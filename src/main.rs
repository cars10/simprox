#[macro_use]
extern crate lazy_static;

use hyper_tls::{HttpsConnector};
use warp::hyper::body::{Body, Bytes};
use warp::hyper::client::connect::HttpConnector;
use warp::hyper::{Client, Request};
use warp::{
    http::{method::Method, HeaderMap, Response},
    path::FullPath,
    Filter, Rejection,
};

fn https_client() -> Client<HttpsConnector<HttpConnector>, Body> {
    let mut tls_builder = hyper_tls::native_tls::TlsConnector::builder();
    let tls_builder = tls_builder.danger_accept_invalid_certs(true);
    let tls_builder = tls_builder.danger_accept_invalid_hostnames(true);

    let tls = tls_builder.build().unwrap();

    let mut http = HttpConnector::new();
    http.enforce_http(false);
    let https = HttpsConnector::from((http, tls.into()));

    Client::builder().build(https)
}

lazy_static! {
    static ref STATIC_CLIENT: Client<HttpsConnector<HttpConnector>, Body> = https_client();
}

fn log_start_request(method: &Method, path: &FullPath) {
    let method_str = format!("[{}]", method.as_str());
    println!("{:6} {}", method_str, path.as_str())
}

fn log_done_request(status: warp::hyper::http::StatusCode) {
    println!(" => {}", status)
}

async fn proxy_request(
    method: Method,
    path: FullPath,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, Rejection> {
    log_start_request(&method, &path);

    let request = build_request(&method, &path, &headers, body);
    let response = STATIC_CLIENT.request(request).await.unwrap();
    let response_status = response.status();
    let response_body = response.into_body();
    log_done_request(response_status);

    Ok(Response::builder()
        .status(response_status)
        .body(response_body)
        .unwrap())
}

fn build_request(
    method: &Method,
    path: &FullPath,
    headers: &HeaderMap,
    body: Bytes,
) -> Request<Body> {
    let location = format!("https://localhost{}", path.as_str());

    let mut request = Request::builder().method(method.as_str()).uri(location);

    for (key, value) in headers {
        request = request.header(key, value);
    }

    request.body(Body::from(body)).unwrap()
}

#[tokio::main]
async fn main() {
    let routes = warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(proxy_request);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
