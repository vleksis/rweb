use std::str::FromStr;

use crate::loader::url::UrlError;

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
