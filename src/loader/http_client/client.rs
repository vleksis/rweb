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
