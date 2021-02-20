use hyper_tls::{native_tls::TlsConnector, HttpsConnector};
use log::log;
use std::convert::Infallible;
use std::sync::Arc;
use warp::hyper::{
    body::{Body, Bytes},
    client::connect::HttpConnector,
    Client, Request,
};
use warp::{
    http::{method::Method, HeaderMap, Response},
    path::FullPath,
    Filter, Rejection,
};

type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

mod args;
mod log;

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
    log(format!("{:6} {}", method_str, path.as_str()))
}

async fn proxy_request(
    original_request: OriginalRequest,
    client: Arc<HttpsClient>,
    target_host: Arc<String>,
) -> Result<Response<Body>, Rejection> {
    log_start_request(&original_request.method, &original_request.path);

    let request = build_request(original_request, target_host);

    match client.request(request).await {
        Ok(proxy_response) => {
            let proxy_status = proxy_response.status();
            let proxy_headers = proxy_response.headers().clone();
            let proxy_body = proxy_response.into_body();

            let mut response = Response::new(proxy_body);
            *response.status_mut() = proxy_status;
            *response.headers_mut() = proxy_headers;
            log(format!(" => {}", proxy_status));

            Ok(response)
        }
        Err(e) => {
            log(format!(" FAILED: proxy server unavailable"));
            log(format!(" {:?}", e));
            Ok(Response::builder()
                .status(503)
                .body("proxy server unavailable".into())
                .unwrap())
        }
    }
}

fn build_request(original_request: OriginalRequest, target_host: Arc<String>) -> Request<Body> {
    let location = format!("{}{}", target_host, original_request.path.as_str());

    let mut request = Request::new(Body::from(original_request.body));
    *request.method_mut() = original_request.method;
    *request.uri_mut() = location.parse().unwrap();
    *request.headers_mut() = original_request.headers;
    request
}

fn with_client(
    client: Arc<HttpsClient>,
) -> impl Filter<Extract = (Arc<HttpsClient>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_target_host(
    target_host: Arc<String>,
) -> impl Filter<Extract = (Arc<String>,), Error = Infallible> + Clone {
    warp::any().map(move || target_host.clone())
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
    println!("Proxy target: {}", config.target_host);
    println!("Skip ssl verify: {}", config.skip_ssl_verify);

    let client = Arc::new(https_client(config.skip_ssl_verify));
    let target_host = Arc::new(config.target_host);

    let routes = warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .map(OriginalRequest::new)
        .and(with_client(client))
        .and(with_target_host(target_host))
        .and_then(proxy_request)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
