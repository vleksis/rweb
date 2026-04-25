use std::str::FromStr;

use crate::loader::data;
use crate::loader::file;
use crate::loader::http;

#[derive(Debug)]
pub enum Url {
    Http(http::Url),
    File(file::Url),
    Data(data::Url),
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

impl From<data::Url> for Url {
    fn from(url: data::Url) -> Self {
        Url::Data(url)
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
        } else if s.starts_with("data:") {
            let url = s.parse()?;
            Ok(Url::Data(url))
        } else {
            Err(anyhow::anyhow!("Invalid URL"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_http_urls() {
        let url: Url = "https://example.com/".parse().unwrap();

        assert!(matches!(url, Url::Http(_)));
    }

    #[test]
    fn dispatches_file_urls() {
        let url: Url = "file:///tmp/index.html".parse().unwrap();

        assert!(matches!(url, Url::File(_)));
    }

    #[test]
    fn dispatches_data_urls() {
        let url: Url = "data:text/html,Hello world!".parse().unwrap();

        assert!(matches!(url, Url::Data(_)));
    }

    #[test]
    fn rejects_unknown_scheme() {
        let result: Result<Url, _> = "brainrot://example.com/".parse();

        assert!(result.is_err());
    }
}
