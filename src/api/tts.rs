//! Deprecated compatibility layer for the pre-0.9 text-to-speech API names.
//!
//! Use [`crate::api::audio`] for the canonical `/audio/speech` surface.

use crate::{api::audio, error::OpenRouterError};

#[deprecated(note = "use api::audio::SpeechResponseFormat")]
pub type TtsResponseFormat = audio::SpeechResponseFormat;

#[deprecated(note = "use api::audio::SpeechProviderOptions")]
pub type TtsProviderOptions = audio::SpeechProviderOptions;

#[deprecated(note = "use api::audio::SpeechRequest")]
pub type TtsRequest = audio::SpeechRequest;

#[deprecated(note = "use api::audio::SpeechRequestBuilder")]
pub type TtsRequestBuilder = audio::SpeechRequestBuilder;

/// Submit a speech request and return raw audio bytes.
#[deprecated(note = "use api::audio::create_speech")]
pub async fn create_tts(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &audio::SpeechRequest,
) -> Result<Vec<u8>, OpenRouterError> {
    audio::create_speech(
        base_url,
        api_key,
        x_title,
        http_referer,
        app_categories,
        request,
    )
    .await
}
