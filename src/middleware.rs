use axum::{
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

/// Keep snippet data and credentials out of shared and browser response caches.
pub(crate) async fn response_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}
