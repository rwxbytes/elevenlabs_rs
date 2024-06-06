use super::*;
use crate::client::BASE_URL;
use crate::endpoints::Endpoint;
use crate::error::Error;
use reqwest::multipart::{Form, Part};
use reqwest::{Method, Response, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;

pub const ADD_VOICE_PATH: &str = "/add";
const EDIT_VOICE_PATH: &str = "/edit";
const EDIT_VOICE_SETTINGS_PATH: &str = "/settings/edit";
const DEFAULT_SETTINGS_PATH: &str = "/v1/voices/settings/default";
pub const VOICES_PATH: &str = "/v1/voices";
const VOICE_SETTINGS_PATH: &str = "/settings";
const WITH_SETTINGS_QUERY: &str = "with_settings=true";

#[derive(Clone, Debug)]
pub struct GetVoices;

impl Endpoint for GetVoices {
    type ResponseBody = Voices;

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

#[derive(Clone, Debug)]
pub struct GetVoiceIDByName(pub String);

impl Endpoint for GetVoiceIDByName {
    type ResponseBody = VoiceID;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let voices = resp.json::<Voices>().await?;
        let voice = voices.voices.iter().find(|v| v.name == self.0);
        let voice_id = voice
            .ok_or(Box::new(Error::VoiceNotFound))?
            .voice_id
            .clone();
        Ok(VoiceID(voice_id))
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(VOICES_PATH);
        url
    }
}

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

#[derive(Clone, Debug)]
pub struct GetVoiceSettings(pub VoiceID);

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

#[derive(Clone, Debug)]
pub struct GetVoice(pub VoiceID);

impl Endpoint for GetVoice {
    type ResponseBody = Voice;

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

#[derive(Clone, Debug)]
pub struct GetVoiceWithSettings(pub VoiceID);

impl Endpoint for GetVoiceWithSettings {
    type ResponseBody = Voice;

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

#[derive(Clone, Debug)]
pub struct DeleteVoice(pub VoiceID);

impl Endpoint for DeleteVoice {
    type ResponseBody = Status;

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

#[derive(Clone, Debug)]
pub struct EditVoiceSettings {
    pub voice_id: VoiceID,
    pub body: EditVoiceSettingsBody,
}

impl Endpoint for EditVoiceSettings {
    type ResponseBody = Status;

    fn method(&self) -> Method {
        Method::POST
    }
    fn json_request_body(&self) -> Option<Result<serde_json::Value>> {
        Some(serde_json::to_value(&self.body).map_err(Into::into))
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

#[derive(Clone, Debug)]
pub struct AddVoice(pub AddVoiceBody);

impl Endpoint for AddVoice {
    type ResponseBody = AddVoiceResponse;

    fn method(&self) -> Method {
        Method::POST
    }
    fn multipart_request_body(&self) -> Option<Result<Form>> {
        Some(to_multipart(
            self.0.name.clone(),
            Some(self.0.files.clone()),
            self.0.description.clone(),
            self.0.labels.clone(),
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

#[derive(Clone, Debug, Deserialize)]
pub struct AddVoiceResponse {
    voice_id: String,
}

#[derive(Clone, Debug)]
pub struct EditVoice {
    pub voice_id: VoiceID,
    pub body: EditVoiceBody,
}

impl Endpoint for EditVoice {
    type ResponseBody = Status;

    fn method(&self) -> Method {
        Method::POST
    }
    fn multipart_request_body(&self) -> Option<Result<Form>> {
        Some(to_multipart(
            self.body.name.clone(),
            self.body.files.clone(),
            self.body.description.clone(),
            self.body.labels.clone(),
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

#[derive(Clone, Debug)]
pub struct VoiceID(pub(crate) String);

impl Deref for VoiceID {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for VoiceID {
    fn from(id: &str) -> Self {
        VoiceID(id.to_string())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Voices {
    voices: Vec<Voice>,
}

// TODO: update this to use the new Voice struct
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Voice {
    voice_id: String,
    name: String,
    samples: Option<Vec<VoiceSample>>,
    category: Option<String>,
    labels: Option<HashMap<String, String>>,
    description: Option<String>,
    preview_url: Option<String>,
    settings: Option<VoiceSettings>,
}

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

// TODO: impl getters & setters.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VoiceSettings {
    pub similarity_boost: f32,
    pub stability: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_speaker_boost: Option<bool>,
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

impl Voice {
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

pub fn to_multipart<P: AsRef<Path>>(
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
