//! The music generation endpoints.
//!
//! Compose a song from a simple text prompt or from a detailed composition
//! plan, stream the audio as it is generated, separate an existing track into
//! stems, or upload a song for later inpainting.
//!
//! See the [Music API reference](https://elevenlabs.io/docs/api-reference/music/compose).

use super::*;
use crate::shared::{audio_mime_from_extension, query_params::OutputFormat, FilePart};
use futures_util::{Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;

// =============================================================================
// Shared query
// =============================================================================

/// Query parameters shared by the composing and stem-separation endpoints.
#[derive(Clone, Debug, Default)]
pub struct MusicQuery {
    params: QueryValues,
}

impl MusicQuery {
    /// Output format of the generated audio, formatted as `codec_sample_rate_bitrate`.
    ///
    /// The composing endpoints also accept `OutputFormat::custom("auto")` (the
    /// default), which lets the API pick the best format for the selected model:
    /// `mp3_44100_128` for `music_v1` and `mp3_48000_192` for `music_v2`.
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.params
            .push(("output_format", output_format.to_string()));
        self
    }
}

// =============================================================================
// Models and composition-plan types
// =============================================================================

/// The model to use for the generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MusicModel {
    /// The `music_v1` model. Pairs with a [`MusicPrompt`] composition plan.
    #[default]
    MusicV1,
    /// The `music_v2` model. Pairs with a [`CompositionPlan`] composition plan.
    MusicV2,
}

/// A detailed composition plan to guide the music generation.
///
/// Cannot be used in conjunction with a prompt. The variant determines which
/// model the plan is compatible with: [`MusicPrompt`] for `music_v1` and
/// [`CompositionPlan`] for `music_v2`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MusicCompositionPlan {
    /// Composition plan for the `music_v1` model.
    V1(MusicPrompt),
    /// Composition plan for the `music_v2` model.
    V2(CompositionPlan),
}

impl MusicCompositionPlan {
    /// The model this composition plan is compatible with.
    pub fn model(&self) -> MusicModel {
        match self {
            Self::V1(_) => MusicModel::MusicV1,
            Self::V2(_) => MusicModel::MusicV2,
        }
    }
}

impl From<MusicPrompt> for MusicCompositionPlan {
    fn from(plan: MusicPrompt) -> Self {
        Self::V1(plan)
    }
}

impl From<CompositionPlan> for MusicCompositionPlan {
    fn from(plan: CompositionPlan) -> Self {
        Self::V2(plan)
    }
}

/// Composition plan for the `music_v1` model.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MusicPrompt {
    /// The styles and musical directions that should be present in the entire
    /// song. Use English for best results.
    pub positive_global_styles: Vec<String>,
    /// The styles and musical directions that should not be present in the
    /// entire song. Use English for best results.
    pub negative_global_styles: Vec<String>,
    /// The sections of the song.
    pub sections: Vec<SongSection>,
}

impl MusicPrompt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_positive_global_styles<I, S>(mut self, styles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.positive_global_styles = styles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_negative_global_styles<I, S>(mut self, styles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.negative_global_styles = styles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_sections(mut self, sections: impl IntoIterator<Item = SongSection>) -> Self {
        self.sections = sections.into_iter().collect();
        self
    }

    pub fn add_section(mut self, section: SongSection) -> Self {
        self.sections.push(section);
        self
    }
}

/// A section of a [`MusicPrompt`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongSection {
    /// The name of the section. Must be between 1 and 100 characters.
    pub section_name: String,
    /// The styles and musical directions that should be present in this section.
    pub positive_local_styles: Vec<String>,
    /// The styles and musical directions that should not be present in this section.
    pub negative_local_styles: Vec<String>,
    /// The duration of the section in milliseconds. Must be between 3000ms and 120000ms.
    pub duration_ms: u32,
    /// The lyrics of the section. Max 30 lines per section, max 200 characters per line.
    pub lines: Vec<String>,
    /// Optional source to extract the section from. Used for inpainting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_from: Option<SectionSource>,
}

impl SongSection {
    pub fn new(section_name: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            section_name: section_name.into(),
            positive_local_styles: Vec::new(),
            negative_local_styles: Vec::new(),
            duration_ms,
            lines: Vec::new(),
            source_from: None,
        }
    }

    pub fn with_positive_local_styles<I, S>(mut self, styles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.positive_local_styles = styles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_negative_local_styles<I, S>(mut self, styles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.negative_local_styles = styles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_lines<I, S>(mut self, lines: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.lines = lines.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_source_from(mut self, source: SectionSource) -> Self {
        self.source_from = Some(source);
        self
    }
}

/// The source song and time range a section is extracted from, for inpainting.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionSource {
    /// The ID of the song to source the section from. Found in the `song-id`
    /// response header returned when generating a song.
    pub song_id: String,
    /// The range to extract from the source song.
    pub range: TimeRange,
    /// The ranges to exclude from `range`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_ranges: Option<Vec<TimeRange>>,
}

impl SectionSource {
    pub fn new(song_id: impl Into<String>, range: TimeRange) -> Self {
        Self {
            song_id: song_id.into(),
            range,
            negative_ranges: None,
        }
    }

    pub fn with_negative_ranges(mut self, ranges: impl IntoIterator<Item = TimeRange>) -> Self {
        self.negative_ranges = Some(ranges.into_iter().collect());
        self
    }
}

/// A time range within a song, in milliseconds.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_ms: u32,
    pub end_ms: u32,
}

impl TimeRange {
    pub fn new(start_ms: u32, end_ms: u32) -> Self {
        Self { start_ms, end_ms }
    }
}

/// Composition plan for the `music_v2` model.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompositionPlan {
    /// The chunks that make up the generation.
    pub chunks: Vec<CompositionChunk>,
}

impl CompositionPlan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_chunks(mut self, chunks: impl IntoIterator<Item = CompositionChunk>) -> Self {
        self.chunks = chunks.into_iter().collect();
        self
    }

    pub fn add_chunk(mut self, chunk: impl Into<CompositionChunk>) -> Self {
        self.chunks.push(chunk.into());
        self
    }
}

/// A single chunk of a [`CompositionPlan`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompositionChunk {
    /// A chunk generated from text and styles.
    Generation(GenerationChunk),
    /// A reference to a range of an existing song.
    AudioRef(AudioRefChunk),
}

impl From<GenerationChunk> for CompositionChunk {
    fn from(chunk: GenerationChunk) -> Self {
        Self::Generation(chunk)
    }
}

impl From<AudioRefChunk> for CompositionChunk {
    fn from(chunk: AudioRefChunk) -> Self {
        Self::AudioRef(chunk)
    }
}

/// How strongly the model adheres to the context of surrounding chunks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextAdherence {
    Low,
    Medium,
    #[default]
    High,
}

/// How strongly the model adheres to a conditioning reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionStrength {
    Low,
    Medium,
    High,
    Xhigh,
}

/// A chunk generated from text and styles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationChunk {
    /// The text config to be generated for this chunk. Can contain a section
    /// name in square brackets, e.g. `[Verse 1]`, lyrics lines, and inline
    /// directions in curly braces, e.g. `{scratching}`.
    pub text: String,
    /// The duration of the chunk in milliseconds. Must be between 3000ms and 120000ms.
    pub duration_ms: u32,
    /// The styles and musical directions that should be present in this chunk.
    pub positive_styles: Vec<String>,
    /// The styles and musical directions that should not be present in this chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_styles: Option<Vec<String>>,
    /// How much the model adheres to the context of its surrounding chunks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_adherence: Option<ContextAdherence>,
    /// The audio reference to condition the generation on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditioning_ref: Option<AudioRefChunk>,
    /// How strongly the model adheres to the conditioning reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_strength: Option<ConditionStrength>,
}

impl GenerationChunk {
    pub fn new(text: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            text: text.into(),
            duration_ms,
            positive_styles: Vec::new(),
            negative_styles: None,
            context_adherence: None,
            conditioning_ref: None,
            condition_strength: None,
        }
    }

    pub fn with_positive_styles<I, S>(mut self, styles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.positive_styles = styles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_negative_styles<I, S>(mut self, styles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.negative_styles = Some(styles.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_context_adherence(mut self, context_adherence: ContextAdherence) -> Self {
        self.context_adherence = Some(context_adherence);
        self
    }

    pub fn with_conditioning_ref(mut self, conditioning_ref: AudioRefChunk) -> Self {
        self.conditioning_ref = Some(conditioning_ref);
        self
    }

    pub fn with_condition_strength(mut self, condition_strength: ConditionStrength) -> Self {
        self.condition_strength = Some(condition_strength);
        self
    }
}

/// A reference to a range of an existing song.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioRefChunk {
    /// The ID of the song to source the chunk from. Found in the `song-id`
    /// response header returned when generating a song.
    pub song_id: String,
    /// The time range to extract from the song.
    pub range: TimeRange,
}

impl AudioRefChunk {
    pub fn new(song_id: impl Into<String>, range: TimeRange) -> Self {
        Self {
            song_id: song_id.into(),
            range,
        }
    }
}

/// A word-level timestamp transcribed from a generated or uploaded song.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordTimestamp {
    pub word: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

/// Metadata describing a generated song.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SongMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    pub is_explicit: Option<bool>,
}

// =============================================================================
// Compose body (shared by compose, compose-detailed, and stream)
// =============================================================================

/// Request body for composing a song.
///
/// Provide either a [prompt](MusicComposeBody::from_prompt) or a
/// [composition plan](MusicComposeBody::from_composition_plan); the two are
/// mutually exclusive.
#[derive(Clone, Debug, Default, Serialize)]
pub struct MusicComposeBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    composition_plan: Option<MusicCompositionPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    music_length_ms: Option<u32>,
    model_id: MusicModel,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    force_instrumental: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    respect_sections_durations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    store_for_inpainting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sign_with_c2pa: Option<bool>,
}

impl MusicComposeBody {
    /// Compose from a simple text prompt. Defaults to the `music_v1` model.
    pub fn from_prompt(prompt: impl Into<String>) -> Self {
        Self {
            prompt: Some(prompt.into()),
            ..Default::default()
        }
    }

    /// Compose from a detailed composition plan. The model is set to match the
    /// plan variant (`music_v1` for [`MusicPrompt`], `music_v2` for
    /// [`CompositionPlan`]).
    pub fn from_composition_plan(plan: impl Into<MusicCompositionPlan>) -> Self {
        let plan = plan.into();
        Self {
            model_id: plan.model(),
            composition_plan: Some(plan),
            ..Default::default()
        }
    }

    /// Override the model to use for the generation.
    pub fn with_model(mut self, model_id: MusicModel) -> Self {
        self.model_id = model_id;
        self
    }

    /// The length of the song to generate in milliseconds. Used only with a
    /// prompt. Must be between 3000ms and 600000ms.
    pub fn with_music_length_ms(mut self, music_length_ms: u32) -> Self {
        self.music_length_ms = Some(music_length_ms);
        self
    }

    /// Random seed to initialize the generation. Cannot be used with a prompt.
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// If true, guarantees that the generated song will be instrumental. Can
    /// only be used with a prompt.
    pub fn with_force_instrumental(mut self, force_instrumental: bool) -> Self {
        self.force_instrumental = Some(force_instrumental);
        self
    }

    /// Controls how strictly section durations in the composition plan are
    /// enforced. Only used with a composition plan and only applies to `music_v1`.
    pub fn with_respect_sections_durations(mut self, respect: bool) -> Self {
        self.respect_sections_durations = Some(respect);
        self
    }

    /// Whether to store the generated song for inpainting.
    pub fn with_store_for_inpainting(mut self, store: bool) -> Self {
        self.store_for_inpainting = Some(store);
        self
    }

    /// Whether to sign the generated song with C2PA. Applicable only for mp3 files.
    pub fn with_sign_with_c2pa(mut self, sign: bool) -> Self {
        self.sign_with_c2pa = Some(sign);
        self
    }
}

impl TryFrom<&MusicComposeBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &MusicComposeBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

// =============================================================================
// POST /v1/music — Compose Music
// =============================================================================

/// Compose a song from a prompt or a composition plan.
///
/// The response is the raw audio; the unique song ID is returned in the
/// `song-id` response header (use [`ComposeMusicDetailed`] to capture it).
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::utils::save;
/// use elevenlabs_rs::endpoints::genai::music::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = MusicComposeBody::from_prompt("An uplifting indie pop anthem")
///         .with_music_length_ms(30_000);
///     let resp = c.hit(ComposeMusic::new(body)).await?;
///     save("music.mp3", resp)?;
///     Ok(())
/// }
/// ```
/// See [Compose Music API reference](https://elevenlabs.io/docs/api-reference/music/compose).
#[derive(Clone, Debug)]
pub struct ComposeMusic {
    body: MusicComposeBody,
    query: Option<MusicQuery>,
}

impl ComposeMusic {
    pub fn new(body: MusicComposeBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: MusicQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl crate::endpoints::sealed::Sealed for ComposeMusic {}

impl ElevenLabsEndpoint for ComposeMusic {
    const PATH: &'static str = "/v1/music";

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

// =============================================================================
// POST /v1/music/stream — Stream Composed Music
// =============================================================================

/// Stream a composed song as the audio is generated.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::utils::stream_audio;
/// use elevenlabs_rs::endpoints::genai::music::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = MusicComposeBody::from_prompt("A lo-fi hip hop beat to study to");
///     let mut stream = c.hit(StreamMusic::new(body)).await?;
///     stream_audio(&mut stream).await?;
///     Ok(())
/// }
/// ```
/// See [Stream Composed Music API reference](https://elevenlabs.io/docs/api-reference/music/stream).
#[derive(Clone, Debug)]
pub struct StreamMusic {
    body: MusicComposeBody,
    query: Option<MusicQuery>,
}

impl StreamMusic {
    pub fn new(body: MusicComposeBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: MusicQuery) -> Self {
        self.query = Some(query);
        self
    }
}

type StreamMusicResponse = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
impl crate::endpoints::sealed::Sealed for StreamMusic {}

impl ElevenLabsEndpoint for StreamMusic {
    const PATH: &'static str = "/v1/music/stream";

    const METHOD: Method = Method::POST;

    type ResponseBody = StreamMusicResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream().map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
}

// =============================================================================
// POST /v1/music/detailed — Compose Music With A Detailed Response
// =============================================================================

/// Compose a song and return the audio together with the composition plan,
/// song metadata, and optional word timestamps.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::utils::save;
/// use elevenlabs_rs::endpoints::genai::music::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = MusicComposeBody::from_prompt("A cinematic orchestral build-up");
///     let resp = c.hit(ComposeMusicDetailed::new(body).with_timestamps(true)).await?;
///     if let Some(metadata) = &resp.song_metadata {
///         println!("title: {:?}", metadata.title);
///     }
///     save("music.mp3", resp.audio)?;
///     Ok(())
/// }
/// ```
/// See [Compose Music With A Detailed Response API reference](https://elevenlabs.io/docs/api-reference/music/compose-detailed).
#[derive(Clone, Debug)]
pub struct ComposeMusicDetailed {
    body: MusicComposeBody,
    query: Option<MusicQuery>,
    with_timestamps: bool,
}

impl ComposeMusicDetailed {
    pub fn new(body: MusicComposeBody) -> Self {
        Self {
            body,
            query: None,
            with_timestamps: false,
        }
    }

    pub fn with_query(mut self, query: MusicQuery) -> Self {
        self.query = Some(query);
        self
    }

    /// Whether to return the word timestamps of the generated song.
    pub fn with_timestamps(mut self, with_timestamps: bool) -> Self {
        self.with_timestamps = with_timestamps;
        self
    }
}

/// The parsed response of [`ComposeMusicDetailed`].
#[derive(Clone, Debug)]
pub struct DetailedMusic {
    /// The unique identifier of the generated song, from the `song-id` header.
    pub song_id: Option<String>,
    /// The composition plan used to generate the song.
    pub composition_plan: Option<MusicCompositionPlan>,
    /// The metadata of the generated song.
    pub song_metadata: Option<SongMetadata>,
    /// The word timestamps of the generated song, if requested.
    pub words_timestamps: Option<Vec<WordTimestamp>>,
    /// The generated audio.
    pub audio: Bytes,
}

#[derive(Debug, Deserialize)]
struct DetailedMusicMetadata {
    composition_plan: Option<MusicCompositionPlan>,
    song_metadata: Option<SongMetadata>,
    words_timestamps: Option<Vec<WordTimestamp>>,
}

impl crate::endpoints::sealed::Sealed for ComposeMusicDetailed {}

impl ElevenLabsEndpoint for ComposeMusicDetailed {
    const PATH: &'static str = "/v1/music/detailed";

    const METHOD: Method = Method::POST;

    type ResponseBody = DetailedMusic;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn request_body(&self) -> Result<RequestBody> {
        let mut value = serde_json::to_value(&self.body)?;
        if self.with_timestamps {
            if let Some(object) = value.as_object_mut() {
                object.insert("with_timestamps".to_owned(), Value::Bool(true));
            }
        }
        Ok(RequestBody::Json(value))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let song_id = resp
            .headers()
            .get("song-id")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                crate::error::Error::InvalidInput(
                    "detailed music response is missing a content-type header".to_owned(),
                )
            })?;

        let boundary = multipart_boundary(&content_type).ok_or_else(|| {
            crate::error::Error::InvalidInput(format!(
                "detailed music response is not multipart: {content_type}"
            ))
        })?;

        let body = resp.bytes().await?;
        let parts = parse_multipart(&boundary, &body);

        let mut metadata = DetailedMusicMetadata {
            composition_plan: None,
            song_metadata: None,
            words_timestamps: None,
        };
        let mut audio = Bytes::new();

        for part in parts {
            let part_type = part
                .headers
                .get("content-type")
                .map(String::as_str)
                .unwrap_or_default();

            if part_type.contains("application/json") {
                metadata = serde_json::from_slice(&part.body)?;
            } else if part_type.starts_with("audio/") || part_type.contains("octet-stream") {
                audio = Bytes::copy_from_slice(&part.body);
            }
        }

        Ok(DetailedMusic {
            song_id,
            composition_plan: metadata.composition_plan,
            song_metadata: metadata.song_metadata,
            words_timestamps: metadata.words_timestamps,
            audio,
        })
    }
}

// =============================================================================
// POST /v1/music/plan — Generate Composition Plan
// =============================================================================

/// Request body for [`GenerateCompositionPlan`].
#[derive(Clone, Debug, Serialize)]
pub struct CompositionPlanBody {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    music_length_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_composition_plan: Option<MusicCompositionPlan>,
    model_id: MusicModel,
}

impl CompositionPlanBody {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            music_length_ms: None,
            source_composition_plan: None,
            model_id: MusicModel::default(),
        }
    }

    /// The model to generate the plan for. Determines the returned plan format.
    pub fn with_model(mut self, model_id: MusicModel) -> Self {
        self.model_id = model_id;
        self
    }

    /// The length of the composition plan in milliseconds. Must be between
    /// 3000ms and 600000ms.
    pub fn with_music_length_ms(mut self, music_length_ms: u32) -> Self {
        self.music_length_ms = Some(music_length_ms);
        self
    }

    /// An optional composition plan to use as a source for the new plan.
    pub fn with_source_composition_plan(
        mut self,
        plan: impl Into<MusicCompositionPlan>,
    ) -> Self {
        self.source_composition_plan = Some(plan.into());
        self
    }
}

/// Generate a composition plan from a prompt.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::music::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = CompositionPlanBody::new("A three-part synthwave track")
///         .with_music_length_ms(45_000);
///     let plan = c.hit(GenerateCompositionPlan::new(body)).await?;
///     // Feed the plan straight into a composition.
///     let compose = MusicComposeBody::from_composition_plan(plan);
///     let _audio = c.hit(ComposeMusic::new(compose)).await?;
///     Ok(())
/// }
/// ```
/// See [Generate Composition Plan API reference](https://elevenlabs.io/docs/api-reference/music/composition-plan/create).
#[derive(Clone, Debug)]
pub struct GenerateCompositionPlan {
    body: CompositionPlanBody,
}

impl GenerateCompositionPlan {
    pub fn new(body: CompositionPlanBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for GenerateCompositionPlan {}

impl ElevenLabsEndpoint for GenerateCompositionPlan {
    const PATH: &'static str = "/v1/music/plan";

    const METHOD: Method = Method::POST;

    type ResponseBody = MusicCompositionPlan;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/music/stem-separation — Stem Separation
// =============================================================================

/// The stem variation to use for separation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StemVariation {
    /// Separate the song into two stems.
    TwoStems,
    /// Separate the song into six stems.
    #[default]
    SixStems,
}

impl StemVariation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TwoStems => "two_stems_v1",
            Self::SixStems => "six_stems_v1",
        }
    }
}

/// Request body for [`SeparateStems`].
#[derive(Clone, Debug)]
pub struct StemSeparationBody {
    file: FilePart,
    stem_variation: StemVariation,
    sign_with_c2pa: bool,
}

impl StemSeparationBody {
    pub fn new(file: impl Into<FilePart>) -> Self {
        Self {
            file: file.into(),
            stem_variation: StemVariation::default(),
            sign_with_c2pa: false,
        }
    }

    pub fn from_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(FilePart::bytes(file_name, mime, bytes))
    }

    pub fn with_stem_variation(mut self, stem_variation: StemVariation) -> Self {
        self.stem_variation = stem_variation;
        self
    }

    pub fn with_sign_with_c2pa(mut self, sign: bool) -> Self {
        self.sign_with_c2pa = sign;
        self
    }
}

impl TryFrom<&StemSeparationBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &StemSeparationBody) -> Result<Self> {
        let inferred_mime = inferred_audio_mime(&body.file)?;
        let form = Form::new()
            .part("file", body.file.clone().into_part(inferred_mime)?)
            .text("stem_variation_id", body.stem_variation.as_str())
            .text("sign_with_c2pa", body.sign_with_c2pa.to_string());
        Ok(RequestBody::Multipart(form))
    }
}

/// Separate an audio file into individual stems.
///
/// The response is a ZIP archive containing one audio file per stem (e.g.
/// `vocals.mp3`, `drums.mp3`, `bass.mp3`, ...).
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::utils::save;
/// use elevenlabs_rs::endpoints::genai::music::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = StemSeparationBody::new("song.mp3").with_stem_variation(StemVariation::TwoStems);
///     let zip = c.hit(SeparateStems::new(body)).await?;
///     save("stems.zip", zip)?;
///     Ok(())
/// }
/// ```
/// See [Stem Separation API reference](https://elevenlabs.io/docs/api-reference/music/stem-separation).
#[derive(Clone, Debug)]
pub struct SeparateStems {
    body: StemSeparationBody,
    query: Option<MusicQuery>,
}

impl SeparateStems {
    pub fn new(body: StemSeparationBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: MusicQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl crate::endpoints::sealed::Sealed for SeparateStems {}

impl ElevenLabsEndpoint for SeparateStems {
    const PATH: &'static str = "/v1/music/stem-separation";

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

// =============================================================================
// POST /v1/music/upload — Upload Music
// =============================================================================

/// Which composition-plan format to extract from an uploaded song.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtractCompositionPlan {
    /// Extract a `music_v1` ([`MusicPrompt`]) composition plan.
    V1,
    /// Extract a `music_v2` ([`CompositionPlan`]) composition plan.
    V2,
}

impl ExtractCompositionPlan {
    fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "music_v1",
            Self::V2 => "music_v2",
        }
    }
}

/// Request body for [`UploadMusic`].
#[derive(Clone, Debug)]
pub struct UploadMusicBody {
    file: FilePart,
    extract_composition_plan: Option<ExtractCompositionPlan>,
    with_timestamps: bool,
}

impl UploadMusicBody {
    pub fn new(file: impl Into<FilePart>) -> Self {
        Self {
            file: file.into(),
            extract_composition_plan: None,
            with_timestamps: false,
        }
    }

    pub fn from_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(FilePart::bytes(file_name, mime, bytes))
    }

    /// Generate and return the composition plan for the uploaded song in the
    /// given format. Increases latency.
    pub fn with_extract_composition_plan(mut self, format: ExtractCompositionPlan) -> Self {
        self.extract_composition_plan = Some(format);
        self
    }

    /// Transcribe the uploaded song and return word-level timestamps. Increases
    /// latency.
    pub fn with_timestamps(mut self, with_timestamps: bool) -> Self {
        self.with_timestamps = with_timestamps;
        self
    }
}

impl TryFrom<&UploadMusicBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &UploadMusicBody) -> Result<Self> {
        let inferred_mime = inferred_audio_mime(&body.file)?;
        let mut form = Form::new().part("file", body.file.clone().into_part(inferred_mime)?);
        if let Some(format) = body.extract_composition_plan {
            form = form.text("extract_composition_plan", format.as_str());
        }
        if body.with_timestamps {
            form = form.text("with_timestamps", "true");
        }
        Ok(RequestBody::Multipart(form))
    }
}

/// The response of [`UploadMusic`].
#[derive(Clone, Debug, Deserialize)]
pub struct MusicUploadResponse {
    /// Unique identifier for the uploaded song.
    pub song_id: String,
    /// The composition plan extracted from the uploaded song, if requested.
    pub composition_plan: Option<MusicCompositionPlan>,
    /// Word-level timestamps transcribed from the uploaded song, if requested.
    pub words_timestamps: Option<Vec<WordTimestamp>>,
}

/// Upload a music file to be later used for inpainting.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::music::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = UploadMusicBody::new("song.mp3")
///         .with_extract_composition_plan(ExtractCompositionPlan::V2);
///     let resp = c.hit(UploadMusic::new(body)).await?;
///     println!("song id: {}", resp.song_id);
///     Ok(())
/// }
/// ```
/// See [Upload Music API reference](https://elevenlabs.io/docs/api-reference/music/upload).
#[derive(Clone, Debug)]
pub struct UploadMusic {
    body: UploadMusicBody,
}

impl UploadMusic {
    pub fn new(body: UploadMusicBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for UploadMusic {}

impl ElevenLabsEndpoint for UploadMusic {
    const PATH: &'static str = "/v1/music/upload";

    const METHOD: Method = Method::POST;

    type ResponseBody = MusicUploadResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn inferred_audio_mime(file: &FilePart) -> Result<Option<String>> {
    if file.mime().is_some() {
        return Ok(None);
    }
    let extension = file.extension()?;
    Ok(Some(audio_mime_from_extension(&extension)?.to_owned()))
}

/// A single parsed part of a `multipart/mixed` response.
struct MultipartPart {
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

/// Extract the boundary token from a `multipart/*` content-type header.
fn multipart_boundary(content_type: &str) -> Option<String> {
    if !content_type.to_ascii_lowercase().contains("multipart/") {
        return None;
    }
    content_type.split(';').find_map(|param| {
        let param = param.trim();
        param
            .strip_prefix("boundary=")
            .or_else(|| param.strip_prefix("boundary ="))
            .map(|value| value.trim().trim_matches('"').to_owned())
    })
}

/// Find every start index of `needle` within `haystack`.
fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return Vec::new();
    }
    let mut indices = Vec::new();
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        if &haystack[i..i + needle.len()] == needle {
            indices.push(i);
            i += needle.len();
        } else {
            i += 1;
        }
    }
    indices
}

/// Parse a `multipart/mixed` body into its constituent parts.
fn parse_multipart(boundary: &str, body: &[u8]) -> Vec<MultipartPart> {
    let delimiter = format!("--{boundary}");
    let positions = find_all(body, delimiter.as_bytes());
    let mut parts = Vec::new();

    for window in positions.windows(2) {
        let start = window[0] + delimiter.len();
        let end = window[1];
        if start > end {
            continue;
        }
        let mut segment = &body[start..end];

        // Skip the CRLF (or LF) immediately following the boundary line.
        if segment.starts_with(b"\r\n") {
            segment = &segment[2..];
        } else if segment.starts_with(b"\n") {
            segment = &segment[1..];
        } else if segment.starts_with(b"--") {
            // Closing boundary's trailing "--"; not a part.
            continue;
        }

        // Drop the trailing CRLF that precedes the next boundary.
        if segment.ends_with(b"\r\n") {
            segment = &segment[..segment.len() - 2];
        } else if segment.ends_with(b"\n") {
            segment = &segment[..segment.len() - 1];
        }

        let header_end = find_all(segment, b"\r\n\r\n")
            .first()
            .copied()
            .map(|i| (i, 4))
            .or_else(|| find_all(segment, b"\n\n").first().copied().map(|i| (i, 2)));

        let Some((header_len, sep_len)) = header_end else {
            continue;
        };

        let headers = parse_headers(&segment[..header_len]);
        let part_body = segment[header_len + sep_len..].to_vec();
        parts.push(MultipartPart {
            headers,
            body: part_body,
        });
    }

    parts
}

/// Parse the header block of a multipart part into a lowercase-keyed map.
fn parse_headers(bytes: &[u8]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let text = String::from_utf8_lossy(bytes);
    for line in text.split("\r\n").flat_map(|line| line.split('\n')) {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    headers
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compose_body_from_prompt_serializes_minimally() {
        let body = MusicComposeBody::from_prompt("an upbeat track").with_music_length_ms(30_000);
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            json!({
                "prompt": "an upbeat track",
                "music_length_ms": 30_000,
                "model_id": "music_v1",
            })
        );
    }

    #[test]
    fn compose_body_from_v2_plan_sets_model() {
        let plan = CompositionPlan::new().add_chunk(
            GenerationChunk::new("[Verse]\nCity lights", 15_000)
                .with_positive_styles(["pop", "warm synths"]),
        );
        let body = MusicComposeBody::from_composition_plan(plan);
        let value = serde_json::to_value(&body).unwrap();

        assert_eq!(value["model_id"], "music_v2");
        assert_eq!(value["composition_plan"]["chunks"][0]["duration_ms"], 15_000);
        assert!(value.get("prompt").is_none());
    }

    #[test]
    fn composition_plan_union_round_trips_both_variants() {
        let v1 = json!({
            "positive_global_styles": ["pop"],
            "negative_global_styles": ["metal"],
            "sections": [],
        });
        let plan: MusicCompositionPlan = serde_json::from_value(v1).unwrap();
        assert!(matches!(plan, MusicCompositionPlan::V1(_)));
        assert_eq!(plan.model(), MusicModel::MusicV1);

        let v2 = json!({ "chunks": [] });
        let plan: MusicCompositionPlan = serde_json::from_value(v2).unwrap();
        assert!(matches!(plan, MusicCompositionPlan::V2(_)));
        assert_eq!(plan.model(), MusicModel::MusicV2);
    }

    #[test]
    fn composition_chunk_union_distinguishes_generation_and_audio_ref() {
        let chunk: CompositionChunk =
            serde_json::from_value(json!({ "song_id": "abc", "range": { "start_ms": 0, "end_ms": 5000 } }))
                .unwrap();
        assert!(matches!(chunk, CompositionChunk::AudioRef(_)));

        let chunk: CompositionChunk = serde_json::from_value(json!({
            "text": "[Verse]",
            "duration_ms": 5000,
            "positive_styles": ["pop"],
        }))
        .unwrap();
        assert!(matches!(chunk, CompositionChunk::Generation(_)));
    }

    #[test]
    fn boundary_is_extracted_from_content_type() {
        assert_eq!(
            multipart_boundary("multipart/mixed; boundary=\"abc123\"").as_deref(),
            Some("abc123")
        );
        assert_eq!(
            multipart_boundary("multipart/mixed; boundary=xyz").as_deref(),
            Some("xyz")
        );
        assert_eq!(multipart_boundary("application/json"), None);
    }

    #[test]
    fn multipart_response_is_parsed_into_json_and_audio() {
        let boundary = "boundary42";
        let json_part = r#"{"composition_plan":{"chunks":[]},"song_metadata":{"title":"Song","description":null,"genres":["pop"],"languages":["en"],"is_explicit":false},"words_timestamps":null}"#;
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
        body.extend_from_slice(json_part.as_bytes());
        body.extend_from_slice(format!("\r\n--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Type: audio/mpeg\r\n\r\n");
        body.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let parts = parse_multipart(boundary, &body);
        assert_eq!(parts.len(), 2);

        let metadata: DetailedMusicMetadata =
            serde_json::from_slice(&parts[0].body).unwrap();
        assert_eq!(
            metadata.song_metadata.unwrap().title.as_deref(),
            Some("Song")
        );
        assert!(matches!(
            metadata.composition_plan,
            Some(MusicCompositionPlan::V2(_))
        ));

        assert_eq!(
            parts[1].headers.get("content-type").map(String::as_str),
            Some("audio/mpeg")
        );
        assert_eq!(parts[1].body, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }
}
