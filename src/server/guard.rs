use super::config::ServerConfig;
use dioxus::server::axum::{
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Rejects requests for unknown host names (DNS rebinding) and cross-origin browser
/// writes (CSRF), then adds baseline security headers to every response.
///
/// Requests without an `Origin` header are allowed through: browsers always send it on
/// cross-origin writes, while the native desktop client never does.
pub async fn request_guard(
    State(config): State<Arc<ServerConfig>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request_host(&request);
    if !host.is_some_and(|host| config.host_allowed(host)) {
        return (StatusCode::MISDIRECTED_REQUEST, "Unknown host").into_response();
    }
    if !is_safe(request.method())
        && let Some(origin) = request.headers().get(header::ORIGIN)
        && !origin
            .to_str()
            .is_ok_and(|origin| config.origin_allowed(origin))
    {
        return (StatusCode::FORBIDDEN, "Cross-origin request blocked").into_response();
    }

    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );
    response
}

/// The `Host` header, or the URI authority for HTTP/2 requests.
fn request_host(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .or_else(|| {
            request
                .uri()
                .authority()
                .map(|authority| authority.as_str())
        })
}

fn is_safe(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}
