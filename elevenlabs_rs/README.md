An unofficial lib crate for [ElevenLabs](https://elevenlabs.io/) 

## Text-to-Speech

```rust
 use elevenlabs_rs::{ElevenLabsClient, Result, DefaultVoice, Model};
 use elevenlabs_rs::endpoints::genai::tts::{TextToSpeech, TextToSpeechBody};
 use elevenlabs_rs::utils::play;

 #[tokio::main]
 async fn main() -> Result<()> {
     let client = ElevenLabsClient::from_env()?;

     let txt = "Hello! 你好! Hola! नमस्ते! Bonjour! \
         こんにちは! مرحبا! 안녕하세요! Ciao! Cześć! Привіт! வணக்கம்!";

     let body = TextToSpeechBody::new(txt)
        .with_model_id(Model::ElevenMultilingualV2);

     let endpoint = TextToSpeech::new(DefaultVoice::Brian, body);

     let speech = client.hit(endpoint).await?;

     play(speech)?;

     Ok(())
 }
 ```