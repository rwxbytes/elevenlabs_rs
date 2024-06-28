//! The history endpoints
#![allow(dead_code)]

use super::*;
use crate::endpoints::voice::VoiceSettings;

const HISTORY_PATH: &str = "/v1/history";
const AUDIO_PATH: &str = "/audio";
const PAGE_SIZE_QUERY: &str = "page_size";
const HISTORY_ITEM_IDS: &str = "history_item_ids";
const START_AFTER_HISTORY_ITEM_ID_QUERY: &str = "start_after_history_item_id";
const VOICE_ID_QUERY: &str = "voice_id";

#[derive(Clone, Debug)]
pub struct DeleteHistoryItem(HistoryItemID);

impl DeleteHistoryItem {
    pub fn new<T: Into<String>>(history_item_id: T) -> Self {
        Self(HistoryItemID(history_item_id.into()))
    }
}

impl Endpoint for DeleteHistoryItem {
    type ResponseBody = StatusResponseBody;
    fn method(&self) -> Method {
        Method::DELETE
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", HISTORY_PATH, self.0 .0));
        url
    }
}

/// Download one or more history items.
/// If one history item ID is provided, we will return a single audio file.
/// If more than one history item IDs are provided, we will provide the history items packed into a .zip file.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::default()?;
///     let history_item_ids = client
///         .hit(GetGeneratedItems::new(HistoryQuery::default()))
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
#[derive(Clone, Debug)]
pub struct DownloadHistoryItems(DownloadBody);

impl DownloadHistoryItems {
    pub fn new(body: DownloadBody) -> Self {
        Self(body)
    }
}

impl Endpoint for DownloadHistoryItems {
    type ResponseBody = Bytes;
    fn method(&self) -> Method {
        Method::POST
    }
    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.0)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}{}", HISTORY_PATH, DOWNLOAD_PATH));
        url
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
        Self  {
            history_item_ids,
            output_format: DownloadOutputFormat::default(),
        }
    }
    pub fn with_output_format(mut self, output_format: DownloadOutputFormat) -> Self {
        self.output_format = output_format;
        self
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadOutputFormat {
    #[default]
    Mp3,
    Wav,
}

impl DownloadOutputFormat {
    fn is_mp3(&self) -> bool {
        match self {
            DownloadOutputFormat::Mp3 => true,
            DownloadOutputFormat::Wav => false,
        }
    }
}

/// Get the audio file of a history item.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::default()?;
///     let query = HistoryQuery::default().with_page_size(1);
///     let resp = client.hit(GetGeneratedItems::new(query))
///         .await?;
///     let item_id = resp
///         .history()
///         .first().unwrap()
///         .history_item_id();
///     let audio = client.hit(GetAudio::new(item_id)).await?;
///     play(audio)?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetAudio(HistoryItemID);

impl GetAudio {
    pub fn new<T: Into<String>>(history_item_id: T) -> Self {
        Self(HistoryItemID(history_item_id.into()))
    }
}

impl Endpoint for GetAudio {
    type ResponseBody = Bytes;
    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}{}", HISTORY_PATH, self.0 .0, AUDIO_PATH));
        url
    }
}

/// Get the generated items endpoint.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let query = HistoryQuery::default()
///         .with_page_size(10)
///         .with_voice_id(PreMadeVoiceID::Alice);
///     let endpoint = GetGeneratedItems::new(query);
///     let resp = c.hit(endpoint).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetGeneratedItems(HistoryQuery);

impl GetGeneratedItems {
    pub fn new(query: HistoryQuery) -> Self {
        Self(query)
    }
}

impl Endpoint for GetGeneratedItems {
    type ResponseBody = GeneratedItems;
    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut ego = self.clone();
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(HISTORY_PATH);
        url.set_query(ego.0.join().as_deref());
        url
    }
}

#[derive(Clone, Debug)]
pub struct GetHistoryItem(HistoryItemID);

impl GetHistoryItem {
    pub fn new<T: Into<String>>(history_item_id: T) -> Self {
        Self(HistoryItemID(history_item_id.into()))
    }
}

impl Endpoint for GetHistoryItem {
    type ResponseBody = HistoryItem;
    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", HISTORY_PATH, self.0 .0));
        url
    }
}

#[derive(Clone, Debug, Default)]
pub struct HistoryQuery {
    pub page_size: Option<String>,
    pub start_after_history_item_id: Option<String>,
    pub voice_id: Option<String>,
}

impl HistoryQuery {
    pub fn with_page_size(mut self, page_size: u16) -> Self {
        self.page_size = Some(format!("{}={}", PAGE_SIZE_QUERY, page_size));
        self
    }
    pub fn with_start_after_history_item_id(mut self, start_after_history_item_id: &str) -> Self {
        self.start_after_history_item_id = Some(format!(
            "{}={}",
            START_AFTER_HISTORY_ITEM_ID_QUERY, start_after_history_item_id
        ));
        self
    }

    pub fn with_voice_id<T: Into<String>>(mut self, voice_id: T) -> Self {
        self.voice_id = Some(format!("{}={}", VOICE_ID_QUERY, voice_id.into()));
        self
    }
    pub fn join(&mut self) -> Option<String> {
        let mut result = String::new();

        if let Some(value) = self.page_size.take() {
            result.push_str(&value);
        }
        if let Some(value) = self.start_after_history_item_id.take() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.voice_id.take() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GeneratedItems {
    history: Vec<HistoryItem>,
    last_history_item_id: String,
    has_more: bool,
}

impl GeneratedItems {
    pub fn history(&self) -> &Vec<HistoryItem> {
        &self.history
    }
    pub fn last_history_item_id(&self) -> &str {
        &self.last_history_item_id
    }
    pub fn has_more(&self) -> bool {
        self.has_more
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct HistoryItem {
    history_item_id: String,
    request_id: String,
    voice_id: String,
    voice_name: String,
    voice_category: Option<String>,
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
    source: Option<String>,
}
impl HistoryItem {
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
    pub fn voice_category(&self) -> Option<&str> {
        self.voice_category.as_deref()
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
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Feedback {
    thumbs_up: bool,
    feedback: String,
    emotions: bool,
    inaccurate_clone: bool,
    glitches: bool,
    audio_quality: bool,
    review_status: String,
    other: bool,
}
impl Feedback {
    pub fn thumbs_up(&self) -> bool {
        self.thumbs_up
    }
    pub fn feedback(&self) -> &str {
        &self.feedback
    }
    pub fn emotions(&self) -> bool {
        self.emotions
    }
    pub fn inaccurate_clone(&self) -> bool {
        self.inaccurate_clone
    }
    pub fn glitches(&self) -> bool {
        self.glitches
    }
    pub fn audio_quality(&self) -> bool {
        self.audio_quality
    }
    pub fn review_status(&self) -> &str {
        &self.review_status
    }
    pub fn other(&self) -> bool {
        self.other
    }
}
