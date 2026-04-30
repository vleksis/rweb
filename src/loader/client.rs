use anyhow::Context;

use crate::http;
use crate::http::HeaderName;
use crate::http::Method;
use crate::http::Version;
use crate::loader::Response;
use crate::loader::Url;
use crate::loader::data_client;
use crate::loader::file_client;
use crate::loader::http_client;

#[derive(Debug)]
pub struct Client {
    http: http_client::Client,

    max_redirects: usize,
    connection_policy: ConnectionPolicy,
    user_agent: String,
}

#[derive(Debug)]
pub enum ConnectionPolicy {
    Close,
    KeepAlive,
}

impl Client {
    pub fn builder() -> Builder {
        Builder::new()
    }

    async fn load_http(&mut self, mut url: http::Url) -> anyhow::Result<Response> {
        let mut redirect_count = 0;
        loop {
            let request = self.build_http_request(&url)?;
            let response = self.http.load(request).await?;

            if response.head().status().is_redirect() {
                redirect_count += 1;
                if redirect_count > self.max_redirects {
                    return Err(anyhow::anyhow!("Too many redirects"));
                }

                let location = response
                    .head()
                    .location()
                    .context("no Location with redirect response")?;
                url = url.resolve(location)?;

                continue;
            }

            return Ok(response.into());
        }
    }

    async fn load_file(&mut self, url: file_client::Url) -> anyhow::Result<Response> {
        let request = file_client::Request::builder().url(&url).build()?;
        let response = file_client::load(request).await?;
        Ok(response.into())
    }

    async fn load_data(&mut self, url: data_client::Url) -> anyhow::Result<Response> {
        let request = data_client::Request::builder().url(&url).build()?;
        let response = data_client::load(request).await?;
        Ok(response.into())
    }

    pub async fn load_url(&mut self, url: &Url) -> anyhow::Result<Response> {
        match url {
            Url::Http(url) => self.load_http(url.clone()).await,
            Url::File(url) => self.load_file(url.clone()).await,
            Url::Data(url) => self.load_data(url.clone()).await,
        }
    }

    fn build_http_request(&self, url: &http::Url) -> anyhow::Result<http::Request> {
        let connection = match self.connection_policy {
            ConnectionPolicy::Close => "close",
            ConnectionPolicy::KeepAlive => "keep-alive",
        };

        http::Request::builder()
            .url(url.clone())
            .method(Method::GET)
            .version(Version::HTTP11)
            .header(HeaderName::HOST, &url.host_header())
            .header(HeaderName::CONNECTION, connection)
            .header(HeaderName::USER_AGENT, &self.user_agent)
            .build()
    }
}

#[derive(Debug)]
pub struct Builder {
    max_redirects: usize,
    connection_policy: ConnectionPolicy,
    user_agent: String,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            max_redirects: 10,
            connection_policy: ConnectionPolicy::KeepAlive,
            user_agent: "RwebBrowser/0.1".to_string(),
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Client {
        Client {
            http: http_client::Client::default(),
            max_redirects: self.max_redirects,
            connection_policy: self.connection_policy,
            user_agent: self.user_agent,
        }
    }

    pub fn max_redirects(mut self, max_redirects: usize) -> Self {
        self.max_redirects = max_redirects;

        self
    }

    pub fn connection_policy(mut self, connection_policy: ConnectionPolicy) -> Self {
        self.connection_policy = connection_policy;

        self
    }

    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_owned();

        self
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;
    use tokio::net::TcpStream;

    use super::*;
    use crate::http::header_end;

    #[test]
    fn builds_http_request_with_default_headers() {
        let url: http::Url = "http://localhost:8080/path".parse().unwrap();
        let client = Client::builder().user_agent("test-agent").build();

        let request = client.build_http_request(&url).unwrap().to_string();

        assert!(request.starts_with("GET /path HTTP/1.1\r\n"));
        assert!(request.contains("host: localhost:8080\r\n"));
        assert!(request.contains("connection: keep-alive\r\n"));
        assert!(request.contains("user-agent: test-agent\r\n"));
    }

    #[test]
    fn builds_http_request_with_close_connection_policy() {
        let url: http::Url = "http://localhost/path".parse().unwrap();
        let client = Client::builder()
            .connection_policy(ConnectionPolicy::Close)
            .build();

        let request = client.build_http_request(&url).unwrap().to_string();

        assert!(request.contains("connection: close\r\n"));
    }

    #[tokio::test]
    async fn follows_same_origin_redirect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(serve_requests(listener, 2, |request| {
            match request_path(request) {
                "/start" => response("302 Found", &[("Location", "/final")], ""),
                "/final" => response("200 OK", &[], "done"),
                path => panic!("unexpected path: {path}"),
            }
        }));
        let url: Url = format!("http://{addr}/start").parse().unwrap();
        let mut client = Client::builder().build();

        let response = client.load_url(&url).await.unwrap();
        let requests = server.await.unwrap();

        assert_eq!(response.body_as_str().unwrap(), "done");
        assert!(requests[0].starts_with("GET /start HTTP/1.1\r\n"));
        assert!(requests[1].starts_with("GET /final HTTP/1.1\r\n"));
    }

    #[tokio::test]
    async fn rejects_redirect_without_location() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(serve_requests(listener, 1, |_| {
            response("302 Found", &[], "")
        }));
        let url: Url = format!("http://{addr}/start").parse().unwrap();
        let mut client = Client::builder().build();

        let err = client.load_url(&url).await.unwrap_err();
        server.await.unwrap();

        assert_eq!(err.to_string(), "no Location with redirect response");
    }

    #[tokio::test]
    async fn stops_after_max_redirects() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(serve_requests(listener, 2, |_| {
            response("302 Found", &[("Location", "/loop")], "")
        }));
        let url: Url = format!("http://{addr}/loop").parse().unwrap();
        let mut client = Client::builder().max_redirects(1).build();

        let err = client.load_url(&url).await.unwrap_err();
        let requests = server.await.unwrap();

        assert_eq!(err.to_string(), "Too many redirects");
        assert_eq!(requests.len(), 2);
    }

    #[tokio::test]
    async fn max_redirects_zero_rejects_first_redirect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(serve_requests(listener, 1, |_| {
            response("302 Found", &[("Location", "/final")], "")
        }));
        let url: Url = format!("http://{addr}/start").parse().unwrap();
        let mut client = Client::builder().max_redirects(0).build();

        let err = client.load_url(&url).await.unwrap_err();
        let requests = server.await.unwrap();

        assert_eq!(err.to_string(), "Too many redirects");
        assert_eq!(requests.len(), 1);
    }

    #[tokio::test]
    async fn rejects_redirect_to_unsupported_scheme() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(serve_requests(listener, 1, |_| {
            response("302 Found", &[("Location", "data:text/html,Hello")], "")
        }));
        let url: Url = format!("http://{addr}/start").parse().unwrap();
        let mut client = Client::builder().build();

        let err = client.load_url(&url).await.unwrap_err();
        server.await.unwrap();

        assert_eq!(
            err.to_string(),
            "unsupported redirect location: data:text/html,Hello"
        );
    }

    #[tokio::test]
    async fn rebuilds_host_header_after_absolute_redirect() {
        let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_addr = target_listener.local_addr().unwrap();
        let target_server = tokio::spawn(serve_requests(target_listener, 1, |_| {
            response("200 OK", &[], "target")
        }));

        let origin_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin_addr = origin_listener.local_addr().unwrap();
        let origin_server = tokio::spawn(serve_requests(origin_listener, 1, move |_| {
            let location = format!("http://{target_addr}/final");
            response("302 Found", &[("Location", &location)], "")
        }));

        let url: Url = format!("http://{origin_addr}/start").parse().unwrap();
        let mut client = Client::builder().build();

        let response = client.load_url(&url).await.unwrap();
        let origin_requests = origin_server.await.unwrap();
        let target_requests = target_server.await.unwrap();

        assert_eq!(response.body_as_str().unwrap(), "target");
        assert!(origin_requests[0].contains(&format!("host: {origin_addr}\r\n")));
        assert!(target_requests[0].contains(&format!("host: {target_addr}\r\n")));
    }

    async fn serve_requests(
        listener: TcpListener,
        expected: usize,
        respond: impl Fn(&str) -> Vec<u8> + Send + 'static,
    ) -> Vec<String> {
        let mut requests = Vec::new();

        for _ in 0..expected {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await.unwrap();
            let response = respond(&request);

            stream.write_all(&response).await.unwrap();
            requests.push(request);
        }

        requests
    }

    async fn read_request(stream: &mut TcpStream) -> anyhow::Result<String> {
        let mut raw = Vec::new();
        let mut buf = [0; 1024];

        loop {
            let read = stream.read(&mut buf).await?;
            if read == 0 {
                anyhow::bail!("connection closed before request completed");
            }

            raw.extend_from_slice(&buf[..read]);

            if header_end(&raw).is_some() {
                return Ok(String::from_utf8(raw)?);
            }
        }
    }

    fn request_path(request: &str) -> &str {
        request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap()
    }

    fn response(status: &str, headers: &[(&str, &str)], body: &str) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );

        for (name, value) in headers {
            response.push_str(name);
            response.push_str(": ");
            response.push_str(value);
            response.push_str("\r\n");
        }

        response.push_str("\r\n");
        response.push_str(body);
        response.into_bytes()
    }
}
