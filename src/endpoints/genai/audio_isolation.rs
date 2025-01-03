#![allow(dead_code)]
//! The audio isolation endpoint
//!
//! # Pricing
//!
//!The API is charged at 1000 characters per minute of audio.
use crate::error::Error;
use crate::shared::path_segments::STREAM_PATH;
use std::path::Path;
//use base64::{engine::general_purpose, Engine as _};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;


use super::*;

const AUDIO_ISOLATION_PATH: &str = "v1/audio-isolation";

/// The audio isolation endpoint
///
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::utils::{play, save,};
/// use elevenlabs_rs::*;
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
#[derive(Clone, Debug)]
pub struct AudioIsolation {
    pub audio_file: String,
}

impl AudioIsolation {
    pub fn new<T: Into<String> >(audio_file: T) -> Self {
        Self { audio_file: audio_file.into() }
    }
}

impl Endpoint for AudioIsolation {
    //type ResponseBody = AudioIsolationResponse;
    type ResponseBody = Bytes;

    fn method(&self) -> Method {
        Method::POST
    }
    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(to_form(&self.audio_file)?))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        //Ok(resp.json().await?)
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(AUDIO_ISOLATION_PATH);
        url
    }
}


/// The audio isolation stream endpoint
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::utils::{save, stream_audio};
/// use elevenlabs_rs::*;
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
#[derive(Clone, Debug)]
pub struct AudioIsolationStream {
    pub audio_file: String,
}

impl AudioIsolationStream {
    pub fn new<T: Into<String> >(audio_file: T) -> Self {
        Self { audio_file: audio_file.into() }
    }
}

type AudioIsolationStreamResponse = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
impl Endpoint for AudioIsolationStream {
    type ResponseBody = AudioIsolationStreamResponse;

    fn method(&self) -> Method {
        Method::POST
    }
    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(to_form(&self.audio_file)?))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream.map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}{}", AUDIO_ISOLATION_PATH, STREAM_PATH));
        url
    }
}


fn to_form(audio_file: &str) -> Result<Form> {
    let mut form = Form::new();
    let path = Path::new(audio_file);
    let audio_bytes = std::fs::read(audio_file)?;
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
    form = form.part("audio", part);
    Ok(form)
}


//#[derive(Clone, Debug, Deserialize)]
//pub struct AudioIsolationResponse {
//    audio: IsolatedAudio,
//    waveform_base_64: String,
//}
//
//#[derive(Clone, Debug, Deserialize)]
//pub struct IsolatedAudio {
//    audio_isolation_id: String,
//    created_at_unix: u64,
//}
//
//impl AudioIsolationResponse {
//    pub fn audio(&self) -> &IsolatedAudio {
//        &self.audio
//    }
//    pub fn waveform_base_64(&self) -> &str {
//        &self.waveform_base_64
//    }
//
//    pub fn audio_as_bytes(&self) -> Result<Bytes> {
//        Ok(Bytes::from(general_purpose::STANDARD.decode(&self.waveform_base_64)?))
//    }
//}
//
//impl IsolatedAudio {
//    pub fn audio_isolation_id(&self) -> &str {
//        &self.audio_isolation_id
//    }
//    pub fn created_at_unix(&self) -> u64 {
//        self.created_at_unix
//    }
//
//}