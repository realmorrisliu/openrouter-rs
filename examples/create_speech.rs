use openrouter_rs::{
    OpenRouterClient,
    api::audio::{SpeechRequest, SpeechResponseFormat},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = SpeechRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello from openrouter-rs")
        .voice("alloy")
        .response_format(SpeechResponseFormat::Mp3)
        .build()?;

    let audio = client.audio().speech().create(&request).await?;
    std::fs::write("speech-output.mp3", &audio)?;
    println!("wrote {} bytes to speech-output.mp3", audio.len());

    Ok(())
}
