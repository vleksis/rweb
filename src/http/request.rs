use anyhow::Context;

use crate::http::HeaderMap;
use crate::http::HeaderName;
use crate::http::Method;
use crate::http::Scheme;
use crate::http::Url;
use crate::http::Version;

#[derive(Debug, Clone)]
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

    pub fn url(&self) -> &Url {
        &self.url
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_get_request() {
        let url: Url = "http://example.com/path".parse().unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .version(Version::HTTP10)
            .url(url)
            .header(HeaderName::HOST, "example.com")
            .header(HeaderName::CONNECTION, "close")
            .build()
            .unwrap();

        let request = request.to_string();

        assert!(request.starts_with("GET /path HTTP/1.0\r\n"));
        assert!(request.contains("host: example.com\r\n"));
        assert!(request.contains("connection: close\r\n"));
        assert!(request.ends_with("\r\n\r\n"));
    }
}
