//! Speech-to-speech endpoints

use reqwest::multipart::Part;
use super::*;
pub use crate::endpoints::tts::{Latency,SpeechQuery, OutputFormat, STREAM_PATH};
pub use crate::endpoints::voice::VoiceSettings;
use crate::endpoints::voice::VoiceID;
use crate::error::Error;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

const STS_PATH: &str = "/v1/speech-to-speech";

/// Speech-to-speech endpoint
///
/// Use Speech to Speech API to transform uploaded speech, 
/// so it sounds like it was spoken by another voice.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::client::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::sts::*;
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let model = "eleven_multilingual_sts_v2"; // default is eleven_english_sts_v2
///    let body = SpeechToSpeechBody::new("some_audio.mp3").with_model_id(model);
///    let client = ElevenLabsClient::new()?;
///    let resp = client.hit(SpeechToSpeech::new("voice_id", body)).await?;
///    play(resp).await?;
///    Ok(())
/// }
///  ```
/// See the [ElevenLabs docs](https://elevenlabs.io/docs/api-reference/speech-to-speech) for more information.
#[derive(Debug, Clone)]
pub struct SpeechToSpeech {
    voice_id: VoiceID,
    speech_to_speech_body: SpeechToSpeechBody,
    speech_query: Option<SpeechQuery>,
}

/// Speech-to-speech body
#[derive(Debug, Clone)]
pub struct SpeechToSpeechBody {
    audio: String,
    model_id: Option<String>,
    voice_settings: Option<VoiceSettings>,
    seed: Option<u64>,
}

impl SpeechToSpeechBody {
    /// Create a new SpeechToSpeechBody
    pub fn new(audio: &str) -> Self {
        SpeechToSpeechBody {
            audio: audio.to_string(),
            model_id: None,
            voice_settings: None,
            seed: None,
        }
    }
    /// Set the model id
    pub fn with_model_id(mut self, model_id: &str) -> Self {
        self.model_id = Some(model_id.to_string());
        self
    }
    /// Set the voice settings
    pub fn with_voice_settings(mut self, voice_settings: VoiceSettings) -> Self {
        self.voice_settings = Some(voice_settings);
        self
    }
    /// Set the seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
    fn to_form(&self) -> Result<Form> {
        let mut form = Form::new();
        let path = std::path::Path::new(&self.audio);
        let audio_bytes = std::fs::read(path)?;
        let file_name = path.to_str().
            ok_or(Box::new(Error::PathNotValidUTF8))?
            .to_string();
        let audio = Part::bytes(audio_bytes)
            .file_name(file_name)
            // TODO: what about wav files?
            .mime_str("audio/mpeg")?;
        form = form.part("audio", audio);
        if let Some(model_id) = &self.model_id {
            form = form.text("model_id", model_id.clone());
        }
        if let Some(voice_settings) = &self.voice_settings {
            form = form.text("voice_settings", serde_json::to_string(voice_settings)?);
        }
        if let Some(seed) = &self.seed {
            form = form.text("seed", seed.to_string());
        }
        Ok(form)
    }
}


impl SpeechToSpeech {
    /// Create a new SpeechToSpeech endpoint
    pub fn new(voice_id: &str, speech_to_speech_body: SpeechToSpeechBody) -> Self {
        let voice_id = VoiceID::from(voice_id);
        SpeechToSpeech {
            voice_id,
            speech_to_speech_body,
            speech_query: None,
        }
    }
    /// Add a query to the endpoint
    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
        self.speech_query = Some(speech_query);
        self
    }

    /// Get the query as a string if it exists
    fn any_query(&self) -> Option<String> {
        if let Some(query) = &self.speech_query {
            Some(query.to_string())
        } else {
            None
        }
    }
}

impl Endpoint for SpeechToSpeech {
    type ResponseBody = Bytes;

    fn method(&self) -> Method {
        Method::POST
    }
    fn multipart_request_body(&self) -> Option<Result<Form>> {
       Some(self.speech_to_speech_body.to_form().map_err(Into::into))
    }
    
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", STS_PATH, self.voice_id.0));
        url.set_query(self.any_query().as_deref());
        url
    }
}
/// Speech-to-speech stream endpoint
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::client::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::sts::*;
/// use elevenlabs_rs::utils::stream_audio;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let model = "eleven_multilingual_sts_v2";
///    let body = SpeechToSpeechBody::new("some_audio.mp3")
///        .with_model_id(model);
///    let client = ElevenLabsClient::new()?;
///    let resp_stream = client
///        .hit(SpeechToSpeechStream::new("voice_id", body))
///        .await?;
///    stream_audio(resp_stream).await?;
///    Ok(())
///}
/// ```
/// See the [ElevenLabs docs](https://elevenlabs.io/docs/api-reference/speech-to-speech-streaming) for more information.

#[derive(Clone, Debug)]
pub struct  SpeechToSpeechStream {
    voice_id: VoiceID,
    speech_to_speech_body: SpeechToSpeechBody,
    speech_query: Option<SpeechQuery>,
}

impl SpeechToSpeechStream {
    pub fn new(voice_id: &str, speech_to_speech_body: SpeechToSpeechBody) -> Self {
        let voice_id = VoiceID::from(voice_id);
        SpeechToSpeechStream {
            voice_id,
            speech_to_speech_body,
            speech_query: None,
        }
    }
    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
        self.speech_query = Some(speech_query);
        self
    }
    fn any_query(&self) -> Option<String> {
        if let Some(query) = &self.speech_query {
            Some(query.to_string())
        } else {
            None
        }
    }
}

impl Endpoint for SpeechToSpeechStream {
    type ResponseBody = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
    fn method(&self) -> Method {
        Method::POST
    }
    fn multipart_request_body(&self) -> Option<Result<Form>> {
        Some(self.speech_to_speech_body.to_form().map_err(Into::into))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream.map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}{}", STS_PATH, self.voice_id.0, STREAM_PATH));
        url.set_query(self.any_query().as_deref());
        url
    }
}

