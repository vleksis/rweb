use crate::loader::file;
use crate::loader::http;

#[derive(Debug)]
pub enum Request {
    Http(http::Request),
    File(file::Request),
}

impl From<http::Request> for Request {
    fn from(req: http::Request) -> Self {
        Self::Http(req)
    }
}

impl From<file::Request> for Request {
    fn from(req: file::Request) -> Self {
        Self::File(req)
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

    pub fn file(self) -> file::Builder {
        file::Builder::new()
    }
}
