use std::str::FromStr;

use anyhow::anyhow;

#[derive(Debug, Clone, Copy)]
pub struct Version(VersionInner);

#[derive(Debug, Clone, Copy)]
pub enum VersionInner {
    Http10,
    Http11,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            VersionInner::Http10 => write!(f, "HTTP/1.0"),
            VersionInner::Http11 => write!(f, "HTTP/1.1"),
        }
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(Self(VersionInner::Http10)),
            "HTTP/1.1" => Ok(Self(VersionInner::Http11)),
            _ => Err(anyhow!("invalid HTTP version")),
        }
    }
}

impl Version {
    pub const HTTP10: Self = Self(VersionInner::Http10);
    pub const HTTP11: Self = Self(VersionInner::Http11);
}
