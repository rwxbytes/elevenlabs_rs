//! Voice endpoints
//!
//! See the [ElevenLabs docs](https://elevenlabs.io/docs/api-reference/get-voices) for more information.
use super::*;
//use crate::endpoints::shared::{
//    identifiers::VoiceID,
//    path_segments::{VOICES_PATH, ADD_VOICE_PATH},
//    response_bodies::StatusResponseBody,
//};
use crate::error::Error;
use std::collections::HashMap;
use std::path::Path;

const EDIT_VOICE_PATH: &str = "/edit";
const EDIT_VOICE_SETTINGS_PATH: &str = "/settings/edit";
const DEFAULT_SETTINGS_PATH: &str = "/v1/voices/settings/default";
const VOICE_SETTINGS_PATH: &str = "/settings";
const WITH_SETTINGS_QUERY: &str = "with_settings=true";

/// Get all voices endpoint
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let voices = c.hit(GetVoices).await?;
///    println!("{:#?}", voices);
///    Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetVoices;

impl Endpoint for GetVoices {
    type ResponseBody = VoicesResponseBody;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(VOICES_PATH);
        url
    }
}

/// Hits [GetVoices] endpoint then finds the voices by name given
#[derive(Clone, Debug)]
pub struct GetVoiceIDByName(String);

impl GetVoiceIDByName {
    pub fn new(name: &str) -> Self {
        GetVoiceIDByName(name.to_string())
    }
}

impl Endpoint for GetVoiceIDByName {
    type ResponseBody = String;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let resp = resp.json::<VoicesResponseBody>().await?;
        let voice = resp.voices.iter().find(|v| v.name == self.0);
        let voice_id = voice
            .ok_or(Box::new(Error::VoiceNotFound))?
            .voice_id
            .clone();
        Ok(voice_id)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(VOICES_PATH);
        url
    }
}

/// Get the default voice settings endpoint
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let default_settings = c.hit(GetDefaultSettings).await?;
///    println!("{:#?}", default_settings);
///    Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetDefaultSettings;

impl Endpoint for GetDefaultSettings {
    type ResponseBody = VoiceSettings;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(DEFAULT_SETTINGS_PATH);
        url
    }
}

/// Get the voice settings endpoint
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::default()?;
///   // Or for a premade voice: GetVoiceSettings::new(PreMadeVoiceID::Adam)
///   let voice_settings = c.hit(GetVoiceSettings::new("some_voice_id")).await?;
///   println!("{:#?}", voice_settings);
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetVoiceSettings(VoiceID);

impl GetVoiceSettings {
    pub fn new<T: Into<String>>(voice_id: T) -> Self {
        GetVoiceSettings(VoiceID::from(voice_id.into()))
    }
}

impl Endpoint for GetVoiceSettings {
    type ResponseBody = VoiceSettings;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            VOICES_PATH, self.0 .0, VOICE_SETTINGS_PATH
        ));
        url
    }
}

/// Get a voice endpoint
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    // Or for IVC's & PVC's: GetVoice::new("some_voice_id")
///    let voice = c.hit(GetVoice::new(PreMadeVoiceID::Brian)).await?;
///    println!("{:#?}", voice);
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetVoice(VoiceID);

impl GetVoice {
    pub fn new<T: Into<String>>(voice_id: T) -> Self {
        GetVoice(VoiceID::from(voice_id.into()))
    }
}

impl Endpoint for GetVoice {
    type ResponseBody = VoiceResponseBody;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", VOICES_PATH, self.0 .0));
        url
    }
}

/// Hits [GetVoice] endpoint with the query `with_settings=true`
#[derive(Clone, Debug)]
pub struct GetVoiceWithSettings(VoiceID);

impl GetVoiceWithSettings {
    pub fn new<T: Into<String>>(voice_id: T) -> Self {
        GetVoiceWithSettings(VoiceID::from(voice_id.into()))
    }
}

impl Endpoint for GetVoiceWithSettings {
    type ResponseBody = VoiceResponseBody;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", VOICES_PATH, self.0 .0));
        url.set_query(Some(WITH_SETTINGS_QUERY));
        url
    }
}

/// Delete a voice endpoint
#[derive(Clone, Debug)]
pub struct DeleteVoice(VoiceID);

impl DeleteVoice {
    pub fn new<T: Into<String>>(voice_id: T) -> Self {
        DeleteVoice(VoiceID::from(voice_id.into()))
    }
}

impl Endpoint for DeleteVoice {
    type ResponseBody = StatusResponseBody;

    fn method(&self) -> Method {
        Method::DELETE
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", VOICES_PATH, self.0 .0));
        url
    }
}

/// Edit voice settings endpoint
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::default()?;
///   let body = EditVoiceSettingsBody::new(0.5, 0.7)
///         .with_style(0.5)
///         .with_use_speaker_boost(true);
///   let endpoint = EditVoiceSettings::new("some_voice_id", body);
///   let resp = c.hit(endpoint).await?;
///   println!("{:#?}", resp);
///   Ok(())
/// }
#[derive(Clone, Debug)]
pub struct EditVoiceSettings {
    voice_id: VoiceID,
    body: EditVoiceSettingsBody,
}

impl EditVoiceSettings {
    pub fn new<T: Into<String>>(voice_id: T, body: EditVoiceSettingsBody) -> Self {
        EditVoiceSettings {
            voice_id: VoiceID::from(voice_id.into()),
            body,
        }
    }
}

impl Endpoint for EditVoiceSettings {
    type ResponseBody = StatusResponseBody;

    fn method(&self) -> Method {
        Method::POST
    }
    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            VOICES_PATH, self.voice_id.0, EDIT_VOICE_SETTINGS_PATH
        ));
        url
    }
}

/// Edit voice settings body
#[derive(Clone, Debug, Serialize)]
pub struct EditVoiceSettingsBody {
    similarity_boost: f32,
    stability: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_speaker_boost: Option<bool>,
}

impl EditVoiceSettingsBody {
    pub fn new(similarity_boost: f32, stability: f32) -> Self {
        Self {
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
}

/// Add a voice endpoint
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
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
#[derive(Clone, Debug)]
pub struct AddVoice(AddVoiceBody);

impl AddVoice {
    pub fn new(body: AddVoiceBody) -> Self {
        AddVoice(body)
    }
}

impl Endpoint for AddVoice {
    type ResponseBody = AddVoiceResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(
            to_multipart(
                self.0.name.clone(),
                Some(self.0.files.clone()),
                self.0.description.clone(),
                self.0.labels.clone(),
            )?,
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}{}", VOICES_PATH, ADD_VOICE_PATH));
        url
    }
}

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

/// Edit a voice endpoint
#[derive(Clone, Debug)]
pub struct EditVoice {
    pub voice_id: VoiceID,
    pub body: EditVoiceBody,
}

impl EditVoice {
    pub fn new<T: Into<String>>(voice_id: T, body: EditVoiceBody) -> Self {
        EditVoice {
            voice_id: VoiceID::from(voice_id.into()),
            body,
        }
    }
}

impl Endpoint for EditVoice {
    type ResponseBody = StatusResponseBody;

    fn method(&self) -> Method {
        Method::POST
    }
    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(
            to_multipart(
                self.body.name.clone(),
                self.body.files.clone(),
                self.body.description.clone(),
                self.body.labels.clone(),
            )?,
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            VOICES_PATH, self.voice_id.0, EDIT_VOICE_PATH
        ));
        url
    }
}

/// Edit voice body
#[derive(Clone, Debug)]
pub struct EditVoiceBody {
    name: String,
    files: Option<Vec<String>>,
    description: Option<String>,
    labels: Option<Vec<(String, String)>>,
}

impl EditVoiceBody {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            files: None,
            description: None,
            labels: None,
        }
    }
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = Some(files);
        self
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



/// Get all voices response body
#[derive(Clone, Debug, Deserialize)]
pub struct VoicesResponseBody {
    voices: Vec<VoiceResponseBody>,
}

impl VoicesResponseBody {
    pub fn get_voices(&self) -> &Vec<VoiceResponseBody> {
        &self.voices
    }
}

// TODO: update this
/// Voice response body
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct VoiceResponseBody {
    voice_id: String,
    name: String,
    samples: Option<Vec<VoiceSample>>,
    category: Option<String>,
    labels: Option<HashMap<String, String>>,
    description: Option<String>,
    preview_url: Option<String>,
    settings: Option<VoiceSettings>,
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

impl VoiceResponseBody {
    pub fn get_voice_id(&self) -> &String {
        &self.voice_id
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_samples(&self) -> Option<&Vec<VoiceSample>> {
        self.samples.as_ref()
    }
    pub fn get_category(&self) -> Option<&String> {
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

fn to_multipart<P: AsRef<Path>>(
    voice_name: String,
    file_paths: Option<Vec<P>>,
    description: Option<String>,
    labels: Option<Vec<(String, String)>>,
) -> Result<Form> {
    let mut form = Form::new();
    form = form.text("name", voice_name);

    if let Some(file_paths) = file_paths {
        for file_path in file_paths {
            let fp = file_path.as_ref();
            let audio_bytes = std::fs::read(fp)?;
            let mut part = Part::bytes(audio_bytes);
            let file_path_str = fp.to_str().ok_or(Box::new(Error::PathNotValidUTF8))?;
            part = part.file_name(file_path_str.to_string());
            let mime_subtype = fp
                .extension()
                .ok_or(Box::new(Error::FileExtensionNotFound))?
                .to_str()
                .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?;
            let mime = format!("audio/{}", mime_subtype);
            part = part.mime_str(&mime)?;
            form = form.part("files", part);
        }
        if let Some(description) = description {
            form = form.text("description", description)
        }
        if let Some(labels) = labels {
            let mut label_map = HashMap::new();
            for (k, v) in labels {
                label_map.insert(k, v);
            }
            form = form.text("labels", serde_json::to_string(&label_map)?)
        }
    }
    Ok(form)
}
