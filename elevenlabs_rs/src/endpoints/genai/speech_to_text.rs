//! Speech to Text endpoints
use super::*;
use crate::error::Error;
use std::string::ToString;
use strum::{self, Display};

pub enum SpeechToTextModel {
    ScribeV1,
    ScribeV1Base,
}

impl From<SpeechToTextModel> for String {
    fn from(model: SpeechToTextModel) -> Self {
        match model {
            SpeechToTextModel::ScribeV1 => "scribe_v1".to_string(),
            SpeechToTextModel::ScribeV1Base => "scribe_v1_base".to_string(),
        }
    }
}

/// Transcribe an audio or video file.
/// 
/// # Example
/// 
/// ```no_run
/// 
/// use elevenlabs_rs::{ElevenLabsClient, Result,};
/// use elevenlabs_rs::endpoints::genai::speech_to_text::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///
///    let body = CreateTranscriptBody::new(SpeechToTextModel::ScribeV1, "some_audio.mp3")
///    .with_tag_audio_events(true)
///    .with_num_speakers(2)
///    .with_timestamps_granularity(Granularity::Character)
///    // a helper to distinguish between webm and mp4
///    //.prefer_video()
///    .with_diarize(true);
///
///    let endpoint = CreateTranscript::new(body);
///
///    let resp = client.hit(endpoint).await?;
///
///    let text = &resp.text;
///    println!("{}", text);
///    println!("--------------------------------");
///    println!("--------------------------------");
///
///
///    for word in resp {
///        println!("{:?}", word);
///    }
///
///    Ok(())
///}
/// ```
/// See [Create Transcript API reference](https://elevenlabs.io/docs/api-reference/speech-to-text/convert)
#[derive(Clone, Debug)]
pub struct CreateTranscript {
    pub body: CreateTranscriptBody,
}

impl CreateTranscript {
    pub fn new(body: CreateTranscriptBody) -> Self {
        Self { body }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateTranscriptBody {
    model_id: String,
    file: String,
    language_code: Option<String>,
    tag_audio_events: Option<bool>,
    num_speakers: Option<u32>,
    timestamps_granularity: Option<Granularity>,
    diarize: Option<bool>,
    #[serde(skip)]
    prefer_video: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Granularity {
    Word,
    Character,
    None,
}

impl From<&str> for Granularity {
    fn from(s: &str) -> Self {
        match s {
            "word" => Granularity::Word,
            "character" => Granularity::Character,
            "none" => Granularity::None,
            _ => Granularity::Word,
        }
    }
}

impl CreateTranscriptBody {
    pub fn new(model_id: impl Into<String>, file: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            file: file.into(),
            ..Default::default()
        }
    }
    pub fn with_language_code(mut self, language_code: impl Into<String>) -> Self {
        self.language_code = Some(language_code.into());
        self
    }
    pub fn with_tag_audio_events(mut self, tag_audio_events: bool) -> Self {
        self.tag_audio_events = Some(tag_audio_events);
        self
    }
    pub fn with_num_speakers(mut self, num_speakers: u32) -> Self {
        self.num_speakers = Some(num_speakers);
        self
    }
    pub fn with_timestamps_granularity(
        mut self,
        timestamps_granularity: impl Into<Granularity>,
    ) -> Self {
        self.timestamps_granularity = Some(timestamps_granularity.into());
        self
    }
    pub fn with_diarize(mut self, diarize: bool) -> Self {
        self.diarize = Some(diarize);
        self
    }
    pub fn prefer_video(mut self) -> Self {
        self.prefer_video = Some(true);
        self
    }
}

impl ElevenLabsEndpoint for CreateTranscript {
    const PATH: &'static str = "/v1/speech-to-text";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateTranscriptResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(self.body.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateTranscriptResponse {
    pub language_code: String,
    pub language_probability: f32,
    pub text: String,
    pub words: Vec<Word>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Word {
    pub text: String,
    pub r#type: WordType,
    pub start: Option<f32>,
    pub end: Option<f32>,
    pub speaker_id: Option<String>,
    pub characters: Option<Vec<Character>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WordType {
    Word,
    Spacing,
    AudioEvent,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Character {
    pub text: String,
    pub start: Option<f32>,
    pub end: Option<f32>,
}

impl TryFrom<CreateTranscriptBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: CreateTranscriptBody) -> Result<Self> {
        let path = std::path::Path::new(&body.file);

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(Error::PathNotValidUTF8)?;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or(Error::FileExtensionNotFound)?;

        let is_preference = body.prefer_video.unwrap_or_default();

        let file_type = TranscriptFileType::from_extension(ext, is_preference)?;

        let content = std::fs::read(path)?;

        let part = Part::bytes(content)
            .file_name(filename.to_string())
            .mime_str(&file_type.mime_type())?;

        let mut form = Form::new();
        form = form.text("model_id", body.model_id);
        form = form.part("file", part);

        if let Some(language_code) = body.language_code {
            form = form.text("language_code", language_code);
        }

        if let Some(tag_audio_events) = body.tag_audio_events {
            form = form.text("tag_audio_events", tag_audio_events.to_string());
        }

        if let Some(num_speakers) = body.num_speakers {
            form = form.text("num_speakers", num_speakers.to_string());
        }

        if let Some(timestamps_granularity) = body.timestamps_granularity {
            form = form.text("timestamps_granularity", timestamps_granularity.to_string());
        }

        if let Some(diarize) = body.diarize {
            form = form.text("diarize", diarize.to_string());
        }

        Ok(RequestBody::Multipart(form))
    }
}

#[derive(Debug, Clone)]
pub enum TranscriptFileType<'a> {
    Audio(&'a str),
    Video(&'a str),
}

const AAC: &str = "aac";
const X_AIFF: &str = "x-aiff";
const OGG: &str = "ogg";
const MPEG: &str = "mpeg";
const WAV: &str = "wav";
const WEBM: &str = "webm";
const FLAC: &str = "flac";
const X_M4A: &str = "x-m4a";
const OPUS: &str = "opus";
const MP4: &str = "mp4";
const X_MSVIDEO: &str = "x-msvideo";
const X_MATROSKA: &str = "x-matroska";
const QUICKTIME: &str = "quicktime";
const X_MS_WMV: &str = "x-ms-wmv";
const X_FLV: &str = "x-flv";
const THREEGPP: &str = "3gpp";

impl<'a> TranscriptFileType<'a> {
    pub fn mime_type(self) -> String {
        match self {
            Self::Audio(s) => format!("audio/{}", s),
            Self::Video(s) => format!("video/{}", s),
        }
    }
    pub fn from_extension(ext: &str, prefer_video: bool) -> Result<TranscriptFileType<'a>> {
        match ext.to_lowercase().as_str() {
            "aac" => Ok(Self::Audio(AAC)),
            "aif" | "aiff" => Ok(Self::Audio(X_AIFF)),
            "ogg" | "oga" | "spx" => Ok(Self::Audio(OGG)),
            "mp3" | "m2a" | "m3a" | "mp2" | "mp2a" | "mpga" => Ok(Self::Audio(MPEG)),
            "opus" => Ok(Self::Audio(OPUS)),
            "wav" | "wave" => Ok(Self::Audio(WAV)),
            "flac" => Ok(Self::Audio(FLAC)),
            "m4a" => Ok(Self::Audio(X_M4A)),

            "webm" => {
                if prefer_video {
                    Ok(Self::Video(WEBM))
                } else {
                    Ok(Self::Audio(WEBM))
                }
            }
            "mp4" => {
                if prefer_video {
                    Ok(Self::Video(MP4))
                } else {
                    Ok(Self::Audio(MP4))
                }
            }

            "avi" => Ok(Self::Video(X_MSVIDEO)),
            "mkv" => Ok(Self::Video(X_MATROSKA)),
            "mov" | "qt" => Ok(Self::Video(QUICKTIME)),
            "wmv" => Ok(Self::Video(X_MS_WMV)),
            "flv" => Ok(Self::Video(X_FLV)),
            "mpg" | "mpeg" => Ok(Self::Video(MPEG)),
            "3gp" => Ok(Self::Video(THREEGPP)),

            _ => Err(Error::FileExtensionNotSupported.into()),
        }
    }
}

impl IntoIterator for CreateTranscriptResponse {
    type Item = Word;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.words.into_iter()
    }
}

impl<'a> IntoIterator for &'a CreateTranscriptResponse {
    type Item = &'a Word;

    type IntoIter = std::slice::Iter<'a, Word>;

    fn into_iter(self) -> Self::IntoIter {
        self.words.iter()
    }
}
