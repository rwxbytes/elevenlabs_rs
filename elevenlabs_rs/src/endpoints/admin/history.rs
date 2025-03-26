//! The history endpoints

use super::*;
use crate::shared::{VoiceCategory, VoiceSettings};

/// Returns metadata about all your generated audio.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result, DefaultVoice};
/// use elevenlabs_rs::endpoints::admin::history::{GetGeneratedItems, HistoryQuery};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let query = HistoryQuery::default()
///         .with_page_size(10)
///         .with_voice_id(DefaultVoice::Brian);
///    let endpoint = GetGeneratedItems::with_query(query);
///    let resp = c.hit(endpoint).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See the [Get Generated Items API reference ](https://elevenlabs.io/docs/api-reference/history/get-all)
#[derive(Clone, Debug, Default)]
pub struct GetGeneratedItems {
    query: Option<HistoryQuery>,
}

impl GetGeneratedItems {
    pub fn with_query(query: HistoryQuery) -> Self {
        Self { query: Some(query) }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HistoryQuery {
    params: QueryValues,
}

impl HistoryQuery {
    pub fn with_page_size(mut self, page_size: u16) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }
    pub fn with_start_after_history_item_id(mut self, start_after_history_item_id: &str) -> Self {
        self.params.push((
            "start_after_history_item_id",
            start_after_history_item_id.to_string(),
        ));
        self
    }

    pub fn with_voice_id(mut self, voice_id: impl Into<String>) -> Self {
        self.params.push(("voice_id", voice_id.into()));
        self
    }
}

impl ElevenLabsEndpoint for GetGeneratedItems {
    const PATH: &'static str = "/v1/history";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetGeneratedItemsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetGeneratedItemsResponse {
    pub history: Vec<GetHistoryItemResponse>,
    pub last_history_item_id: String,
    pub has_more: bool,
}

/// Returns information about a history item by its ID.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::history::GetHistoryItem;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let history_item_id = "some_history_item_id";
///    let resp = client.hit(GetHistoryItem::new(history_item_id)).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See the [Get History Item API reference](https://elevenlabs.io/docs/api-reference/history/get)
#[derive(Clone, Debug)]
pub struct GetHistoryItem {
    history_item_id: String,
}

impl GetHistoryItem {
    pub fn new(history_item_id: impl Into<String>) -> Self {
        Self {
            history_item_id: history_item_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetHistoryItem {
    const PATH: &'static str = "/v1/history/:history_item_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetHistoryItemResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.history_item_id.and_param(PathParam::HistoryItemID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetHistoryItemResponse {
    pub history_item_id: String,
    pub request_id: String,
    pub voice_id: String,
    pub voice_name: String,
    pub voice_category: Option<VoiceCategory>,
    pub model_id: Option<String>,
    pub text: String,
    pub date_unix: u64,
    pub character_count_change_from: u64,
    pub character_count_change_to: u64,
    pub content_type: String,
    pub state: String,
    pub settings: VoiceSettings,
    pub feedback: Option<Feedback>,
    pub share_link_id: Option<String>,
    pub source: Option<Source>,
    pub alignments: Option<Value>,
}

/// Delete a history item by its ID.
///
/// See the [Delete History Item API reference](https://elevenlabs.io/docs/api-reference/history/delete)
#[derive(Clone, Debug)]
pub struct DeleteHistoryItem {
    history_item_id: String,
}

impl DeleteHistoryItem {
    pub fn new(history_item_id: impl Into<String>) -> Self {
        Self {
            history_item_id: history_item_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for DeleteHistoryItem {
    const PATH: &'static str = "/v1/history/:history_item_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.history_item_id.and_param(PathParam::HistoryItemID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Download one or more history items.
/// If one history item ID is provided, we will return a single audio file.
/// If more than one history item IDs are provided, we will provide the history items packed into a .zip file.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::history::{
///     DownloadBody, DownloadHistoryItems, HistoryQuery, GetGeneratedItems};
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let history_item_ids = client
///         .hit(GetGeneratedItems::with_query(HistoryQuery::default()))
///         .await?
///         .history
///         .iter()
///         .map(|i| i.history_item_id.to_string())
///         .collect::<Vec<String>>();
///     let body = DownloadBody::new(history_item_ids);
///     let downloaded_items = client.hit(DownloadHistoryItems::new(body)).await?;
///     save("last_100_items.zip", downloaded_items)?;
///     Ok(())
/// }
/// ```
/// See the [Download History Items API reference](https://elevenlabs.io/docs/api-reference/history/download)
#[derive(Clone, Debug)]
pub struct DownloadHistoryItems {
    body: DownloadBody,
}

impl DownloadHistoryItems {
    pub fn new(body: DownloadBody) -> Self {
        Self { body }
    }
}

impl ElevenLabsEndpoint for DownloadHistoryItems {
    const PATH: &'static str = "/v1/history/download";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DownloadBody {
    pub history_item_ids: Vec<String>,
    /// Output format to transcode the audio file, can be wav or default (Mp3).
    #[serde(skip_serializing_if = "DownloadOutputFormat::is_mp3")]
    pub output_format: DownloadOutputFormat,
}

impl DownloadBody {
    pub fn new(history_item_ids: Vec<String>) -> Self {
        Self {
            history_item_ids,
            output_format: DownloadOutputFormat::default(),
        }
    }
    pub fn with_output_format(mut self, output_format: DownloadOutputFormat) -> Self {
        self.output_format = output_format;
        self
    }
}

/// Returns the audio of a history item.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::history::{GetGeneratedItems, GetAudio, HistoryQuery};
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let query = HistoryQuery::default().with_page_size(1);
///     let resp = client.hit(GetGeneratedItems::with_query(query)).await?;
///     let item_id = resp.into_iter().next().unwrap().history_item_id;
///     let audio = client.hit(GetAudio::new(item_id)).await?;
///     play(audio)?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetAudio {
    history_item_id: String,
}

impl GetAudio {
    pub fn new(history_item_id: impl Into<String>) -> Self {
        Self {
            history_item_id: history_item_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetAudio {
    const PATH: &'static str = "/v1/history/:history_item_id/audio";

    const METHOD: Method = Method::GET;

    type ResponseBody = Bytes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.history_item_id.and_param(PathParam::HistoryItemID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Feedback {
    pub thumbs_up: bool,
    pub feedback: String,
    pub emotions: bool,
    pub inaccurate_clone: bool,
    pub glitches: bool,
    pub audio_quality: bool,
    pub review_status: String,
    pub other: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Source {
    Tts,
    Sts,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadOutputFormat {
    #[default]
    Mp3,
    Wav,
}

impl DownloadOutputFormat {
    pub(crate) fn is_mp3(&self) -> bool {
        match self {
            DownloadOutputFormat::Mp3 => true,
            DownloadOutputFormat::Wav => false,
        }
    }
}

impl<'a> IntoIterator for &'a GetGeneratedItemsResponse {
    type Item = &'a GetHistoryItemResponse;
    type IntoIter = std::slice::Iter<'a, GetHistoryItemResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.history.iter()
    }
}

impl IntoIterator for GetGeneratedItemsResponse {
    type Item = GetHistoryItemResponse;
    type IntoIter = std::vec::IntoIter<GetHistoryItemResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.history.into_iter()
    }
}
