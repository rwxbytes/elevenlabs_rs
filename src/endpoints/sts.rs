#![allow(dead_code)]
//! The speech-to-speech endpoints
use super::*;
pub use crate::endpoints::tts::SpeechQuery;
pub use crate::endpoints::voice::VoiceSettings;
use crate::error::Error;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

const STS_PATH: &str = "/v1/speech-to-speech";

/// Speech-to-speech endpoint
///
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::sts::*;
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let model = Model::ElevenMultilingualV2STS;
///    let body = SpeechToSpeechBody::new("some_audio.mp3").with_model_id(model);
///    let client = ElevenLabsClient::default()?;
///    let resp = client.hit(SpeechToSpeech::new("voice_id", body)).await?;
///    play(resp)?;
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
    pub fn with_model_id<T: Into<String>>(mut self, model_id: T) -> Self {
        self.model_id = Some(model_id.into());
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
        let file_name = path
            .to_str()
            .ok_or(Box::new(Error::PathNotValidUTF8))?
            .to_string();
        let audio = Part::bytes(audio_bytes)
            .file_name(file_name)
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
    pub fn new<T: Into<String>>(voice_id: T, speech_to_speech_body: SpeechToSpeechBody) -> Self {
        SpeechToSpeech {
            voice_id: VoiceID::from(voice_id.into()),
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
    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(
            self.speech_to_speech_body.to_form()?,
        ))
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
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::sts::*;
/// use elevenlabs_rs::utils::stream_audio;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let model = Model::ElevenMultilingualV2STS;
///    let body = SpeechToSpeechBody::new("some_audio.mp3")
///        .with_model_id(model);
///    let client = ElevenLabsClient::default()?;
///    let resp_stream = client
///        .hit(SpeechToSpeechStream::new("voice_id", body))
///        .await?;
///    stream_audio(resp_stream).await?;
///    Ok(())
///}
/// ```
/// See the [ElevenLabs docs](https://elevenlabs.io/docs/api-reference/speech-to-speech-streaming) for more information.

#[derive(Clone, Debug)]
pub struct SpeechToSpeechStream {
    voice_id: VoiceID,
    speech_to_speech_body: SpeechToSpeechBody,
    speech_query: Option<SpeechQuery>,
}

impl SpeechToSpeechStream {
    pub fn new<T: Into<String>>(voice_id: T, speech_to_speech_body: SpeechToSpeechBody) -> Self {
        SpeechToSpeechStream {
            voice_id: VoiceID::from(voice_id.into()),
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
    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(
            self.speech_to_speech_body.to_form()?,
        ))
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
