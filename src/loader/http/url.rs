use std::str::FromStr;

use thiserror::Error;

use crate::loader::http::Scheme;

#[derive(Debug, Error)]
pub enum UrlError {
    #[error("invalid scheme: {0}")]
    InvalidScheme(String),
}

#[derive(Debug, Clone)]
pub struct Url {
    scheme: Scheme,
    host: String,
    port: u16,
    path: String,
}

impl FromStr for Url {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((scheme, path)) = s.split_once("://") else {
            anyhow::bail!("invalid url: {s}");
        };

        let scheme = scheme.parse()?;

        let (host, path) = if let Some((host, path)) = path.split_once('/') {
            (host, format!("/{path}"))
        } else {
            (path, "/".to_string())
        };

        let (host, port) = Self::parse_host(scheme, host)?;

        Ok(Self {
            scheme,
            host,
            port,
            path,
        })
    }
}

impl Url {
    fn parse_host(scheme: Scheme, raw: &str) -> anyhow::Result<(String, u16)> {
        let Some((host, port)) = raw.rsplit_once(':') else {
            match scheme {
                Scheme::Http => return Ok((raw.to_string(), 80)),
                Scheme::Https => return Ok((raw.to_string(), 443)),
            }
        };

        let host = host.to_string();
        let port = port.parse()?;

        Ok((host, port))
    }

    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn host_header(&self) -> String {
        if let Scheme::Http = self.scheme
            && self.port == 80
        {
            self.host.clone()
        } else if let Scheme::Https = self.scheme
            && self.port == 443
        {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_http_url_with_defaults() {
        let url: Url = "http://example.com/foo".parse().unwrap();

        assert!(matches!(url.scheme(), Scheme::Http));
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.port(), 80);
        assert_eq!(url.path(), "/foo");
    }

    #[test]
    fn parses_https_url_with_default_port() {
        let url: Url = "https://example.com/".parse().unwrap();

        assert!(matches!(url.scheme(), Scheme::Https));
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.port(), 443);
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn parses_explicit_port() {
        let url: Url = "http://localhost:8000/".parse().unwrap();

        assert_eq!(url.host(), "localhost");
        assert_eq!(url.port(), 8000);
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn host_header() {
        let http_default: Url = "http://localhost/".parse().unwrap();
        let http_explicit_default: Url = "http://localhost:80/".parse().unwrap();
        let http_non_default: Url = "http://localhost:8080/".parse().unwrap();
        let https_default: Url = "https://example.com/".parse().unwrap();
        let https_explicit_default: Url = "https://example.com:443/".parse().unwrap();
        let https_non_default: Url = "https://example.com:8443/".parse().unwrap();

        assert_eq!(http_default.host_header(), "localhost");
        assert_eq!(http_explicit_default.host_header(), "localhost");
        assert_eq!(http_non_default.host_header(), "localhost:8080");
        assert_eq!(https_default.host_header(), "example.com");
        assert_eq!(https_explicit_default.host_header(), "example.com");
        assert_eq!(https_non_default.host_header(), "example.com:8443");
    }
}
