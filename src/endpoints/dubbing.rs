#![allow(dead_code)]
//! The dubbing endpoints
use super::*;
use crate::error::Error;
use std::path::Path;

const DUBBING_PATH: &str = "v1/dubbing";
const AUDIO_PATH: &str = "/audio";

#[derive(Clone, Debug)]
pub struct DubbingID(String);

impl From<String> for DubbingID {
    fn from(id: String) -> Self {
        DubbingID(id)
    }
}

// TODO: Add CSV file example
/// The dubbing endpoint for the ElevenLabs API.
#[derive(Clone, Debug)]
pub struct DubAVideoOrAnAudioFile(DubbingBody);

impl DubAVideoOrAnAudioFile {
    pub fn new(body: DubbingBody) -> Self {
        DubAVideoOrAnAudioFile(body)
    }
    /// Create a dub from a video or audio file.
    ///
    /// # Example
    ///```no_run
    /// use elevenlabs_rs::*;
    /// use elevenlabs_rs::endpoints::dubbing::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///    let c = ElevenLabsClient::default()?;
    ///    let file = "some_video_file.mp4";
    ///    let endpoint = DubAVideoOrAnAudioFile::from_file(file, "ja", "en");
    ///    let resp = c.hit(endpoint).await?;
    ///    println!("{:#?}", resp);
    ///    Ok(())
    /// }
    ///```
    pub fn from_file(file: &str, source_language: &str, target_language: &str) -> Self {
        let mut body = DubbingBody::default();
        body = body
            .with_file(file.to_string())
            .with_source_lang(source_language)
            .with_target_lang(target_language)
            .with_mode(Mode::Automatic)
            .with_num_speakers(1)
            .with_watermark(false);
        DubAVideoOrAnAudioFile::new(body)
    }

    /// Create a dub from a link to a video or audio file.
    ///
    /// # Example
    /// ```no_run
    /// use elevenlabs_rs::*;
    /// use elevenlabs_rs::endpoints::dubbing::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///    let c = ElevenLabsClient::default()?;
    ///    let url = "some_url";
    ///    let endpoint = DubAVideoOrAnAudioFile::from_url(url, "en", "fr");
    ///    let resp = c.hit(endpoint).await?;
    ///    println!("{:#?}", resp);
    ///    Ok(())
    /// }
    /// ```
    pub fn from_url(source_url: &str, source_language: &str, target_language: &str) -> Self {
        let mut body = DubbingBody::default();
        body = body
            .with_source_url(source_url)
            .with_source_lang(source_language)
            .with_target_lang(target_language)
            .with_mode(Mode::Automatic)
            .with_num_speakers(1)
            .with_watermark(true);
        DubAVideoOrAnAudioFile::new(body)
    }
}

impl Endpoint for DubAVideoOrAnAudioFile {
    type ResponseBody = DubAVideoOrAnAudioFileResponse;

    fn method(&self) -> Method {
        Method::POST
    }
    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(to_form(self.0.clone())?))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(DUBBING_PATH);
        url
    }
}

/// Response body for dubbing a video or audio file.
#[derive(Clone, Debug, Deserialize)]
pub struct DubAVideoOrAnAudioFileResponse {
    dubbing_id: String,
    expected_duration_sec: f32,
}

impl DubAVideoOrAnAudioFileResponse {
    pub fn dubbing_id(&self) -> &str {
        &self.dubbing_id
    }
    pub fn expected_duration_sec(&self) -> f32 {
        self.expected_duration_sec
    }
}

/// Request body for dubbing a video or audio file.
///
/// # Example
/// ```
/// use elevenlabs_rs::endpoints::dubbing::*;
///
/// let mut body= DubbingBody::default();
/// body = body
///    .with_file("some_file.mp4".to_string())
///    // The source language of the content to be dubbed, default is auto.
///    .with_source_lang("en")
///    // The Target language to dub the content into. Can be none if dubbing studio editor is enabled and running manual mode.
///    .with_target_lang("fr")
///    // Automatic or manual. Manual mode is only supported when creating a dubbing studio project.
///    .with_mode(Mode::Automatic)
///    // Number of speakers to use for the dubbing. Set to 0 to automatically detect the number of speakers. Default is 0.
///    .with_num_speakers(0)
///    // Whether to apply watermark to the output video.
///    .with_watermark(true)
///    // Start time of the source video/audio to dub from.
///    .with_start_time(0.0)
///    // End time of the source video/audio to dub to.
///    .with_end_time(10.0)
///    // Whether to use the highest resolution available for the dubbing. Default is false.
///    .with_highest_resolution(true)
///    // Whether to prepare the dub for edits in the dubbing studio. Default is false.
///    .with_dubbing_studio(false);
/// ```
/// See [ElevenLabs API documentation](https://elevenlabs.io/docs/api-reference/create-dub) for more information.
///
///
#[derive(Clone, Debug, Default)]
pub struct DubbingBody {
    mode: Option<Mode>,
    file: Option<String>,
    csv_file: Option<String>,
    foreground_audio_file: Option<String>,
    background_audio_file: Option<String>,
    name: Option<String>,
    source_url: Option<String>,
    source_lang: Option<String>,
    target_lang: Option<String>,
    num_speakers: Option<u32>,
    watermark: Option<bool>,
    start_time: Option<f32>,
    end_time: Option<f32>,
    highest_resolution: Option<bool>,
    dubbing_studio: Option<bool>,
}

#[derive(Clone, Debug)]
pub enum Mode {
    Automatic,
    Manual,
}

impl Mode {
    pub fn to_string(&self) -> String {
        match self {
            Mode::Automatic => "automatic".to_string(),
            Mode::Manual => "manual".to_string(),
        }
    }
}

impl DubbingBody {
    pub fn new(target_lang: &str) -> Self {
        DubbingBody {
            mode: None,
            file: None,
            csv_file: None,
            foreground_audio_file: None,
            background_audio_file: None,
            name: None,
            source_url: None,
            source_lang: None,
            target_lang: Some(target_lang.to_string()),
            num_speakers: None,
            watermark: None,
            start_time: None,
            end_time: None,
            highest_resolution: None,
            dubbing_studio: None,
        }
    }

    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_csv_file(mut self, csv_file: String) -> Self {
        self.csv_file = Some(csv_file);
        self
    }

    pub fn with_foreground_audio_file(mut self, foreground_audio_file: String) -> Self {
        self.foreground_audio_file = Some(foreground_audio_file);
        self
    }

    pub fn with_background_audio_file(mut self, background_audio_file: String) -> Self {
        self.background_audio_file = Some(background_audio_file);
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_source_url(mut self, source_url: &str) -> Self {
        self.source_url = Some(source_url.to_string());
        self
    }

    pub fn with_source_lang(mut self, source_language: &str) -> Self {
        self.source_lang = Some(source_language.to_string());
        self
    }

    pub fn with_target_lang(mut self, target_language: &str) -> Self {
        self.target_lang = Some(target_language.to_string());
        self
    }

    pub fn with_num_speakers(mut self, num_speakers: u32) -> Self {
        self.num_speakers = Some(num_speakers);
        self
    }

    pub fn with_watermark(mut self, watermark: bool) -> Self {
        self.watermark = Some(watermark);
        self
    }

    pub fn with_start_time(mut self, start_time: f32) -> Self {
        self.start_time = Some(start_time);
        self
    }

    pub fn with_end_time(mut self, end_time: f32) -> Self {
        self.end_time = Some(end_time);
        self
    }

    pub fn with_highest_resolution(mut self, highest_resolution: bool) -> Self {
        self.highest_resolution = Some(highest_resolution);
        self
    }
    pub fn with_dubbing_studio(mut self, dubbing_studio: bool) -> Self {
        self.dubbing_studio = Some(dubbing_studio);
        self
    }
}

#[derive(Clone, Debug)]
pub struct GetDubbingProjectMetadata(DubbingID);

impl GetDubbingProjectMetadata {
    pub fn new(dubbing_id: DubbingID) -> Self {
        GetDubbingProjectMetadata(dubbing_id)
    }
}

impl Endpoint for GetDubbingProjectMetadata {
    type ResponseBody = GetDubbingProjectMetadataResponse;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", DUBBING_PATH, self.0 .0));
        url
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetDubbingProjectMetadataResponse {
    dubbing_id: String,
    name: String,
    status: String,
    target_languages: Vec<String>,
    error: Option<String>,
}

impl GetDubbingProjectMetadataResponse {
    pub fn dubbing_id(&self) -> &str {
        &self.dubbing_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn target_languages(&self) -> &Vec<String> {
        &self.target_languages
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Get the dubbed file for a dubbing project.
///
/// Returns dubbed file as a streamed file.
/// Videos will be returned in MP4 format and audio only dubs will be returned in MP3.
///
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::dubbing::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let dub_params = GetDubbedFileParams::new("some dubbing id", "en");
///     let resp = c.hit(GetDubbedFile(dub_params)).await?;
///     save("dubbed_vid.mp4", resp)?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetDubbedFile(pub GetDubbedFileParams);

impl Endpoint for GetDubbedFile {
    type ResponseBody = Bytes;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}/{}",
            DUBBING_PATH, self.0.dubbing_id.0, AUDIO_PATH, self.0.language_code
        ));
        url
    }
}

#[derive(Clone, Debug)]
pub struct GetDubbedFileParams {
    dubbing_id: DubbingID,
    language_code: String,
}

impl GetDubbedFileParams {
    pub fn new(dubbing_id: &str, language_code: &str) -> Self {
        GetDubbedFileParams {
            dubbing_id: DubbingID::from(dubbing_id.to_string()),
            language_code: language_code.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeleteDubbingProject(DubbingID);

impl DeleteDubbingProject {
    pub fn new(dubbing_id: &str) -> Self {
        DeleteDubbingProject(DubbingID::from(dubbing_id.to_string()))
    }
}

impl Endpoint for DeleteDubbingProject {
    type ResponseBody = StatusResponseBody;

    fn method(&self) -> Method {
        Method::DELETE
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", DUBBING_PATH, self.0 .0));
        url
    }
}
fn to_form(body: DubbingBody) -> Result<Form> {
    let mut form = Form::new();
    if let Some(mode) = body.mode {
        form = form.text("mode", mode.to_string());
    }
    if let Some(file) = body.file {
        let path = Path::new(&file);
        let dubbing_file = std::fs::read(path)?;
        let mut part = Part::bytes(dubbing_file);
        part = part.file_name(
            path.to_str()
                .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?
                .to_string(),
        );
        let mime_subtype = path
            .extension()
            .ok_or(Box::new(Error::FileExtensionNotFound))?
            .to_str()
            .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?;
        if mime_subtype == "mp4" {
            part = part.mime_str("video/mp4")?;
        } else if mime_subtype == "mp3" {
            part = part.mime_str("audio/mp3")?;
        } else {
            return Err(Box::new(Error::FileExtensionNotSupported));
        }
        form = form.part("file", part);
    }
    if let Some(csv_file) = body.csv_file {
        form = form.text("csv_file", csv_file);
    }
    if let Some(file) = body.foreground_audio_file {
        let path = Path::new(&file);
        let foreground_audio_file = std::fs::read(path)?;
        let mut part = Part::bytes(foreground_audio_file);
        part = part.file_name(
            path.to_str()
                .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?
                .to_string(),
        );
        let mime_subtype = path
            .extension()
            .ok_or(Box::new(Error::FileExtensionNotFound))?
            .to_str()
            .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?;
        if mime_subtype == "mp3" {
            part = part.mime_str("audio/mp3")?;
        } else if mime_subtype == "wav" {
            part = part.mime_str("audio/wav")?;
        } else {
            return Err(Box::new(Error::FileExtensionNotSupported));
        }
        form = form.part("foreground_audio_file", part);
    }
    if let Some(file) = body.background_audio_file {
        let path = Path::new(&file);
        let background_audio_file = std::fs::read(path)?;
        let mut part = Part::bytes(background_audio_file);
        part = part.file_name(
            path.to_str()
                .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?
                .to_string(),
        );
        let mime_subtype = path
            .extension()
            .ok_or(Box::new(Error::FileExtensionNotFound))?
            .to_str()
            .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?;
        if mime_subtype == "mp3" {
            part = part.mime_str("audio/mp3")?;
        } else if mime_subtype == "wav" {
            part = part.mime_str("audio/wav")?;
        } else {
            return Err(Box::new(Error::FileExtensionNotSupported));
        }
        form = form.part("background_audio_file", part);
    }
    if let Some(name) = body.name {
        form = form.text("name", name);
    }
    if let Some(source_url) = body.source_url {
        form = form.text("source_url", source_url);
    }
    if let Some(source_lang) = body.source_lang {
        form = form.text("source_lang", source_lang);
    }
    if let Some(target_lang) = body.target_lang {
        form = form.text("target_lang", target_lang);
    }
    if let Some(num_speakers) = body.num_speakers {
        form = form.text("num_speakers", num_speakers.to_string());
    }
    if let Some(watermark) = body.watermark {
        form = form.text("watermark", watermark.to_string());
    }
    if let Some(start_time) = body.start_time {
        form = form.text("start_time", start_time.to_string());
    }
    if let Some(end_time) = body.end_time {
        form = form.text("end_time", end_time.to_string());
    }
    if let Some(highest_resolution) = body.highest_resolution {
        form = form.text("highest_resolution", highest_resolution.to_string());
    }
    if let Some(dubbing_studio) = body.dubbing_studio {
        form = form.text("dubbing_studio", dubbing_studio.to_string());
    }
    Ok(form)
}
