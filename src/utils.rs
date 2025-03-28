use crate::{api::errors::ApiErrorResponse, error::OpenRouterError};
use surf::Response;

#[macro_export]
macro_rules! setter {
    ($name:ident, $type:ty) => {
        pub fn $name(mut self, value: $type) -> Self {
            self.$name = Some(value);
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
