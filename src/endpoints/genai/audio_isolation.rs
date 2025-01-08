//! The audio isolation endpoint

use super::*;
use crate::error::Error;
use futures_util::{Stream, StreamExt};
use std::path::Path;
use std::pin::Pin;


/// Removes background noise from audio.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::audio_isolation::AudioIsolation;
/// use elevenlabs_rs::utils::{play, save,};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint = AudioIsolation::new("some_audio_file.mp3");
///     let resp = client.hit(endpoint).await?;
///     save("audio_file_isolated.mp3", resp.clone())?;
///     play(resp)?;
///     Ok(())
/// }
/// ```
/// See [Audio Isolation API reference](https://elevenlabs.io/docs/api-reference/audio-isolation/audio-isolation).
#[derive(Clone, Debug)]
pub struct AudioIsolation {
    body: AudioIsolationBody,
}

impl AudioIsolation {
    pub fn new(body: impl Into<AudioIsolationBody>) -> Self {
        Self { body: body.into() }
    }
}

#[derive(Clone, Debug)]
pub struct AudioIsolationBody {
    audio_file: String,
}

impl AudioIsolationBody {
    pub fn new(audio_file: impl Into<String>) -> Self {
        Self {
            audio_file: audio_file.into(),
        }
    }
}

impl From<&str> for AudioIsolationBody {
    fn from(audio_file: &str) -> Self {
        Self {
            audio_file: audio_file.to_string(),
        }
    }
}

impl From<String> for AudioIsolationBody {
    fn from(audio_file: String) -> Self {
        Self { audio_file }
    }
}

impl ElevenLabsEndpoint for AudioIsolation {
    const PATH: &'static str = "v1/audio-isolation";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// Removes background noise from audio.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::audio_isolation::AudioIsolationStream;
/// use elevenlabs_rs::utils::{save, stream_audio};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint = AudioIsolationStream::new("some_audio_file.mp3");
///     let resp = client.hit(endpoint).await?;
///     stream_audio(resp).await?;
///     Ok(())
/// }
/// ```
/// See [Audio Isolation Stream API reference](https://elevenlabs.io/docs/api-reference/audio-isolation/audio-isolation-stream).
#[derive(Clone, Debug)]
pub struct AudioIsolationStream {
    body: AudioIsolationBody,
}

impl AudioIsolationStream {
    pub fn new(body: impl Into<AudioIsolationBody>) -> Self {
        Self { body: body.into() }
    }
}

type AudioIsolationStreamResponse = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
impl ElevenLabsEndpoint for AudioIsolationStream {
    const PATH: &'static str = "v1/audio-isolation/stream";

    const METHOD: Method = Method::POST;

    type ResponseBody = AudioIsolationStreamResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream.map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
}

impl TryFrom<&AudioIsolationBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &AudioIsolationBody) -> Result<Self> {
        let path = Path::new(&body.audio_file);
        let audio_bytes = std::fs::read(&body.audio_file)?;
        let mut part = Part::bytes(audio_bytes);
        let file_path_str = path.to_str().ok_or(Box::new(Error::PathNotValidUTF8))?;
        part = part.file_name(file_path_str.to_string());
        let mime_subtype = path
            .extension()
            .ok_or(Box::new(Error::FileExtensionNotFound))?
            .to_str()
            .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?;
        let mime = format!("audio/{}", mime_subtype);
        part = part.mime_str(&mime)?;
        Ok(RequestBody::Multipart(Form::new().part("audio", part)))
    }
}