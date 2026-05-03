use openrouter_rs::{
    OpenRouterClient,
    api::audio::{TranscriptionInputAudio, TranscriptionRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .build()?;

    let audio_data = std::env::var("OPENROUTER_TRANSCRIPTION_AUDIO_BASE64")
        .expect("set OPENROUTER_TRANSCRIPTION_AUDIO_BASE64 to raw base64 audio bytes");
    let audio_format =
        std::env::var("OPENROUTER_TRANSCRIPTION_AUDIO_FORMAT").unwrap_or_else(|_| "wav".into());
    let model = std::env::var("OPENROUTER_TRANSCRIPTION_MODEL")
        .unwrap_or_else(|_| "openai/whisper-large-v3".into());

    let mut request = TranscriptionRequest::builder();
    request
        .model(model)
        .input_audio(TranscriptionInputAudio::new(audio_data, audio_format));
    if let Ok(language) = std::env::var("OPENROUTER_TRANSCRIPTION_LANGUAGE") {
        request.language(language);
    }

    let response = client
        .audio()
        .transcriptions()
        .create(&request.build()?)
        .await?;

    println!("{}", response.text);
    if let Some(usage) = response.usage {
        println!("usage: {:?}", usage);
    }

    Ok(())
}
