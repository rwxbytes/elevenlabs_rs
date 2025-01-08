//! The voice changer endpoints
use super::*;
use crate::error::Error;
pub use crate::shared::VoiceSettings;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

/// Transform audio from one voice to another. Maintain full control over emotion, timing and delivery.
///
/// # Example
/// ```no_run
///
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::play;
/// use elevenlabs_rs::endpoints::genai::voice_changer::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let query = VoiceChangerQuery::default()
///         .with_output_format(OutputFormat::Mp3_44100Hz64kbps);
///
///     let body = VoiceChangerBody::new("some_audio.mp3")
///         .with_model_id(Model::ElevenMultilingualV2STS);
///
///     let endpoint = VoiceChanger::new("voice_id", body)
///         .with_query(query);
///
///     let resp = client.hit(endpoint).await?;
///
///     play(resp)?;
///
///     Ok(())
/// }
/// ```
/// See [Voice Changer API reference](https://elevenlabs.io/docs/api-reference/speech-to-speech/convert)
#[derive(Debug, Clone)]
pub struct VoiceChanger {
    voice_id: VoiceID,
    body: VoiceChangerBody,
    query: Option<VoiceChangerQuery>,
}

impl VoiceChanger {
    pub fn new(voice_id: impl Into<VoiceID>, body: VoiceChangerBody) -> Self {
        VoiceChanger {
            voice_id: voice_id.into(),
            body,
            query: None,
        }
    }
    pub fn with_query(mut self, query: VoiceChangerQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl ElevenLabsEndpoint for VoiceChanger {
    const PATH: &'static str = "v1/speech-to-speech/:voice_id";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.as_path_param()]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

#[derive(Clone, Debug, Default)]
pub struct VoiceChangerQuery {
    params: QueryValues,
}

impl VoiceChangerQuery {
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.params
            .push(("output_format", output_format.to_string()));
        self
    }
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.params
            .push(("enable_logging", enable_logging.to_string()));
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct VoiceChangerBody {
    audio: String,
    model_id: Option<ModelID>,
    voice_settings: Option<VoiceSettings>,
    seed: Option<u64>,
    remove_background_noise: Option<bool>,
}

impl VoiceChangerBody {
    pub fn new(audio: impl Into<String>) -> Self {
        VoiceChangerBody {
            audio: audio.into(),
            ..Default::default()
        }
    }
    pub fn with_model_id(mut self, model_id: impl Into<ModelID>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }
    pub fn with_voice_settings(mut self, voice_settings: VoiceSettings) -> Self {
        self.voice_settings = Some(voice_settings);
        self
    }
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_remove_background_noise(mut self, remove_background_noise: bool) -> Self {
        self.remove_background_noise = Some(remove_background_noise);
        self
    }
}



/// Stream audio from one voice to another. Maintain full control over emotion, timing and delivery.
///
/// # Example
/// ```no_run
///
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::stream_audio;
/// use elevenlabs_rs::endpoints::genai::voice_changer::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let query = VoiceChangerQuery::default()
///         .with_output_format(OutputFormat::Mp3_44100Hz64kbps);
///
///     let body = VoiceChangerBody::new("some_audio.mp3")
///         .with_model_id(Model::ElevenMultilingualV2STS);
///
///     let endpoint = VoiceChangerStream::new("voice_id", body)
///         .with_query(query);
///
///     let resp = client.hit(endpoint).await?;
///
///     stream_audio(resp).await?;
///
///     Ok(())
/// }
/// ```
/// See [Voice Changer Stream API reference](https://elevenlabs.io/docs/api-reference/speech-to-speech/convert-as-stream)
#[derive(Clone, Debug)]
pub struct VoiceChangerStream {
    voice_id: VoiceID,
    body: VoiceChangerBody,
    query: Option<VoiceChangerQuery>,
}

impl VoiceChangerStream {
    pub fn new(voice_id: impl Into<VoiceID>, body: VoiceChangerBody) -> Self {
        VoiceChangerStream {
            voice_id: voice_id.into(),
            body,
            query: None,
        }
    }
    pub fn with_query(mut self, query: VoiceChangerQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl ElevenLabsEndpoint for VoiceChangerStream {
    const PATH: &'static str = "v1/speech-to-speech/:voice_id/stream";

    const METHOD: Method = Method::POST;

    type ResponseBody = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.as_path_param()]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream.map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
}

impl TryFrom<&VoiceChangerBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &VoiceChangerBody) -> Result<Self> {
        let path = std::path::Path::new(&body.audio);
        let audio_bytes = std::fs::read(path)?;
        let mut part = Part::bytes(audio_bytes);
        let file_path_str = path.to_str().ok_or(Box::new(Error::PathNotValidUTF8))?;
        part = part.file_name(file_path_str.to_string());
        let mime_subtype = path
            .extension()
            .ok_or(Box::new(Error::FileExtensionNotFound))?
            .to_str()
            .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?;
        let mime_type = format!("audio/{}", mime_subtype);
        part = part.mime_str(&mime_type)?;
        let mut form = Form::new().part("audio", part);
        if let Some(model_id) = &body.model_id {
            form = form.text("model_id", model_id._inner.clone());
        }
        if let Some(voice_settings) = &body.voice_settings {
            form = form.text("voice_settings", serde_json::to_string(voice_settings)?);
        }
        if let Some(seed) = &body.seed {
            form = form.text("seed", seed.to_string());
        }
        Ok(RequestBody::Multipart(form))
    }
}