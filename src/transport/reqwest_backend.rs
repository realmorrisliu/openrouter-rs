//! Sends through a `reqwest::Client` (feature `reqwest`).

use futures_util::{StreamExt, TryStreamExt};

use super::client::{PreparedRequest, RequestBody, Response};
use crate::error::HttpRequestError;

pub(super) async fn send(
    client: &reqwest::Client,
    request: PreparedRequest,
) -> Result<Response, HttpRequestError> {
    let mut builder = client
        .request(request.method, request.url)
        .headers(request.headers);
    builder = match request.body {
        RequestBody::Empty => builder,
        RequestBody::Bytes(body) => builder.body(body),
        RequestBody::Multipart(form) => builder.multipart(into_reqwest_form(form)?),
    };
    let response = builder.send().await.map_err(transport_error)?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes_stream().map_err(transport_error).boxed();
    Ok(Response::new(status, headers, body))
}

fn into_reqwest_form(
    form: super::multipart::Form,
) -> Result<reqwest::multipart::Form, HttpRequestError> {
    let mut reqwest_form = reqwest::multipart::Form::new();
    for (name, part) in form.into_parts() {
        let mut reqwest_part = reqwest::multipart::Part::bytes(part.bytes);
        if let Some(file_name) = part.file_name {
            reqwest_part = reqwest_part.file_name(file_name);
        }
        if let Some(mime) = part.mime {
            reqwest_part = reqwest_part
                .mime_str(mime.as_ref())
                .map_err(transport_error)?;
        }
        reqwest_form = reqwest_form.part(name, reqwest_part);
    }
    Ok(reqwest_form)
}

pub(crate) fn transport_error(error: reqwest::Error) -> HttpRequestError {
    HttpRequestError::new(error.to_string())
}
