use reqwest::{Client, Method, RequestBuilder};

use crate::types::OpenRouterExperimentalMetadata;

pub(crate) fn request(client: &Client, method: Method, url: &str) -> RequestBuilder {
    client.request(method, url)
}

pub(crate) fn get(client: &Client, url: &str) -> RequestBuilder {
    request(client, Method::GET, url)
}

pub(crate) fn post(client: &Client, url: &str) -> RequestBuilder {
    request(client, Method::POST, url)
}

pub(crate) fn put(client: &Client, url: &str) -> RequestBuilder {
    request(client, Method::PUT, url)
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
    app_categories: &Option<Vec<String>>,
) -> Result<RequestBuilder, crate::error::OpenRouterError> {
    if let Some(x_title) = x_title {
        req = req.header("X-OpenRouter-Title", x_title);
        req = req.header("X-Title", x_title);
    }
    if let Some(http_referer) = http_referer {
        req = req.header("HTTP-Referer", http_referer);
    }
    if let Some(app_categories) = app_categories {
        req = req.header(
            "X-OpenRouter-Categories",
            serialize_app_categories(app_categories)?,
        );
    }

    Ok(req)
}

pub(crate) fn with_client_request_headers(
    req: RequestBuilder,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
) -> Result<RequestBuilder, crate::error::OpenRouterError> {
    with_request_metadata(
        with_bearer_auth(req, api_key),
        x_title,
        http_referer,
        app_categories,
    )
}

pub(crate) fn with_experimental_metadata_header(
    req: RequestBuilder,
    experimental_metadata: &Option<OpenRouterExperimentalMetadata>,
) -> RequestBuilder {
    if let Some(level) = experimental_metadata {
        req.header("X-OpenRouter-Metadata", level.as_header_value())
    } else {
        req
    }
}

fn serialize_app_categories(
    app_categories: &[String],
) -> Result<String, crate::error::OpenRouterError> {
    if app_categories.is_empty() {
        return Err(crate::error::OpenRouterError::ConfigError(
            "app_categories cannot be empty when provided".to_string(),
        ));
    }
    if app_categories.len() > 2 {
        return Err(crate::error::OpenRouterError::ConfigError(
            "app_categories supports at most 2 categories per request".to_string(),
        ));
    }

    let mut serialized = Vec::with_capacity(app_categories.len());
    for category in app_categories {
        let trimmed = category.trim();
        if trimmed.is_empty() {
            return Err(crate::error::OpenRouterError::ConfigError(
                "app_categories cannot contain empty values".to_string(),
            ));
        }
        if trimmed.len() > 30 {
            return Err(crate::error::OpenRouterError::ConfigError(format!(
                "app category `{trimmed}` exceeds the 30 character OpenRouter limit"
            )));
        }
        if !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(crate::error::OpenRouterError::ConfigError(format!(
                "app category `{trimmed}` must be lowercase and hyphen-separated"
            )));
        }
        serialized.push(trimmed.to_string());
    }

    Ok(serialized.join(","))
}

#[cfg(test)]
mod tests {
    use reqwest::header::AUTHORIZATION;

    use crate::types::OpenRouterExperimentalMetadata;

    use super::{post, with_client_request_headers, with_experimental_metadata_header};

    #[test]
    fn test_with_client_request_headers_sets_auth_and_metadata() {
        let client = reqwest::Client::new();
        let request = with_client_request_headers(
            post(&client, "http://example.com/test"),
            "test-key",
            &Some("openrouter-rs-tests".to_string()),
            &Some("https://example.com".to_string()),
            &Some(vec!["cli-agent".to_string(), "cloud-agent".to_string()]),
        )
        .expect("request builder should be created")
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
        assert_eq!(
            request
                .headers()
                .get("X-OpenRouter-Categories")
                .expect("x-openrouter-categories should exist"),
            "cli-agent,cloud-agent"
        );
    }

    #[test]
    fn test_with_client_request_headers_rejects_invalid_app_categories() {
        let client = reqwest::Client::new();
        let error = with_client_request_headers(
            post(&client, "http://example.com/test"),
            "test-key",
            &Some("openrouter-rs-tests".to_string()),
            &Some("https://example.com".to_string()),
            &Some(vec!["CLI-Agent".to_string()]),
        )
        .expect_err("invalid categories should fail");

        assert!(
            matches!(error, crate::error::OpenRouterError::ConfigError(_)),
            "expected config error, got {error:?}"
        );
    }

    #[test]
    fn test_with_experimental_metadata_header_sets_level() {
        let client = reqwest::Client::new();
        let request = with_experimental_metadata_header(
            post(&client, "http://example.com/test"),
            &Some(OpenRouterExperimentalMetadata::Enabled),
        )
        .build()
        .expect("request should build");

        assert_eq!(
            request
                .headers()
                .get("X-OpenRouter-Metadata")
                .expect("experimental metadata header should exist"),
            "enabled"
        );
    }
}
