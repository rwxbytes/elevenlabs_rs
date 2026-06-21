//! The text-to-dialogue endpoint
pub use super::tts::{Alignment, Normalization};
use super::*;
use crate::shared::{query_params::OutputFormat, DictionaryLocator, VoiceSettings};
use async_stream::try_stream;
use base64::{engine::general_purpose, Engine};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

/// Converts a list of text and voice ID pairs into speech (dialogue) and returns audio.
///
/// See [Text-to-Dialogue API reference](https://elevenlabs.io/docs/api-reference/text-to-dialogue/convert)
#[derive(Clone, Debug)]
pub struct TextToDialogue {
    body: TextToDialogueBody,
    query: Option<TextToDialogueQuery>,
}

impl TextToDialogue {
    pub fn new(body: TextToDialogueBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: TextToDialogueQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl ElevenLabsEndpoint for TextToDialogue {
    const PATH: &'static str = "/v1/text-to-dialogue";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// Converts a list of text and voice ID pairs into speech (dialogue) and returns
/// an audio stream.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::genai::text_to_dialogue::{
///     DialogueInput, TextToDialogueBody, TextToDialogueStream,
/// };
/// use elevenlabs_rs::utils::stream_audio;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let body = TextToDialogueBody::new(vec![
///         DialogueInput::new("[giggling] Knock knock", "voice_id_1"),
///         DialogueInput::new("[curious] Who is there?", "voice_id_2"),
///     ]);
///
///     let mut stream = c.hit(TextToDialogueStream::new(body)).await?;
///     stream_audio(&mut stream).await?;
///
///     Ok(())
/// }
/// ```
///
/// See [Text-to-Dialogue Stream API reference](https://elevenlabs.io/docs/api-reference/text-to-dialogue/stream)
#[derive(Clone, Debug)]
pub struct TextToDialogueStream {
    body: TextToDialogueBody,
    query: Option<TextToDialogueQuery>,
}

impl TextToDialogueStream {
    pub fn new(body: TextToDialogueBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: TextToDialogueQuery) -> Self {
        self.query = Some(query);
        self
    }
}

type TextToDialogueStreamResponse = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;

impl ElevenLabsEndpoint for TextToDialogueStream {
    const PATH: &'static str = "/v1/text-to-dialogue/stream";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToDialogueStreamResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream.map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
}

/// Single dialogue turn input consisting of text and a voice ID
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueInput {
    pub text: String,
    pub voice_id: String,
}

impl DialogueInput {
    pub fn new(text: impl Into<String>, voice_id: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            voice_id: voice_id.into(),
        }
    }
}

/// Request body for Text-to-Dialogue API
#[derive(Clone, Debug, Serialize, Default)]
pub struct TextToDialogueBody {
    pub inputs: Vec<DialogueInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<VoiceSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronunciation_dictionary_locators: Option<Vec<DictionaryLocator>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_text_normalization: Option<Normalization>,
}

impl TextToDialogueBody {
    pub fn new(inputs: Vec<DialogueInput>) -> Self {
        Self {
            inputs,
            ..Default::default()
        }
    }

    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn with_settings(mut self, settings: VoiceSettings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn with_pronunciation_dictionary_locators(
        mut self,
        locators: Vec<DictionaryLocator>,
    ) -> Self {
        self.pronunciation_dictionary_locators = Some(locators);
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_language_code(mut self, language_code: impl Into<String>) -> Self {
        self.language_code = Some(language_code.into());
        self
    }

    pub fn with_text_normalization(mut self, normalization: Normalization) -> Self {
        self.apply_text_normalization = Some(normalization);
        self
    }
}

impl TryFrom<&TextToDialogueBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(value: &TextToDialogueBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(value)?))
    }
}

/// Query parameters for Text-to-Dialogue API
#[derive(Clone, Debug, Default)]
pub struct TextToDialogueQuery {
    pub params: QueryValues,
}

impl TextToDialogueQuery {
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.params
            .push(("output_format", output_format.to_string()));
        self
    }

    /// When set to `false`, zero retention mode is used for the request, which
    /// disables history features (including request stitching). Zero retention
    /// mode may only be used by enterprise customers.
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.params
            .push(("enable_logging", enable_logging.to_string()));
        self
    }
}

/// Converts a list of text and voice ID pairs into speech (dialogue) and returns audio
/// together with character-level timing information for audio-text synchronization.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::genai::text_to_dialogue::{
///     DialogueInput, TextToDialogueBody, TextToDialogueWithTimestamps,
/// };
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let body = TextToDialogueBody::new(vec![
///         DialogueInput::new("Hello, how are you?", "voice_id_1"),
///         DialogueInput::new("I'm doing well, thank you!", "voice_id_2"),
///     ]);
///
///     let resp = c.hit(TextToDialogueWithTimestamps::new(body)).await?;
///
///     for (segment, text) in resp.segments_with_text() {
///         println!("[{:.1}s] {}: {}", segment.start_time_seconds, segment.voice_id, text);
///     }
///
///     play(resp.audio()?)?;
///
///     Ok(())
/// }
/// ```
///
/// See [Text-to-Dialogue with timestamps API reference](https://elevenlabs.io/docs/api-reference/text-to-dialogue/convert-with-timestamps)
#[derive(Clone, Debug)]
pub struct TextToDialogueWithTimestamps {
    body: TextToDialogueBody,
    query: Option<TextToDialogueQuery>,
}

impl TextToDialogueWithTimestamps {
    pub fn new(body: TextToDialogueBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: TextToDialogueQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl ElevenLabsEndpoint for TextToDialogueWithTimestamps {
    const PATH: &'static str = "/v1/text-to-dialogue/with-timestamps";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToDialogueWithTimestampsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Response for the Text-to-Dialogue with timestamps API, containing the audio,
/// character-level alignment and the per-line voice segments.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextToDialogueWithTimestampsResponse {
    /// Base64 encoded audio data.
    pub audio_base64: String,
    /// Timestamp information for each character in the original text.
    pub alignment: Option<Alignment>,
    /// Timestamp information for each character in the normalized text.
    pub normalized_alignment: Option<Alignment>,
    /// Voice segments for the audio.
    pub voice_segments: Vec<VoiceSegment>,
}

impl TextToDialogueWithTimestampsResponse {
    /// Decodes the base64 audio data into raw bytes.
    pub fn audio(&self) -> Result<Bytes> {
        let decoded = general_purpose::STANDARD.decode(&self.audio_base64)?;
        Ok(Bytes::from(decoded))
    }

    /// Returns the text spoken in a given [`VoiceSegment`], reconstructed from the
    /// alignment's `characters` (which the segment's indices point into).
    ///
    /// The text is returned verbatim, including any `[audio tags]` present in the
    /// original input. Falls back to the normalized alignment, and yields an empty
    /// string if no alignment is available or the indices are out of range.
    ///
    /// Voice-segment indices are global across the whole dialogue, but a streamed
    /// chunk only carries its own slice of `characters`. This offsets the indices
    /// by the index of this response's first character, so it works both for the
    /// full response and for an individual stream chunk.
    pub fn segment_text(&self, segment: &VoiceSegment) -> String {
        let base = self.character_offset();
        let start = segment.character_start_index.saturating_sub(base);
        let end = segment.character_end_index.saturating_sub(base);
        self.alignment
            .as_ref()
            .or(self.normalized_alignment.as_ref())
            .and_then(|a| a.characters.get(start..end))
            .map(|chars| chars.concat())
            .unwrap_or_default()
    }

    /// The global character index of this response's first character — i.e. how
    /// far into the overall dialogue this (possibly partial) response begins. For
    /// the full, non-streamed response this is `0`.
    fn character_offset(&self) -> usize {
        self.voice_segments
            .iter()
            .map(|s| s.character_start_index)
            .min()
            .unwrap_or(0)
    }

    /// Iterates over the voice segments paired with their spoken text.
    ///
    /// See [`segment_text`](Self::segment_text) for how the text is reconstructed.
    pub fn segments_with_text(&self) -> impl Iterator<Item = (&VoiceSegment, String)> {
        self.voice_segments
            .iter()
            .map(|segment| (segment, self.segment_text(segment)))
    }
}

/// A contiguous span of the generated audio produced by a single voice.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceSegment {
    /// The voice ID used for this segment.
    pub voice_id: String,
    /// Start time of this voice segment.
    pub start_time_seconds: f64,
    /// End time of this voice segment.
    pub end_time_seconds: f64,
    /// Start index in the characters array.
    pub character_start_index: usize,
    /// End index in the characters array (exclusive).
    pub character_end_index: usize,
    /// Line of the dialogue (script) that this segment is a part of.
    pub dialogue_input_index: usize,
}

impl VoiceSegment {
    /// Duration of this voice segment in seconds.
    pub fn duration(&self) -> f64 {
        self.end_time_seconds - self.start_time_seconds
    }
}

/// Converts a list of text and voice ID pairs into speech (dialogue) and returns a
/// stream of JSON chunks, each containing a slice of audio together with its
/// character-level timing information.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::genai::text_to_dialogue::{
///     DialogueInput, TextToDialogueBody, TextToDialogueStreamWithTimestamps,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let body = TextToDialogueBody::new(vec![
///         DialogueInput::new("Hello, how are you?", "voice_id_1"),
///         DialogueInput::new("I'm doing well, thank you!", "voice_id_2"),
///     ]);
///
///     let mut stream = c.hit(TextToDialogueStreamWithTimestamps::new(body)).await?;
///
///     while let Some(chunk) = stream.next().await {
///         let chunk = chunk?;
///         for (segment, text) in chunk.segments_with_text() {
///             println!("[{:.1}s] {}: {}", segment.start_time_seconds, segment.voice_id, text);
///         }
///     }
///
///     Ok(())
/// }
/// ```
///
/// See [Text-to-Dialogue Stream with Timestamps API reference](https://elevenlabs.io/docs/api-reference/text-to-dialogue/stream-with-timestamps)
#[derive(Clone, Debug)]
pub struct TextToDialogueStreamWithTimestamps {
    body: TextToDialogueBody,
    query: Option<TextToDialogueQuery>,
}

impl TextToDialogueStreamWithTimestamps {
    pub fn new(body: TextToDialogueBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: TextToDialogueQuery) -> Self {
        self.query = Some(query);
        self
    }
}

type TextToDialogueStreamWithTimestampsResponse =
    Pin<Box<dyn Stream<Item = Result<TextToDialogueWithTimestampsResponse>> + Send>>;

impl ElevenLabsEndpoint for TextToDialogueStreamWithTimestamps {
    const PATH: &'static str = "/v1/text-to-dialogue/stream/with-timestamps";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToDialogueStreamWithTimestampsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream_chunks_to_json(stream);
        Ok(Box::pin(stream))
    }
}

// Parses the streamed JSON chunks of the streaming-with-timestamps response.
//
// HTTP chunks don't align with message boundaries, so we buffer raw bytes and let
// serde pull off each complete JSON value, tracking how many bytes it consumed.
// This handles partial messages, several messages in one chunk, and UTF-8
// characters split across chunks — independent of the delimiter the server uses.
fn stream_chunks_to_json(
    stream: impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static,
) -> impl Stream<Item = Result<TextToDialogueWithTimestampsResponse>> + Send {
    try_stream! {
        let mut buffer: Vec<u8> = Vec::new();

        for await chunk in stream {
            buffer.extend_from_slice(&chunk?);

            loop {
                let mut iter = serde_json::Deserializer::from_slice(&buffer)
                    .into_iter::<TextToDialogueWithTimestampsResponse>();
                match iter.next() {
                    Some(Ok(value)) => {
                        let consumed = iter.byte_offset();
                        buffer.drain(..consumed);
                        yield value;
                    }
                    // Incomplete value: wait for more bytes.
                    Some(Err(e)) if e.is_eof() => break,
                    Some(Err(e)) => Err(e)?,
                    // Only trailing whitespace left.
                    None => break,
                }
            }
        }
    }
}
