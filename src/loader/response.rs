use crate::loader::http;

#[derive(Debug)]
pub enum Response {
    Http(http::Response),
}

impl From<http::Response> for Response {
    fn from(resp: http::Response) -> Self {
        Self::Http(resp)
    }
}

impl Response {
    pub fn body_as_str(&self) -> anyhow::Result<&str> {
        match self {
            Self::Http(resp) => resp.body_as_str(),
        }
    }
}
