use crate::{api::errors::parse_api_error, error::OpenRouterError};
use surf::Response;
use surf::http::headers::AUTHORIZATION;

#[macro_export]
macro_rules! strip_option_vec_setter {
    ($field:ident, $item_ty:ty) => {
        pub fn $field<T, S>(&mut self, items: T) -> &mut Self
        where
            T: IntoIterator<Item = S>,
            S: Into<$item_ty>,
        {
            self.$field = Some(Some(items.into_iter().map(Into::into).collect()));
            self
        }
    };
}

#[macro_export]
macro_rules! strip_option_map_setter {
    ($field:ident, $key_ty:ty, $val_ty:ty) => {
        pub fn $field<K, V, T>(&mut self, items: T) -> &mut Self
        where
            T: IntoIterator<Item = (K, V)>,
            K: Into<$key_ty>,
            V: Into<$val_ty>,
        {
            let map: std::collections::HashMap<$key_ty, $val_ty> = items
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect();

            self.$field = Some(Some(map));
            self
        }
    };
}

pub fn with_bearer_auth(req: surf::RequestBuilder, api_key: &str) -> surf::RequestBuilder {
    req.header(AUTHORIZATION, format!("Bearer {api_key}"))
}

pub fn with_request_metadata(
    mut req: surf::RequestBuilder,
    x_title: &Option<String>,
    http_referer: &Option<String>,
) -> surf::RequestBuilder {
    if let Some(x_title) = x_title {
        req = req.header("X-OpenRouter-Title", x_title);
        req = req.header("X-Title", x_title);
    }
    if let Some(http_referer) = http_referer {
        req = req.header("HTTP-Referer", http_referer);
    }

    req
}

pub fn with_client_request_headers(
    req: surf::RequestBuilder,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
) -> surf::RequestBuilder {
    with_request_metadata(with_bearer_auth(req, api_key), x_title, http_referer)
}

pub async fn handle_error(mut response: Response) -> Result<(), OpenRouterError> {
    let status = response.status();
    let request_id = response
        .header("x-request-id")
        .and_then(|values| values.get(0))
        .map(|value| value.as_str().to_string())
        .or_else(|| {
            response
                .header("request-id")
                .and_then(|values| values.get(0))
                .map(|value| value.as_str().to_string())
        });
    let text = response
        .body_string()
        .await
        .unwrap_or_else(|_| "Failed to read response text".to_string());

    Err(parse_api_error(status, request_id, &text))
}
