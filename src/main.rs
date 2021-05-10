#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
use chrono::SecondsFormat;
use hyper_tls::{native_tls::TlsConnector, HttpsConnector};
use std::convert::Infallible;
use std::error::Error;
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

pub fn setup_logger() -> Result<(), Box<dyn Error>> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {} -- {}",
                chrono::Local::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("warp", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

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

async fn proxy_request(
    original_request: OriginalRequest,
    client: Arc<HttpsClient>,
    target_host: Arc<String>,
) -> Result<Response<Body>, Rejection> {
    info!(
        "[{}] {}{}",
        &original_request.method.as_str(),
        &original_request.path.as_str(),
        &original_request.query_string()
    );

    let request = build_request(original_request, target_host);

    match client.request(request).await {
        Ok(proxy_response) => {
            let proxy_status = proxy_response.status();
            let proxy_headers = proxy_response.headers().clone();
            let proxy_body = proxy_response.into_body();

            let mut response = Response::new(proxy_body);
            *response.status_mut() = proxy_status;
            *response.headers_mut() = proxy_headers;
            info!(" => {}", proxy_status);

            Ok(response)
        }
        Err(e) => {
            error!("FAILED: proxy server unavailable");
            error!("{:?}", e);
            Ok(Response::builder()
                .status(503)
                .body("proxy target unavailable".into())
                .unwrap())
        }
    }
}

fn build_request(original_request: OriginalRequest, target_host: Arc<String>) -> Request<Body> {
    let location = format!(
        "{}{}{}",
        target_host,
        original_request.path.as_str(),
        original_request.query_string()
    );

    let mut request = Request::new(Body::from(original_request.body));
    *request.method_mut() = original_request.method;
    *request.uri_mut() = location.parse().expect("invalid uri");
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

fn with_raw_query() -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::filters::query::raw()
        .or(warp::any().map(String::default))
        .unify()
}

struct OriginalRequest {
    method: Method,
    path: FullPath,
    query: String,
    headers: HeaderMap,
    body: Bytes,
}

impl OriginalRequest {
    fn new(method: Method, path: FullPath, query: String, headers: HeaderMap, body: Bytes) -> Self {
        OriginalRequest {
            method,
            path,
            query,
            headers,
            body,
        }
    }

    fn query_string(&self) -> String {
        if self.query.is_empty() {
            String::default()
        } else {
            format!("?{}", self.query)
        }
    }
}

#[tokio::main]
async fn main() {
    setup_logger().expect("Error setting up logger");

    ctrlc::set_handler(|| {
        info!("Stopping simprox...");
        std::process::exit(0);
    })
    .expect("Error setting exit handler");

    let config = args::Config::build();

    let addr: std::net::SocketAddr = config.listen_host.parse().unwrap_or_else(|_| {
        error!("Invalid listen host: {}", config.listen_host);
        std::process::exit(-1)
    });

    info!("Listening on: {}", addr);
    info!("Proxy target: {}", config.target_host);
    info!("Skip ssl verify: {}", config.skip_ssl_verify);
    info!("");

    let client = Arc::new(https_client(config.skip_ssl_verify));
    let target_host = Arc::new(config.target_host);

    let routes = warp::method()
        .and(warp::path::full())
        .and(with_raw_query())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .map(OriginalRequest::new)
        .and(with_client(client))
        .and(with_target_host(target_host))
        .and_then(proxy_request)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(addr).await;
}
