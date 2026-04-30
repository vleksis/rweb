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
    use super::*;

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
}
