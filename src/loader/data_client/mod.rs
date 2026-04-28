use std::str::FromStr;

use anyhow::Context;

use crate::mime::MediaType;

#[derive(Debug, Clone)]
pub struct Url {
    media_type: MediaType,
    body: Vec<u8>,
}

impl FromStr for Url {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("data:")
            .context("data URL should start with data:")?;
        let (media_type, body) = s
            .split_once(',')
            .context("data URL should contain a media type and body")?;

        let media_type = if media_type.is_empty() {
            MediaType::default()
        } else {
            media_type.parse().context("invalid media type")?
        };

        Ok(Self {
            media_type,
            body: body.as_bytes().to_vec(),
        })
    }
}

impl Url {
    pub fn media_type(&self) -> &MediaType {
        &self.media_type
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub url: Url,
}

impl Request {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

#[derive(Debug, Default)]
pub struct Builder {
    url: Option<Url>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn url(self, url: &Url) -> Self {
        Self {
            url: Some(url.clone()),
        }
    }

    pub fn build(self) -> anyhow::Result<Request> {
        Ok(Request {
            url: self.url.context("url is required")?,
        })
    }
}

#[derive(Debug)]
pub struct Response {
    body: Vec<u8>,
}

impl Response {
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn body_as_str(&self) -> anyhow::Result<&str> {
        std::str::from_utf8(&self.body).context("failed to parse body as UTF-8")
    }
}

pub async fn load(request: Request) -> anyhow::Result<Response> {
    let body = request.url.body;
    Ok(Response { body })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_data_url_with_media_type() {
        let url: Url = "data:text/html,Hello world!".parse().unwrap();

        assert_eq!(url.media_type(), &MediaType::TextHtml);
        assert_eq!(url.body(), b"Hello world!");
    }

    #[tokio::test]
    async fn loads_data_request_body() {
        let url: Url = "data:text/html,Hello world!".parse().unwrap();
        let request = Request::builder().url(&url).build().unwrap();

        let response = load(request).await.unwrap();

        assert_eq!(response.body_as_str().unwrap(), "Hello world!");
    }

    #[test]
    fn defaults_to_text_plain() {
        let url: Url = "data:,Hello world!".parse().unwrap();

        assert_eq!(url.media_type(), &MediaType::TextPlain);
        assert_eq!(url.body(), b"Hello world!");
    }

    #[test]
    fn parses_json_media_type() {
        let url: Url = "data:application/json,{\"ok\":true}".parse().unwrap();

        assert_eq!(url.media_type(), &MediaType::ApplicationJson);
    }
}
