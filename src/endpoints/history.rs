use super::*;
use crate::endpoints::voice::VoiceSettings;

pub const HISTORY_PATH: &str = "/v1/history";
pub const AUDIO_PATH: &str = "/audio";
pub const DOWNLOAD_PATH: &str = "/download";
pub const PAGE_SIZE_QUERY: &str = "page_size";
pub const HISTORY_ITEM_IDS: &str = "history_item_ids";
pub const START_AFTER_HISTORY_ITEM_ID_QUERY: &str = "start_after_history_item_id";
pub const VOICE_ID_QUERY: &str = "voice_id";

#[derive(Clone, Debug)]
pub struct DeleteHistoryItem(pub HistoryItemID);
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
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::history::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let history_item_ids = c
///         .hit(GetGeneratedItems(HistoryQuery::default())) // Returns 100 history items by default
///         .await?
///         .history()
///         .iter()
///         .map(|i| i.history_item_id().to_string())
///         .collect::<Vec<String>>();
///     let items = c
///         .hit(DownloadHistoryItems(DownloadBody {
///             history_item_ids,
///             output_format: OutputFormat::default(),
///         }))
///         .await?;
///     save("last_100_items.zip", items)?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct DownloadHistoryItems(pub DownloadBody);

impl Endpoint for DownloadHistoryItems {
    type ResponseBody = Bytes;
    fn method(&self) -> Method {
        Method::POST
    }
    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.0)?))

    }
    //fn json_request_body(&self) -> Option<Result<serde_json::Value>> {
    //    Some(serde_json::to_value(&self.0).map_err(Into::into))
    //}
    async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(&format!("{}{}", HISTORY_PATH, DOWNLOAD_PATH));
        url
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct DownloadBody {
    pub history_item_ids: Vec<String>,
    /// Output format to transcode the audio file, can be wav or default (Mp3).
    #[serde(skip_serializing_if = "OutputFormat::is_mp3")]
    pub output_format: OutputFormat,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Mp3,
    Wav,
}

impl OutputFormat {
    fn is_mp3(&self) -> bool {
        match self {
            OutputFormat::Mp3 => true,
            OutputFormat::Wav => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GetAudio(pub HistoryItemID);
impl Endpoint for GetAudio {
    type ResponseBody = Bytes;
    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }
    async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(&format!("{}/{}{}", HISTORY_PATH, self.0 .0, AUDIO_PATH));
        url
    }
}

#[derive(Clone, Debug)]
pub struct GetGeneratedItems(pub HistoryQuery);
impl Endpoint for GetGeneratedItems {
    type ResponseBody = GeneratedItems;
    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }
    async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> reqwest::Url {
        let mut ego = self.clone();
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(HISTORY_PATH);
        url.set_query(ego.0.join().as_deref());
        url
    }
}

#[derive(Clone, Debug)]
pub struct GetHistoryItem(pub HistoryItemID);
impl Endpoint for GetHistoryItem {
    type ResponseBody = HistoryItem;
    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }
    async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(&format!("{}/{}", HISTORY_PATH, self.0 .0));
        url
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct HistoryItemID(String);

impl From<&str> for HistoryItemID {
    fn from(s: &str) -> Self {
        Self(s.to_string())
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
    pub fn with_start_after_history_item_id(
        mut self,
        start_after_history_item_id: HistoryItemID,
    ) -> Self {
        self.start_after_history_item_id = Some(format!(
            "{}={}",
            START_AFTER_HISTORY_ITEM_ID_QUERY,
            start_after_history_item_id.0.as_str()
        ));
        self
    }

    pub fn with_voice_id(mut self, voice_id: VoiceID) -> Self {
        self.voice_id = Some(format!("{}={}", VOICE_ID_QUERY, voice_id.0));
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
