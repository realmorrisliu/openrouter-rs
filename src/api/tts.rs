//! Deprecated compatibility layer for the pre-0.9 text-to-speech API names.
//!
//! Use [`crate::api::audio`] for the canonical `/audio/speech` surface.

use crate::api::audio;

#[deprecated(note = "use api::audio::SpeechResponseFormat")]
pub type TtsResponseFormat = audio::SpeechResponseFormat;

#[deprecated(note = "use api::audio::SpeechProviderOptions")]
pub type TtsProviderOptions = audio::SpeechProviderOptions;

#[deprecated(note = "use api::audio::SpeechRequest")]
pub type TtsRequest = audio::SpeechRequest;

#[deprecated(note = "use api::audio::SpeechRequestBuilder")]
pub type TtsRequestBuilder = audio::SpeechRequestBuilder;
