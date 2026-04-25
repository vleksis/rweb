use crate::loader::file;
use crate::loader::http;

#[derive(Debug)]
pub enum Response {
    Http(http::Response),
    File(file::Response),
}

impl From<http::Response> for Response {
    fn from(resp: http::Response) -> Self {
        Self::Http(resp)
    }
}

impl From<file::Response> for Response {
    fn from(resp: file::Response) -> Self {
        Self::File(resp)
    }
}

impl Response {
    pub fn body_as_str(&self) -> anyhow::Result<&str> {
        match self {
            Self::Http(resp) => resp.body_as_str(),
            Self::File(resp) => resp.body_as_str(),
        }
    }
}
