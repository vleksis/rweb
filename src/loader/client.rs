use crate::loader::Request;
use crate::loader::Response;
use crate::loader::data_client;
use crate::loader::file_client;
use crate::loader::http_client;

#[derive(Debug)]
pub struct Client {
    http: http_client::Client,
}

impl Client {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub async fn load(&mut self, request: impl Into<Request>) -> anyhow::Result<Response> {
        let resp: Response = match request.into() {
            Request::Http(request) => self.http.load(request).await?.into(),
            Request::File(request) => file_client::load(request).await?.into(),
            Request::Data(request) => data_client::load(request).await?.into(),
        };

        Ok(resp)
    }
}

#[derive(Debug, Default)]
pub struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self) -> Client {
        Client {
            http: http_client::Client::default(),
        }
    }
}
