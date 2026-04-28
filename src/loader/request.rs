use crate::http;
use crate::loader::data_client;
use crate::loader::file_client;

#[derive(Debug)]
pub enum Request {
    Http(http::Request),
    File(file_client::Request),
    Data(data_client::Request),
}

impl From<http::Request> for Request {
    fn from(req: http::Request) -> Self {
        Self::Http(req)
    }
}

impl From<file_client::Request> for Request {
    fn from(req: file_client::Request) -> Self {
        Self::File(req)
    }
}

impl From<data_client::Request> for Request {
    fn from(req: data_client::Request) -> Self {
        Self::Data(req)
    }
}

impl Request {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

#[derive(Debug, Default)]
pub struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn http(self) -> http::Builder {
        http::Builder::new()
    }

    pub fn file(self) -> file_client::Builder {
        file_client::Builder::new()
    }

    pub fn data(self) -> data_client::Builder {
        data_client::Builder::new()
    }
}
