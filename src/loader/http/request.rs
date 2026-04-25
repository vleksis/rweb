use anyhow::Context;

use crate::loader::http::HeaderMap;
use crate::loader::http::HeaderName;
use crate::loader::http::Method;
use crate::loader::http::Scheme;
use crate::loader::http::Url;
use crate::loader::http::Version;

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
