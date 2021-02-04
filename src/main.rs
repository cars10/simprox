use warp::hyper;
use warp::{
    http::{method::Method, HeaderMap, Response},
    hyper::body::Bytes,
    path::FullPath,
    Filter,
};

fn log_request(method: &Method, path: &FullPath) {
    println!("[{}] {}", method.as_str(), path.as_str())
}

fn build_request(
    method: &Method,
    path: &FullPath,
    headers: &HeaderMap,
    body: Bytes,
) -> hyper::Request<hyper::body::Body> {
    let location = format!("http://localhost:9200{}", path.as_str());

    let mut request = hyper::Request::builder()
        .method(method.as_str())
        .uri(location);

    for (key, value) in headers {
        request = request.header(key, value);
    }

    request.body(hyper::Body::from(body)).unwrap()
}

async fn proxy_request(
    method: Method,
    path: FullPath,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    log_request(&method, &path);

    let client = hyper::Client::new();
    let request = build_request(&method, &path, &headers, body);
    let response = client.request(request).await.unwrap();
    let response_status = response.status();
    let response_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_text = String::from_utf8(response_body.to_vec()).unwrap();

    Ok(Box::new(
        Response::builder().status(response_status).body(response_text).unwrap(),
    ))
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
