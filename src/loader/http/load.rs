use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use tokio_native_tls::native_tls;

use crate::loader::http::HeaderName;
use crate::loader::http::Request;
use crate::loader::http::Response;
use crate::loader::http::Scheme;

pub async fn load(request: Request) -> anyhow::Result<Response> {
    let mut dst = Vec::new();
    match request.scheme() {
        Scheme::Http => {
            let mut stream = TcpStream::connect((request.host(), request.port())).await?;
            stream.write_all(request.to_string().as_bytes()).await?;
            stream.read_to_end(&mut dst).await?;
        }
        Scheme::Https => {
            let stream = TcpStream::connect((request.host(), request.port())).await?;
            let connector = native_tls::TlsConnector::new()?;
            let connector = TlsConnector::from(connector);
            let mut stream = connector.connect(request.host(), stream).await?;

            stream.write_all(request.to_string().as_bytes()).await?;
            stream.read_to_end(&mut dst).await?;
        }
    }

    let response = Response::try_from(dst.as_slice())?;

    // unsupported for now
    assert!(
        response
            .headers()
            .get(&HeaderName::TRANSFER_ENCODING)
            .is_none()
    );
    assert!(
        response
            .headers()
            .get(&HeaderName::CONTENT_ENCODING)
            .is_none()
    );

    Ok(response)
}
