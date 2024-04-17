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

const BASE_PATH: &str = "/text-to-speech";
const STREAM_PATH: &str = "/stream";
const OPTIMIZE_QUERY: &str = "optimize_streaming_latency";
const OUTPUT_QUERY: &str = "output_format";

/// See Elevenlabs documentation on [supported output formats](https://help.elevenlabs.io/hc/en-us/articles/15754340124305-What-audio-formats-do-you-support).
pub enum OutputFormat {
    Mp3_22050Hz32kbps,
    Mp3_44100Hz32kbps,
    Mp3_44100Hz64kbps,
    Mp3_44100Hz96kbps,
    Mp3_44100Hz192kbps,
    Pcm16000Hz,
    Pcm22050Hz,
    Pcm24000Hz,
    Pcm44100Hz,
    MuLaw8000Hz,
}
impl OutputFormat {
    fn to_query(&self) -> &str {
        match self {
            OutputFormat::Pcm16000Hz => "pcm_16000",
            OutputFormat::Pcm22050Hz => "pcm_22050",
            OutputFormat::Pcm24000Hz => "pcm_24000",
            OutputFormat::Pcm44100Hz => "pcm_44100",
            OutputFormat::Mp3_22050Hz32kbps => "mp3_22050_32",
            OutputFormat::Mp3_44100Hz32kbps => "mp3_44100_32",
            OutputFormat::Mp3_44100Hz64kbps => "mp3_44100_64",
            OutputFormat::Mp3_44100Hz96kbps => "mp3_44100_96",
            OutputFormat::Mp3_44100Hz192kbps => "mp3_44100_192",
            OutputFormat::MuLaw8000Hz => "ulaw_8000",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Speech;

    #[tokio::test]
    #[ignore]
    async fn speechs_new_is_erring_with_invalid_voice_name() {
        let speech = Speech::new("Test", "Bogus Voice", "eleven_monolingual_v1", 0, None).await;
        assert!(speech.is_err());
    }
    #[tokio::test]
    #[ignore]
    async fn speechs_new_is_erring_with_invalid_model_id() {
        let speech = Speech::new("Test", "Adam", "bogus_model_v1", 0, None).await;
        assert!(speech.is_err());
    }
    #[tokio::test]
    #[ignore]
    async fn speechs_new_is_erring_when_latency_value_is_beyond_the_limit_of_22() {
        let speech = Speech::new("Test", "Adam", "eleven_monolingual_v1", 23, None).await;
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
            None,
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
            None,
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
    ///        None,
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
    pub async fn new(
        text: &str,
        voice_name: &str,
        model: &str,
        latency: u32,
        format: Option<OutputFormat>,
    ) -> Result<Self> {
        if latency > 22 {
            return Err(Box::new(Error::SpeechGenerationError(
                "Latency value must be between 0 and 22".to_string(),
            )));
        }
        let voice = Voice::with_settings(voice_name).await?;

        let format_query = if let Some(format) = format {
            format!("&{}={}", OUTPUT_QUERY, format.to_query())
        } else {
            "".to_string()
        };

        let cb = ClientBuilder::new()?;
        let c = cb
            .method(POST)?
            .path(format!(
                "{}/{}{}?{}={}{}",
                BASE_PATH, voice.voice_id, STREAM_PATH, OPTIMIZE_QUERY, latency, format_query,
            ))?
            .header(ACCEPT, APPLICATION_JSON)?
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
    ///        None,
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
        format: Option<OutputFormat>,
    ) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::new(&text, voice_name, model, latency, format).await?)
    }

    pub fn play(&self) -> Result<()> {
        play(self.audio.clone())?;
        Ok(())
    }

    pub fn save(&self, filename: Option<String>) -> Result<()> {
        let filename = match filename {
            Some(f) => f,
            None => format!("{}_{}.mp3", self.voice.name.clone(), Utc::now().timestamp()),
        };
        save(&filename, self.audio.clone())?;
        Ok(())
    }
}

impl AsRef<[u8]> for Speech {
    fn as_ref(&self) -> &[u8] {
        self.audio.as_ref()
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
