#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::SecondsFormat;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::{Bytes, Incoming};
use hyper::header::{
    ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_HEADERS, HOST, HeaderValue, VIA,
};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, Uri};
use hyper_rustls::{ConfigBuilderExt, HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as ServerBuilder;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, ring, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error as TlsError, SignatureScheme};
use tokio::net::TcpListener;

mod args;

type BoxError = Box<dyn Error + Send + Sync>;
type HttpsClient = Client<HttpsConnector<HttpConnector>, Incoming>;
type ProxyBody = BoxBody<Bytes, hyper::Error>;

// Accepts any certificate but still checks signatures, so the handshake itself stays sound.
#[derive(Debug)]
struct NoCertificateVerification(CryptoProvider);

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

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
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn https_client(skip_ssl_verify: bool) -> Result<HttpsClient, BoxError> {
    let builder = ClientConfig::builder_with_provider(Arc::new(ring::default_provider()))
        .with_safe_default_protocol_versions()?;
    let tls = if skip_ssl_verify {
        builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification(
                ring::default_provider(),
            )))
            .with_no_client_auth()
    } else {
        builder.with_native_roots()?.with_no_client_auth()
    };

    let https = HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .build();
    Ok(Client::builder(TokioExecutor::new()).build(https))
}

fn text_response(status: StatusCode, text: &'static str) -> Response<ProxyBody> {
    let mut resp = Response::new(Full::new(Bytes::from(text)).map_err(|n| match n {}).boxed());
    *resp.status_mut() = status;
    resp
}

fn preflight_response(req: &Request<Incoming>) -> Response<ProxyBody> {
    let mut resp = Response::new(Empty::new().map_err(|n| match n {}).boxed());
    *resp.status_mut() = StatusCode::NO_CONTENT;
    let headers = resp.headers_mut();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS"),
    );
    // Echo the requested headers: a "*" wildcard does not cover Authorization.
    let allow_headers = req
        .headers()
        .get(ACCESS_CONTROL_REQUEST_HEADERS)
        .cloned()
        .unwrap_or(HeaderValue::from_static("*"));
    headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, allow_headers);
    headers.insert(ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("86400"));
    resp
}

async fn proxy_request(
    mut req: Request<Incoming>,
    client: HttpsClient,
    target_host: Arc<String>,
    skip_cors: bool,
) -> Result<Response<ProxyBody>, Infallible> {
    let path_and_query = req.uri().path_and_query().map_or("/", |p| p.as_str());
    info!("[{}] {}", req.method(), path_and_query);

    if skip_cors && req.method() == Method::OPTIONS {
        return Ok(preflight_response(&req));
    }

    let uri: Uri = match format!("{}{}", target_host, path_and_query).parse() {
        Ok(uri) => uri,
        Err(e) => {
            error!("Invalid target uri: {:?}", e);
            return Ok(text_response(StatusCode::BAD_GATEWAY, "invalid target uri"));
        }
    };
    *req.uri_mut() = uri;
    // Drop our own Host so the client sets the target's; virtual hosts need it.
    req.headers_mut().remove(HOST);
    req.headers_mut()
        .append(VIA, HeaderValue::from_static("1.1 simprox"));

    match client.request(req).await {
        Ok(resp) => {
            info!(" => {}", resp.status());
            let mut resp = resp.map(|b| b.boxed());
            let headers = resp.headers_mut();
            if skip_cors || !headers.contains_key(ACCESS_CONTROL_ALLOW_ORIGIN) {
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
            }
            Ok(resp)
        }
        Err(e) => {
            error!("FAILED: proxy target unavailable");
            error!("{:?}", e);
            Ok(text_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "proxy target unavailable",
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    setup_logger().expect("Error setting up logger");

    ctrlc::set_handler(|| {
        info!("Stopping simprox...");
        std::process::exit(0);
    })
    .expect("Error setting exit handler");

    let config = args::Config::build();
    let addr: SocketAddr = config.listen.parse().unwrap_or_else(|_| {
        error!("Invalid listen host: {}", config.listen);
        std::process::exit(-1)
    });

    info!("Listening on: {}", addr);
    info!("Proxy target: {}", config.target_host);
    info!("Skip ssl verify: {}", config.skip_ssl_verify);
    info!("Skip cors: {}", config.skip_cors);
    info!("");

    let client = https_client(config.skip_ssl_verify)?;
    let target_host = Arc::new(config.target_host);
    let skip_cors = config.skip_cors;
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let client = client.clone();
        let target_host = target_host.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                proxy_request(req, client.clone(), target_host.clone(), skip_cors)
            });
            if let Err(err) = ServerBuilder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(stream), service)
                .await
            {
                error!("server connection error: {}", err);
            }
        });
    }
}
