use anyhow::Context;

use crate::loader::http::HeaderMap;
use crate::loader::http::StatusCode;
use crate::loader::http::Version;

#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    version: Version,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl TryFrom<&[u8]> for Response {
    type Error = anyhow::Error;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let (head, body) = {
            let header_end = s
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
                .context("missing header/body separator")?;

            (&s[..header_end], &s[header_end + 4..])
        };

        let head = std::str::from_utf8(head).context("invalid UTF-8 in header")?;
        let mut lines = head.lines();

        let mut status_line = lines.next().context("missing status line")?.splitn(3, " ");
        let version = status_line
            .next()
            .context("missing HTTP version")?
            .parse()?;
        let status = status_line.next().context("missing status code")?.parse()?;
        let _explanation = status_line.next().unwrap_or("");

        let mut headers = HeaderMap::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }

            if let Some((name, value)) = line.split_once(':') {
                let name = name.trim();
                let header = name.parse()?;
                let value = value.trim();
                headers.append(header, value);
            }
        }

        Ok(Self {
            status,
            version,
            headers,
            body: body.to_vec(),
        })
    }
}

impl Response {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
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
