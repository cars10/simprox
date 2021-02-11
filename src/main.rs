#[macro_use]
extern crate lazy_static;

use hyper_tls::HttpsConnector;
use warp::hyper::{
    body::{Body, Bytes},
    client::connect::HttpConnector,
    http::StatusCode,
    Client, Request,
};
use warp::{
    http::{method::Method, HeaderMap, Response},
    path::FullPath,
    Filter, Rejection,
};

mod args;

lazy_static! {
    static ref STATIC_CLIENT: Client<HttpsConnector<HttpConnector>, Body> = https_client();
}

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

fn log_start_request(method: &Method, path: &FullPath) {
    let method_str = format!("[{}]", method.as_str());
    println!("{:6} {}", method_str, path.as_str())
}

fn log_done_request(status: StatusCode) {
    println!(" => {}", status)
}

fn log_error_request() {
    println!(" FAILED: proxy server unavailable")
}

async fn proxy_request(
    method: Method,
    path: FullPath,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, Rejection> {
    log_start_request(&method, &path);

    let request = build_request(method, path, headers, body);

    if let Ok(proxy_response) = STATIC_CLIENT.request(request).await {
        let proxy_status = proxy_response.status();
        let proxy_headers = proxy_response.headers().clone();
        let proxy_body = proxy_response.into_body();

        let mut response = Response::new(proxy_body);
        *response.status_mut() = proxy_status;
        *response.headers_mut() = proxy_headers;
        log_done_request(proxy_status);

        Ok(response)
    } else {
        log_error_request();
        Ok(Response::builder()
            .status(503)
            .body("proxy server unavailable".into())
            .unwrap())
    }
}

fn build_request(method: Method, path: FullPath, headers: HeaderMap, body: Bytes) -> Request<Body> {
    let location = format!("https://localhost{}", path.as_str());

    let mut request = Request::new(Body::from(body));
    *request.method_mut() = method;
    *request.uri_mut() = location.parse().unwrap();
    *request.headers_mut() = headers;
    request
}

#[tokio::main]
async fn main() {
    let config = args::Config::build();
    println!("Host: {}", config.host);
    println!("Use ssl: {}", config.use_ssl);
    println!("Skip ssl verify: {}", config.skip_ssl_verify);

    let routes = warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(proxy_request);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
