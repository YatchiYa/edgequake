//! Reverse-proxy-safe SSE responses.
//!
//! Gzip and nginx buffering coalesce `text/event-stream` into one chunk at EOF
//! ([tower-http #420](https://github.com/tower-rs/tower-http/issues/420)).
//! Attach `Cache-Control: no-cache` and `X-Accel-Buffering: no` on every SSE
//! body so proxies that honor those headers cannot hide the same bug.

use std::convert::Infallible;
use std::time::Duration;

use axum::http::{header, HeaderName, HeaderValue};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures::Stream;

/// Nginx: disable response buffering for this stream (`proxy_buffering` analogue).
pub const X_ACCEL_BUFFERING: HeaderName = HeaderName::from_static("x-accel-buffering");

/// Stamp SSE proxy headers on an already-built response (MCP keep-alive text differs).
pub fn attach_sse_proxy_headers(response: &mut Response) {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(X_ACCEL_BUFFERING, HeaderValue::from_static("no"));
}

/// SSE body with 15s keep-alive plus reverse-proxy no-buffer headers.
pub fn live_sse<S>(stream: S) -> Response
where
    S: Stream<Item = Result<Event, Infallible>> + Send + 'static,
{
    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    );
    let mut response = sse.into_response();
    attach_sse_proxy_headers(&mut response);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[tokio::test]
    async fn live_sse_sets_proxy_headers() {
        let stream = stream::once(async { Ok(Event::default().data("{}")) });
        let response = live_sse(stream);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );
        assert_eq!(response.headers().get(X_ACCEL_BUFFERING).unwrap(), "no");
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("text/event-stream"),
            "content-type={content_type}"
        );
    }
}
