An unofficial lib crate for [ElevenLabs](https://elevenlabs.io/) 

## Text-to-Speech

```rust
use elevenlabs_rs::*;
use elevenlabs_rs::utils::play;

#[tokio::main]
async fn main() -> Result<()> {
    let client = ElevenLabsClient::default()?;
    let body = TextToSpeechBody::new(
        "This is the way the world ends, not with a bang but a whimper",
        Model::ElevenMultilingualV2,
    );
    let endpoint = TextToSpeech::new(PreMadeVoiceID::Clyde, body);
    let speech = client.hit(endpoint).await?;
    play(speech)?;

    Ok(())
}
 ```