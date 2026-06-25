//! The audio isolation endpoint

use super::*;
use crate::shared::{audio_mime_from_extension, FilePart};
use futures_util::{Stream, StreamExt};
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
    audio_file: FilePart,
}

impl AudioIsolationBody {
    pub fn new(audio_file: impl Into<FilePart>) -> Self {
        Self {
            audio_file: audio_file.into(),
        }
    }

    pub fn from_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(FilePart::bytes(file_name, mime, bytes))
    }
}

impl From<&str> for AudioIsolationBody {
    fn from(audio_file: &str) -> Self {
        Self {
            audio_file: FilePart::from(audio_file),
        }
    }
}

impl From<String> for AudioIsolationBody {
    fn from(audio_file: String) -> Self {
        Self {
            audio_file: FilePart::from(audio_file),
        }
    }
}

impl crate::endpoints::sealed::Sealed for AudioIsolation {}

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
impl crate::endpoints::sealed::Sealed for AudioIsolationStream {}

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
    type Error = crate::error::Error;

    fn try_from(body: &AudioIsolationBody) -> Result<Self> {
        let inferred_mime = inferred_audio_mime(&body.audio_file)?;
        Ok(RequestBody::Multipart(Form::new().part(
            "audio",
            body.audio_file.clone().into_part(inferred_mime)?,
        )))
    }
}

fn inferred_audio_mime(file: &FilePart) -> Result<Option<String>> {
    if file.mime().is_some() {
        return Ok(None);
    }

    let extension = file.extension()?;
    Ok(Some(audio_mime_from_extension(&extension)?.to_owned()))
}

/// Get metadata about your audio isolation history.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::audio_isolation::{
///     GetAudioIsolationHistory, AudioIsolationHistoryQuery,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let query = AudioIsolationHistoryQuery::default().with_page_size(10);
///     let resp = client.hit(GetAudioIsolationHistory::default().with_query(query)).await?;
///     for item in &resp.items {
///         println!("{} ({})", item.id, item.format);
///     }
///     Ok(())
/// }
/// ```
/// See [Get Audio Isolation History API reference](https://elevenlabs.io/docs/api-reference/audio-isolation/history).
#[derive(Clone, Debug, Default)]
pub struct GetAudioIsolationHistory {
    query: Option<AudioIsolationHistoryQuery>,
}

impl GetAudioIsolationHistory {
    pub fn with_query(mut self, query: AudioIsolationHistoryQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetAudioIsolationHistory`].
#[derive(Clone, Debug, Default)]
pub struct AudioIsolationHistoryQuery {
    params: QueryValues,
}

impl AudioIsolationHistoryQuery {
    /// How many history items to return at maximum. Defaults to 100, max 1000.
    pub fn with_page_size(mut self, page_size: u16) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    /// Page number for search pagination (1-based). Only used with `search`.
    pub fn with_page(mut self, page: u32) -> Self {
        self.params.push(("page", page.to_string()));
        self
    }

    /// Optional search term used to filter history by title or text.
    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.params.push(("search", search.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetAudioIsolationHistory {}

impl ElevenLabsEndpoint for GetAudioIsolationHistory {
    const PATH: &'static str = "/v1/audio-isolation/history";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetAudioIsolationHistoryResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetAudioIsolationHistoryResponse {
    pub items: Vec<AudioIsolationHistoryItem>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AudioIsolationHistoryItem {
    pub id: String,
    pub title: Option<String>,
    pub created_at_unix: u64,
    pub format: String,
    pub duration_seconds: Option<f64>,
    pub download_url: Option<String>,
    pub icon_url: Option<String>,
    pub source_video_url: Option<String>,
    pub supports_video: bool,
    pub processing: bool,
    pub video_processing_failed: bool,
    pub preview_b64: Option<String>,
}

impl IntoIterator for GetAudioIsolationHistoryResponse {
    type Item = AudioIsolationHistoryItem;
    type IntoIter = std::vec::IntoIter<AudioIsolationHistoryItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

/// Delete an audio isolation history item by its ID.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::audio_isolation::DeleteAudioIsolationHistoryItem;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     client.hit(DeleteAudioIsolationHistoryItem::new("history_item_id")).await?;
///     Ok(())
/// }
/// ```
/// See [Delete Audio Isolation History Item API reference](https://elevenlabs.io/docs/api-reference/audio-isolation/delete-history-item).
#[derive(Clone, Debug)]
pub struct DeleteAudioIsolationHistoryItem {
    history_item_id: String,
}

impl DeleteAudioIsolationHistoryItem {
    pub fn new(history_item_id: impl Into<String>) -> Self {
        Self {
            history_item_id: history_item_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteAudioIsolationHistoryItem {}

impl ElevenLabsEndpoint for DeleteAudioIsolationHistoryItem {
    const PATH: &'static str = "/v1/audio-isolation/history/:history_item_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.history_item_id.and_param(PathParam::HistoryItemID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}
