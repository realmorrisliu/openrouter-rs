use crate::{api::errors::ApiErrorResponse, error::OpenRouterError};
use surf::Response;

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

pub async fn handle_error(mut response: Response) -> Result<(), OpenRouterError> {
    let status = response.status();
    let text = response
        .body_string()
        .await
        .unwrap_or_else(|_| "Failed to read response text".to_string());
    let api_error_response: Result<ApiErrorResponse, _> = serde_json::from_str(&text);

    if let Ok(api_error_response) = api_error_response {
        Err(OpenRouterError::from(api_error_response))
    } else {
        Err(OpenRouterError::ApiError {
            code: status,
            message: text,
        })
    }
}
