use std::str::FromStr;

use crate::http::UrlError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
