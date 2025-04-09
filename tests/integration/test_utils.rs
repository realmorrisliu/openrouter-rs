use std::{
    env,
    time::{Duration, Instant},
};

use openrouter_rs::client::OpenRouterClient;
use tokio::time::sleep;

pub fn get_test_api_key() -> String {
    env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable not set for integration tests")
}

pub fn create_test_client() -> OpenRouterClient {
    OpenRouterClient::builder(get_test_api_key())
        .base_url("https://openrouter.ai/api/v1")
        .build()
}

pub async fn rate_limit_delay() {
    static LAST_CALL: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);

    let mut last_call = LAST_CALL.lock().unwrap();
    if let Some(last) = *last_call {
        let elapsed = last.elapsed();
        if elapsed < Duration::from_millis(500) {
            sleep(Duration::from_millis(500) - elapsed).await;
        }
    }
    *last_call = Some(Instant::now());
}
