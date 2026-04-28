use crate::http;
use crate::loader::data_client;
use crate::loader::file_client;

#[derive(Debug)]
pub enum Response {
    Http(http::Response),
    File(file_client::Response),
    Data(data_client::Response),
}

impl From<http::Response> for Response {
    fn from(resp: http::Response) -> Self {
        Self::Http(resp)
    }
}

impl From<file_client::Response> for Response {
    fn from(resp: file_client::Response) -> Self {
        Self::File(resp)
    }
}

impl From<data_client::Response> for Response {
    fn from(resp: data_client::Response) -> Self {
        Self::Data(resp)
    }
}

impl Response {
    pub fn body_as_str(&self) -> anyhow::Result<&str> {
        match self {
            Self::Http(resp) => resp.body_as_str(),
            Self::File(resp) => resp.body_as_str(),
            Self::Data(resp) => resp.body_as_str(),
        }
    }
}
