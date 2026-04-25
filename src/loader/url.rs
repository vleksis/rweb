use std::str::FromStr;

use crate::loader::file;
use crate::loader::http;

#[derive(Debug)]
pub enum Url {
    Http(http::Url),
    File(file::Url),
}

impl From<http::Url> for Url {
    fn from(url: http::Url) -> Self {
        Url::Http(url)
    }
}

impl From<file::Url> for Url {
    fn from(url: file::Url) -> Self {
        Url::File(url)
    }
}

impl FromStr for Url {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") || s.starts_with("https://") {
            let url = s.parse()?;
            Ok(Url::Http(url))
        } else if s.starts_with("file://") {
            let url = s.parse()?;
            Ok(Url::File(url))
        } else {
            Err(anyhow::anyhow!("Invalid URL"))
        }
    }
}
