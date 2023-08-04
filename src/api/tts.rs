use crate::{
    api::{
        voice::{Voice, VoiceSettings},
        ClientBuilder,
    },
    error::Error,
    prelude::*,
    utils::{play, save},
};
use chrono::prelude::*;
use http_body_util::Full;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

const POST: &str = "POST";
const BASE_PATH: &str = "/text-to-speech";
const STREAM_PATH: &str = "/stream";
const OPTIMIZE_QUERY: &str = "optimize_streaming_latency";

#[cfg(test)]
mod tests {
    use super::Speech;

    #[tokio::test]
    #[ignore]
    async fn speechs_new_is_erring_with_invalid_voice_name() {
        let speech = Speech::new("Test", "Bogus Voice", "eleven_monolingual_v1", 0).await;
        assert!(speech.is_err());
    }
    #[tokio::test]
    #[ignore]
    async fn speechs_new_is_erring_with_invalid_model_id() {
        let speech = Speech::new("Test", "Adam", "bogus_model_v1", 0).await;
        assert!(speech.is_err());
    }
    #[tokio::test]
    #[ignore]
    async fn speechs_new_is_erring_when_latency_value_is_beyond_the_limit_of_22() {
        let speech = Speech::new("Test", "Adam", "eleven_monolingual_v1", 23).await;
        assert!(speech.is_err());
    }
    #[tokio::test]
    #[ignore]
    async fn speechs_from_file_is_erring_with_invalid_voice_name() {
        let speech = Speech::from_file(
            "tests/testdata/speech_from_file_test.txt",
            "Bogus Voice",
            "eleven_monolingual_v1",
            0,
        )
        .await;
        assert!(speech.is_err());
    }
    #[tokio::test]
    #[ignore]
    async fn speechs_from_file_is_erring_with_invalid_model_id() {
        let speech = Speech::from_file(
            "tests/testdata/speech_from_file_test.txt",
            "Adam",
            "bogus_model_v1",
            0,
        )
        .await;
        assert!(speech.is_err());
    }
}

#[derive(Debug, Clone)]
pub struct Speech {
    audio: Bytes,
    voice: Voice,
}

impl Speech {
    /// Generate a new speech from a text, a voice name, a model name and a latency.
    ///
    /// # Examples
    /// ```no_run
    ///use elevenlabs_rs::api::tts::Speech;
    ///use elevenlabs_rs::prelude::*;
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<()> {
    ///    let speech = Speech::new(
    ///        "By the pricking of my thumbs",
    ///        "Glinda",
    ///        "eleven_multilingual_v1",
    ///        0,
    ///    )
    ///    .await?;
    ///     
    ///     // None will generate a filename with the voice name and the current utc timestamp
    ///     // e.g. Glinda_1624299999.mp3
    ///    speech.save(None)?;
    ///
    ///     // And/or play it directly
    ///    speech.play()?;
    ///
    ///    Ok(())
    ///}
    /// ```
    pub async fn new(text: &str, voice_name: &str, model: &str, latency: u32) -> Result<Self> {
        if latency > 22 {
            return Err(Box::new(Error::SpeechGenerationError(
                "Latency value must be between 0 and 22".to_string(),
            )));
        }
        let voice = Voice::with_settings(voice_name).await?;

        let cb = ClientBuilder::new()?;
        let c = cb
            .method(POST)?
            .path(format!(
                "{}/{}{}?{}={}",
                BASE_PATH, voice.voice_id, STREAM_PATH, OPTIMIZE_QUERY, latency
            ))?
            .header("ACCEPT", "application/json")?
            .build()?;

        let body = TTSBody {
            text: text.to_string(),
            model_id: model.to_string(),
            voice_settings: voice.settings.clone().unwrap(),
        }
        .to_json()?;

        let audio = c.send_request(Full::<Bytes>::new(body.into())).await?;

        Ok(Self { audio, voice })
    }

    /// Generate a new speech from a file, a voice name, a model name and a latency.
    ///
    /// # Examples
    /// ```no_run
    ///use elevenlabs_rs::api::tts::Speech;
    ///use elevenlabs_rs::prelude::*;
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<()> {
    ///    let speech = Speech::from_file(
    ///        "sonnet_11.txt",
    ///        "Ethan",
    ///        "eleven_monolingual_v1",
    ///        0,
    ///    )
    ///    .await?;
    ///     
    ///    speech.save(Some("ethans_sonnet_11_recitation.mp3".to_string()))?;
    ///
    ///    Ok(())
    ///}
    /// ```
    pub async fn from_file(
        path: &str,
        voice_name: &str,
        model: &str,
        latency: u32,
    ) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::new(&text, voice_name, model, latency).await?)
    }

    pub fn play(&self) -> Result<()> {
        play(self.audio.clone())?;
        Ok(())
    }

    pub fn save(&self, filename: Option<String>) -> Result<()> {
        let filename = match filename {
            Some(f) => f,
            None => format!(
                "{}_{}.mp3",
                self.voice.name.clone().unwrap(),
                Utc::now().timestamp()
            ),
        };
        let _saved = save(&filename, self.audio.clone())?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TTSBody {
    text: String,
    model_id: String,
    voice_settings: VoiceSettings,
}

impl TTSBody {
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}
