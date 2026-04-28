use anyhow::Context;

use crate::http::HeaderMap;
use crate::http::HeaderName;
use crate::http::StatusCode;
use crate::http::Version;

#[derive(Debug)]
pub struct Head {
    status: StatusCode,
    version: Version,
    headers: HeaderMap,
}

impl Head {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn content_length(&self) -> Option<usize> {
        self.headers.get(&HeaderName::CONTENT_LENGTH)?.parse().ok()
    }

    pub fn transfer_encoding(&self) -> Option<&str> {
        self.headers.get(&HeaderName::TRANSFER_ENCODING)
    }

    pub fn content_encoding(&self) -> Option<&str> {
        self.headers.get(&HeaderName::CONTENT_ENCODING)
    }

    pub fn connection(&self) -> Option<&str> {
        self.headers.get(&HeaderName::CONNECTION)
    }

    pub fn is_connection_closed(&self) -> bool {
        match self.connection() {
            Some(value) => value == "close",
            None => false,
        }
    }
}

impl TryFrom<&[u8]> for Head {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw = std::str::from_utf8(value).context("invalid UTF-8 in header")?;
        let mut lines = raw.lines();

        let mut status_line = lines.next().context("missing status line")?.splitn(3, ' ');
        let version: Version = status_line
            .next()
            .context("missing HTTP version")?
            .parse()?;
        let status: StatusCode = status_line.next().context("missing status code")?.parse()?;
        let _explanation = status_line.next().unwrap_or("");

        let mut headers = HeaderMap::new();
        for line in lines {
            if let Some((name, value)) = line.split_once(':') {
                let name = name.trim();
                let header = name.parse()?;
                let value = value.trim();
                headers.append(header, value);
            }
        }

        Ok(Head {
            status,
            version,
            headers,
        })
    }
}

pub fn header_end(raw: &[u8]) -> Option<usize> {
    raw.windows(4).position(|w| w == b"\r\n\r\n")
}

#[derive(Debug)]
pub struct Response {
    head: Head,
    body: Vec<u8>,
}

impl TryFrom<&[u8]> for Response {
    type Error = anyhow::Error;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let (head, body) = {
            let header_end = header_end(s).context("missing header/body separator")?;
            (&s[..header_end], &s[header_end + 4..])
        };

        let head = Head::try_from(head)?;

        Ok(Self {
            head,
            body: body.to_vec(),
        })
    }
}

impl Response {
    pub fn new(head: Head, body: Vec<u8>) -> Self {
        Self { head, body }
    }

    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    pub fn version(&self) -> Version {
        self.head.version
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.head.headers
    }

    pub fn head(&self) -> &Head {
        &self.head
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn body_as_str(&self) -> anyhow::Result<&str> {
        std::str::from_utf8(&self.body).context("response body is not valid utf-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_headers_and_preserves_body_bytes() {
        let raw = b"HTTP/1.0 200 OK\r\nContent-Type: text/html\r\nX-Test: one\r\nX-Test: two\r\n\r\nhello\nworld";

        let response = Response::try_from(raw.as_slice()).unwrap();
        let content_type = "content-type".parse().unwrap();
        let x_test = "x-test".parse().unwrap();

        assert_eq!(response.headers().get(&content_type), Some("text/html"));
        assert_eq!(
            response.headers().get_all(&x_test).collect::<Vec<_>>(),
            vec!["one", "two"]
        );
        assert_eq!(response.body(), b"hello\nworld");
    }

    #[test]
    fn accepts_non_utf8_body() {
        let raw = b"HTTP/1.0 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n\xff\x00";

        let response = Response::try_from(raw.as_slice()).unwrap();

        assert_eq!(response.body(), b"\xff\x00");
        assert!(response.body_as_str().is_err());
    }
}
