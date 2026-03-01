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

#[allow(clippy::result_large_err)]
pub fn create_test_client() -> Result<OpenRouterClient, OpenRouterError> {
    OpenRouterClient::builder()
        .api_key(get_test_api_key())
        .base_url("https://openrouter.ai/api/v1")
        .build()
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
