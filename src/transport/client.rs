//! The backend-neutral HTTP surface the endpoint modules are written against. It keeps the
//! shape of the slice of reqwest they used (`header`, `json`, `query`, `send`, `status`,
//! `text`, …) so the backend is chosen by cargo feature, not at each call site.

use bytes::Bytes;
#[cfg(feature = "reqwest")]
use futures_util::StreamExt;
use futures_util::{TryStreamExt, stream::BoxStream};
use http::{
    HeaderMap, HeaderName, HeaderValue, Method, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::Serialize;
use url::Url;

use super::multipart;
use crate::error::HttpRequestError;

/// The HTTP backend a client sends through.
#[derive(Clone, Debug)]
pub(crate) enum HttpClient {
    #[cfg(feature = "reqwest")]
    Reqwest(reqwest::Client),
    /// The host's `wasi:http/client`, which holds no client state of its own.
    #[cfg(all(feature = "wasip3", target_os = "wasi"))]
    Wasip3,
}

impl HttpClient {
    pub(crate) fn request(&self, method: Method, url: &str) -> RequestBuilder {
        let (url, error) = match Url::parse(url) {
            Ok(url) => (Some(url), None),
            Err(error) => (None, Some(builder_error(error))),
        };
        RequestBuilder {
            client: self.clone(),
            method,
            url,
            headers: HeaderMap::new(),
            body: RequestBody::Empty,
            error,
        }
    }
}

/// What a backend sends: the request with every builder step applied.
pub(crate) struct PreparedRequest {
    pub(crate) method: Method,
    pub(crate) url: Url,
    pub(crate) headers: HeaderMap,
    pub(crate) body: RequestBody,
}

pub(crate) enum RequestBody {
    Empty,
    Bytes(Vec<u8>),
    Multipart(multipart::Form),
}

/// A request under construction. Like reqwest's, a builder step that fails is recorded and
/// reported by [`RequestBuilder::send`].
pub(crate) struct RequestBuilder {
    client: HttpClient,
    method: Method,
    url: Option<Url>,
    headers: HeaderMap,
    body: RequestBody,
    error: Option<HttpRequestError>,
}

impl std::fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestBuilder")
            .field("method", &self.method)
            .field("url", &self.url.as_ref().map(Url::as_str))
            .field("headers", &self.headers)
            .finish_non_exhaustive()
    }
}

impl RequestBuilder {
    pub(crate) fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<http::Error>,
        V: TryInto<HeaderValue>,
        V::Error: Into<http::Error>,
    {
        if self.error.is_none() {
            match (key.try_into(), value.try_into()) {
                (Ok(key), Ok(value)) => {
                    self.headers.append(key, value);
                }
                (Err(error), _) => self.error = Some(builder_error(error.into())),
                (_, Err(error)) => self.error = Some(builder_error(error.into())),
            }
        }
        self
    }

    pub(crate) fn bearer_auth(mut self, token: impl std::fmt::Display) -> Self {
        if self.error.is_none() {
            match HeaderValue::try_from(format!("Bearer {token}")) {
                Ok(mut value) => {
                    value.set_sensitive(true);
                    self.headers.append(AUTHORIZATION, value);
                }
                Err(error) => self.error = Some(builder_error(error)),
            }
        }
        self
    }

    pub(crate) fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
        if self.error.is_none() {
            match serde_json::to_vec(json) {
                Ok(body) => {
                    self.headers
                        .entry(CONTENT_TYPE)
                        .or_insert(HeaderValue::from_static("application/json"));
                    self.body = RequestBody::Bytes(body);
                }
                Err(error) => self.error = Some(builder_error(error)),
            }
        }
        self
    }

    pub(crate) fn form<T: Serialize + ?Sized>(mut self, form: &T) -> Self {
        if self.error.is_none() {
            match serde_urlencoded::to_string(form) {
                Ok(body) => {
                    self.headers
                        .entry(CONTENT_TYPE)
                        .or_insert(HeaderValue::from_static(
                            "application/x-www-form-urlencoded",
                        ));
                    self.body = RequestBody::Bytes(body.into_bytes());
                }
                Err(error) => self.error = Some(builder_error(error)),
            }
        }
        self
    }

    pub(crate) fn query<T: Serialize + ?Sized>(mut self, query: &T) -> Self {
        if self.error.is_some() {
            return self;
        }
        if let Some(url) = &mut self.url {
            let result = {
                let mut pairs = url.query_pairs_mut();
                query
                    .serialize(serde_urlencoded::Serializer::new(&mut pairs))
                    .map(|_| ())
            };
            if url.query() == Some("") {
                url.set_query(None);
            }
            if let Err(error) = result {
                self.error = Some(builder_error(error));
            }
        }
        self
    }

    pub(crate) fn multipart(mut self, form: multipart::Form) -> Self {
        self.body = RequestBody::Multipart(form);
        self
    }

    pub(crate) async fn send(self) -> Result<Response, HttpRequestError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let Some(url) = self.url else {
            return Err(HttpRequestError::new("builder error: missing URL"));
        };
        let request = PreparedRequest {
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
        };
        match self.client {
            #[cfg(feature = "reqwest")]
            HttpClient::Reqwest(client) => super::reqwest_backend::send(&client, request).await,
            #[cfg(all(feature = "wasip3", target_os = "wasi"))]
            HttpClient::Wasip3 => super::wasip3_backend::send(request).await,
        }
    }

    #[cfg(test)]
    pub(crate) fn build(self) -> Result<PreparedRequest, HttpRequestError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(PreparedRequest {
            method: self.method,
            url: self
                .url
                .ok_or_else(|| HttpRequestError::new("builder error: missing URL"))?,
            headers: self.headers,
            body: self.body,
        })
    }
}

#[cfg(test)]
impl std::fmt::Debug for PreparedRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedRequest")
            .field("method", &self.method)
            .field("url", &self.url.as_str())
            .field("headers", &self.headers)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
impl PreparedRequest {
    pub(crate) fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

/// A response whose status and headers have arrived; the body is read on demand.
pub(crate) enum Response {
    #[cfg(feature = "reqwest")]
    Reqwest(reqwest::Response),
    #[cfg(all(feature = "wasip3", target_os = "wasi"))]
    Wasip3 {
        status: StatusCode,
        headers: HeaderMap,
        body: BoxStream<'static, Result<Bytes, HttpRequestError>>,
    },
}

impl Response {
    #[cfg(all(feature = "wasip3", target_os = "wasi"))]
    pub(crate) fn new(
        status: StatusCode,
        headers: HeaderMap,
        body: BoxStream<'static, Result<Bytes, HttpRequestError>>,
    ) -> Self {
        Self::Wasip3 {
            status,
            headers,
            body,
        }
    }

    pub(crate) fn status(&self) -> StatusCode {
        match self {
            #[cfg(feature = "reqwest")]
            Self::Reqwest(response) => response.status(),
            #[cfg(all(feature = "wasip3", target_os = "wasi"))]
            Self::Wasip3 { status, .. } => *status,
        }
    }

    pub(crate) fn headers(&self) -> &HeaderMap {
        match self {
            #[cfg(feature = "reqwest")]
            Self::Reqwest(response) => response.headers(),
            #[cfg(all(feature = "wasip3", target_os = "wasi"))]
            Self::Wasip3 { headers, .. } => headers,
        }
    }

    pub(crate) async fn bytes(self) -> Result<Bytes, HttpRequestError> {
        match self {
            #[cfg(feature = "reqwest")]
            Self::Reqwest(response) => response.bytes().await.map_err(super::reqwest_error),
            #[cfg(all(feature = "wasip3", target_os = "wasi"))]
            Self::Wasip3 { body, .. } => {
                let mut chunks: Vec<Bytes> = body.try_collect().await?;
                if chunks.len() <= 1 {
                    return Ok(chunks.pop().unwrap_or_default());
                }
                let mut body = Vec::with_capacity(chunks.iter().map(Bytes::len).sum());
                for chunk in &chunks {
                    body.extend_from_slice(chunk);
                }
                Ok(Bytes::from(body))
            }
        }
    }

    pub(crate) async fn text(self) -> Result<String, HttpRequestError> {
        match self {
            // Preserve downstream-enabled reqwest charset/BOM decoding.
            #[cfg(feature = "reqwest")]
            Self::Reqwest(response) => response.text().await.map_err(super::reqwest_error),
            #[cfg(all(feature = "wasip3", target_os = "wasi"))]
            response @ Self::Wasip3 { .. } => {
                let bytes = response.bytes().await?;
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            }
        }
    }

    pub(crate) fn bytes_stream(self) -> BoxStream<'static, Result<Bytes, HttpRequestError>> {
        match self {
            #[cfg(feature = "reqwest")]
            Self::Reqwest(response) => response
                .bytes_stream()
                .map_err(super::reqwest_error)
                .boxed(),
            #[cfg(all(feature = "wasip3", target_os = "wasi"))]
            Self::Wasip3 { body, .. } => body,
        }
    }
}

pub(crate) fn builder_error(error: impl std::fmt::Display) -> HttpRequestError {
    HttpRequestError::new(format!("builder error: {error}"))
}

#[cfg(test)]
mod tests {
    use http::{Method, header::CONTENT_TYPE};
    use serde::Serialize;

    use super::RequestBody;

    #[derive(Serialize)]
    struct Page<'a> {
        limit: Option<u32>,
        cursor: Option<&'a str>,
    }

    fn get(url: &str) -> super::RequestBuilder {
        crate::transport::new_client()
            .expect("default client should build")
            .request(Method::GET, url)
    }

    #[cfg(feature = "reqwest")]
    #[tokio::test]
    async fn test_native_text_preserves_reqwest_decoding() {
        for (content_type, body) in [
            ("text/plain; charset=windows-1252", vec![0xe9]),
            ("application/json", b"\xef\xbb\xbf{}".to_vec()),
        ] {
            let response = || {
                reqwest::Response::from(
                    http::Response::builder()
                        .header(CONTENT_TYPE, content_type)
                        .body(body.clone())
                        .unwrap(),
                )
            };
            let expected = response().text().await.unwrap();
            let actual = super::Response::Reqwest(response()).text().await.unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_query_appends_pairs_and_skips_none() {
        let request = get("http://example.com/files?a=1")
            .query(&Page {
                limit: Some(5),
                cursor: None,
            })
            .query(&[("after", "x y")])
            .build()
            .expect("request should build");

        assert_eq!(
            request.url.as_str(),
            "http://example.com/files?a=1&limit=5&after=x+y"
        );
    }

    #[test]
    fn test_empty_query_leaves_no_question_mark() {
        let request = get("http://example.com/files")
            .query(&Page {
                limit: None,
                cursor: None,
            })
            .build()
            .expect("request should build");

        assert_eq!(request.url.as_str(), "http://example.com/files");
    }

    #[test]
    fn test_form_sets_body_and_content_type() {
        let request = get("http://example.com/oauth/token")
            .form(&[("code", "a&b"), ("grant_type", "authorization_code")])
            .build()
            .expect("request should build");

        assert_eq!(
            request.headers()[CONTENT_TYPE],
            "application/x-www-form-urlencoded"
        );
        let RequestBody::Bytes(body) = request.body else {
            panic!("form should set a byte body");
        };
        assert_eq!(body, b"code=a%26b&grant_type=authorization_code");
    }

    #[test]
    fn test_json_keeps_an_explicit_content_type() {
        let request = get("http://example.com/x")
            .header(CONTENT_TYPE, "application/vnd.test+json")
            .json(&serde_json::json!({ "a": 1 }))
            .build()
            .expect("request should build");

        assert_eq!(request.headers()[CONTENT_TYPE], "application/vnd.test+json");
    }

    #[test]
    fn test_builder_errors_are_reported_at_build() {
        let error = get("http://example.com/x")
            .header("X-Bad", "line\nbreak")
            .build()
            .expect_err("an invalid header value should fail");
        assert!(error.message().starts_with("builder error"), "{error}");

        let error = get("not a url")
            .build()
            .expect_err("an invalid url should fail");
        assert!(error.message().starts_with("builder error"), "{error}");
    }
}
