#![allow(dead_code)]

use reqwest::{Client, Method, RequestBuilder};

pub(crate) fn request(client: &Client, method: Method, url: &str) -> RequestBuilder {
    client.request(method, url)
}

pub(crate) fn get(client: &Client, url: &str) -> RequestBuilder {
    request(client, Method::GET, url)
}

pub(crate) fn post(client: &Client, url: &str) -> RequestBuilder {
    request(client, Method::POST, url)
}

pub(crate) fn patch(client: &Client, url: &str) -> RequestBuilder {
    request(client, Method::PATCH, url)
}

pub(crate) fn delete(client: &Client, url: &str) -> RequestBuilder {
    request(client, Method::DELETE, url)
}

pub(crate) fn with_bearer_auth(req: RequestBuilder, api_key: &str) -> RequestBuilder {
    req.bearer_auth(api_key)
}

pub(crate) fn with_request_metadata(
    mut req: RequestBuilder,
    x_title: &Option<String>,
    http_referer: &Option<String>,
) -> RequestBuilder {
    if let Some(x_title) = x_title {
        req = req.header("X-OpenRouter-Title", x_title);
        req = req.header("X-Title", x_title);
    }
    if let Some(http_referer) = http_referer {
        req = req.header("HTTP-Referer", http_referer);
    }

    req
}

pub(crate) fn with_client_request_headers(
    req: RequestBuilder,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
) -> RequestBuilder {
    with_request_metadata(with_bearer_auth(req, api_key), x_title, http_referer)
}

#[cfg(test)]
mod tests {
    use reqwest::header::AUTHORIZATION;

    use super::{post, with_client_request_headers};

    #[test]
    fn test_with_client_request_headers_sets_auth_and_metadata() {
        let client = reqwest::Client::new();
        let request = with_client_request_headers(
            post(&client, "http://example.com/test"),
            "test-key",
            &Some("openrouter-rs-tests".to_string()),
            &Some("https://example.com".to_string()),
        )
        .build()
        .expect("request should build");

        assert_eq!(
            request
                .headers()
                .get(AUTHORIZATION)
                .expect("authorization header should exist"),
            "Bearer test-key"
        );
        assert_eq!(
            request
                .headers()
                .get("X-OpenRouter-Title")
                .expect("x-openrouter-title should exist"),
            "openrouter-rs-tests"
        );
        assert_eq!(
            request
                .headers()
                .get("X-Title")
                .expect("x-title should exist"),
            "openrouter-rs-tests"
        );
        assert_eq!(
            request
                .headers()
                .get("HTTP-Referer")
                .expect("http-referer should exist"),
            "https://example.com"
        );
    }
}
