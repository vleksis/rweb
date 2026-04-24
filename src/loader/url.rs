use thiserror::Error;

use crate::loader::scheme::Scheme;

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

impl Url {
    pub fn parse(raw: &str) -> anyhow::Result<Self> {
        let Some((scheme, path)) = raw.split_once("://") else {
            anyhow::bail!("invalid url: {raw}");
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

    fn parse_host(scheme: Scheme, raw: &str) -> anyhow::Result<(String, u16)> {
        let Some((host, port)) = raw.rsplit_once(':') else {
            match scheme {
                Scheme::Http => return Ok((raw.to_string(), 80)),
                Scheme::Https => return Ok((raw.to_string(), 443)),
            }
        };

        Ok((host.to_string(), port.parse()?))
    }

    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
