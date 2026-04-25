use crate::loader::http;

#[derive(Debug)]
pub enum Request {
    Http(http::Request),
}

impl From<http::Request> for Request {
    fn from(req: http::Request) -> Self {
        Self::Http(req)
    }
}

impl Request {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

#[derive(Debug)]
pub struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn http(self) -> http::Builder {
        http::Builder::new()
    }
}
