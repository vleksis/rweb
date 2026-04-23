use std::collections::HashMap;
use std::env;
use std::str::FromStr;

use anyhow::Context;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use tokio_native_tls::native_tls;

#[derive(Debug, Clone, Copy)]
pub enum Scheme {
    Http,
    Https,
}

impl FromStr for Scheme {
    type Err = UrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            _ => Err(UrlError::InvalidScheme(s.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum UrlError {
    #[error("invalid scheme: {0}")]
    InvalidScheme(String),
}

#[derive(Debug)]
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
            path: format!("/{path}"),
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

    pub async fn request(&self) -> anyhow::Result<String> {
        let req = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", self.path, self.host);
        let mut dst = String::new();

        match self.scheme {
            Scheme::Http => {
                let mut stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
                stream.write_all(req.as_bytes()).await?;
                stream.read_to_string(&mut dst).await?;
            }
            Scheme::Https => {
                let stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
                let connector = native_tls::TlsConnector::new()?;
                let connector = TlsConnector::from(connector);
                let mut stream = connector.connect(&self.host, stream).await?;

                stream.write_all(req.as_bytes()).await?;
                stream.read_to_string(&mut dst).await?;
            }
        }

        let mut lines = dst.lines();

        let mut status_line = lines.next().context("missing status line")?.splitn(3, " ");
        let _version = status_line.next().context("missing HTTP version")?;
        let _status = status_line.next().context("missing status code")?;
        let _explanation = status_line.next().unwrap_or("");

        let mut headers = HashMap::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();
                headers.insert(key, value);
            }
        }

        // unsupported for now
        assert!(!headers.contains_key("transfer-encoding"));
        assert!(!headers.contains_key("content-encoding"));

        let content = lines.collect();

        Ok(content)
    }
}

pub fn show(html: &str) {
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            print!("{}", c);
        }
    }
}

async fn load(url: &Url) -> anyhow::Result<()> {
    let resp = url.request().await?;
    show(&resp);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let url = Url::parse(&args[1])?;
    load(&url).await?;

    Ok(())
}
