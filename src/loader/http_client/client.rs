use std::collections::HashMap;

use anyhow::Context;
use anyhow::bail;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use tokio_native_tls::native_tls;

use crate::http::Head;
use crate::http::Request;
use crate::http::Response;
use crate::http::Scheme;
use crate::http::header_end;

#[derive(Debug, Default)]
pub struct Client {
    connections: HashMap<Origin, Connection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Origin {
    scheme: Scheme,
    host: String,
    port: u16,
}

impl From<&Request> for Origin {
    fn from(value: &Request) -> Self {
        Self {
            scheme: value.scheme(),
            host: value.host().into(),
            port: value.port(),
        }
    }
}

#[derive(Debug)]
enum Connection {
    Http(TcpStream),
    Https(tokio_native_tls::TlsStream<TcpStream>),
}

impl Connection {
    async fn send_request(&mut self, request: &Request) -> anyhow::Result<()> {
        let bytes = request.as_bytes();

        match self {
            Self::Http(stream) => stream.write_all(&bytes).await?,
            Self::Https(stream) => stream.write_all(&bytes).await?,
        }

        Ok(())
    }

    async fn read_response(&mut self) -> anyhow::Result<Response> {
        let (head, body_prefix) = self.read_head().await?;

        // unsupported for now
        if head.transfer_encoding().is_some() {
            bail!("Transfer-Encoding is not supported yet")
        }
        if head.content_encoding().is_some() {
            bail!("Content-Encoding is not supported yet");
        }

        let len: usize = head
            .content_length()
            .context("no Content-Length provided")?;
        let body = self.read_body(len, body_prefix).await?;

        let response = Response::new(head, body);

        Ok(response)
    }

    async fn read_head(&mut self) -> anyhow::Result<(Head, Vec<u8>)> {
        let mut raw = Vec::new();
        let mut buf = [0; 1024];

        let header_end = loop {
            let read = match self {
                Self::Http(stream) => stream.read(&mut buf).await?,
                Self::Https(stream) => stream.read(&mut buf).await?,
            };

            if read == 0 {
                bail!("connection closed before response headers completed");
            }

            raw.extend_from_slice(&buf[..read]);

            if let Some(header_end) = header_end(&raw) {
                break header_end;
            }
        };

        let head = Head::try_from(&raw[..header_end])?;
        let body_prefix = raw[header_end + 4..].to_vec();

        Ok((head, body_prefix))
    }

    async fn read_body(&mut self, len: usize, mut body: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let already = body.len();
        body.resize(len, 0);
        let buf = &mut body[already..];

        match self {
            Self::Http(stream) => stream.read_exact(buf).await?,
            Self::Https(stream) => stream.read_exact(buf).await?,
        };

        Ok(body)
    }

    async fn connect(origin: &Origin) -> anyhow::Result<Connection> {
        match origin.scheme {
            Scheme::Http => {
                let stream = TcpStream::connect((origin.host.as_str(), origin.port)).await?;
                Ok(Connection::Http(stream))
            }
            Scheme::Https => {
                let stream = TcpStream::connect((origin.host.as_str(), origin.port)).await?;
                let connector = native_tls::TlsConnector::new()?;
                let connector = TlsConnector::from(connector);
                let stream = connector.connect(&origin.host, stream).await?;

                Ok(Connection::Https(stream))
            }
        }
    }

    pub async fn exchange(&mut self, request: &Request) -> anyhow::Result<Response> {
        self.send_request(request).await?;
        self.read_response().await
    }
}

impl Client {
    async fn take_connection(&mut self, origin: &Origin) -> anyhow::Result<Connection> {
        if let Some(connection) = self.connections.remove(origin) {
            return Ok(connection);
        }

        let connection = Connection::connect(origin).await?;

        Ok(connection)
    }

    fn put_connection(&mut self, origin: &Origin, connection: Connection) {
        self.connections.insert(origin.clone(), connection);
    }

    pub async fn load(&mut self, request: Request) -> anyhow::Result<Response> {
        let origin = Origin::from(&request);

        let mut retry = 0;
        let mut connection = self.take_connection(&origin).await?;

        let response = loop {
            if let Ok(response) = connection.exchange(&request).await {
                break response;
            }

            if retry == 1 {
                bail!("Failed to send request")
            }

            connection = Connection::connect(&origin).await?;
            retry += 1;
        };

        if !response.head().is_connection_closed() {
            self.put_connection(&origin, connection);
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;
    use tokio::net::TcpStream;

    use super::*;
    use crate::http::HeaderName;
    use crate::http::Method;
    use crate::http::Request;
    use crate::http::Url;
    use crate::http::Version;
    use crate::http::header_end;

    #[tokio::test]
    async fn reuses_keep_alive_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let accepted = Arc::new(AtomicUsize::new(0));
        let handled = Arc::new(AtomicUsize::new(0));

        tokio::spawn({
            let accepted = Arc::clone(&accepted);
            let handled = Arc::clone(&handled);

            async move {
                loop {
                    let (stream, _) = listener.accept().await.unwrap();
                    accepted.fetch_add(1, Ordering::SeqCst);

                    tokio::spawn(handle_connection(stream, Arc::clone(&handled)));
                }
            }
        });

        let url: Url = format!("http://{addr}/").parse().unwrap();
        let mut client = Client::default();

        let req = Request::builder()
            .method(Method::GET)
            .version(Version::HTTP11)
            .url(url.clone())
            .header(HeaderName::HOST, &url.host_header())
            .header(HeaderName::CONNECTION, "keep-alive")
            .build()
            .unwrap();

        for i in 1..100 {
            client.load(req.clone()).await.unwrap();
            assert_eq!(handled.load(Ordering::SeqCst), i);
            assert_eq!(client.connections.len(), 1);
        }

        assert_eq!(accepted.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn does_not_store_connection_when_response_says_close() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            read_request(&mut stream).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .await
                .unwrap();
        });

        let url: Url = format!("http://{addr}/").parse().unwrap();
        let mut client = Client::default();
        let req = get(&url);

        let response = client.load(req).await.unwrap();
        server.await.unwrap();

        assert_eq!(response.body_as_str().unwrap(), "OK");
        assert_eq!(client.connections.len(), 0);
    }

    #[tokio::test]
    async fn reconnects_when_pooled_connection_was_closed() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            read_request(&mut stream).await.unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nOK",
                )
                .await
                .unwrap();
            drop(stream);

            let (mut stream, _) = listener.accept().await.unwrap();
            read_request(&mut stream).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .await
                .unwrap();
        });

        let url: Url = format!("http://{addr}/").parse().unwrap();
        let mut client = Client::default();
        let req = get(&url);

        let response = client.load(req.clone()).await.unwrap();
        assert_eq!(response.body_as_str().unwrap(), "OK");
        assert_eq!(client.connections.len(), 1);

        let response = client.load(req).await.unwrap();
        server.await.unwrap();

        assert_eq!(response.body_as_str().unwrap(), "OK");
        assert_eq!(client.connections.len(), 0);
    }

    async fn handle_connection(mut stream: TcpStream, handled: Arc<AtomicUsize>) {
        loop {
            if read_request(&mut stream).await.is_err() {
                return;
            }

            handled.fetch_add(1, Ordering::SeqCst);

            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nOK",
                )
                .await
                .unwrap();
        }
    }

    async fn read_request(stream: &mut TcpStream) -> anyhow::Result<()> {
        let mut raw = Vec::new();
        let mut buf = [0; 1024];

        loop {
            let read = stream.read(&mut buf).await?;
            if read == 0 {
                anyhow::bail!("connection closed before request completed");
            }

            raw.extend_from_slice(&buf[..read]);

            if header_end(&raw).is_some() {
                return Ok(());
            }
        }
    }

    fn get(url: &Url) -> Request {
        Request::builder()
            .method(Method::GET)
            .version(Version::HTTP11)
            .url(url.clone())
            .header(HeaderName::HOST, &url.host_header())
            .header(HeaderName::CONNECTION, "keep-alive")
            .build()
            .unwrap()
    }
}
