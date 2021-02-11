use hyper_tls::{native_tls::TlsConnector, HttpsConnector};
use std::sync::Arc;
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

type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

mod args;

fn https_client(skip_ssl_verify: bool) -> HttpsClient {
    let mut tls_builder = TlsConnector::builder();
    let tls_builder = tls_builder.danger_accept_invalid_certs(skip_ssl_verify);
    let tls_builder = tls_builder.danger_accept_invalid_hostnames(skip_ssl_verify);

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
    original_request: OriginalRequest,
    client: Arc<HttpsClient>,
) -> Result<Response<Body>, Rejection> {
    log_start_request(&original_request.method, &original_request.path);

    let request = build_request(original_request);

    if let Ok(proxy_response) = client.request(request).await {
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

fn build_request(original_request: OriginalRequest) -> Request<Body> {
    let location = format!("http://localhost:9200{}", original_request.path.as_str());

    let mut request = Request::new(Body::from(original_request.body));
    *request.method_mut() = original_request.method;
    *request.uri_mut() = location.parse().unwrap();
    *request.headers_mut() = original_request.headers;
    request
}

use std::convert::Infallible;

fn with_client(
    client: Arc<HttpsClient>,
) -> impl Filter<Extract = (Arc<HttpsClient>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

struct OriginalRequest {
    method: Method,
    path: FullPath,
    headers: HeaderMap,
    body: Bytes,
}

impl OriginalRequest {
    fn new(method: Method, path: FullPath, headers: HeaderMap, body: Bytes) -> Self {
        OriginalRequest {
            method,
            path,
            headers,
            body,
        }
    }
}

#[tokio::main]
async fn main() {
    let config = args::Config::build();
    println!("Host: {}", config.host);
    println!("Skip ssl verify: {}", config.skip_ssl_verify);

    let client = Arc::new(https_client(config.skip_ssl_verify));

    let routes = warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .map(OriginalRequest::new)
        .and(with_client(client))
        .and_then(proxy_request)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
