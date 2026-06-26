//! The Professional Voice Cloning (PVC) voice endpoints.
//!
//! These endpoints manage the lifecycle of a PVC voice: creating and editing
//! the voice, uploading and managing samples, separating speakers within a
//! sample, solving the verification captcha, requesting manual verification,
//! and kicking off training.
//!
//! See the [PVC Voices API reference](https://elevenlabs.io/docs/api-reference/voices/pvc).

use super::*;
use crate::shared::{audio_mime_from_extension, FilePart};
use std::collections::HashMap;

// =============================================================================
// POST /v1/voices/pvc — Create PVC Voice
// =============================================================================

/// Creates a new PVC voice with metadata but no samples.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pvc_voices::{CreatePvcVoice, CreatePvcVoiceBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = CreatePvcVoiceBody::new("My Voice", "en").with_description("A warm narration voice");
///     let resp = c.hit(CreatePvcVoice::new(body)).await?;
///     println!("{}", resp.voice_id);
///     Ok(())
/// }
/// ```
/// See [Create PVC Voice API reference](https://elevenlabs.io/docs/api-reference/voices/create-pvc-voice).
#[derive(Clone, Debug)]
pub struct CreatePvcVoice {
    body: CreatePvcVoiceBody,
}

impl CreatePvcVoice {
    pub fn new(body: CreatePvcVoiceBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for CreatePvcVoice {}

impl ElevenLabsEndpoint for CreatePvcVoice {
    const PATH: &'static str = "/v1/voices/pvc";

    const METHOD: Method = Method::POST;

    type ResponseBody = PvcVoiceResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Create-PVC-voice body.
#[derive(Clone, Debug, Serialize)]
pub struct CreatePvcVoiceBody {
    /// The name that identifies this voice.
    name: String,
    /// Language used in the samples (e.g. `en`).
    language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Labels for the voice. Keys can be `language`, `accent`, `gender`, or `age`.
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<HashMap<String, String>>,
}

impl CreatePvcVoiceBody {
    pub fn new(name: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            language: language.into(),
            description: None,
            labels: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = Some(labels);
        self
    }
}

/// Response holding the ID of a created or edited PVC voice.
#[derive(Clone, Debug, Deserialize)]
pub struct PvcVoiceResponse {
    pub voice_id: String,
}

// =============================================================================
// POST /v1/voices/pvc/{voice_id} — Edit PVC Voice
// =============================================================================

/// Edits the metadata of an existing PVC voice. All fields are optional.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pvc_voices::{EditPvcVoice, EditPvcVoiceBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = EditPvcVoiceBody::default().with_name("Renamed Voice");
///     let resp = c.hit(EditPvcVoice::new("voice_id", body)).await?;
///     println!("{}", resp.voice_id);
///     Ok(())
/// }
/// ```
/// See [Edit PVC Voice API reference](https://elevenlabs.io/docs/api-reference/voices/edit-pvc-voice).
#[derive(Clone, Debug)]
pub struct EditPvcVoice {
    voice_id: String,
    body: EditPvcVoiceBody,
}

impl EditPvcVoice {
    pub fn new(voice_id: impl Into<String>, body: EditPvcVoiceBody) -> Self {
        Self {
            voice_id: voice_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for EditPvcVoice {}

impl ElevenLabsEndpoint for EditPvcVoice {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id";

    const METHOD: Method = Method::POST;

    type ResponseBody = PvcVoiceResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Edit-PVC-voice body. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditPvcVoiceBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<HashMap<String, String>>,
}

impl EditPvcVoiceBody {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = Some(labels);
        self
    }
}

// =============================================================================
// GET /v1/voices/pvc/{voice_id}/captcha — Get PVC Voice Captcha
// =============================================================================

/// Retrieves the captcha that must be solved to verify a PVC voice. The
/// response is the raw captcha payload (e.g. an image) to be presented to the
/// user.
///
/// See [Get PVC Voice Captcha API reference](https://elevenlabs.io/docs/api-reference/voices/get-pvc-captcha).
#[derive(Clone, Debug)]
pub struct GetPvcVoiceCaptcha {
    voice_id: String,
}

impl GetPvcVoiceCaptcha {
    pub fn new(voice_id: impl Into<String>) -> Self {
        Self {
            voice_id: voice_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetPvcVoiceCaptcha {}

impl ElevenLabsEndpoint for GetPvcVoiceCaptcha {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/captcha";

    const METHOD: Method = Method::GET;

    type ResponseBody = Bytes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

// =============================================================================
// POST /v1/voices/pvc/{voice_id}/captcha — Verify PVC Voice Captcha
// =============================================================================

/// Verifies a PVC voice captcha by submitting an audio recording of the user.
///
/// See [Verify PVC Voice Captcha API reference](https://elevenlabs.io/docs/api-reference/voices/verify-pvc-captcha).
#[derive(Clone, Debug)]
pub struct VerifyPvcVoiceCaptcha {
    voice_id: String,
    recording: FilePart,
}

impl VerifyPvcVoiceCaptcha {
    pub fn new(voice_id: impl Into<String>, recording: impl Into<FilePart>) -> Self {
        Self {
            voice_id: voice_id.into(),
            recording: recording.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for VerifyPvcVoiceCaptcha {}

impl ElevenLabsEndpoint for VerifyPvcVoiceCaptcha {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/captcha";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        let form = Form::new().part("recording", audio_part(&self.recording)?);
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/voices/pvc/{voice_id}/samples — Add Samples To PVC Voice
// =============================================================================

/// Adds one or more audio samples to a PVC voice.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pvc_voices::{AddPvcVoiceSamples, AddPvcVoiceSamplesBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = AddPvcVoiceSamplesBody::new(["sample1.mp3", "sample2.mp3"])
///         .with_remove_background_noise(true);
///     let samples = c.hit(AddPvcVoiceSamples::new("voice_id", body)).await?;
///     println!("added {} samples", samples.len());
///     Ok(())
/// }
/// ```
/// See [Add Samples To PVC Voice API reference](https://elevenlabs.io/docs/api-reference/voices/add-pvc-samples).
#[derive(Clone, Debug)]
pub struct AddPvcVoiceSamples {
    voice_id: String,
    body: AddPvcVoiceSamplesBody,
}

impl AddPvcVoiceSamples {
    pub fn new(voice_id: impl Into<String>, body: AddPvcVoiceSamplesBody) -> Self {
        Self {
            voice_id: voice_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for AddPvcVoiceSamples {}

impl ElevenLabsEndpoint for AddPvcVoiceSamples {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/samples";

    const METHOD: Method = Method::POST;

    type ResponseBody = Vec<SampleResponse>;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        let mut form = Form::new();
        for file in &self.body.files {
            form = form.part("files", audio_part(file)?);
        }
        if let Some(remove) = self.body.remove_background_noise {
            form = form.text("remove_background_noise", remove.to_string());
        }
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Add-samples body.
#[derive(Clone, Debug)]
pub struct AddPvcVoiceSamplesBody {
    files: Vec<FilePart>,
    remove_background_noise: Option<bool>,
}

impl AddPvcVoiceSamplesBody {
    pub fn new(files: impl IntoIterator<Item = impl Into<FilePart>>) -> Self {
        Self {
            files: files.into_iter().map(Into::into).collect(),
            remove_background_noise: None,
        }
    }

    pub fn add_file(mut self, file: impl Into<FilePart>) -> Self {
        self.files.push(file.into());
        self
    }

    /// Remove background noise from the samples using the audio isolation model.
    pub fn with_remove_background_noise(mut self, remove: bool) -> Self {
        self.remove_background_noise = Some(remove);
        self
    }
}

// =============================================================================
// POST /v1/voices/pvc/{voice_id}/samples/{sample_id} — Update PVC Voice Sample
// =============================================================================

/// Updates an existing PVC voice sample, e.g. trimming it or selecting speakers
/// for training.
///
/// See [Update PVC Voice Sample API reference](https://elevenlabs.io/docs/api-reference/voices/update-pvc-sample).
#[derive(Clone, Debug)]
pub struct UpdatePvcVoiceSample {
    voice_id: String,
    sample_id: String,
    body: UpdatePvcVoiceSampleBody,
}

impl UpdatePvcVoiceSample {
    pub fn new(
        voice_id: impl Into<String>,
        sample_id: impl Into<String>,
        body: UpdatePvcVoiceSampleBody,
    ) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for UpdatePvcVoiceSample {}

impl ElevenLabsEndpoint for UpdatePvcVoiceSample {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/samples/:sample_id";

    const METHOD: Method = Method::POST;

    type ResponseBody = PvcVoiceResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.and_param(PathParam::VoiceID),
            self.sample_id.and_param(PathParam::SampleID),
        ]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Update-sample body. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdatePvcVoiceSampleBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    remove_background_noise: Option<bool>,
    /// Speaker IDs to use for PVC training. Send all desired speaker IDs in one
    /// request; the last request overrides previous ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_speaker_ids: Option<Vec<String>>,
    /// The start time of the audio to use for training, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    trim_start_time: Option<u32>,
    /// The end time of the audio to use for training, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    trim_end_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<String>,
}

impl UpdatePvcVoiceSampleBody {
    pub fn with_remove_background_noise(mut self, remove: bool) -> Self {
        self.remove_background_noise = Some(remove);
        self
    }

    pub fn with_selected_speaker_ids<I, S>(mut self, speaker_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.selected_speaker_ids = Some(speaker_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_trim_start_time(mut self, trim_start_time: u32) -> Self {
        self.trim_start_time = Some(trim_start_time);
        self
    }

    pub fn with_trim_end_time(mut self, trim_end_time: u32) -> Self {
        self.trim_end_time = Some(trim_end_time);
        self
    }

    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }
}

// =============================================================================
// DELETE /v1/voices/pvc/{voice_id}/samples/{sample_id} — Delete PVC Voice Sample
// =============================================================================

/// Deletes a sample from a PVC voice.
///
/// See [Delete PVC Voice Sample API reference](https://elevenlabs.io/docs/api-reference/voices/delete-pvc-sample).
#[derive(Clone, Debug)]
pub struct DeletePvcVoiceSample {
    voice_id: String,
    sample_id: String,
}

impl DeletePvcVoiceSample {
    pub fn new(voice_id: impl Into<String>, sample_id: impl Into<String>) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeletePvcVoiceSample {}

impl ElevenLabsEndpoint for DeletePvcVoiceSample {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/samples/:sample_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.and_param(PathParam::VoiceID),
            self.sample_id.and_param(PathParam::SampleID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// GET /v1/voices/pvc/{voice_id}/samples/{sample_id}/audio — Retrieve Sample Audio
// =============================================================================

/// Retrieves the audio of a PVC voice sample.
///
/// See [Retrieve Voice Sample Audio API reference](https://elevenlabs.io/docs/api-reference/voices/get-pvc-sample-audio).
#[derive(Clone, Debug)]
pub struct GetPvcSampleAudio {
    voice_id: String,
    sample_id: String,
    query: Option<GetPvcSampleAudioQuery>,
}

impl GetPvcSampleAudio {
    pub fn new(voice_id: impl Into<String>, sample_id: impl Into<String>) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: GetPvcSampleAudioQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetPvcSampleAudio`].
#[derive(Clone, Debug, Default)]
pub struct GetPvcSampleAudioQuery {
    params: QueryValues,
}

impl GetPvcSampleAudioQuery {
    /// Return the audio with background noise removed, if available.
    pub fn with_remove_background_noise(mut self, remove: bool) -> Self {
        self.params
            .push(("remove_background_noise", remove.to_string()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetPvcSampleAudio {}

impl ElevenLabsEndpoint for GetPvcSampleAudio {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/samples/:sample_id/audio";

    const METHOD: Method = Method::GET;

    type ResponseBody = VoiceSamplePreview;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.and_param(PathParam::VoiceID),
            self.sample_id.and_param(PathParam::SampleID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A base64-encoded preview of a voice sample's audio.
#[derive(Clone, Debug, Deserialize)]
pub struct VoiceSamplePreview {
    pub audio_base_64: String,
    pub voice_id: String,
    pub sample_id: String,
    pub media_type: String,
    pub duration_secs: Option<f64>,
}

// =============================================================================
// POST .../samples/{sample_id}/separate-speakers — Start Speaker Separation
// =============================================================================

/// Starts speaker separation for a PVC voice sample. Separation runs
/// asynchronously; poll [`GetSpeakerSeparationStatus`] for progress.
///
/// See [Start Speaker Separation API reference](https://elevenlabs.io/docs/api-reference/voices/start-speaker-separation).
#[derive(Clone, Debug)]
pub struct StartSpeakerSeparation {
    voice_id: String,
    sample_id: String,
}

impl StartSpeakerSeparation {
    pub fn new(voice_id: impl Into<String>, sample_id: impl Into<String>) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for StartSpeakerSeparation {}

impl ElevenLabsEndpoint for StartSpeakerSeparation {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/samples/:sample_id/separate-speakers";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.and_param(PathParam::VoiceID),
            self.sample_id.and_param(PathParam::SampleID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// GET .../samples/{sample_id}/speakers — Retrieve Speaker Separation Status
// =============================================================================

/// Retrieves the status (and, once complete, the speakers) of a sample's
/// speaker separation.
///
/// See [Retrieve Speaker Separation Status API reference](https://elevenlabs.io/docs/api-reference/voices/get-speaker-separation-status).
#[derive(Clone, Debug)]
pub struct GetSpeakerSeparationStatus {
    voice_id: String,
    sample_id: String,
}

impl GetSpeakerSeparationStatus {
    pub fn new(voice_id: impl Into<String>, sample_id: impl Into<String>) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetSpeakerSeparationStatus {}

impl ElevenLabsEndpoint for GetSpeakerSeparationStatus {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/samples/:sample_id/speakers";

    const METHOD: Method = Method::GET;

    type ResponseBody = SpeakerSeparation;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.and_param(PathParam::VoiceID),
            self.sample_id.and_param(PathParam::SampleID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Speaker separation status and result for a sample.
#[derive(Clone, Debug, Deserialize)]
pub struct SpeakerSeparation {
    pub voice_id: String,
    pub sample_id: String,
    pub status: SpeakerSeparationStatus,
    /// Map of speaker ID to speaker, present once separation has completed.
    pub speakers: Option<HashMap<String, Speaker>>,
    pub selected_speaker_ids: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerSeparationStatus {
    NotStarted,
    Pending,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Speaker {
    pub speaker_id: String,
    pub duration_secs: f64,
    pub utterances: Option<Vec<Utterance>>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Utterance {
    pub start: f64,
    pub end: f64,
}

// =============================================================================
// GET .../speakers/{speaker_id}/audio — Retrieve Separated Speaker Audio
// =============================================================================

/// Retrieves the separated audio of a single speaker within a sample.
///
/// See [Retrieve Separated Speaker Audio API reference](https://elevenlabs.io/docs/api-reference/voices/get-speaker-audio).
#[derive(Clone, Debug)]
pub struct GetSeparatedSpeakerAudio {
    voice_id: String,
    sample_id: String,
    speaker_id: String,
}

impl GetSeparatedSpeakerAudio {
    pub fn new(
        voice_id: impl Into<String>,
        sample_id: impl Into<String>,
        speaker_id: impl Into<String>,
    ) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
            speaker_id: speaker_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetSeparatedSpeakerAudio {}

impl ElevenLabsEndpoint for GetSeparatedSpeakerAudio {
    const PATH: &'static str =
        "/v1/voices/pvc/:voice_id/samples/:sample_id/speakers/:speaker_id/audio";

    const METHOD: Method = Method::GET;

    type ResponseBody = SpeakerAudio;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.and_param(PathParam::VoiceID),
            self.sample_id.and_param(PathParam::SampleID),
            self.speaker_id.and_param(PathParam::SpeakerID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A base64-encoded audio clip of a separated speaker.
#[derive(Clone, Debug, Deserialize)]
pub struct SpeakerAudio {
    pub audio_base_64: String,
    pub media_type: String,
    pub duration_secs: f64,
}

// =============================================================================
// GET .../samples/{sample_id}/waveform — Retrieve Sample Visual Waveform
// =============================================================================

/// Retrieves the visual waveform of a PVC voice sample.
///
/// See [Retrieve Voice Sample Visual Waveform API reference](https://elevenlabs.io/docs/api-reference/voices/get-pvc-sample-waveform).
#[derive(Clone, Debug)]
pub struct GetPvcSampleWaveform {
    voice_id: String,
    sample_id: String,
}

impl GetPvcSampleWaveform {
    pub fn new(voice_id: impl Into<String>, sample_id: impl Into<String>) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetPvcSampleWaveform {}

impl ElevenLabsEndpoint for GetPvcSampleWaveform {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/samples/:sample_id/waveform";

    const METHOD: Method = Method::GET;

    type ResponseBody = VoiceSampleVisualWaveform;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.and_param(PathParam::VoiceID),
            self.sample_id.and_param(PathParam::SampleID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The visual waveform of a voice sample.
#[derive(Clone, Debug, Deserialize)]
pub struct VoiceSampleVisualWaveform {
    pub sample_id: String,
    pub visual_waveform: Vec<f64>,
}

// =============================================================================
// POST /v1/voices/pvc/{voice_id}/train — Run PVC Training
// =============================================================================

/// Starts training for a PVC voice.
///
/// See [Run PVC Training API reference](https://elevenlabs.io/docs/api-reference/voices/run-pvc-training).
#[derive(Clone, Debug)]
pub struct RunPvcTraining {
    voice_id: String,
    body: RunPvcTrainingBody,
}

impl RunPvcTraining {
    pub fn new(voice_id: impl Into<String>) -> Self {
        Self {
            voice_id: voice_id.into(),
            body: RunPvcTrainingBody::default(),
        }
    }

    pub fn with_body(mut self, body: RunPvcTrainingBody) -> Self {
        self.body = body;
        self
    }
}

impl crate::endpoints::sealed::Sealed for RunPvcTraining {}

impl ElevenLabsEndpoint for RunPvcTraining {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/train";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Run-PVC-training body.
#[derive(Clone, Debug, Default, Serialize)]
pub struct RunPvcTrainingBody {
    /// The model ID to use for the conversion.
    #[serde(skip_serializing_if = "Option::is_none")]
    model_id: Option<String>,
}

impl RunPvcTrainingBody {
    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }
}

// =============================================================================
// POST /v1/voices/pvc/{voice_id}/verification — Request Manual Verification
// =============================================================================

/// Requests manual verification of a PVC voice by uploading verification
/// documents.
///
/// See [Request Manual Verification API reference](https://elevenlabs.io/docs/api-reference/voices/request-pvc-verification).
#[derive(Clone, Debug)]
pub struct RequestPvcManualVerification {
    voice_id: String,
    body: RequestPvcManualVerificationBody,
}

impl RequestPvcManualVerification {
    pub fn new(voice_id: impl Into<String>, body: RequestPvcManualVerificationBody) -> Self {
        Self {
            voice_id: voice_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for RequestPvcManualVerification {}

impl ElevenLabsEndpoint for RequestPvcManualVerification {
    const PATH: &'static str = "/v1/voices/pvc/:voice_id/verification";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        let mut form = Form::new();
        for file in &self.body.files {
            form = form.part("files", document_part(file)?);
        }
        if let Some(extra_text) = &self.body.extra_text {
            form = form.text("extra_text", extra_text.clone());
        }
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Request-manual-verification body.
#[derive(Clone, Debug)]
pub struct RequestPvcManualVerificationBody {
    files: Vec<FilePart>,
    extra_text: Option<String>,
}

impl RequestPvcManualVerificationBody {
    pub fn new(files: impl IntoIterator<Item = impl Into<FilePart>>) -> Self {
        Self {
            files: files.into_iter().map(Into::into).collect(),
            extra_text: None,
        }
    }

    pub fn add_file(mut self, file: impl Into<FilePart>) -> Self {
        self.files.push(file.into());
        self
    }

    pub fn with_extra_text(mut self, extra_text: impl Into<String>) -> Self {
        self.extra_text = Some(extra_text.into());
        self
    }
}

/// The response of [`AddPvcVoiceSamples`] — metadata for each uploaded sample.
#[derive(Clone, Debug, Deserialize)]
pub struct SampleResponse {
    pub sample_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub hash: String,
    pub duration_secs: Option<f64>,
    pub remove_background_noise: Option<bool>,
    pub has_isolated_audio: Option<bool>,
    pub has_isolated_audio_preview: Option<bool>,
    pub speaker_separation: Option<SpeakerSeparation>,
    pub trim_start: Option<u32>,
    pub trim_end: Option<u32>,
}

// =============================================================================
// Helpers
// =============================================================================

/// Build a multipart part for an audio file, inferring the MIME type from the
/// extension when the [`FilePart`] does not carry one.
fn audio_part(file: &FilePart) -> Result<Part> {
    let inferred_mime = if file.mime().is_some() {
        None
    } else {
        Some(audio_mime_from_extension(&file.extension()?)?.to_owned())
    };
    file.clone().into_part(inferred_mime)
}

/// Build a multipart part for a verification document. Falls back to
/// `application/octet-stream` when the MIME type cannot be inferred, since
/// verification documents are not necessarily audio.
fn document_part(file: &FilePart) -> Result<Part> {
    let inferred_mime = if file.mime().is_some() {
        None
    } else {
        let from_extension = file
            .extension()
            .ok()
            .and_then(|extension| audio_mime_from_extension(&extension).ok())
            .map(ToOwned::to_owned);
        Some(from_extension.unwrap_or_else(|| "application/octet-stream".to_owned()))
    };
    file.clone().into_part(inferred_mime)
}
