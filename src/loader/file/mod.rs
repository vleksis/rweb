use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use tokio::fs::read;

#[derive(Debug, Clone)]
pub struct Url {
    path: PathBuf,
}

impl FromStr for Url {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("file://")
            .context("file URL should start with file://")?;

        let path = PathBuf::from(s);
        if !path.is_absolute() {
            return Err(anyhow::anyhow!("file path should be absolute"));
        }

        Ok(Self { path })
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    url: Url,
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
    let path = request.url.path;
    let file = read(path).await.context("failed to read file")?;
    Ok(Response { body: file })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_absolute_file_url() {
        let url: Url = "file:///tmp/index.html".parse().unwrap();

        assert_eq!(url.path, PathBuf::from("/tmp/index.html"));
    }

    #[test]
    fn rejects_relative_file_url() {
        let result = "file://fixtures/index.html".parse::<Url>();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn loads_file_body_as_bytes() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/linus.txt");
        let url: Url = format!("file://{}", path.display()).parse().unwrap();
        let request = Request::builder().url(&url).build().unwrap();

        let response = load(request).await.unwrap();

        assert_eq!(
            response.body_as_str().unwrap(),
            "Talk is cheap. Show me the code.\n"
        );
    }
}
