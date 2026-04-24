use anyhow::Context;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use tokio_native_tls::native_tls;

use crate::loader::HeaderMap;
use crate::loader::HeaderName;
use crate::loader::Scheme;
use crate::loader::Url;

#[derive(Debug)]
pub struct Request {
    method: Method,
    url: Url,
    version: Version,
    headers: HeaderMap,
}

impl Request {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn host(&self) -> &str {
        self.url.host()
    }

    pub fn port(&self) -> u16 {
        self.url.port()
    }

    pub fn scheme(&self) -> Scheme {
        self.url.scheme()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    pub fn path(&self) -> &str {
        self.url.path()
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}\r\n{}\r\n",
            self.method,
            self.path(),
            self.version,
            self.headers
        )
    }
}

pub async fn send(request: Request) -> anyhow::Result<String> {
    let mut dst = String::new();
    match request.scheme() {
        Scheme::Http => {
            let mut stream = TcpStream::connect((request.host(), request.port())).await?;
            stream.write_all(request.to_string().as_bytes()).await?;
            stream.read_to_string(&mut dst).await?;
        }
        Scheme::Https => {
            let stream = TcpStream::connect((request.host(), request.port())).await?;
            let connector = native_tls::TlsConnector::new()?;
            let connector = TlsConnector::from(connector);
            let mut stream = connector.connect(request.host(), stream).await?;

            stream.write_all(request.to_string().as_bytes()).await?;
            stream.read_to_string(&mut dst).await?;
        }
    }

    let (head, body) = dst
        .split_once("\r\n\r\n")
        .context("missing header/body separator")?;
    let mut lines = head.lines();

    let mut status_line = lines.next().context("missing status line")?.splitn(3, " ");
    let _version = status_line.next().context("missing HTTP version")?;
    let _status = status_line.next().context("missing status code")?;
    let _explanation = status_line.next().unwrap_or("");

    let mut headers = HeaderMap::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }

        if let Some((name, value)) = line.split_once(':') {
            let header = name.parse()?;
            headers.append(header, value);
        }
    }

    // unsupported for now
    assert!(headers.get(&HeaderName::TRANSFER_ENCODING).is_none());
    assert!(headers.get(&HeaderName::CONTENT_ENCODING).is_none());

    Ok(body.to_string())
}

#[derive(Debug)]
pub struct Method(MethodInner);

#[derive(Debug)]
pub enum MethodInner {
    Get,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            MethodInner::Get => write!(f, "GET"),
        }
    }
}

impl Method {
    pub const GET: Self = Self(MethodInner::Get);
}

#[derive(Debug)]
pub struct Version(VersionInner);

#[derive(Debug)]
pub enum VersionInner {
    Http10,
    Http11,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            VersionInner::Http10 => write!(f, "HTTP/1.0"),
            VersionInner::Http11 => write!(f, "HTTP/1.1"),
        }
    }
}

impl Version {
    pub const HTTP10: Self = Self(VersionInner::Http10);
    pub const HTTP11: Self = Self(VersionInner::Http11);
}

#[derive(Debug, Default)]
pub struct Builder {
    method: Option<Method>,
    url: Option<Url>,
    version: Option<Version>,
    headers: HeaderMap,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn method(self, method: Method) -> Self {
        Self {
            method: Some(method),
            ..self
        }
    }

    pub fn url(self, url: Url) -> Self {
        Self {
            url: Some(url),
            ..self
        }
    }

    pub fn version(self, version: Version) -> Self {
        Self {
            version: Some(version),
            ..self
        }
    }

    pub fn header(self, name: HeaderName, value: &str) -> Self {
        let mut headers = self.headers;
        headers.append(name, value);
        Self { headers, ..self }
    }

    pub fn build(self) -> anyhow::Result<Request> {
        let method = self.method.context("missing method")?;
        let url = self.url.context("missing url")?;
        let version = self.version.context("missing version")?;
        Ok(Request {
            method,
            url,
            version,
            headers: self.headers,
        })
    }
}
