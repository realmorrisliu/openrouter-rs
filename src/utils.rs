use crate::{api::errors::ApiErrorResponse, error::OpenRouterError};
use surf::Response;

/// A macro for generating builder-style setter methods.
///
/// This macro provides two variants:
/// 1. Direct assignment: `setter!(field_name, FieldType)`
/// 2. Automatic Into conversion: `setter!(field_name, into TargetType)`
///
/// # Examples
///
/// ```rust
/// struct Builder {
///     name: Option<String>,
///     count: Option<u32>,
/// }
///
/// impl Builder {
///     setter!(name, into String);  // Accepts any type that implements Into<String>
///     setter!(count, u32);         // Only accepts u32
///
///     pub fn build(self) -> Self {
///         self
///     }
/// }
///
/// let builder = Builder {
///     name: None,
///     count: None,
/// }
/// .name("test")    // &str automatically converted to String
/// .count(42);      // Direct u32 assignment
/// ```
#[macro_export]
macro_rules! setter {
    // Direct value assignment
    ($name:ident, $type:ty) => {
        #[doc = concat!("Sets the `", stringify!($name), "` field directly.")]
        pub fn $name(mut self, value: $type) -> Self {
            self.$name = Some(value);
            self
        }
    };

    // Automatic Into conversion
    ($name:ident, into $type:ty) => {
        #[doc = concat!("Sets the `", stringify!($name), "` field with automatic conversion using `Into<", stringify!($type), ">`.")]
        pub fn $name(mut self, value: impl Into<$type>) -> Self {
            self.$name = Some(value.into());
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
