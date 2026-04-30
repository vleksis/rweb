use rweb::http::header_end;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

pub async fn serve_requests(
    listener: TcpListener,
    expected: usize,
    respond: impl Fn(&str) -> Vec<u8> + Send + 'static,
) -> Vec<String> {
    let mut requests = Vec::new();

    for _ in 0..expected {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await.unwrap();
        let response = respond(&request);

        stream.write_all(&response).await.unwrap();
        requests.push(request);
    }

    requests
}

pub fn request_path(request: &str) -> &str {
    request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap()
}

pub fn response(status: &str, headers: &[(&str, &str)], body: &str) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );

    for (name, value) in headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }

    response.push_str("\r\n");
    response.push_str(body);
    response.into_bytes()
}

async fn read_request(stream: &mut TcpStream) -> anyhow::Result<String> {
    let mut raw = Vec::new();
    let mut buf = [0; 1024];

    loop {
        let read = stream.read(&mut buf).await?;
        if read == 0 {
            anyhow::bail!("connection closed before request completed");
        }

        raw.extend_from_slice(&buf[..read]);

        if header_end(&raw).is_some() {
            return Ok(String::from_utf8(raw)?);
        }
    }
}
