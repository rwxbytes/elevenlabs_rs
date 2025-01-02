//! The history endpoints

use super::*;
use crate::components::admin::history::*;
use crate::components::common::{VoiceCategory, VoiceSettings};

/// Returns metadata about all your generated audio.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result, DefaultVoice};
/// use elevenlabs_rs::endpoints::admin::history::{GetGeneratedItems, HistoryQuery};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
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

    pub fn with_voice_id(mut self, voice_id: impl Into<VoiceID>) -> Self {
        self.params.push(("voice_id", voice_id.into()._inner));
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
    history: Vec<GetHistoryItemResponse>,
    last_history_item_id: String,
    has_more: bool,
}

impl GetGeneratedItemsResponse {
    pub fn history(&self) -> &[GetHistoryItemResponse] {
        &self.history
    }
    pub fn last_history_item_id(&self) -> &str {
        &self.last_history_item_id
    }
    pub fn has_more(&self) -> bool {
        self.has_more
    }
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
///    let client = ElevenLabsClient::default()?;
///    let history_item_id = "some_history_item_id";
///    let resp = client.hit(GetHistoryItem::new(history_item_id)).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See the [Get History Item API reference](https://elevenlabs.io/docs/api-reference/history/get)
#[derive(Clone, Debug)]
pub struct GetHistoryItem {
    history_item_id: HistoryItemID,
}

impl GetHistoryItem {
    pub fn new(history_item_id: impl Into<HistoryItemID>) -> Self {
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
        vec![self.history_item_id.as_path_param()]
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetHistoryItemResponse {
    history_item_id: String,
    request_id: String,
    voice_id: String,
    voice_name: String,
    voice_category: Option<VoiceCategory>,
    model_id: Option<String>,
    text: String,
    date_unix: u64,
    character_count_change_from: u64,
    character_count_change_to: u64,
    content_type: String,
    state: String,
    settings: VoiceSettings,
    feedback: Option<Feedback>,
    share_link_id: Option<String>,
    source: Option<Source>,
    // TODO: impl type
    alignments: Option<Value>,
}
impl GetHistoryItemResponse {
    pub fn history_item_id(&self) -> &str {
        &self.history_item_id
    }
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
    pub fn voice_id(&self) -> &str {
        &self.voice_id
    }
    pub fn voice_name(&self) -> &str {
        &self.voice_name
    }
    pub fn voice_category(&self) -> Option<&VoiceCategory> {
        self.voice_category.as_ref()
    }
    pub fn model_id(&self) -> Option<&str> {
        self.model_id.as_deref()
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn date_unix(&self) -> u64 {
        self.date_unix
    }
    pub fn character_count_change_from(&self) -> u64 {
        self.character_count_change_from
    }
    pub fn character_count_change_to(&self) -> u64 {
        self.character_count_change_to
    }
    pub fn content_type(&self) -> &str {
        &self.content_type
    }
    pub fn state(&self) -> &str {
        &self.state
    }
    pub fn settings(&self) -> &VoiceSettings {
        &self.settings
    }
    pub fn feedback(&self) -> Option<&Feedback> {
        self.feedback.as_ref()
    }
    pub fn share_link_id(&self) -> Option<&str> {
        self.share_link_id.as_deref()
    }
    pub fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }
    pub fn alignments(&self) -> Option<&Value> {
        self.alignments.as_ref()
    }
}

/// Delete a history item by its ID.
///
/// See the [Delete History Item API reference](https://elevenlabs.io/docs/api-reference/history/delete)
#[derive(Clone, Debug)]
pub struct DeleteHistoryItem {
    history_item_id: HistoryItemID,
}

impl DeleteHistoryItem {
    pub fn new(history_item_id: impl Into<HistoryItemID>) -> Self {
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
        vec![self.history_item_id.as_path_param()]
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
///     let client = ElevenLabsClient::default()?;
///     let history_item_ids = client
///         .hit(GetGeneratedItems::with_query(HistoryQuery::default()))
///         .await?
///         .history()
///         .iter()
///         .map(|i| i.history_item_id().to_string())
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
///     let client = ElevenLabsClient::default()?;
///     let query = HistoryQuery::default().with_page_size(1);
///     let resp = client.hit(GetGeneratedItems::with_query(query)).await?;
///     let item_id = resp.into_iter().next().unwrap().history_item_id();
///     let audio = client.hit(GetAudio::new(item_id)).await?;
///     play(audio)?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetAudio {
    history_item_id: HistoryItemID,
}

impl GetAudio {
    pub fn new(history_item_id: impl Into<HistoryItemID>) -> Self {
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
        vec![self.history_item_id.as_path_param()]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
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
