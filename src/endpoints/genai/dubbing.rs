//! The dubbing endpoints

use super::*;
use crate::error::Error;
use std::path::Path;
use std::string::ToString;
use strum::Display;

/// Dubs provided audio or video file into given language.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::dubbing::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     // Both file and source_url cannot be provided
///     let body = DubbingBody::new("en")
///         //.with_file("some_audio_or_video.mp3/mp4")
///         .with_highest_resolution(true)
///         .with_num_speakers(2)
///          // perhaps a scene from Crouching Tiger, Hidden Dragon
///         .with_source_url("some_url")
///         .with_source_lang("zh");
///
///     let endpoint = DubAVideoOrAnAudioFile::new(body);
///
///     let response = client.hit(endpoint).await?;
///
///     println!("{:?}", response);
///
///     Ok(())
/// }
/// ```
/// See [Dub a Video or Audio File API reference](https://elevenlabs.io/docs/api-reference/dubbing/dub-a-video-or-an-audio-file)
#[derive(Clone, Debug)]
pub struct DubAVideoOrAnAudioFile {
    body: DubbingBody,
}

impl DubAVideoOrAnAudioFile {
    pub fn new(body: DubbingBody) -> Self {
        DubAVideoOrAnAudioFile { body }
    }
}

impl ElevenLabsEndpoint for DubAVideoOrAnAudioFile {
    const PATH: &'static str = "v1/dubbing";

    const METHOD: Method = Method::POST;

    type ResponseBody = DubAVideoOrAnAudioFileResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// TODO: wrap `file` and `source_url` in an enum an make it required in fn new
#[derive(Clone, Debug, Default)]
pub struct DubbingBody {
    file: Option<String>,
    name: Option<String>,
    source_url: Option<String>,
    source_lang: Option<String>,
    target_lang: Option<String>,
    num_speakers: Option<u32>,
    watermark: Option<bool>,
    start_time: Option<f32>,
    end_time: Option<f32>,
    highest_resolution: Option<bool>,
    drop_background_audio: Option<bool>,
    /// [BETA] Whether transcripts should have profanities censored with the words ‘[censored]’
    use_profanity_filter: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DubAVideoOrAnAudioFileResponse {
    pub dubbing_id: String,
    pub expected_duration_sec: f32,
}

impl DubbingBody {
    pub fn new(target_lang: impl Into<String>) -> Self {
        DubbingBody {
            target_lang: Some(target_lang.into()),
            ..Default::default()
        }
    }
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_source_url(mut self, source_url: impl Into<String>) -> Self {
        self.source_url = Some(source_url.into());
        self
    }

    pub fn with_source_lang(mut self, source_language: impl Into<String>) -> Self {
        self.source_lang = Some(source_language.into());
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

    pub fn with_drop_background_audio(mut self, drop_background_audio: bool) -> Self {
        self.drop_background_audio = Some(drop_background_audio);
        self
    }

    pub fn with_use_profanity_filter(mut self, use_profanity_filter: bool) -> Self {
        self.use_profanity_filter = Some(use_profanity_filter);
        self
    }
}

/// Returns metadata about a dubbing project, including whether it’s still in progress or not.
///
/// # Example
///
/// ```no_run
///use elevenlabs_rs::{ElevenLabsClient, Result};
///use elevenlabs_rs::endpoints::genai::dubbing::*;
///
///#[tokio::main]
///async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///
///    let endpoint = GetDubbing::new("some_id");
///
///    let response = client.hit(endpoint).await?;
///
///    println!("{:?}", response);
///
///    Ok(())
///}
/// ```
/// See [Get Dubbing API reference](https://elevenlabs.io/docs/api-reference/dubbing/get-dubbing-project-metadata)
#[derive(Clone, Debug)]
pub struct GetDubbing {
    dubbing_id: DubbingID,
}

impl GetDubbing {
    pub fn new(dubbing_id: impl Into<DubbingID>) -> Self {
        GetDubbing {
            dubbing_id: dubbing_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetDubbing {
    const PATH: &'static str = "v1/dubbing/:dubbing_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetDubbingResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.dubbing_id.as_path_param()]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetDubbingResponse {
    pub dubbing_id: String,
    pub name: String,
    pub status: String,
    pub target_languages: Vec<String>,
    pub error: Option<String>,
}

/// Returns dubbed file as a streamed file.
/// Videos will be returned in MP4 format and audio only dubs will be returned in MP3.
///
/// # Example
/// ```no_run
///
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::dubbing::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let endpoint = GetDubbedAudio::new("some_id", "en");
///     let resp = client.hit(endpoint).await?;
///     save("dubbed.mp4", resp)?;
///
///     Ok(())
/// }
/// ```
/// See [Get Dubbed Audio API reference](https://elevenlabs.io/docs/api-reference/dubbing/get-dubbed-file)
#[derive(Clone, Debug)]
pub struct GetDubbedAudio {
    dubbing_id: DubbingID,
    language_code_id: LanguageCodeID,
}

impl GetDubbedAudio {
    pub fn new(
        dubbing_id: impl Into<DubbingID>,
        language_code_id: impl Into<LanguageCodeID>,
    ) -> Self {
        GetDubbedAudio {
            dubbing_id: dubbing_id.into(),
            language_code_id: language_code_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetDubbedAudio {
    const PATH: &'static str = "v1/dubbing/:dubbing_id/audio/:language_code";

    const METHOD: Method = Method::GET;

    type ResponseBody = Bytes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.dubbing_id.as_path_param(),
            self.language_code_id.as_path_param(),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// Deletes a dubbing project.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::dubbing::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let endpoint = DeleteDubbing::new("some_id");
///     let resp = client.hit(endpoint).await?;
///     println!("{:?}", resp);
///
///     Ok(())
/// }
/// ```
/// See [Delete Dubbing API reference](https://elevenlabs.io/docs/api-reference/dubbing/delete-dubbing-project)
#[derive(Clone, Debug)]
pub struct DeleteDubbing {
    dubbing_id: DubbingID,
}

impl DeleteDubbing {
    pub fn new(dubbing_id: impl Into<DubbingID>) -> Self {
        DeleteDubbing {
            dubbing_id: dubbing_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for DeleteDubbing {
    const PATH: &'static str = "v1/dubbing/:dubbing_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.dubbing_id.as_path_param()]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Returns transcript for the dub as an SRT file.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::dubbing::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let query = GetDubbedTranscriptQuery::default()
///         .with_format(TranscriptFormat::WebVtt);
///
///     let endpoint = GetDubbedTranscript::new("some_id", "en")
///         .with_query(query);
///
///     let resp = client.hit(endpoint).await?;
///     println!("{:?}", resp);
///
///     Ok(())
/// }
/// ```
/// See [Get Dubbed Transcript API reference](https://elevenlabs.io/docs/api-reference/dubbing/get-transcript-for-dub)
#[derive(Clone, Debug)]
pub struct GetDubbedTranscript {
    dubbing_id: DubbingID,
    language_code_id: LanguageCodeID,
    query: Option<GetDubbedTranscriptQuery>,
}

impl GetDubbedTranscript {
    pub fn new(
        dubbing_id: impl Into<DubbingID>,
        language_code_id: impl Into<LanguageCodeID>,
    ) -> Self {
        GetDubbedTranscript {
            dubbing_id: dubbing_id.into(),
            language_code_id: language_code_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: GetDubbedTranscriptQuery) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetDubbedTranscriptQuery {
    params: QueryValues,
}

impl GetDubbedTranscriptQuery {
    pub fn with_format(mut self, format: TranscriptFormat) -> Self {
        self.params.push(("format_type", format.to_string()));
        self
    }
}

impl ElevenLabsEndpoint for GetDubbedTranscript {
    const PATH: &'static str = "v1/dubbing/:dubbing_id/transcript/:language_code";

    const METHOD: Method = Method::GET;

    // TODO: parse, type, & impl iterator
    type ResponseBody = String;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.dubbing_id.as_path_param(),
            self.language_code_id.as_path_param(),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.text().await?)
    }
}

//#[derive(Clone, Debug, Deserialize)]
//pub struct GetDubbedTranscriptResponse {
//    pub key: String,
//}

#[derive(Clone, Debug, Display)]
#[strum(serialize_all = "lowercase")]
pub enum TranscriptFormat {
    Srt,
    WebVtt,
}

impl TryFrom<&DubbingBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &DubbingBody) -> Result<Self> {
        let mut form = Form::new();

        if let Some(file) = &body.file {
            let path = Path::new(file);
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
        if let Some(source_url) = &body.source_url {
            form = form.text("source_url", source_url.clone());
        }
        if let Some(source_lang) = &body.source_lang {
            form = form.text("source_lang", source_lang.clone());
        }
        if let Some(target_lang) = &body.target_lang {
            form = form.text("target_lang", target_lang.clone());
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
        if let Some(drop_background_audio) = body.drop_background_audio {
            form = form.text("drop_background_audio", drop_background_audio.to_string());
        }

        if let Some(use_profanity_filter) = body.use_profanity_filter {
            form = form.text("use_profanity_filter", use_profanity_filter.to_string());
        }
        Ok(RequestBody::Multipart(form))
    }
}
