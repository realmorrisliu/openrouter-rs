use std::{
    env,
    sync::Once,
    time::{Duration, Instant},
};

use openrouter_rs::{client::OpenRouterClient, error::OpenRouterError};
use tokio::time::sleep;

fn load_integration_env_file() {
    static LOAD_ENV: Once = Once::new();

    LOAD_ENV.call_once(|| {
        // Load the standard local env file used by this repo.
        let _ = dotenvy::dotenv();
    });
}

pub fn get_test_api_key() -> String {
    load_integration_env_file();

    env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable not set for integration tests")
}

pub fn get_test_management_key() -> Option<String> {
    load_integration_env_file();

    env::var("OPENROUTER_MANAGEMENT_KEY")
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

#[allow(clippy::result_large_err)]
pub fn create_test_client() -> Result<OpenRouterClient, OpenRouterError> {
    OpenRouterClient::builder()
        .api_key(get_test_api_key())
        .base_url("https://openrouter.ai/api/v1")
        .x_title("openrouter-rs")
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .build()
}

#[allow(clippy::result_large_err)]
pub fn create_management_test_client() -> Result<Option<OpenRouterClient>, OpenRouterError> {
    let Some(management_key) = get_test_management_key() else {
        return Ok(None);
    };

    OpenRouterClient::builder()
        .management_key(management_key)
        .base_url("https://openrouter.ai/api/v1")
        .x_title("openrouter-rs")
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .build()
        .map(Some)
}

pub fn should_run_management_tests() -> bool {
    load_integration_env_file();

    env::var("OPENROUTER_RUN_MANAGEMENT_TESTS")
        .ok()
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub async fn rate_limit_delay() {
    static LAST_CALL: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);

    let sleep_duration = {
        let last_call = LAST_CALL.lock().unwrap();
        if let Some(last) = *last_call {
            let elapsed = last.elapsed();
            if elapsed < Duration::from_millis(500) {
                Some(Duration::from_millis(500) - elapsed)
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(duration) = sleep_duration {
        sleep(duration).await;
    }

    {
        let mut last_call = LAST_CALL.lock().unwrap();
        *last_call = Some(Instant::now());
    }
}
