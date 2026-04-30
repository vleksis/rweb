use rweb::loader::Client;
use rweb::loader::Url;
use tokio::net::TcpListener;

mod support;

use support::request_path;
use support::response;
use support::serve_requests;

#[tokio::test]
async fn follows_same_origin_redirect() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_requests(listener, 2, |request| {
        match request_path(request) {
            "/start" => response("302 Found", &[("Location", "/final")], ""),
            "/final" => response("200 OK", &[], "done"),
            path => panic!("unexpected path: {path}"),
        }
    }));
    let url: Url = format!("http://{addr}/start").parse().unwrap();
    let mut client = Client::builder().build();

    let response = client.load_url(&url).await.unwrap();
    let requests = server.await.unwrap();

    assert_eq!(response.body_as_str().unwrap(), "done");
    assert!(requests[0].starts_with("GET /start HTTP/1.1\r\n"));
    assert!(requests[1].starts_with("GET /final HTTP/1.1\r\n"));
}

#[tokio::test]
async fn rejects_redirect_without_location() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_requests(listener, 1, |_| {
        response("302 Found", &[], "")
    }));
    let url: Url = format!("http://{addr}/start").parse().unwrap();
    let mut client = Client::builder().build();

    let err = client.load_url(&url).await.unwrap_err();
    server.await.unwrap();

    assert_eq!(err.to_string(), "no Location with redirect response");
}

#[tokio::test]
async fn stops_after_max_redirects() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_requests(listener, 2, |_| {
        response("302 Found", &[("Location", "/loop")], "")
    }));
    let url: Url = format!("http://{addr}/loop").parse().unwrap();
    let mut client = Client::builder().max_redirects(1).build();

    let err = client.load_url(&url).await.unwrap_err();
    let requests = server.await.unwrap();

    assert_eq!(err.to_string(), "Too many redirects");
    assert_eq!(requests.len(), 2);
}

#[tokio::test]
async fn max_redirects_zero_rejects_first_redirect() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_requests(listener, 1, |_| {
        response("302 Found", &[("Location", "/final")], "")
    }));
    let url: Url = format!("http://{addr}/start").parse().unwrap();
    let mut client = Client::builder().max_redirects(0).build();

    let err = client.load_url(&url).await.unwrap_err();
    let requests = server.await.unwrap();

    assert_eq!(err.to_string(), "Too many redirects");
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn rejects_redirect_to_unsupported_scheme() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_requests(listener, 1, |_| {
        response("302 Found", &[("Location", "data:text/html,Hello")], "")
    }));
    let url: Url = format!("http://{addr}/start").parse().unwrap();
    let mut client = Client::builder().build();

    let err = client.load_url(&url).await.unwrap_err();
    server.await.unwrap();

    assert_eq!(
        err.to_string(),
        "unsupported redirect location: data:text/html,Hello"
    );
}

#[tokio::test]
async fn rebuilds_host_header_after_absolute_redirect() {
    let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_addr = target_listener.local_addr().unwrap();
    let target_server = tokio::spawn(serve_requests(target_listener, 1, |_| {
        response("200 OK", &[], "target")
    }));

    let origin_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_addr = origin_listener.local_addr().unwrap();
    let origin_server = tokio::spawn(serve_requests(origin_listener, 1, move |_| {
        let location = format!("http://{target_addr}/final");
        response("302 Found", &[("Location", &location)], "")
    }));

    let url: Url = format!("http://{origin_addr}/start").parse().unwrap();
    let mut client = Client::builder().build();

    let response = client.load_url(&url).await.unwrap();
    let origin_requests = origin_server.await.unwrap();
    let target_requests = target_server.await.unwrap();

    assert_eq!(response.body_as_str().unwrap(), "target");
    assert!(origin_requests[0].contains(&format!("host: {origin_addr}\r\n")));
    assert!(target_requests[0].contains(&format!("host: {target_addr}\r\n")));
}
