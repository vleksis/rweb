use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HeaderName(Inner);

/// It is there only to hide private enum from public API
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Inner {
    Standard(StandardHeaderName),
    Custom(CustomHeaderName),
}

impl FromStr for HeaderName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(standard) = StandardHeaderName::from_str(s) {
            Ok(HeaderName(Inner::Standard(standard)))
        } else {
            let custom = CustomHeaderName::from_str(s)?;
            Ok(HeaderName(Inner::Custom(custom)))
        }
    }
}

impl std::fmt::Display for HeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Inner::Standard(standard) => write!(f, "{}", standard),
            Inner::Custom(custom) => write!(f, "{}", custom),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum StandardHeaderName {
    Host,
    Connection,
    ContentLength,
    UserAgent,
    TransferEncoding,
    ContentEncoding,
}

impl std::fmt::Display for StandardHeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StandardHeaderName::Host => write!(f, "host"),
            StandardHeaderName::Connection => write!(f, "connection"),
            StandardHeaderName::ContentLength => write!(f, "content-length"),
            StandardHeaderName::UserAgent => write!(f, "user-agent"),
            StandardHeaderName::TransferEncoding => write!(f, "transfer-encoding"),
            StandardHeaderName::ContentEncoding => write!(f, "content-encoding"),
        }
    }
}

impl FromStr for StandardHeaderName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "host" => Ok(StandardHeaderName::Host),
            "connection" => Ok(StandardHeaderName::Connection),
            "content-length" => Ok(StandardHeaderName::ContentLength),
            "user-agent" => Ok(StandardHeaderName::UserAgent),
            "transfer-encoding" => Ok(StandardHeaderName::TransferEncoding),
            "content-encoding" => Ok(StandardHeaderName::ContentEncoding),
            _ => Err(anyhow::anyhow!("invalid standard header name")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CustomHeaderName(String);

impl std::fmt::Display for CustomHeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CustomHeaderName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CustomHeaderName(s.trim().to_lowercase()))
    }
}

impl HeaderName {
    pub const HOST: Self = Self(Inner::Standard(StandardHeaderName::Host));
    pub const CONNECTION: Self = Self(Inner::Standard(StandardHeaderName::Connection));
    pub const CONTENT_LENGTH: Self = Self(Inner::Standard(StandardHeaderName::ContentLength));
    pub const USER_AGENT: Self = Self(Inner::Standard(StandardHeaderName::UserAgent));
    pub const TRANSFER_ENCODING: Self = Self(Inner::Standard(StandardHeaderName::TransferEncoding));
    pub const CONTENT_ENCODING: Self = Self(Inner::Standard(StandardHeaderName::ContentEncoding));
}

#[derive(Debug, Clone, Default)]
pub struct HeaderMap {
    headers: HashMap<HeaderName, Vec<String>>,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, name: HeaderName, value: &str) {
        self.headers
            .entry(name)
            .or_default()
            .push(value.to_string());
    }

    pub fn set(&mut self, name: HeaderName, value: &str) {
        self.headers.insert(name, vec![value.to_string()]);
    }

    pub fn get(&self, name: &HeaderName) -> Option<&str> {
        self.headers.get(name)?.first().map(String::as_str)
    }

    pub fn get_all(&self, name: &HeaderName) -> impl Iterator<Item = &str> {
        self.headers
            .get(name)
            .into_iter()
            .flatten()
            .map(String::as_str)
    }
}

impl std::fmt::Display for HeaderMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, values) in &self.headers {
            for value in values {
                write!(f, "{}: {}\r\n", name, value)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_custom_header_names() {
        let name: HeaderName = "   X-Test-Header ".parse().unwrap();

        assert_eq!(name.to_string(), "x-test-header");
    }

    #[test]
    fn preserves_repeated_header_values() {
        let name: HeaderName = "x-test".parse().unwrap();
        let mut headers = HeaderMap::new();

        headers.append(name.clone(), "one");
        headers.append(name.clone(), "two");

        assert_eq!(headers.get(&name), Some("one"));
        assert_eq!(
            headers.get_all(&name).collect::<Vec<_>>(),
            vec!["one", "two"]
        );
    }

    #[test]
    fn set_replaces_existing_values() {
        let name: HeaderName = "x-test".parse().unwrap();
        let mut headers = HeaderMap::new();

        headers.append(name.clone(), "one");
        headers.set(name.clone(), "two");

        assert_eq!(headers.get_all(&name).collect::<Vec<_>>(), vec!["two"]);
    }
}
