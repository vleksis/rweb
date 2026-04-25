use crate::loader::Request;
use crate::loader::Response;
use crate::loader::http;

pub async fn load(request: impl Into<Request>) -> anyhow::Result<Response> {
    let resp: Response = match request.into() {
        Request::Http(request) => http::load(request).await?.into(),
    };

    Ok(resp)
}
