//! The audio native endpoints
use super::*;
use crate::shared::FilePart;

/// Creates AudioNative enabled project, optionally starts conversion and returns project id and embeddable html snippet.
#[derive(Debug, Clone)]
pub struct AudioNative {
    body: AudioNativeBody,
}

impl AudioNative {
    pub fn new(body: AudioNativeBody) -> Self {
        AudioNative { body }
    }
}

impl crate::endpoints::sealed::Sealed for AudioNative {}

impl ElevenLabsEndpoint for AudioNative {
    const PATH: &'static str = "/v1/audio-native";

    const METHOD: Method = Method::POST;

    type ResponseBody = AudioNativeResponseBody;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(self.body.clone().into()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AudioNativeBody {
    name: String,
    image: Option<String>,
    author: Option<String>,
    title: Option<String>,
    small: Option<bool>,
    text_color: Option<String>,
    background_color: Option<String>,
    sessionization: Option<u32>,
    voice_id: Option<String>,
    model_id: Option<String>,
    file: Option<String>,
    auto_convert: Option<bool>,
}

impl AudioNativeBody {
    pub fn new(name: &str) -> Self {
        AudioNativeBody {
            name: name.to_string(),
            ..Default::default()
        }
    }
    pub fn with_image(mut self, image: &str) -> Self {
        self.image = Some(image.to_string());
        self
    }
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    pub fn with_small(mut self, small: bool) -> Self {
        self.small = Some(small);
        self
    }
    pub fn with_text_color(mut self, text_color: &str) -> Self {
        self.text_color = Some(text_color.to_string());
        self
    }
    pub fn with_background_color(mut self, background_color: &str) -> Self {
        self.background_color = Some(background_color.to_string());
        self
    }
    pub fn with_sessionization(mut self, sessionization: u32) -> Self {
        self.sessionization = Some(sessionization);
        self
    }
    pub fn with_voice_id(mut self, voice_id: &str) -> Self {
        self.voice_id = Some(voice_id.to_string());
        self
    }
    pub fn with_model_id(mut self, model_id: &str) -> Self {
        self.model_id = Some(model_id.to_string());
        self
    }
    pub fn with_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());
        self
    }
    pub fn with_auto_convert(mut self) -> Self {
        self.auto_convert = Some(true);
        self
    }
}

impl From<AudioNativeBody> for Form {
    fn from(body: AudioNativeBody) -> Self {
        let mut form = Form::new();
        form = form.text("name", body.name);
        if let Some(image) = body.image {
            form = form.text("image", image);
        }
        if let Some(author) = body.author {
            form = form.text("author", author);
        }
        if let Some(title) = body.title {
            form = form.text("title", title);
        }
        if let Some(small) = body.small {
            form = form.text("small", small.to_string());
        }
        if let Some(text_color) = body.text_color {
            form = form.text("text_color", text_color);
        }
        if let Some(background_color) = body.background_color {
            form = form.text("background_color", background_color);
        }
        if let Some(sessionization) = body.sessionization {
            form = form.text("sessionization", sessionization.to_string());
        }
        if let Some(voice_id) = body.voice_id {
            form = form.text("voice_id", voice_id);
        }
        if let Some(model_id) = body.model_id {
            form = form.text("model_id", model_id);
        }
        if let Some(file) = body.file {
            form = form.text("file", file);
        }
        if let Some(auto_convert) = body.auto_convert {
            form = form.text("auto_convert", auto_convert.to_string());
        }
        form
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AudioNativeResponseBody {
    pub project_id: String,
    pub converting: bool,
    pub html_snippet: String,
}

/// Updates the content of an Audio-Native project from a public URL: the page
/// is scraped and converted into a new project snapshot.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::audio_native::{
///     UpdateAudioNativeContentFromUrl, UpdateAudioNativeContentFromUrlBody,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = UpdateAudioNativeContentFromUrlBody::new(
///         "https://elevenlabs.io/blog/the_first_ai_that_can_laugh/",
///     )
///     .with_title("The first AI that can laugh");
///     let resp = c.hit(UpdateAudioNativeContentFromUrl::new(body)).await?;
///     println!("{}", resp.html_snippet);
///     Ok(())
/// }
/// ```
/// See [Update Audio-Native Content From Url API reference](https://elevenlabs.io/docs/api-reference/audio-native/update-content-from-url).
#[derive(Clone, Debug)]
pub struct UpdateAudioNativeContentFromUrl {
    body: UpdateAudioNativeContentFromUrlBody,
}

impl UpdateAudioNativeContentFromUrl {
    pub fn new(body: UpdateAudioNativeContentFromUrlBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateAudioNativeContentFromUrl {}

impl ElevenLabsEndpoint for UpdateAudioNativeContentFromUrl {
    const PATH: &'static str = "/v1/audio-native/content";

    const METHOD: Method = Method::POST;

    type ResponseBody = AudioNativeEditContentResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateAudioNativeContentFromUrlBody {
    /// URL of the page to extract content from.
    url: String,
    /// Author used in the player and inserted at the start of the article. If
    /// not provided, the default author from the Player settings is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    /// Title used in the player and inserted at the top of the article. If not
    /// provided, the default title from the Player settings is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

impl UpdateAudioNativeContentFromUrlBody {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            author: None,
            title: None,
        }
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Updates the content of an Audio-Native project by uploading a new article
/// file (txt or HTML), optionally converting and publishing it.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::audio_native::{
///     UpdateAudioNativeProjectContent, UpdateAudioNativeProjectContentBody,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = UpdateAudioNativeProjectContentBody::new("article.html").with_auto_convert(true);
///     let resp = c.hit(UpdateAudioNativeProjectContent::new("project_id", body)).await?;
///     println!("converting: {}", resp.converting);
///     Ok(())
/// }
/// ```
/// See [Update Audio-Native Project Content API reference](https://elevenlabs.io/docs/api-reference/audio-native/update-project-content).
#[derive(Clone, Debug)]
pub struct UpdateAudioNativeProjectContent {
    project_id: String,
    body: UpdateAudioNativeProjectContentBody,
}

impl UpdateAudioNativeProjectContent {
    pub fn new(project_id: impl Into<String>, body: UpdateAudioNativeProjectContentBody) -> Self {
        Self {
            project_id: project_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateAudioNativeProjectContent {}

impl ElevenLabsEndpoint for UpdateAudioNativeProjectContent {
    const PATH: &'static str = "/v1/audio-native/:project_id/content";

    const METHOD: Method = Method::POST;

    type ResponseBody = AudioNativeEditContentResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.project_id.and_param(PathParam::ProjectID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug)]
pub struct UpdateAudioNativeProjectContentBody {
    file: FilePart,
    auto_convert: bool,
    auto_publish: bool,
}

impl UpdateAudioNativeProjectContentBody {
    /// Create a body from a txt or HTML article file containing the content.
    pub fn new(file: impl Into<FilePart>) -> Self {
        Self {
            file: file.into(),
            auto_convert: false,
            auto_publish: false,
        }
    }

    pub fn from_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(FilePart::bytes(file_name, mime, bytes))
    }

    /// Whether to auto convert the project to audio or not.
    pub fn with_auto_convert(mut self, auto_convert: bool) -> Self {
        self.auto_convert = auto_convert;
        self
    }

    /// Whether to auto publish the new project snapshot after it's converted.
    pub fn with_auto_publish(mut self, auto_publish: bool) -> Self {
        self.auto_publish = auto_publish;
        self
    }
}

impl TryFrom<&UpdateAudioNativeProjectContentBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &UpdateAudioNativeProjectContentBody) -> Result<Self> {
        let form = Form::new()
            .part("file", body.file.clone().into_part(None)?)
            .text("auto_convert", body.auto_convert.to_string())
            .text("auto_publish", body.auto_publish.to_string());
        Ok(RequestBody::Multipart(form))
    }
}

/// Response of the Audio-Native content-update endpoints.
#[derive(Clone, Debug, Deserialize)]
pub struct AudioNativeEditContentResponse {
    /// The ID of the project.
    pub project_id: String,
    /// Whether the project is currently being converted.
    pub converting: bool,
    /// Whether the project is currently being published.
    pub publishing: bool,
    /// The HTML snippet to embed the Audio-Native player.
    pub html_snippet: String,
}

/// Get the settings of an Audio-Native project.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::audio_native::GetAudioNativeProjectSettings;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let resp = c.hit(GetAudioNativeProjectSettings::new("project_id")).await?;
///     println!("enabled: {}", resp.enabled);
///     Ok(())
/// }
/// ```
/// See [Get Audio-Native Project Settings API reference](https://elevenlabs.io/docs/api-reference/audio-native/get-settings).
#[derive(Clone, Debug)]
pub struct GetAudioNativeProjectSettings {
    project_id: String,
}

impl GetAudioNativeProjectSettings {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetAudioNativeProjectSettings {}

impl ElevenLabsEndpoint for GetAudioNativeProjectSettings {
    const PATH: &'static str = "/v1/audio-native/:project_id/settings";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetAudioNativeProjectSettingsResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.project_id.and_param(PathParam::ProjectID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetAudioNativeProjectSettingsResponse {
    /// Whether the project is enabled.
    pub enabled: bool,
    /// The ID of the latest snapshot of the project.
    pub snapshot_id: Option<String>,
    /// The settings of the project.
    pub settings: Option<AudioNativeProjectSettings>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AudioNativeProjectSettings {
    pub title: String,
    pub image: String,
    pub author: String,
    pub small: bool,
    pub text_color: String,
    pub background_color: String,
    /// For how many minutes to persist the session across page reloads.
    pub sessionization: u32,
    pub audio_path: Option<String>,
    pub audio_url: Option<String>,
    #[serde(default)]
    pub status: AudioNativeProjectStatus,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioNativeProjectStatus {
    Processing,
    #[default]
    Ready,
}
