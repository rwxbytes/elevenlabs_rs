//! The voice endpoints
use super::*;
use crate::endpoints::admin::voice_library::SharedVoice;
pub use crate::shared::{
    FineTuning, FineTuningState, SafetyControl, Sharing, VerifiedLanguage, VoiceCategory,
    VoiceSample, VoiceSettings, VoiceVerification,
};
use std::collections::HashMap;
use std::path::Path;
use strum::Display;

/// Gets a list of all available voices for a user.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{GetVoices, GetVoicesQuery, VoiceCategory, VoiceType};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let query = GetVoicesQuery::default()
///       .with_voice_type(VoiceType::Community)
///       .with_voice_category(VoiceCategory::Professional)
///       .with_page_size(2);
///    let endpoint = GetVoices::with_query(query);
///    let voices = c.hit(endpoint).await?;
///    println!("{:#?}", voices);
///    Ok(())
/// }
/// ```
/// See the [Get Voices API reference](https://elevenlabs.io/docs/api-reference/voices/get-all)
#[derive(Clone, Debug, Default)]
pub struct GetVoices {
    query: Option<GetVoicesQuery>,
}

impl GetVoices {
    pub fn with_query(query: GetVoicesQuery) -> Self {
        GetVoices { query: Some(query) }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetVoicesQuery {
    params: QueryValues,
}
impl GetVoicesQuery {
    pub fn with_next_page_token(mut self, next_page_token: impl Into<String>) -> Self {
        self.params
            .push(("next_page_token", next_page_token.into()));
        self
    }
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }
    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.params.push(("sort", sort.into()));
        self
    }

    pub fn with_sort_direction(mut self, sort_direction: impl Into<String>) -> Self {
        self.params.push(("sort_direction", sort_direction.into()));
        self
    }

    pub fn with_voice_type(mut self, voice_type: VoiceType) -> Self {
        self.params.push(("voice_type", voice_type.to_string()));
        self
    }

    pub fn with_voice_category(mut self, voice_category: VoiceCategory) -> Self {
        self.params
            .push(("voice_category", voice_category.to_string()));
        self
    }

    pub fn with_fine_tuning_state(mut self, fine_tuning_state: FineTuningState) -> Self {
        self.params
            .push(("fine_tuning_state", fine_tuning_state.to_string()));
        self
    }

    pub fn include_total_count(mut self, include_total_count: bool) -> Self {
        self.params
            .push(("include_total_count", include_total_count.to_string()));
        self
    }
}

impl ElevenLabsEndpoint for GetVoices {
    const PATH: &'static str = "/v2/voices";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetVoicesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Display, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum VoiceType {
    Personal,
    Community,
    Default,
    Workspace,
}

#[derive(Debug, Clone,)]
pub struct GetDefaultVoiceSettings;

impl ElevenLabsEndpoint for GetDefaultVoiceSettings {
    const PATH: &'static str = "/v1/voices/settings/default";

    const METHOD: Method = Method::GET;

    type ResponseBody = VoiceSettings;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}


/// Returns the [`VoiceSettings`] for a voice.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{GetVoiceSettings, VoiceSettings};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::from_env()?;
///   // Or for a premade voice: GetVoiceSettings::new(DefaultVoice::Brian)
///   let voice_settings = c.hit(GetVoiceSettings::new("some_voice_id")).await?;
///   println!("{:#?}", voice_settings);
///   Ok(())
/// }
/// ```
/// See the [Get Voice Settings API reference](https://elevenlabs.io/docs/api-reference/voices/get-settings)
#[derive(Clone, Debug)]
pub struct GetVoiceSettings {
    voice_id: String,
}

impl GetVoiceSettings {
    pub fn new(voice_id: impl Into<String>) -> Self {
        GetVoiceSettings {
            voice_id: voice_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetVoiceSettings {
    const PATH: &'static str = "/v1/voices/:voice_id/settings";

    const METHOD: Method = Method::GET;

    type ResponseBody = VoiceSettings;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Returns metadata about a specific voice.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result, DefaultVoice};
/// use elevenlabs_rs::endpoints::admin::voice::{GetVoice, GetVoiceResponse};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    // Or for IVC's & PVC's: GetVoice::new("some_voice_id")
///    let resp = c.hit(GetVoice::new(DefaultVoice::Brian)).await?;
///    println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Get Voice API reference](https://elevenlabs.io/docs/api-reference/voices/get)
#[derive(Clone, Debug)]
pub struct GetVoice {
    voice_id: String,
}

impl GetVoice {
    pub fn new(voice_id: impl Into<String>) -> Self {
        GetVoice {
            voice_id: voice_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetVoice {
    const PATH: &'static str = "/v1/voices/:voice_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetVoiceResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Deletes a voice by its ID.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::DeleteVoice;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::from_env()?;
///   let endpoint = DeleteVoice::new("some_voice_id");
///   let resp = c.hit(endpoint).await?;
///   println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Delete Voice API reference](https://elevenlabs.io/docs/api-reference/voices/delete)
#[derive(Clone, Debug)]
pub struct DeleteVoice {
    voice_id: String,
}

impl DeleteVoice {
    pub fn new(voice_id: impl Into<String>) -> Self {
        DeleteVoice {
            voice_id: voice_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for DeleteVoice {
    const PATH: &'static str = "/v1/voices/:voice_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Edit your settings for a specific voice.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{EditVoiceSettings, EditVoiceSettingsBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::from_env()?;
///   let body = EditVoiceSettingsBody::default()
///        .with_speed(0.8)
///        .use_speaker_boost(true);
///   let endpoint = EditVoiceSettings::new("some_voice_id", body);
///   let resp = c.hit(endpoint).await?;
///   println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Edit Voice Settings API reference](https://elevenlabs.io/docs/api-reference/voices/edit-settings)
#[derive(Clone, Debug)]
pub struct EditVoiceSettings {
    voice_id: String,
    body: EditVoiceSettingsBody,
}

impl EditVoiceSettings {
    pub fn new(voice_id: impl Into<String>, body: EditVoiceSettingsBody) -> Self {
        EditVoiceSettings {
            voice_id: voice_id.into(),
            body,
        }
    }
}

impl ElevenLabsEndpoint for EditVoiceSettings {
    const PATH: &'static str = "/v1/voices/:voice_id/settings/edit";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body.inner)?))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Edit voice settings body
#[derive(Clone, Debug, Default)]
pub struct EditVoiceSettingsBody {
    inner: VoiceSettings,
}

impl EditVoiceSettingsBody {
    pub fn with_similarity_boost(mut self, similarity_boost: f32) -> Self {
        self.inner.similarity_boost = Some(similarity_boost);
        self
    }
    pub fn with_stability(mut self, stability: f32) -> Self {
        self.inner.stability = Some(stability);
        self
    }
    pub fn with_style(mut self, style: f32) -> Self {
        self.inner.style = Some(style);
        self
    }
    pub fn use_speaker_boost(mut self, use_speaker_boost: bool) -> Self {
        self.inner.use_speaker_boost = Some(use_speaker_boost);
        self
    }
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.inner.speed = Some(speed);
        self
    }
}

/// Add a new voice to your collection of voices in VoiceLab.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{AddVoice, VoiceBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let samples = vec!["some_file_path.mp3".to_string(), "another.mp3".into(),];
///     let labels = vec![("age".to_string(), "old".into()), ("gender".into(), "male".into())];
///     let body = VoiceBody::add("John Doe", samples)
///         .with_description("A public intellectual")
///         .with_labels(labels)
///         .with_remove_background_noise(true);
///     let endpoint = AddVoice::new(body);
///     let resp = c.hit(endpoint).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See the [Add Voice API reference](https://elevenlabs.io/docs/api-reference/voices/add)
#[derive(Clone, Debug)]
pub struct AddVoice {
    body: VoiceBody,
}

impl AddVoice {
    pub fn new(body: VoiceBody) -> Self {
        AddVoice { body }
    }
}

impl ElevenLabsEndpoint for AddVoice {
    const PATH: &'static str = "/v1/voices/add";

    const METHOD: Method = Method::POST;

    type ResponseBody = AddVoiceResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        let form = Form::try_from(self.body.clone())?;
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Add voice body
#[derive(Clone, Debug)]
pub struct VoiceBody {
    name: String,
    files: Vec<String>,
    description: Option<String>,
    labels: Option<Vec<(String, String)>>,
    remove_background_noise: Option<bool>,
}

impl VoiceBody {
    pub fn add(name: &str, files: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            files,
            description: None,
            labels: None,
            remove_background_noise: None,
        }
    }
    pub fn edit(name: &str) -> Self {
        Self {
            name: name.to_string(),
            files: vec![],
            description: None,
            labels: None,
            remove_background_noise: None,
        }
    }
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    pub fn with_labels(mut self, labels: Vec<(String, String)>) -> Self {
        self.labels = Some(labels);
        self
    }
    pub fn with_remove_background_noise(mut self, remove: bool) -> Self {
        self.remove_background_noise = Some(remove);
        self
    }
}

impl TryFrom<VoiceBody> for Form {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: VoiceBody) -> Result<Self> {
        let mut form = Form::new();
        form = form.text("name", body.name);

        for file_path in body.files {
            let fp = Path::new(&file_path);
            let audio_bytes = std::fs::read(fp)?;
            let mut part = Part::bytes(audio_bytes);
            let file_path_str = fp.to_str().ok_or("Path is not valid UTF-8")?;
            part = part.file_name(file_path_str.to_string());

            let mime_subtype = fp
                .extension()
                .ok_or("File extension not found")?
                .to_str()
                .ok_or("File extension is not valid UTF-8")?;
            let mime = format!("audio/{}", mime_subtype);
            part = part.mime_str(&mime)?;
            form = form.part("files", part);
        }

        if let Some(description) = body.description {
            form = form.text("description", description);
        }

        if let Some(labels) = body.labels {
            let label_map: HashMap<_, _> = labels.into_iter().collect();
            form = form.text("labels", serde_json::to_string(&label_map)?);
        }

        if let Some(remove_background_noise) = body.remove_background_noise {
            form = form.text(
                "remove_background_noise",
                remove_background_noise.to_string(),
            );
        }

        Ok(form)
    }
}

/// Add voice response
#[derive(Clone, Debug, Deserialize)]
pub struct AddVoiceResponse {
    pub voice_id: String,
    pub requires_verification: bool,
}

/// Edit a voice created by you.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{EditVoice, VoiceBody};
/// #[tokio::main]
///
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///
///    let samples = vec!["sample.mp3".to_string(), "sample_2.mp3".into()];
///    let labels = vec![
///        ("age".to_string(), "old".into()),
///        ("gender".into(), "male".into()),
///    ];
///    let body = VoiceBody::edit("a new name")
///        .with_description("A public intellectual")
///        .with_labels(labels)
///        .with_remove_background_noise(false);
///    let endpoint = EditVoice::new("agent_id", body);
///    let resp = client.hit(endpoint).await?;
///   println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Edit Voice API reference](https://elevenlabs.io/docs/api-reference/voices/edit)
#[derive(Clone, Debug)]
pub struct EditVoice {
    voice_id: String,
    body: VoiceBody,
}

impl EditVoice {
    pub fn new(voice_id: impl Into<String>, body: VoiceBody) -> Self {
        EditVoice {
            voice_id: voice_id.into(),
            body,
        }
    }
}

impl ElevenLabsEndpoint for EditVoice {
    const PATH: &'static str = "/v1/voices/:voice_id/edit";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        let form = Form::try_from(self.body.clone())?;
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Get all voices response body
#[derive(Clone, Debug, Deserialize)]
pub struct GetVoicesResponse {
    pub voices: Vec<GetVoiceResponse>,
    pub has_more: bool,
    pub total_count: u32,
    pub next_page_token: Option<String>,
}

/// Voice response body
#[derive(Clone, Debug, Deserialize)]
pub struct GetVoiceResponse {
    pub voice_id: String,
    pub name: Option<String>,
    pub samples: Option<Vec<VoiceSample>>,
    pub fine_tuning: Option<FineTuning>,
    pub category: Option<VoiceCategory>,
    pub labels: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub preview_url: Option<String>,
    pub available_for_tiers: Option<Vec<String>>,
    pub settings: Option<VoiceSettings>,
    pub sharing: Option<Sharing>,
    pub high_quality_base_model_ids: Option<Vec<String>>,
    pub verified_languages: Option<Vec<VerifiedLanguage>>,
    pub safety_control: Option<SafetyControl>,
    pub voice_verification: Option<VoiceVerification>,
    pub permission_on_resource: Option<String>,
    pub is_owner: Option<bool>,
    pub is_legacy: Option<bool>,
    pub is_mixed: Option<bool>,
    pub created_at_unix: Option<u64>,
}

impl<'a> IntoIterator for &'a GetVoicesResponse {
    type Item = &'a GetVoiceResponse;
    type IntoIter = std::slice::Iter<'a, GetVoiceResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.voices.iter()
    }
}

impl IntoIterator for GetVoicesResponse {
    type Item = GetVoiceResponse;
    type IntoIter = std::vec::IntoIter<GetVoiceResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.voices.into_iter()
    }
}

/// Returns a list of shared voices similar to the provided audio sample.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{ListSimilarVoices, ListSimilarVoicesBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let body = ListSimilarVoicesBody::new("audio_sample.mp3")
///     .with_similarity_threshold(1.75)
///     .with_top_k(5);
///
///    let endpoint = ListSimilarVoices::new(body);
///    let resp = c.hit(endpoint).await?;
///
///    for shared_voice in resp {
///       println!("{:#?}", shared_voice);
///    }
///
///    Ok(())
/// }
/// ```
/// See the [List Similar Voices API reference](https://elevenlabs.io/docs/api-reference/voices/get-similar-library-voices)
#[derive(Clone, Debug)]
pub struct ListSimilarVoices {
    body: ListSimilarVoicesBody,
}

impl ListSimilarVoices {
    pub fn new(body: ListSimilarVoicesBody) -> Self {
        ListSimilarVoices { body }
    }
}

impl ElevenLabsEndpoint for ListSimilarVoices {
    const PATH: &'static str = "/v1/similar-voices";

    const METHOD: Method = Method::POST;

    type ResponseBody = ListSimilarVoicesResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        let form = Form::try_from(self.body.clone())?;
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// List similar voices body
///
///
/// `audio_sample` - Required
///
/// Path to the audio file to compare with library voices.
///
/// `similarity_threshold` - Optional
///
/// Threshold for voice similarity between provided sample and library voices.
/// Must be in range <0, 2>. The smaller the value the more similar voices will be returned.
///
/// `top_k` - Optional
///
/// Number of most similar voices to return.
/// If similarity_threshold is provided, less than this number of voices may be returned. Must be in range <1, 100>.
#[derive(Clone, Debug)]
pub struct ListSimilarVoicesBody {
    audio_sample: String,
    similarity_threshold: Option<f32>,
    top_k: Option<u32>,
}

impl ListSimilarVoicesBody {
    pub fn new(audio_sample: &str) -> Self {
        ListSimilarVoicesBody {
            audio_sample: audio_sample.to_string(),
            similarity_threshold: None,
            top_k: None,
        }
    }
    pub fn with_similarity_threshold(mut self, similarity_threshold: f32) -> Self {
        self.similarity_threshold = Some(similarity_threshold);
        self
    }
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }
}

impl TryFrom<ListSimilarVoicesBody> for Form {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: ListSimilarVoicesBody) -> Result<Self> {
        let mut form = Form::new();
        let audio_bytes = std::fs::read(&body.audio_sample)?;
        let mut part = Part::bytes(audio_bytes);
        let file_path_str = Path::new(&body.audio_sample)
            .to_str()
            .ok_or("Path is not valid UTF-8")?;
        part = part.file_name(file_path_str.to_string());
        part = part.mime_str("audio/mpeg")?;
        form = form.part("audio_file", part);

        if let Some(similarity_threshold) = body.similarity_threshold {
            form = form.text("similarity_threshold", similarity_threshold.to_string());
        }

        if let Some(top_k) = body.top_k {
            form = form.text("top_k", top_k.to_string());
        }

        Ok(form)
    }
}

/// List similar voices response
#[derive(Clone, Debug, Deserialize)]
pub struct ListSimilarVoicesResponse {
    pub voices: Vec<SharedVoice>,
    pub has_more: bool,
    pub last_sort_id: Option<String>,
}

impl<'a> IntoIterator for &'a ListSimilarVoicesResponse {
    type Item = &'a SharedVoice;
    type IntoIter = std::slice::Iter<'a, SharedVoice>;

    fn into_iter(self) -> Self::IntoIter {
        self.voices.iter()
    }
}

impl IntoIterator for ListSimilarVoicesResponse {
    type Item = SharedVoice;
    type IntoIter = std::vec::IntoIter<SharedVoice>;

    fn into_iter(self) -> Self::IntoIter {
        self.voices.into_iter()
    }
}
