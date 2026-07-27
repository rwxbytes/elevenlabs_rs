//! Speech to Text endpoints
use super::*;
use crate::error::Error;
use crate::shared::FilePart;
use std::string::ToString;
use strum::{self, Display};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpeechToTextModel {
    ScribeV2,
    #[deprecated(note = "ElevenLabs has deprecated scribe_v1; use ScribeV2")]
    ScribeV1,
    #[deprecated(note = "ElevenLabs has deprecated scribe_v1; use ScribeV2")]
    ScribeV1Base,
}

impl From<SpeechToTextModel> for String {
    #[allow(deprecated)]
    fn from(model: SpeechToTextModel) -> Self {
        match model {
            SpeechToTextModel::ScribeV2 => "scribe_v2".to_string(),
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
///    let body = CreateTranscriptBody::new(SpeechToTextModel::ScribeV2, "some_audio.mp3")
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
    pub query: Option<CreateTranscriptQuery>,
}

impl CreateTranscript {
    pub fn new(body: CreateTranscriptBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: CreateTranscriptQuery) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Clone, Debug)]
pub struct GetTranscript {
    transcription_id: String,
}

impl GetTranscript {
    pub fn new(transcription_id: impl Into<String>) -> Self {
        Self {
            transcription_id: transcription_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetTranscript {}

impl ElevenLabsEndpoint for GetTranscript {
    const PATH: &'static str = "/v1/speech-to-text/transcripts/:transcription_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetTranscriptResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.transcription_id.and_param(PathParam::TranscriptionID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug)]
pub struct DeleteTranscript {
    transcription_id: String,
}

impl DeleteTranscript {
    pub fn new(transcription_id: impl Into<String>) -> Self {
        Self {
            transcription_id: transcription_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteTranscript {}

impl ElevenLabsEndpoint for DeleteTranscript {
    const PATH: &'static str = "/v1/speech-to-text/transcripts/:transcription_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.transcription_id.and_param(PathParam::TranscriptionID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct CreateTranscriptQuery {
    params: QueryValues,
}

impl CreateTranscriptQuery {
    /// When enable_logging is set to false zero retention mode will be used for the request.
    /// This will mean history features are unavailable for this request, including request stitching.
    /// Zero retention mode may only be used by enterprise customers.
    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.params.push(("enable_logging", enable.to_string()));
        self
    }

    /// Authenticate with a single-use batch Scribe token.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.params.push(("token", token.into()));
        self
    }
}

#[derive(Clone, Copy, Debug, Display, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MultichannelOutputStyle {
    Separate,
    Combined,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateTranscriptBody {
    model_id: String,
    #[serde(skip)]
    file: FilePart,
    language_code: Option<String>,
    tag_audio_events: Option<bool>,
    num_speakers: Option<u32>,
    timestamps_granularity: Option<Granularity>,
    diarize: Option<bool>,
    multichannel_output_style: Option<MultichannelOutputStyle>,
    additional_formats: Option<Vec<AdditionalFormat>>,
    #[serde(skip)]
    prefer_video: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct FormatCommonOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_speakers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_timestamps: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_segment_chars: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_segment_duration_s: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_on_silence_longer_than_s: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum AdditionalFormat {
    Docx {
        #[serde(flatten)]
        common_opts: FormatCommonOptions,
    },
    Html {
        #[serde(flatten)]
        common_opts: FormatCommonOptions,
    },
    Pdf {
        #[serde(flatten)]
        common_opts: FormatCommonOptions,
    },
    SegmentedJson {
        #[serde(skip_serializing_if = "Option::is_none")]
        max_segment_chars: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_segment_duration_s: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        segment_on_silence_longer_than_s: Option<f32>,
    },
    Srt {
        #[serde(flatten)]
        common_opts: FormatCommonOptions,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_character_per_line: Option<u32>,
    },
    Txt {
        #[serde(flatten)]
        common_opts: FormatCommonOptions,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_character_per_line: Option<u32>,
    },
}

impl AdditionalFormat {
    pub fn new_docx() -> Self {
        Self::Docx {
            common_opts: FormatCommonOptions::default(),
        }
    }

    pub fn new_html() -> Self {
        Self::Html {
            common_opts: FormatCommonOptions::default(),
        }
    }

    pub fn new_pdf() -> Self {
        Self::Pdf {
            common_opts: FormatCommonOptions::default(),
        }
    }

    pub fn new_segmented_json() -> Self {
        Self::SegmentedJson {
            max_segment_chars: None,
            max_segment_duration_s: None,
            segment_on_silence_longer_than_s: None,
        }
    }

    pub fn new_srt() -> Self {
        Self::Srt {
            common_opts: FormatCommonOptions::default(),
            max_character_per_line: None,
        }
    }

    pub fn new_txt() -> Self {
        Self::Txt {
            common_opts: FormatCommonOptions::default(),
            max_character_per_line: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Display)]
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
    pub fn new(model_id: impl Into<String>, file: impl Into<FilePart>) -> Self {
        Self {
            model_id: model_id.into(),
            file: file.into(),
            ..Default::default()
        }
    }

    pub fn from_bytes(
        model_id: impl Into<String>,
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(model_id, FilePart::bytes(file_name, mime, bytes))
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

    /// Choose whether multichannel input is returned as separate transcripts
    /// or as one combined transcript.
    pub fn with_multichannel_output_style(mut self, output_style: MultichannelOutputStyle) -> Self {
        self.multichannel_output_style = Some(output_style);
        self
    }

    /// Add additional formats to the request.
    ///
    /// # Example
    ///
    /// ```
    /// # use elevenlabs_rs::endpoints::genai::speech_to_text::*;
    /// let mut additional_formats = Vec::new();
    ///
    /// let mut docx = AdditionalFormat::new_docx();
    /// if let AdditionalFormat::Docx {common_opts} = &mut docx {
    ///     common_opts.include_speakers = Some(false);
    ///     common_opts.include_timestamps = Some(false);
    /// };
    ///
    /// let mut srt = AdditionalFormat::new_srt();
    /// if let AdditionalFormat::Srt {mut max_character_per_line, ..} = &mut srt {
    ///     max_character_per_line = Some(40);
    /// };
    ///
    /// let segmented_json = AdditionalFormat::new_segmented_json();
    ///
    /// additional_formats.push(docx);
    /// additional_formats.push(srt);
    /// additional_formats.push(segmented_json);
    ///
    /// let body = CreateTranscriptBody::new(SpeechToTextModel::ScribeV2, "file")
    ///     .with_diarize(true) // Must be set to true to use additional formats
    ///     .with_additional_formats(additional_formats);
    /// ```
    pub fn with_additional_formats(mut self, additional_formats: Vec<AdditionalFormat>) -> Self {
        self.additional_formats = Some(additional_formats);
        self
    }

    pub fn prefer_video(mut self) -> Self {
        self.prefer_video = Some(true);
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateTranscript {}

impl ElevenLabsEndpoint for CreateTranscript {
    const PATH: &'static str = "/v1/speech-to-text";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateTranscriptResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

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
    pub channel_index: Option<u32>,
    pub additional_formats: Option<Vec<RequestedAdditionalFormat>>,
    pub transcription_id: Option<String>,
    pub entities: Option<Vec<DetectedEntity>>,
    pub audio_duration_secs: Option<f32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum GetTranscriptResponse {
    Single(CreateTranscriptResponse),
    Multichannel(MultichannelTranscriptResponse),
}

#[derive(Clone, Debug, Deserialize)]
pub struct MultichannelTranscriptResponse {
    pub transcripts: Vec<CreateTranscriptResponse>,
    pub transcription_id: Option<String>,
    pub audio_duration_secs: Option<f32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RequestedAdditionalFormat {
    pub requested_format: String,
    pub file_extension: String,
    pub content_type: String,
    pub is_base64_encoded: bool,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Word {
    pub text: String,
    pub r#type: WordType,
    pub start: Option<f32>,
    pub end: Option<f32>,
    pub speaker_id: Option<String>,
    pub logprob: Option<f32>,
    pub characters: Option<Vec<Character>>,
    pub channel_index: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Debug, Deserialize)]
pub struct DetectedEntity {
    pub text: String,
    pub entity_type: String,
    pub start_char: u32,
    pub end_char: u32,
}

impl TryFrom<CreateTranscriptBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: CreateTranscriptBody) -> Result<Self> {
        let inferred_mime = if body.file.mime().is_some() {
            None
        } else {
            let ext = body.file.extension()?;
            let prefer_video = body.prefer_video.unwrap_or_default();
            Some(TranscriptFileType::from_extension(&ext, prefer_video)?.mime_type())
        };

        let mut form = Form::new();
        form = form.text("model_id", body.model_id);
        form = form.part("file", body.file.into_part(inferred_mime)?);

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

        if let Some(output_style) = body.multichannel_output_style {
            form = form.text("multichannel_output_style", output_style.to_string());
        }

        if let Some(additional_formats) = body.additional_formats {
            let additional_formats_json = serde_json::to_string(&additional_formats)?;
            form = form.text("additional_formats", additional_formats_json);
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

            _ => Err(Error::FileExtensionNotSupported),
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

#[cfg(feature = "ws")]
pub mod ws {
    //! Realtime Speech to Text WebSocket endpoint.

    use super::*;
    use base64::{engine::general_purpose, Engine};
    use futures_util::StreamExt;
    use serde::de;
    use serde_json::{Map, Value};
    use std::pin::Pin;

    const WS_BASE_URL: &str = "wss://api.elevenlabs.io";
    const WS_PATH: &str = "/v1/speech-to-text/realtime";

    /// Realtime speech-to-text transcription using WebSockets.
    ///
    /// See [Realtime Speech to Text API reference](https://elevenlabs.io/docs/api-reference/speech-to-text/v-1-speech-to-text-realtime).
    pub struct RealtimeSpeechToText<S>
    where
        S: futures_util::Stream<Item = RealtimeSpeechToTextInput> + Send + 'static,
    {
        pub(crate) model_id: String,
        pub(crate) input_stream: S,
        pub(crate) query: RealtimeSpeechToTextQuery,
        #[cfg(test)]
        pub(crate) base_url: String,
    }

    impl<S> RealtimeSpeechToText<S>
    where
        S: futures_util::Stream<Item = RealtimeSpeechToTextInput> + Send + 'static,
    {
        pub fn new(model_id: impl Into<String>, input_stream: S) -> Self {
            Self {
                model_id: model_id.into(),
                input_stream,
                query: RealtimeSpeechToTextQuery::default(),
                #[cfg(test)]
                base_url: WS_BASE_URL.to_owned(),
            }
        }

        pub fn with_query(mut self, query: RealtimeSpeechToTextQuery) -> Self {
            self.query = query;
            self
        }

        #[cfg(test)]
        pub(crate) fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
            self.base_url = base_url.into();
            self
        }

        pub(crate) fn url(&self) -> Result<String> {
            let mut query_params = vec![("model_id", self.model_id.as_str())];
            query_params.extend(
                self.query
                    .params
                    .iter()
                    .map(|(name, value)| (*name, value.as_str())),
            );

            #[cfg(test)]
            let base_url = self.base_url.as_str();
            #[cfg(not(test))]
            let base_url = WS_BASE_URL;

            crate::ws::websocket_url(base_url, WS_PATH, [], query_params)
        }

        pub(crate) fn auth(&self) -> crate::ws::WebSocketAuth {
            if self.query.uses_token_auth() {
                crate::ws::WebSocketAuth::None
            } else {
                crate::ws::WebSocketAuth::XiApiKeyHeader
            }
        }
    }

    impl<S> crate::ws::sealed::Sealed for RealtimeSpeechToText<S> where
        S: futures_util::Stream<Item = RealtimeSpeechToTextInput> + Send + 'static
    {
    }

    impl<S> crate::ws::WebSocketEndpoint for RealtimeSpeechToText<S>
    where
        S: futures_util::Stream<Item = RealtimeSpeechToTextInput> + Send + 'static,
    {
        type Codec =
            crate::ws::JsonTextCodec<RealtimeSpeechToTextInput, RealtimeSpeechToTextResponse>;
        type InputStream =
            Pin<Box<dyn futures_util::Stream<Item = Result<RealtimeSpeechToTextInput>> + Send>>;

        fn url(&self) -> Result<String> {
            RealtimeSpeechToText::url(self)
        }

        fn auth(&self) -> crate::ws::WebSocketAuth {
            RealtimeSpeechToText::auth(self)
        }

        fn close_after_inputs(&self) -> bool {
            true
        }

        fn endpoint_name(&self) -> &'static str {
            "speech_to_text.realtime"
        }

        fn input_stream(self, _api_key: &str) -> Result<Self::InputStream> {
            let input_stream = self.input_stream;

            Ok(Box::pin(async_stream::try_stream! {
                futures_util::pin_mut!(input_stream);
                while let Some(input) = input_stream.next().await {
                    yield input;
                }
            }))
        }
    }

    /// Query parameters for realtime speech-to-text sessions.
    #[derive(Clone, Debug, Default)]
    pub struct RealtimeSpeechToTextQuery {
        params: QueryValues,
    }

    impl RealtimeSpeechToTextQuery {
        /// Authenticate with a single-use token instead of the `xi-api-key` header.
        pub fn with_token(mut self, token: impl Into<String>) -> Self {
            self.params.push(("token", token.into()));
            self
        }

        /// Receive committed transcript events with word-level timestamps.
        pub fn with_timestamps(mut self, include_timestamps: bool) -> Self {
            self.params
                .push(("include_timestamps", include_timestamps.to_string()));
            self
        }

        /// Include language detection in committed transcript-with-timestamps events.
        pub fn with_language_detection(mut self, include_language_detection: bool) -> Self {
            self.params.push((
                "include_language_detection",
                include_language_detection.to_string(),
            ));
            self
        }

        /// Set the source audio encoding format, for example `pcm_16000`.
        pub fn with_audio_format(mut self, audio_format: impl Into<String>) -> Self {
            self.params.push(("audio_format", audio_format.into()));
            self
        }

        /// Set the expected language code in ISO 639-1 or ISO 639-3 format.
        pub fn with_language_code(mut self, language_code: impl Into<String>) -> Self {
            self.params.push(("language_code", language_code.into()));
            self
        }

        /// Set whether transcripts are committed manually or by voice activity detection.
        pub fn with_commit_strategy(mut self, commit_strategy: RealtimeCommitStrategy) -> Self {
            self.params
                .push(("commit_strategy", commit_strategy.to_string()));
            self
        }

        /// Bias transcription towards key terms. The API currently allows at most
        /// 50 key terms of up to 20 characters each.
        pub fn with_keyterms<I, T>(mut self, keyterms: I) -> Self
        where
            I: IntoIterator<Item = T>,
            T: Into<String>,
        {
            self.params
                .extend(keyterms.into_iter().map(|term| ("keyterms", term.into())));
            self
        }

        /// Remove filler words, false starts, and disfluencies.
        pub fn with_no_verbatim(mut self, no_verbatim: bool) -> Self {
            self.params.push(("no_verbatim", no_verbatim.to_string()));
            self
        }

        /// Set the VAD silence threshold in seconds.
        pub fn with_vad_silence_threshold_secs(mut self, seconds: f64) -> Self {
            self.params
                .push(("vad_silence_threshold_secs", seconds.to_string()));
            self
        }

        /// Set the VAD threshold.
        pub fn with_vad_threshold(mut self, threshold: f64) -> Self {
            self.params.push(("vad_threshold", threshold.to_string()));
            self
        }

        /// Set the minimum speech duration for VAD.
        pub fn with_min_speech_duration_ms(mut self, milliseconds: u32) -> Self {
            self.params
                .push(("min_speech_duration_ms", milliseconds.to_string()));
            self
        }

        /// Set the minimum silence duration for VAD.
        pub fn with_min_silence_duration_ms(mut self, milliseconds: u32) -> Self {
            self.params
                .push(("min_silence_duration_ms", milliseconds.to_string()));
            self
        }

        /// Enable or disable logging for the session.
        pub fn with_logging(mut self, enable_logging: bool) -> Self {
            self.params
                .push(("enable_logging", enable_logging.to_string()));
            self
        }

        pub(crate) fn uses_token_auth(&self) -> bool {
            self.params.iter().any(|(name, _)| *name == "token")
        }
    }

    #[derive(Clone, Copy, Debug, Deserialize, Display, PartialEq, Eq, Serialize)]
    #[serde(rename_all = "lowercase")]
    #[strum(serialize_all = "lowercase")]
    pub enum RealtimeCommitStrategy {
        Manual,
        Vad,
    }

    /// A client-to-server realtime speech-to-text message.
    #[derive(Clone, Debug, Serialize)]
    #[serde(tag = "message_type", rename_all = "snake_case")]
    pub enum RealtimeSpeechToTextInput {
        InputAudioChunk(RealtimeAudioChunk),
    }

    impl RealtimeSpeechToTextInput {
        /// Create an audio chunk from raw bytes. The bytes are base64-encoded for
        /// the WebSocket JSON message.
        pub fn audio(audio: impl AsRef<[u8]>) -> Self {
            Self::InputAudioChunk(RealtimeAudioChunk::audio(audio))
        }

        /// Create an audio chunk from already base64-encoded audio.
        pub fn audio_base64(audio_base_64: impl Into<String>) -> Self {
            Self::InputAudioChunk(RealtimeAudioChunk::audio_base64(audio_base_64))
        }

        pub fn with_commit(mut self, commit: bool) -> Self {
            let Self::InputAudioChunk(chunk) = &mut self;
            chunk.commit = Some(commit);
            self
        }

        pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
            let Self::InputAudioChunk(chunk) = &mut self;
            chunk.sample_rate = Some(sample_rate);
            self
        }

        pub fn with_previous_text(mut self, previous_text: impl Into<String>) -> Self {
            let Self::InputAudioChunk(chunk) = &mut self;
            chunk.previous_text = Some(previous_text.into());
            self
        }
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct RealtimeAudioChunk {
        pub audio_base_64: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub commit: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sample_rate: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub previous_text: Option<String>,
    }

    impl RealtimeAudioChunk {
        pub fn audio(audio: impl AsRef<[u8]>) -> Self {
            Self::audio_base64(general_purpose::STANDARD.encode(audio))
        }

        pub fn audio_base64(audio_base_64: impl Into<String>) -> Self {
            Self {
                audio_base_64: audio_base_64.into(),
                commit: None,
                sample_rate: None,
                previous_text: None,
            }
        }
    }

    /// A server-to-client realtime speech-to-text message.
    #[derive(Clone, Debug)]
    pub enum RealtimeSpeechToTextResponse {
        SessionStarted(RealtimeSessionStarted),
        PartialTranscript(RealtimeTranscript),
        CommittedTranscript(RealtimeTranscript),
        CommittedTranscriptWithTimestamps(RealtimeTranscriptWithTimestamps),
        ScribeError(RealtimeSpeechToTextError),
        ScribeAuthError(RealtimeSpeechToTextError),
        ScribeQuotaExceededError(RealtimeSpeechToTextError),
        ScribeThrottledError(RealtimeSpeechToTextError),
        ScribeUnacceptedTermsError(RealtimeSpeechToTextError),
        ScribeRateLimitedError(RealtimeSpeechToTextError),
        ScribeQueueOverflowError(RealtimeSpeechToTextError),
        ScribeResourceExhaustedError(RealtimeSpeechToTextError),
        ScribeSessionTimeLimitExceededError(RealtimeSpeechToTextError),
        ScribeInputError(RealtimeSpeechToTextError),
        ScribeChunkSizeExceededError(RealtimeSpeechToTextError),
        ScribeInsufficientAudioActivityError(RealtimeSpeechToTextError),
        ScribeTranscriberError(RealtimeSpeechToTextError),
        InvalidRequest(RealtimeSpeechToTextError),
        InputError(RealtimeSpeechToTextError),
        Unknown(UnknownRealtimeSpeechToTextMessage),
    }

    impl RealtimeSpeechToTextResponse {
        pub fn message_type(&self) -> &str {
            match self {
                Self::SessionStarted(_) => "session_started",
                Self::PartialTranscript(_) => "partial_transcript",
                Self::CommittedTranscript(_) => "committed_transcript",
                Self::CommittedTranscriptWithTimestamps(_) => {
                    "committed_transcript_with_timestamps"
                }
                Self::ScribeError(_) => "scribe_error",
                Self::ScribeAuthError(_) => "scribe_auth_error",
                Self::ScribeQuotaExceededError(_) => "scribe_quota_exceeded_error",
                Self::ScribeThrottledError(_) => "scribe_throttled_error",
                Self::ScribeUnacceptedTermsError(_) => "scribe_unaccepted_terms_error",
                Self::ScribeRateLimitedError(_) => "scribe_rate_limited_error",
                Self::ScribeQueueOverflowError(_) => "scribe_queue_overflow_error",
                Self::ScribeResourceExhaustedError(_) => "scribe_resource_exhausted_error",
                Self::ScribeSessionTimeLimitExceededError(_) => {
                    "scribe_session_time_limit_exceeded_error"
                }
                Self::ScribeInputError(_) => "scribe_input_error",
                Self::ScribeChunkSizeExceededError(_) => "scribe_chunk_size_exceeded_error",
                Self::ScribeInsufficientAudioActivityError(_) => {
                    "scribe_insufficient_audio_activity_error"
                }
                Self::ScribeTranscriberError(_) => "scribe_transcriber_error",
                Self::InvalidRequest(_) => "invalid_request",
                Self::InputError(_) => "input_error",
                Self::Unknown(message) => &message.message_type,
            }
        }

        pub fn transcript_text(&self) -> Option<&str> {
            match self {
                Self::PartialTranscript(transcript) | Self::CommittedTranscript(transcript) => {
                    Some(&transcript.text)
                }
                Self::CommittedTranscriptWithTimestamps(transcript) => Some(&transcript.text),
                _ => None,
            }
        }

        pub fn error(&self) -> Option<&RealtimeSpeechToTextError> {
            match self {
                Self::ScribeError(error)
                | Self::ScribeAuthError(error)
                | Self::ScribeQuotaExceededError(error)
                | Self::ScribeThrottledError(error)
                | Self::ScribeUnacceptedTermsError(error)
                | Self::ScribeRateLimitedError(error)
                | Self::ScribeQueueOverflowError(error)
                | Self::ScribeResourceExhaustedError(error)
                | Self::ScribeSessionTimeLimitExceededError(error)
                | Self::ScribeInputError(error)
                | Self::ScribeChunkSizeExceededError(error)
                | Self::ScribeInsufficientAudioActivityError(error)
                | Self::ScribeTranscriberError(error)
                | Self::InvalidRequest(error)
                | Self::InputError(error) => Some(error),
                _ => None,
            }
        }

        pub fn is_error(&self) -> bool {
            self.error().is_some()
        }
    }

    impl<'de> Deserialize<'de> for RealtimeSpeechToTextResponse {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = Value::deserialize(deserializer)?;
            let message_type = value
                .get("message_type")
                .and_then(Value::as_str)
                .ok_or_else(|| de::Error::missing_field("message_type"))?
                .to_owned();

            match message_type.as_str() {
                "session_started" => deserialize_realtime_message(value).map(Self::SessionStarted),
                "partial_transcript" => {
                    deserialize_realtime_message(value).map(Self::PartialTranscript)
                }
                "committed_transcript" => {
                    deserialize_realtime_message(value).map(Self::CommittedTranscript)
                }
                "committed_transcript_with_timestamps" => {
                    deserialize_realtime_message(value).map(Self::CommittedTranscriptWithTimestamps)
                }
                "scribe_error" => deserialize_realtime_message(value).map(Self::ScribeError),
                "scribe_auth_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeAuthError)
                }
                "scribe_quota_exceeded_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeQuotaExceededError)
                }
                "scribe_throttled_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeThrottledError)
                }
                "scribe_unaccepted_terms_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeUnacceptedTermsError)
                }
                "scribe_rate_limited_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeRateLimitedError)
                }
                "scribe_queue_overflow_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeQueueOverflowError)
                }
                "scribe_resource_exhausted_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeResourceExhaustedError)
                }
                "scribe_session_time_limit_exceeded_error" => deserialize_realtime_message(value)
                    .map(Self::ScribeSessionTimeLimitExceededError),
                "scribe_input_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeInputError)
                }
                "scribe_chunk_size_exceeded_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeChunkSizeExceededError)
                }
                "scribe_insufficient_audio_activity_error" => deserialize_realtime_message(value)
                    .map(Self::ScribeInsufficientAudioActivityError),
                "scribe_transcriber_error" => {
                    deserialize_realtime_message(value).map(Self::ScribeTranscriberError)
                }
                "invalid_request" => deserialize_realtime_message(value).map(Self::InvalidRequest),
                "input_error" => deserialize_realtime_message(value).map(Self::InputError),
                unknown => unknown_realtime_message(unknown, value).map(Self::Unknown),
            }
        }
    }

    fn deserialize_realtime_message<T, E>(value: Value) -> std::result::Result<T, E>
    where
        T: serde::de::DeserializeOwned,
        E: de::Error,
    {
        serde_json::from_value(value).map_err(E::custom)
    }

    fn unknown_realtime_message<E>(
        message_type: &str,
        value: Value,
    ) -> std::result::Result<UnknownRealtimeSpeechToTextMessage, E>
    where
        E: de::Error,
    {
        let Value::Object(mut payload) = value else {
            return Err(E::custom(
                "realtime websocket message must be a JSON object",
            ));
        };
        payload.remove("message_type");

        Ok(UnknownRealtimeSpeechToTextMessage {
            message_type: message_type.to_owned(),
            payload,
        })
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct RealtimeSessionStarted {
        pub session_id: String,
        pub config: RealtimeSessionConfig,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct RealtimeSessionConfig {
        pub sample_rate: Option<u32>,
        pub audio_format: Option<String>,
        pub language_code: Option<String>,
        pub timestamps_granularity: Option<Granularity>,
        pub vad_commit_strategy: Option<bool>,
        pub vad_silence_threshold_secs: Option<f64>,
        pub vad_threshold: Option<f64>,
        pub min_speech_duration_ms: Option<u32>,
        pub min_silence_duration_ms: Option<u32>,
        pub max_tokens_to_recompute: Option<u32>,
        pub model_id: String,
        pub disable_logging: Option<bool>,
        pub include_timestamps: Option<bool>,
        pub include_language_detection: Option<bool>,
        pub keyterms: Option<Vec<String>>,
        pub no_verbatim: Option<bool>,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct RealtimeTranscript {
        pub text: String,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct RealtimeTranscriptWithTimestamps {
        pub text: String,
        pub language_code: Option<String>,
        pub words: Vec<RealtimeWord>,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct RealtimeWord {
        pub text: String,
        pub start: Option<f64>,
        pub end: Option<f64>,
        pub r#type: WordType,
        pub logprob: Option<f64>,
        pub characters: Option<Vec<String>>,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct RealtimeSpeechToTextError {
        pub message: Option<String>,
        pub code: Option<String>,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct UnknownRealtimeSpeechToTextMessage {
        pub message_type: String,
        pub payload: Map<String, Value>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::{json, Value};

        #[test]
        fn realtime_audio_chunk_serializes_as_json_message() {
            let input = RealtimeSpeechToTextInput::audio(b"hello")
                .with_commit(true)
                .with_sample_rate(16_000)
                .with_previous_text("context");

            let encoded = serde_json::to_string(&input).unwrap();

            assert_eq!(
                serde_json::from_str::<Value>(&encoded).unwrap(),
                json!({
                    "message_type": "input_audio_chunk",
                    "audio_base_64": "aGVsbG8=",
                    "commit": true,
                    "sample_rate": 16000,
                    "previous_text": "context"
                })
            );
        }

        #[test]
        fn realtime_url_contains_required_model_and_optional_query() {
            let input_stream = futures_util::stream::empty::<RealtimeSpeechToTextInput>();
            let endpoint = RealtimeSpeechToText::new("scribe_v2_realtime", input_stream)
                .with_query(
                    RealtimeSpeechToTextQuery::default()
                        .with_timestamps(true)
                        .with_language_code("en")
                        .with_commit_strategy(RealtimeCommitStrategy::Vad)
                        .with_keyterms(["ElevenLabs", "Scribe"]),
                );

            let url = endpoint.url().unwrap();
            let parsed = Url::parse(&url).unwrap();
            assert_eq!(parsed.scheme(), "wss");
            assert_eq!(parsed.host_str(), Some("api.elevenlabs.io"));
            assert_eq!(parsed.path(), "/v1/speech-to-text/realtime");

            let query_pairs: Vec<_> = parsed.query_pairs().collect();
            assert!(query_pairs.contains(&("model_id".into(), "scribe_v2_realtime".into())));
            assert!(query_pairs.contains(&("include_timestamps".into(), "true".into())));
            assert!(query_pairs.contains(&("language_code".into(), "en".into())));
            assert!(query_pairs.contains(&("commit_strategy".into(), "vad".into())));
            assert!(query_pairs.contains(&("keyterms".into(), "ElevenLabs".into())));
            assert!(query_pairs.contains(&("keyterms".into(), "Scribe".into())));
        }

        #[test]
        fn realtime_auth_uses_header_unless_token_query_is_set() {
            let endpoint = RealtimeSpeechToText::new(
                "scribe_v2_realtime",
                futures_util::stream::empty::<RealtimeSpeechToTextInput>(),
            );
            assert!(matches!(
                endpoint.auth(),
                crate::ws::WebSocketAuth::XiApiKeyHeader
            ));

            let endpoint = endpoint
                .with_query(RealtimeSpeechToTextQuery::default().with_token("single-use-token"));
            assert!(matches!(endpoint.auth(), crate::ws::WebSocketAuth::None));
        }

        #[test]
        fn realtime_responses_deserialize_by_message_type() {
            let partial = r#"{"message_type":"partial_transcript","text":"hello"}"#;
            let response: RealtimeSpeechToTextResponse = serde_json::from_str(partial).unwrap();
            assert!(matches!(
                response,
                RealtimeSpeechToTextResponse::PartialTranscript(RealtimeTranscript { text, .. })
                    if text == "hello"
            ));

            let error = r#"{"message_type":"scribe_rate_limited_error","message":"slow down","retry_after":2}"#;
            let response: RealtimeSpeechToTextResponse = serde_json::from_str(error).unwrap();
            assert!(matches!(
                response,
                RealtimeSpeechToTextResponse::ScribeRateLimitedError(err)
                    if err.message.as_deref() == Some("slow down")
                        && err.extra.get("retry_after") == Some(&json!(2))
            ));
        }

        #[test]
        fn realtime_response_helpers_expose_common_fields() {
            let partial = r#"{"message_type":"partial_transcript","text":"hello","stability":0.8}"#;
            let response: RealtimeSpeechToTextResponse = serde_json::from_str(partial).unwrap();

            assert_eq!(response.message_type(), "partial_transcript");
            assert_eq!(response.transcript_text(), Some("hello"));
            assert!(!response.is_error());

            let error = r#"{"message_type":"scribe_input_error","message":"bad audio"}"#;
            let response: RealtimeSpeechToTextResponse = serde_json::from_str(error).unwrap();

            assert_eq!(response.message_type(), "scribe_input_error");
            assert_eq!(
                response.error().unwrap().message.as_deref(),
                Some("bad audio")
            );
            assert!(response.is_error());
        }

        #[test]
        fn realtime_live_error_events_are_classified_as_errors() {
            let invalid_request = r#"{"message_type":"invalid_request","message":"bad request"}"#;
            let response: RealtimeSpeechToTextResponse =
                serde_json::from_str(invalid_request).unwrap();

            assert!(matches!(
                response,
                RealtimeSpeechToTextResponse::InvalidRequest(_)
            ));
            assert_eq!(response.message_type(), "invalid_request");
            assert_eq!(
                response.error().unwrap().message.as_deref(),
                Some("bad request")
            );
            assert!(response.is_error());

            let input_error = r#"{"message_type":"input_error","message":"bad audio"}"#;
            let response: RealtimeSpeechToTextResponse = serde_json::from_str(input_error).unwrap();

            assert!(matches!(
                response,
                RealtimeSpeechToTextResponse::InputError(_)
            ));
            assert_eq!(response.message_type(), "input_error");
            assert_eq!(
                response.error().unwrap().message.as_deref(),
                Some("bad audio")
            );
            assert!(response.is_error());
        }

        #[test]
        fn realtime_unknown_messages_preserve_type_and_payload() {
            let payload = r#"{
                "message_type": "future_event",
                "text": "still useful",
                "nested": { "ok": true }
            }"#;

            let response: RealtimeSpeechToTextResponse = serde_json::from_str(payload).unwrap();
            let RealtimeSpeechToTextResponse::Unknown(message) = response else {
                panic!("expected unknown event");
            };

            assert_eq!(message.message_type, "future_event");
            assert_eq!(message.payload.get("text"), Some(&json!("still useful")));
            assert_eq!(message.payload["nested"]["ok"], json!(true));
        }

        #[test]
        fn realtime_timestamp_words_accept_character_arrays() {
            let payload = r#"{
                "message_type": "committed_transcript_with_timestamps",
                "text": "The",
                "language_code": "en",
                "words": [
                    {
                        "text": "The",
                        "start": 0,
                        "end": 0.12,
                        "type": "word",
                        "logprob": -0.05,
                        "characters": ["T", "h", "e"]
                    }
                ]
            }"#;

            let response: RealtimeSpeechToTextResponse = serde_json::from_str(payload).unwrap();
            let RealtimeSpeechToTextResponse::CommittedTranscriptWithTimestamps(transcript) =
                response
            else {
                panic!("expected timestamp transcript");
            };

            assert_eq!(
                transcript.words[0].characters.as_ref().unwrap(),
                &vec!["T".to_string(), "h".to_string(), "e".to_string()]
            );
        }
    }
}
