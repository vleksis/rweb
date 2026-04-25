use std::str::FromStr;

use crate::loader::http;

#[derive(Debug)]
pub enum Url {
    Http(http::Url),
}

impl From<http::Url> for Url {
    fn from(url: http::Url) -> Self {
        Url::Http(url)
    }
}

impl FromStr for Url {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") || s.starts_with("https://") {
            let url = http::Url::from_str(s)?;
            Ok(Url::Http(url))
        } else {
            Err(anyhow::anyhow!("Invalid URL"))
        }
    }
}
