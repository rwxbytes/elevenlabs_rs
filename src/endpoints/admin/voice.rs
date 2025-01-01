#![allow(dead_code)]
//! The voice endpoints
use super::*;
use std::collections::HashMap;
use std::path::Path;

/// Gets a list of all available voices for a user.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{GetVoices, GetVoicesQuery};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let query = GetVoicesQuery::default().show_legacy(true);
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

/// # Query Parameters
///
/// `show_legacy` - bool
///
/// Default: false
///
/// If set to true, [`LegacyVoice`]s will be included in [`GetVoicesResponse`]
#[derive(Clone, Debug, Default)]
pub struct GetVoicesQuery {
    params: QueryValues,
}
impl GetVoicesQuery {
    pub fn show_legacy(mut self, show_legacy: bool) -> Self {
        self.params.push(("show_legacy", show_legacy.to_string()));
        self
    }
}

impl ElevenLabsEndpoint for GetVoices {
    const PATH: &'static str = "/v1/voices";
    const METHOD: Method = Method::GET;

    type ResponseBody = GetVoicesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }
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
///   let c = ElevenLabsClient::default()?;
///   // Or for a premade voice: GetVoiceSettings::new(DefaultVoice::Brian)
///   let voice_settings = c.hit(GetVoiceSettings::new("some_voice_id")).await?;
///   println!("{:#?}", voice_settings);
///   Ok(())
/// }
/// ```
/// See the [Get Voice Settings API reference](https://elevenlabs.io/docs/api-reference/voices/get-settings)
#[derive(Clone, Debug)]
pub struct GetVoiceSettings {
    voice_id: VoiceID,
}

impl GetVoiceSettings {
    pub fn new(voice_id: impl Into<VoiceID>) -> Self {
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
        vec![self.voice_id.as_path_param()]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Returns metadata about a specific voice.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{GetVoice, GetVoiceResponse, DefaultVoice};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    // Or for IVC's & PVC's: GetVoice::new("some_voice_id")
///    let resp = c.hit(GetVoice::new(DefaultVoice::Brian)).await?;
///    println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Get Voice API reference](https://elevenlabs.io/docs/api-reference/voices/get)
#[derive(Clone, Debug)]
pub struct GetVoice {
    voice_id: VoiceID,
    query: Option<GetVoiceQuery>,
}

impl GetVoice {
    pub fn new(voice_id: impl Into<VoiceID>) -> Self {
        GetVoice {
            voice_id: voice_id.into(),
            query: None,
        }
    }

    pub fn with_query(voice_id: VoiceID, query: GetVoiceQuery) -> Self {
        GetVoice {
            voice_id,
            query: Some(query),
        }
    }
}

/// # Query Parameters
/// `with_settings` - bool
///
/// Default: false
///
/// If set to true, the [`VoiceSettings`] will be included in the [`GetVoiceResponse`]
#[derive(Clone, Debug, Default)]
pub struct GetVoiceQuery {
    params: QueryValues,
}

impl GetVoiceQuery {
    pub fn with_settings(mut self, with_settings: bool) -> Self {
        self.params
            .push(("with_settings", with_settings.to_string()));
        self
    }
}

impl ElevenLabsEndpoint for GetVoice {
    const PATH: &'static str = "/v1/voices/:voice_id";
    const METHOD: Method = Method::GET;
    type ResponseBody = GetVoiceResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.as_path_param()]
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
///   let c = ElevenLabsClient::default()?;
///   let endpoint = DeleteVoice::new("some_voice_id");
///   let resp = c.hit(endpoint).await?;
///   println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Delete Voice API reference](https://elevenlabs.io/docs/api-reference/voices/delete)
#[derive(Clone, Debug)]
pub struct DeleteVoice {
    voice_id: VoiceID,
}

impl DeleteVoice {
    pub fn new(voice_id: impl Into<VoiceID>) -> Self {
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
        vec![self.voice_id.as_path_param()]
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
///   let c = ElevenLabsClient::default()?;
///   let body = EditVoiceSettingsBody::new(1.0, 0.85)
///        .with_style(0.25)
///        .with_use_speaker_boost(true);
///   let endpoint = EditVoiceSettings::new("some_voice_id", body);
///   let resp = c.hit(endpoint).await?;
///   println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Edit Voice Settings API reference](https://elevenlabs.io/docs/api-reference/voices/edit-settings)
#[derive(Clone, Debug)]
pub struct EditVoiceSettings {
    voice_id: VoiceID,
    body: EditVoiceSettingsBody,
}

impl EditVoiceSettings {
    pub fn new(voice_id: impl Into<VoiceID>, body: EditVoiceSettingsBody) -> Self {
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
        vec![self.voice_id.as_path_param()]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body.inner)?))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Edit voice settings body
#[derive(Clone, Debug)]
pub struct EditVoiceSettingsBody {
    inner: VoiceSettings,
}

impl EditVoiceSettingsBody {
    pub fn new(stability: f32, similarity: f32) -> Self {
        EditVoiceSettingsBody {
            inner: VoiceSettings::new(stability, similarity),
        }
    }

    pub fn with_style(mut self, style: f32) -> Self {
        self.inner.style = Some(style);
        self
    }
    pub fn with_use_speaker_boost(mut self, use_speaker_boost: bool) -> Self {
        self.inner.use_speaker_boost = Some(use_speaker_boost);
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
///     let c = ElevenLabsClient::default()?;
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
    voice_id: String,
    requires_verification: bool,
}

impl AddVoiceResponse {
    pub fn get_voice_id(&self) -> &String {
        &self.voice_id
    }

    pub fn requires_verification(&self) -> bool {
        self.requires_verification
    }
}

/// Edit a voice created by you.
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice::{EditVoice, VoiceBody};
/// #[tokio::main]
///
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::default()?;
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
    voice_id: VoiceID,
    body: VoiceBody,
}

impl EditVoice {
    pub fn new(voice_id: impl Into<VoiceID>, body: VoiceBody) -> Self {
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
        vec![self.voice_id.as_path_param()]
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
    voices: Vec<GetVoiceResponse>,
}

impl GetVoicesResponse {
    pub fn get_voices(&self) -> &Vec<GetVoiceResponse> {
        &self.voices
    }
}

/// Voice response body
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GetVoiceResponse {
    voice_id: String,
    name: Option<String>,
    samples: Option<Vec<VoiceSample>>,
    //TODO: implement type
    fine_tuning: Option<Value>,
    category: Option<VoiceCategory>,
    labels: Option<HashMap<String, String>>,
    description: Option<String>,
    preview_url: Option<String>,
    available_for_tiers: Option<Vec<String>>,
    settings: Option<VoiceSettings>,
    //TODO: implement type
    sharing: Option<Value>,
    high_quality_base_model_ids: Option<Vec<String>>,
    safety_control: Option<SafetyControl>,
    // TODO: implement type
    voice_verification: Option<Value>,
    permission_on_resource: Option<String>,
    is_owner: Option<bool>,
    is_legacy: Option<bool>,
    is_mixed: Option<bool>,
    created_at_unix: Option<u64>,
}
/// Voice sample
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct VoiceSample {
    sample_id: String,
    file_name: String,
    mime_type: String,
    size_bytes: Option<u64>,
    hash: String,
}

impl VoiceSample {
    pub fn get_sample_id(&self) -> &String {
        &self.sample_id
    }
    pub fn get_file_name(&self) -> &String {
        &self.file_name
    }
    pub fn get_mime_type(&self) -> &String {
        &self.mime_type
    }
    pub fn get_size_bytes(&self) -> Option<u64> {
        self.size_bytes
    }
    pub fn get_hash(&self) -> &String {
        &self.hash
    }
}

/// Voice category
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VoiceCategory {
    Generated,
    Cloned,
    Premade,
    Professional,
    Famous,
    HighQuality,
}

/// Voice settings
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VoiceSettings {
    similarity_boost: f32,
    stability: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_speaker_boost: Option<bool>,
}

impl VoiceSettings {
    pub fn new(stability: f32, similarity: f32) -> Self {
        VoiceSettings {
            similarity_boost: similarity,
            stability,
            style: None,
            use_speaker_boost: None,
        }
    }
    pub fn with_similarity_boost(mut self, similarity_boost: f32) -> Self {
        self.similarity_boost = similarity_boost;
        self
    }
    pub fn with_stability(mut self, stability: f32) -> Self {
        self.stability = stability;
        self
    }
    pub fn with_style(mut self, style: f32) -> Self {
        self.style = Some(style);
        self
    }
    pub fn with_use_speaker_boost(mut self, use_speaker_boost: bool) -> Self {
        self.use_speaker_boost = Some(use_speaker_boost);
        self
    }

    pub fn get_similarity_boost(&self) -> f32 {
        self.similarity_boost
    }

    pub fn get_stability(&self) -> f32 {
        self.stability
    }

    pub fn get_style(&self) -> Option<f32> {
        self.style
    }

    pub fn get_use_speaker_boost(&self) -> Option<bool> {
        self.use_speaker_boost
    }
}

impl Default for VoiceSettings {
    fn default() -> Self {
        VoiceSettings {
            similarity_boost: 0.75,
            stability: 0.5,
            style: Some(0.5),
            use_speaker_boost: Some(true),
        }
    }
}

/// Safety control
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SafetyControl {
    None,
    Ban,
    Captcha,
    CaptchaAndModeration,
    EnterpriseBan,
    EnterpriseCaptcha,
}

impl GetVoiceResponse {
    pub fn get_voice_id(&self) -> &String {
        &self.voice_id
    }
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    pub fn get_samples(&self) -> Option<&Vec<VoiceSample>> {
        self.samples.as_ref()
    }
    pub fn get_category(&self) -> Option<&VoiceCategory> {
        self.category.as_ref()
    }
    pub fn get_labels(&self) -> Option<&HashMap<String, String>> {
        self.labels.as_ref()
    }
    pub fn get_description(&self) -> Option<&String> {
        self.description.as_ref()
    }
    pub fn get_preview_url(&self) -> Option<&String> {
        self.preview_url.as_ref()
    }
    pub fn get_settings(&self) -> Option<&VoiceSettings> {
        self.settings.as_ref()
    }
    pub fn get_safety_control(&self) -> Option<&SafetyControl> {
        self.safety_control.as_ref()
    }
    pub fn get_created_at_unix(&self) -> Option<u64> {
        self.created_at_unix
    }

    pub fn is_owner(&self) -> Option<bool> {
        self.is_owner
    }
    pub fn is_legacy(&self) -> Option<bool> {
        self.is_legacy
    }
    pub fn is_mixed(&self) -> Option<bool> {
        self.is_mixed
    }
    pub fn get_available_for_tiers(&self) -> Option<&Vec<String>> {
        self.available_for_tiers.as_ref()
    }
    pub fn get_high_quality_base_model_ids(&self) -> Option<&Vec<String>> {
        self.high_quality_base_model_ids.as_ref()
    }
    pub fn get_voice_verification(&self) -> Option<&Value> {
        self.voice_verification.as_ref()
    }
    pub fn get_permission_on_resource(&self) -> Option<&String> {
        self.permission_on_resource.as_ref()
    }
    pub fn get_sharing(&self) -> Option<&Value> {
        self.sharing.as_ref()
    }
    pub fn get_fine_tuning(&self) -> Option<&Value> {
        self.fine_tuning.as_ref()
    }
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
///    let c = ElevenLabsClient::default()?;
///    let body = ListSimilarVoicesBody::new("audio_sample.mp3")
///     .with_similarity_threshold(1.75)
///     .with_top_k(5);
///    let endpoint = ListSimilarVoices::new(body);
///    let resp = c.hit(endpoint).await?;
///    println!("{:#?}", resp);
/// Ok(())
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
    // TODO: implement type
    voices: Vec<Value>,
}
