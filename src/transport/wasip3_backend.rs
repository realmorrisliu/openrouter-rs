//! Sends through the host's `wasi:http/client` (WASI 0.3, feature `wasip3` on a WASI target).
//!
//! The component model's futures are not `Send`. A WASI component runs on one thread, so they
//! are wrapped in [`SendWrapper`], which keeps the SDK's public futures and streams `Send`
//! without a separate non-`Send` API.

use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt};
use http::header::CONTENT_TYPE;
use http_body_util::{BodyDataStream, Full};
use send_wrapper::SendWrapper;

use super::client::{PreparedRequest, RequestBody, Response};
use crate::error::HttpRequestError;

pub(super) async fn send(request: PreparedRequest) -> Result<Response, HttpRequestError> {
    SendWrapper::new(send_local(request)).await
}

async fn send_local(request: PreparedRequest) -> Result<Response, HttpRequestError> {
    let PreparedRequest {
        method,
        url,
        mut headers,
        body,
    } = request;
    let body = match body {
        RequestBody::Empty => Vec::new(),
        RequestBody::Bytes(body) => body,
        RequestBody::Multipart(form) => {
            let (content_type, body) = form.encode(&boundary());
            let content_type = http::HeaderValue::try_from(content_type)
                .map_err(|error| failed("the multipart content type", &error))?;
            headers.insert(CONTENT_TYPE, content_type);
            body
        }
    };

    let mut builder = http::Request::builder().method(method).uri(url.as_str());
    if let Some(request_headers) = builder.headers_mut() {
        *request_headers = headers;
    }
    let request = builder
        .body(Full::new(Bytes::from(body)))
        .map_err(|error| failed("the request", &error))?;
    let request = wasip3::http_compat::http_into_wasi_request(request)
        .map_err(|error| failed("the request", &error))?;
    let response = wasip3::http::client::send(request)
        .await
        .map_err(|error| failed("the request was not answered", &error))?;
    let response = wasip3::http_compat::http_from_wasi_response(response)
        .map_err(|error| failed("the response", &error))?;

    let (parts, body) = response.into_parts();
    let body = SendWrapper::new(
        BodyDataStream::new(body).map_err(|error| failed("the response body", &error)),
    )
    .boxed();
    Ok(Response::new(parts.status, parts.headers, body))
}

fn boundary() -> String {
    let [a, b, c, d] = std::array::from_fn(|_| wasip3::random::random::get_random_u64());
    format!("{a:016x}-{b:016x}-{c:016x}-{d:016x}")
}

fn failed(what: &str, error: &dyn std::fmt::Debug) -> HttpRequestError {
    HttpRequestError::new(format!("{what}: {error:?}"))
}
