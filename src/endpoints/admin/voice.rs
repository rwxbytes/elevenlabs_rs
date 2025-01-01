#![allow(dead_code)]
//! The voice endpoints
use super::*;
use crate::error::Error;
use std::collections::HashMap;
use std::path::Path;

const EDIT_VOICE_PATH: &str = "/edit";
const EDIT_VOICE_SETTINGS_PATH: &str = "/settings/edit";
const VOICE_SETTINGS_PATH: &str = "/settings";
const WITH_SETTINGS_QUERY: &str = "with_settings=true";

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
///    let query = GetVoicesQuery::new().show_legacy(true);
///    let endpoint = GetVoices::new().with_query(query);
///    let voices = c.hit(endpoint).await?;
///    println!("{:#?}", voices);
///    Ok(())
/// }
/// ```
/// See the [Get Voices API reference](https://elevenlabs.io/docs/api-reference/voices/get-all)
#[derive(Clone, Debug)]
pub struct GetVoices {
    query: Option<GetVoicesQuery>,
}

impl GetVoices {
    pub fn new() -> Self {
        GetVoices { query: None }
    }
    pub fn with_query(mut self, query: GetVoicesQuery) -> Self {
        self.query = Some(query);
        self
    }
}


/// # Query Parameters
///
/// `show_legacy` - bool
///
/// Default: false
///
/// If set to true, [`LegacyVoice`]s will be included in [`GetVoicesResponse`]
#[derive(Clone, Debug)]
pub struct GetVoicesQuery {
    params: QueryValues,
}
impl GetVoicesQuery {
    pub fn new() -> Self {
        GetVoicesQuery { params: vec![] }
    }
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

///// Get a voice endpoint
///// # Example
///// ```no_run
///// use elevenlabs_rs::*;
///// use elevenlabs_rs::endpoints::voice::{GetVoice, VoiceResponseBody};
/////
///// #[tokio::main]
///// async fn main() -> Result<()> {
/////    let c = ElevenLabsClient::default()?;
/////    // Or for IVC's & PVC's: GetVoice::new("some_voice_id")
/////    let voice = c.hit(GetVoice::new(DefaultVoice::Brian)).await?;
/////    println!("{:#?}", voice);
/////   Ok(())
///// }
///// ```
//#[derive(Clone, Debug)]
//pub struct GetVoice {
//    voice_id: VoiceID,
//}
//
//impl GetVoice {
//    pub fn new<T: Into<String>>(voice_id: T) -> Self {
//        GetVoice {
//            voice_id: VoiceID::from(voice_id.into()),
//        }
//    }
//}
//
//impl Endpoint for GetVoice
//where
//    Self: PathAndQueryParams,
//{
//    const PATH: &'static str = "/v1/voices/{voice_id}";
//    type ResponseBody = VoiceResponseBody;
//
//    const METHOD: Method = Method::GET;
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//    fn url(&self) -> Url {
//        build_url(Self::PATH, self.clone())
//    }
//}
//
//impl PathAndQueryParams for GetVoice {
//    fn get_path_params(&self) -> Vec<(&'static str, String)> {
//        vec![self.voice_id.as_path_param()]
//    }
//}
//
///// Hits [GetVoice] endpoint with the query `with_settings=true`
//#[derive(Clone, Debug)]
//pub struct GetVoiceWithSettings(VoiceID);
//
//impl GetVoiceWithSettings {
//    pub fn new<T: Into<String>>(voice_id: T) -> Self {
//        GetVoiceWithSettings(VoiceID::from(voice_id.into()))
//    }
//}
//
//impl Endpoint for GetVoiceWithSettings {
//    type ResponseBody = VoiceResponseBody;
//
//    const METHOD: Method = Method::GET;
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//    fn url(&self) -> Url {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!("{}/{}", VOICES_PATH, self.0 .0));
//        url.set_query(Some(WITH_SETTINGS_QUERY));
//        Ok(url)
//    }
//}
//
///// Delete a voice endpoint
//#[derive(Clone, Debug)]
//pub struct DeleteVoice(VoiceID);
//
//impl DeleteVoice {
//    pub fn new<T: Into<String>>(voice_id: T) -> Self {
//        DeleteVoice(VoiceID::from(voice_id.into()))
//    }
//}
//
//impl Endpoint for DeleteVoice {
//    type ResponseBody = StatusResponseBody;
//
//    const METHOD: Method = Method::DELETE;
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//    fn url(&self) -> Url {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!("{}/{}", VOICES_PATH, self.0 .0));
//        Ok(url)
//    }
//}
//
///// Edit voice settings endpoint
///// # Example
///// ```no_run
///// use elevenlabs_rs::*;
///// use elevenlabs_rs::endpoints::voice::{EditVoiceSettings, EditVoiceSettingsBody};
/////
///// #[tokio::main]
///// async fn main() -> Result<()> {
/////   let c = ElevenLabsClient::default()?;
/////   let body = EditVoiceSettingsBody::new(0.5, 0.7)
/////         .with_style(0.5)
/////         .with_use_speaker_boost(true);
/////   let endpoint = EditVoiceSettings::new("some_voice_id", body);
/////   let resp = c.hit(endpoint).await?;
/////   println!("{:#?}", resp);
/////   Ok(())
///// }
//#[derive(Clone, Debug)]
//pub struct EditVoiceSettings {
//    voice_id: VoiceID,
//    body: EditVoiceSettingsBody,
//}
//
//impl EditVoiceSettings {
//    pub fn new<T: Into<String>>(voice_id: T, body: EditVoiceSettingsBody) -> Self {
//        EditVoiceSettings {
//            voice_id: VoiceID::from(voice_id.into()),
//            body,
//        }
//    }
//}
//
//impl Endpoint for EditVoiceSettings {
//    type ResponseBody = StatusResponseBody;
//
//    const METHOD: Method = Method::POST;
//    async fn request_body(&self) -> Result<RequestBody> {
//        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
//    }
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//    fn url(&self) -> Url {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!(
//            "{}/{}{}",
//            VOICES_PATH, self.voice_id.0, EDIT_VOICE_SETTINGS_PATH
//        ));
//        Ok(url)
//    }
//}
//
///// Edit voice settings body
//#[derive(Clone, Debug, Serialize)]
//pub struct EditVoiceSettingsBody {
//    similarity_boost: f32,
//    stability: f32,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    style: Option<f32>,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    use_speaker_boost: Option<bool>,
//}
//
//impl EditVoiceSettingsBody {
//    pub fn new(similarity_boost: f32, stability: f32) -> Self {
//        Self {
//            similarity_boost,
//            stability,
//            style: None,
//            use_speaker_boost: None,
//        }
//    }
//    pub fn with_style(mut self, style: f32) -> Self {
//        self.style = Some(style);
//        self
//    }
//    pub fn with_use_speaker_boost(mut self, use_speaker_boost: bool) -> Self {
//        self.use_speaker_boost = Some(use_speaker_boost);
//        self
//    }
//}
//

/// Add a new voice to your collection of voices in VoiceLab.
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::admin::voice::{AddVoice, AddVoiceBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let samples = vec!["some_file_path.mp3".to_string(), "another.mp3".into(),];
///     let labels = vec![("age".to_string(), "old".into()), ("gender".into(), "male".into())];
///     let body = AddVoiceBody::new("John Doe", samples)
///         .with_description("A public intellectual")
///         .with_labels(labels);
///     let endpoint = AddVoice::new(body);
///     let resp = c.hit(endpoint).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See the [Add Voice API reference](https://elevenlabs.io/docs/api-reference/voices/add)
#[derive(Clone, Debug)]
pub struct AddVoice {
    body: AddVoiceBody,
}

impl AddVoice {
    pub fn new(body: AddVoiceBody) -> Self {
        AddVoice { body }
    }
}

//impl PathAndQueryParams for AddVoice {}

//impl Endpoint for AddVoice  {
//    const PATH: &'static str = "/v1/voices/add";
//    const METHOD: Method = Method::POST;
//    type ResponseBody = AddVoiceResponse;
//
//    async fn request_body(&self) -> Result<RequestBody> {
//        let form = Form::try_from(self.body.clone())?;
//        Ok(RequestBody::Multipart(form))
//
//    }
//
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//    fn url(&self) -> Url {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(Self::PATH);
//        url
//        //build_url(Self::PATH, self)
//    }
//}

/// Add voice body
#[derive(Clone, Debug)]
pub struct AddVoiceBody {
    name: String,
    files: Vec<String>,
    description: Option<String>,
    labels: Option<Vec<(String, String)>>,
}

impl AddVoiceBody {
    pub fn new(name: &str, files: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            files,
            description: None,
            labels: None,
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
}

impl TryFrom<AddVoiceBody> for Form {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: AddVoiceBody) -> Result<Self> {
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

        Ok(form)
    }
}

/// Add voice response
#[derive(Clone, Debug, Deserialize)]
pub struct AddVoiceResponse {
    voice_id: String,
}

impl AddVoiceResponse {
    pub fn get_voice_id(&self) -> &String {
        &self.voice_id
    }
}

///// Edit a voice endpoint
//#[derive(Clone, Debug)]
//pub struct EditVoice {
//    voice_id: VoiceID,
//    body: EditVoiceBody,
//}
//
//impl EditVoice {
//    pub fn new<T: Into<String>>(voice_id: T, body: EditVoiceBody) -> Self {
//        EditVoice {
//            voice_id: VoiceID::from(voice_id.into()),
//            body,
//        }
//    }
//}
//
//impl Endpoint for EditVoice {
//    type ResponseBody = StatusResponseBody;
//
//    const METHOD: Method = Method::POST;
//    async fn request_body(&self) -> Result<RequestBody> {
//        Ok(RequestBody::Multipart(to_multipart(
//            self.body.name.clone(),
//            self.body.files.clone(),
//            self.body.description.clone(),
//            self.body.labels.clone(),
//        )?))
//    }
//
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//    fn url(&self) -> Url {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!(
//            "{}/{}{}",
//            VOICES_PATH, self.voice_id.0, EDIT_VOICE_PATH
//        ));
//        Ok(url)
//    }
//}
//
///// Edit voice body
//#[derive(Clone, Debug)]
//pub struct EditVoiceBody {
//    name: String,
//    files: Option<Vec<String>>,
//    description: Option<String>,
//    labels: Option<Vec<(String, String)>>,
//}
//
//impl EditVoiceBody {
//    pub fn new(name: &str) -> Self {
//        Self {
//            name: name.to_string(),
//            files: None,
//            description: None,
//            labels: None,
//        }
//    }
//    pub fn with_files(mut self, files: Vec<String>) -> Self {
//        self.files = Some(files);
//        self
//    }
//    pub fn with_description(mut self, description: &str) -> Self {
//        self.description = Some(description.to_string());
//        self
//    }
//    pub fn with_labels(mut self, labels: Vec<(String, String)>) -> Self {
//        self.labels = Some(labels);
//        self
//    }
//}
//
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
    pub fn new(similarity_boost: f32, stability: f32) -> Self {
        VoiceSettings {
            similarity_boost,
            stability,
            style: None,
            use_speaker_boost: None,
        }
    }
    pub fn with_style(mut self, style: f32) -> Self {
        self.style = Some(style);
        self
    }
    pub fn with_use_speaker_boost(mut self, use_speaker_boost: bool) -> Self {
        self.use_speaker_boost = Some(use_speaker_boost);
        self
    }

    pub fn similarity_boost(&self) -> f32 {
        self.similarity_boost
    }

    pub fn stability(&self) -> f32 {
        self.stability
    }

    pub fn style(&self) -> Option<f32> {
        self.style
    }

    pub fn use_speaker_boost(&self) -> Option<bool> {
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
