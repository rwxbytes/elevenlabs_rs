//! Speech Engine endpoints and upstream WebSocket protocol types.

use super::*;
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ListSpeechEngines {
    query: Option<ListSpeechEnginesQuery>,
}

impl ListSpeechEngines {
    pub fn new() -> Self {
        Self { query: None }
    }

    pub fn with_query(query: ListSpeechEnginesQuery) -> Self {
        Self { query: Some(query) }
    }
}

impl Default for ListSpeechEngines {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::endpoints::sealed::Sealed for ListSpeechEngines {}

impl ElevenLabsEndpoint for ListSpeechEngines {
    const PATH: &'static str = "/v1/speech-engine";

    const METHOD: Method = Method::GET;

    type ResponseBody = ListSpeechEnginesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|query| query.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ListSpeechEnginesQuery {
    params: QueryValues,
}

impl ListSpeechEnginesQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.params.push(("search", search.into()));
        self
    }

    pub fn with_sort_direction(mut self, sort_direction: impl Into<String>) -> Self {
        self.params.push(("sort_direction", sort_direction.into()));
        self
    }

    pub fn with_sort_by(mut self, sort_by: impl Into<String>) -> Self {
        self.params.push(("sort_by", sort_by.into()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }
}

#[derive(Clone, Debug)]
pub struct CreateSpeechEngine {
    body: CreateSpeechEngineBody,
}

impl CreateSpeechEngine {
    pub fn new(body: CreateSpeechEngineBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for CreateSpeechEngine {}

impl ElevenLabsEndpoint for CreateSpeechEngine {
    const PATH: &'static str = "/v1/speech-engine";

    const METHOD: Method = Method::POST;

    type ResponseBody = SpeechEngineResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug)]
pub struct GetSpeechEngine {
    speech_engine_id: String,
}

impl GetSpeechEngine {
    pub fn new(speech_engine_id: impl Into<String>) -> Self {
        Self {
            speech_engine_id: speech_engine_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetSpeechEngine {}

impl ElevenLabsEndpoint for GetSpeechEngine {
    const PATH: &'static str = "/v1/speech-engine/:speech_engine_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = SpeechEngineResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.speech_engine_id.and_param(PathParam::SpeechEngineID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug)]
pub struct UpdateSpeechEngine {
    speech_engine_id: String,
    body: UpdateSpeechEngineBody,
}

impl UpdateSpeechEngine {
    pub fn new(speech_engine_id: impl Into<String>, body: UpdateSpeechEngineBody) -> Self {
        Self {
            speech_engine_id: speech_engine_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateSpeechEngine {}

impl ElevenLabsEndpoint for UpdateSpeechEngine {
    const PATH: &'static str = "/v1/speech-engine/:speech_engine_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = SpeechEngineResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.speech_engine_id.and_param(PathParam::SpeechEngineID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug)]
pub struct DeleteSpeechEngine {
    speech_engine_id: String,
}

impl DeleteSpeechEngine {
    pub fn new(speech_engine_id: impl Into<String>) -> Self {
        Self {
            speech_engine_id: speech_engine_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteSpeechEngine {}

impl ElevenLabsEndpoint for DeleteSpeechEngine {
    const PATH: &'static str = "/v1/speech-engine/:speech_engine_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.speech_engine_id.and_param(PathParam::SpeechEngineID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateSpeechEngineBody {
    pub speech_engine: SpeechEngineConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asr: Option<SpeechEngineAsrConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<SpeechEngineTtsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<SpeechEngineTurnConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<SpeechEngineConversationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<SpeechEnginePrivacyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_limits: Option<SpeechEngineCallLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<SpeechEngineOverrides>,
}

impl CreateSpeechEngineBody {
    pub fn new(ws_url: impl Into<String>) -> Self {
        Self {
            speech_engine: SpeechEngineConfig::new(ws_url),
            name: None,
            asr: None,
            tts: None,
            turn: None,
            conversation: None,
            privacy: None,
            call_limits: None,
            language: None,
            tags: None,
            overrides: None,
        }
    }

    pub fn with_speech_engine(mut self, speech_engine: SpeechEngineConfig) -> Self {
        self.speech_engine = speech_engine;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_asr(mut self, asr: SpeechEngineAsrConfig) -> Self {
        self.asr = Some(asr);
        self
    }

    pub fn with_tts(mut self, tts: SpeechEngineTtsConfig) -> Self {
        self.tts = Some(tts);
        self
    }

    pub fn with_turn(mut self, turn: SpeechEngineTurnConfig) -> Self {
        self.turn = Some(turn);
        self
    }

    pub fn with_conversation(mut self, conversation: SpeechEngineConversationConfig) -> Self {
        self.conversation = Some(conversation);
        self
    }

    pub fn with_privacy(mut self, privacy: SpeechEnginePrivacyConfig) -> Self {
        self.privacy = Some(privacy);
        self
    }

    pub fn with_call_limits(mut self, call_limits: SpeechEngineCallLimits) -> Self {
        self.call_limits = Some(call_limits);
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_overrides(mut self, overrides: SpeechEngineOverrides) -> Self {
        self.overrides = Some(overrides);
        self
    }
}

impl TryFrom<&CreateSpeechEngineBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &CreateSpeechEngineBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateSpeechEngineBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_engine: Option<SpeechEngineConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asr: Option<SpeechEngineAsrConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<SpeechEngineTtsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<SpeechEngineTurnConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<SpeechEngineConversationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<SpeechEnginePrivacyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_limits: Option<SpeechEngineCallLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<SpeechEngineOverrides>,
}

impl UpdateSpeechEngineBody {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_ws_url(mut self, ws_url: impl Into<String>) -> Self {
        self.speech_engine = Some(SpeechEngineConfig::new(ws_url));
        self
    }

    pub fn with_speech_engine(mut self, speech_engine: SpeechEngineConfig) -> Self {
        self.speech_engine = Some(speech_engine);
        self
    }

    pub fn with_asr(mut self, asr: SpeechEngineAsrConfig) -> Self {
        self.asr = Some(asr);
        self
    }

    pub fn with_tts(mut self, tts: SpeechEngineTtsConfig) -> Self {
        self.tts = Some(tts);
        self
    }

    pub fn with_turn(mut self, turn: SpeechEngineTurnConfig) -> Self {
        self.turn = Some(turn);
        self
    }

    pub fn with_conversation(mut self, conversation: SpeechEngineConversationConfig) -> Self {
        self.conversation = Some(conversation);
        self
    }

    pub fn with_privacy(mut self, privacy: SpeechEnginePrivacyConfig) -> Self {
        self.privacy = Some(privacy);
        self
    }

    pub fn with_call_limits(mut self, call_limits: SpeechEngineCallLimits) -> Self {
        self.call_limits = Some(call_limits);
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_overrides(mut self, overrides: SpeechEngineOverrides) -> Self {
        self.overrides = Some(overrides);
        self
    }
}

impl TryFrom<&UpdateSpeechEngineBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &UpdateSpeechEngineBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListSpeechEnginesResponse {
    pub speech_engines: Vec<SpeechEngineSummaryResponse>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

impl<'a> IntoIterator for &'a ListSpeechEnginesResponse {
    type Item = &'a SpeechEngineSummaryResponse;
    type IntoIter = std::slice::Iter<'a, SpeechEngineSummaryResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.speech_engines.iter()
    }
}

impl IntoIterator for ListSpeechEnginesResponse {
    type Item = SpeechEngineSummaryResponse;
    type IntoIter = std::vec::IntoIter<SpeechEngineSummaryResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.speech_engines.into_iter()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpeechEngineSummaryResponse {
    pub speech_engine_id: String,
    pub name: String,
    pub created_at_unix_secs: u64,
    pub tags: Vec<String>,
    pub access_info: SpeechEngineAccessInfo,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpeechEngineResponse {
    pub speech_engine_id: String,
    pub name: String,
    pub speech_engine: SpeechEngineConfig,
    pub asr: SpeechEngineAsrConfig,
    pub tts: SpeechEngineTtsConfig,
    pub turn: SpeechEngineTurnConfig,
    pub conversation: SpeechEngineConversationConfig,
    pub privacy: SpeechEnginePrivacyConfig,
    pub call_limits: SpeechEngineCallLimits,
    pub language: String,
    pub tags: Vec<String>,
    pub overrides: SpeechEngineOverrides,
    pub metadata: SpeechEngineMetadata,
    pub access_info: Option<SpeechEngineAccessInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpeechEngineConfig {
    pub ws_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_headers: Option<HashMap<String, SpeechEngineRequestHeaderValue>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEngineConfig {
    pub fn new(ws_url: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
            request_headers: None,
            extra: Map::new(),
        }
    }

    pub fn with_request_header(
        mut self,
        name: impl Into<String>,
        value: impl Into<SpeechEngineRequestHeaderValue>,
    ) -> Self {
        self.request_headers
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), value.into());
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SpeechEngineRequestHeaderValue {
    Static(String),
    Secret { secret_id: String },
    DynamicVariable { variable_name: String },
    Other(Value),
}

impl SpeechEngineRequestHeaderValue {
    pub fn secret(secret_id: impl Into<String>) -> Self {
        Self::Secret {
            secret_id: secret_id.into(),
        }
    }

    pub fn dynamic_variable(variable_name: impl Into<String>) -> Self {
        Self::DynamicVariable {
            variable_name: variable_name.into(),
        }
    }
}

impl From<String> for SpeechEngineRequestHeaderValue {
    fn from(value: String) -> Self {
        Self::Static(value)
    }
}

impl From<&str> for SpeechEngineRequestHeaderValue {
    fn from(value: &str) -> Self {
        Self::Static(value.to_owned())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpeechEngineAsrConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input_audio_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEngineAsrConfig {
    pub fn with_quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_user_input_audio_format(mut self, format: impl Into<String>) -> Self {
        self.user_input_audio_format = Some(format.into());
        self
    }

    pub fn with_keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keywords = Some(keywords.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpeechEngineTtsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_voices: Option<Vec<SpeechEngineSupportedVoice>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expressive_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_audio_tags: Option<Vec<SpeechEngineSuggestedAudioTag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_output_audio_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimize_streaming_latency: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_boost: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_normalisation_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronunciation_dictionary_locators: Option<Vec<SpeechEngineDictionaryLocator>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_phoneme_tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_filter: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEngineTtsConfig {
    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn with_voice_id(mut self, voice_id: impl Into<String>) -> Self {
        self.voice_id = Some(voice_id.into());
        self
    }

    pub fn with_agent_output_audio_format(mut self, format: impl Into<String>) -> Self {
        self.agent_output_audio_format = Some(format.into());
        self
    }

    pub fn with_optimize_streaming_latency(mut self, latency: u8) -> Self {
        self.optimize_streaming_latency = Some(latency);
        self
    }

    pub fn with_stability(mut self, stability: f32) -> Self {
        self.stability = Some(stability);
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }

    pub fn with_similarity_boost(mut self, similarity_boost: f32) -> Self {
        self.similarity_boost = Some(similarity_boost);
        self
    }

    pub fn with_text_normalisation_type(
        mut self,
        text_normalisation_type: impl Into<String>,
    ) -> Self {
        self.text_normalisation_type = Some(text_normalisation_type.into());
        self
    }

    pub fn enable_phoneme_tags(mut self, enable_phoneme_tags: bool) -> Self {
        self.enable_phoneme_tags = Some(enable_phoneme_tags);
        self
    }

    pub fn expressive_mode(mut self, expressive_mode: bool) -> Self {
        self.expressive_mode = Some(expressive_mode);
        self
    }

    pub fn with_supported_voices<I>(mut self, supported_voices: I) -> Self
    where
        I: IntoIterator<Item = SpeechEngineSupportedVoice>,
    {
        self.supported_voices = Some(supported_voices.into_iter().collect());
        self
    }

    pub fn with_suggested_audio_tags<I>(mut self, suggested_audio_tags: I) -> Self
    where
        I: IntoIterator<Item = SpeechEngineSuggestedAudioTag>,
    {
        self.suggested_audio_tags = Some(suggested_audio_tags.into_iter().collect());
        self
    }

    pub fn with_pronunciation_dictionary_locators<I>(
        mut self,
        pronunciation_dictionary_locators: I,
    ) -> Self
    where
        I: IntoIterator<Item = SpeechEngineDictionaryLocator>,
    {
        self.pronunciation_dictionary_locators =
            Some(pronunciation_dictionary_locators.into_iter().collect());
        self
    }

    pub fn with_audio_filter(mut self, audio_filter: impl Into<String>) -> Self {
        self.audio_filter = Some(audio_filter.into());
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpeechEngineSupportedVoice {
    pub label: String,
    pub voice_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimize_streaming_latency: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_boost: Option<f32>,
}

impl SpeechEngineSupportedVoice {
    pub fn new(label: impl Into<String>, voice_id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            voice_id: voice_id.into(),
            description: None,
            language: None,
            model_family: None,
            optimize_streaming_latency: None,
            stability: None,
            speed: None,
            similarity_boost: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpeechEngineSuggestedAudioTag {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl SpeechEngineSuggestedAudioTag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpeechEngineDictionaryLocator {
    pub pronunciation_dictionary_id: String,
    pub version_id: Option<String>,
}

impl SpeechEngineDictionaryLocator {
    pub fn new(
        pronunciation_dictionary_id: impl Into<String>,
        version_id: impl Into<String>,
    ) -> Self {
        Self {
            pronunciation_dictionary_id: pronunciation_dictionary_id.into(),
            version_id: Some(version_id.into()),
        }
    }

    pub fn without_version(pronunciation_dictionary_id: impl Into<String>) -> Self {
        Self {
            pronunciation_dictionary_id: pronunciation_dictionary_id.into(),
            version_id: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpeechEngineTurnConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_timeout: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_wait_time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_end_call_timeout: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_eagerness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spelling_patience: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speculative_turn: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retranscribe_on_turn_timeout: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interruption_ignore_terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcribe_on_disabled_interruptions: Option<bool>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEngineTurnConfig {
    pub fn with_turn_timeout(mut self, turn_timeout: f32) -> Self {
        self.turn_timeout = Some(turn_timeout);
        self
    }

    pub fn with_silence_end_call_timeout(mut self, timeout: f32) -> Self {
        self.silence_end_call_timeout = Some(timeout);
        self
    }

    pub fn with_turn_eagerness(mut self, turn_eagerness: impl Into<String>) -> Self {
        self.turn_eagerness = Some(turn_eagerness.into());
        self
    }

    pub fn with_turn_model(mut self, turn_model: impl Into<String>) -> Self {
        self.turn_model = Some(turn_model.into());
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpeechEngineConversationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_events: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring_events: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_attribution: Option<bool>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEngineConversationConfig {
    pub fn with_max_duration_seconds(mut self, max_duration_seconds: u32) -> Self {
        self.max_duration_seconds = Some(max_duration_seconds);
        self
    }

    pub fn with_client_events<I, S>(mut self, client_events: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.client_events = Some(client_events.into_iter().map(Into::into).collect());
        self
    }

    pub fn text_only(mut self, text_only: bool) -> Self {
        self.text_only = Some(text_only);
        self
    }

    pub fn monitoring_enabled(mut self, monitoring_enabled: bool) -> Self {
        self.monitoring_enabled = Some(monitoring_enabled);
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpeechEnginePrivacyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_voice: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_transcript_and_pii: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_to_existing_conversations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_retention_mode: Option<bool>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEnginePrivacyConfig {
    pub fn record_voice(mut self, record_voice: bool) -> Self {
        self.record_voice = Some(record_voice);
        self
    }

    pub fn with_retention_days(mut self, retention_days: i32) -> Self {
        self.retention_days = Some(retention_days);
        self
    }

    pub fn zero_retention_mode(mut self, zero_retention_mode: bool) -> Self {
        self.zero_retention_mode = Some(zero_retention_mode);
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpeechEngineCallLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_concurrency_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bursting_enabled: Option<bool>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEngineCallLimits {
    pub fn with_agent_concurrency_limit(mut self, agent_concurrency_limit: i32) -> Self {
        self.agent_concurrency_limit = Some(agent_concurrency_limit);
        self
    }

    pub fn with_daily_limit(mut self, daily_limit: i32) -> Self {
        self.daily_limit = Some(daily_limit);
        self
    }

    pub fn bursting_enabled(mut self, bursting_enabled: bool) -> Self {
        self.bursting_enabled = Some(bursting_enabled);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpeechEngineOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<bool>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl SpeechEngineOverrides {
    pub fn first_message(mut self, first_message: bool) -> Self {
        self.first_message = Some(first_message);
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpeechEngineMetadata {
    pub created_at_unix_secs: u64,
    pub updated_at_unix_secs: u64,
    pub created_from: Option<String>,
    pub last_updated_from: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpeechEngineAccessInfo {
    pub is_creator: bool,
    pub creator_name: String,
    pub creator_email: String,
    pub role: String,
    pub anonymous_access_level_override: Option<String>,
    pub access_source: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[cfg(feature = "ws")]
pub mod ws {
    //! Framework-neutral Speech Engine upstream WebSocket helpers.

    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use futures_util::{Sink, SinkExt, Stream, StreamExt};
    use hmac::Mac;
    use serde::{de, Serialize};
    use sha2::Digest;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio_tungstenite::tungstenite::protocol::Message;

    use crate::error::{Error, WebSocketDirection, WebSocketError, WebSocketErrorContext};

    pub const AUTHORIZATION_HEADER: &str = "X-Elevenlabs-Speech-Engine-Authorization";
    pub const JWT_ISSUER: &str = "https://api.elevenlabs.io/convai/speech-engine";
    pub const JWT_SUBJECT: &str = "convai_speech_engine_upstream";
    pub const JWT_CLOCK_SKEW_SECONDS: u64 = 60;

    const ENDPOINT_NAME: &str = "speech_engine.upstream";
    const INBOUND_CONTEXT: WebSocketErrorContext =
        WebSocketErrorContext::new(ENDPOINT_NAME, WebSocketDirection::Inbound);
    const OUTBOUND_CONTEXT: WebSocketErrorContext =
        WebSocketErrorContext::new(ENDPOINT_NAME, WebSocketDirection::Outbound);

    type HmacSha256 = hmac::Hmac<sha2::Sha256>;

    #[derive(Clone, Debug, Deserialize)]
    pub struct SpeechEngineAuthorizationClaims {
        pub iss: String,
        pub sub: String,
        pub exp: u64,
        #[serde(default, flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct SpeechEngineJwtHeader {
        alg: String,
    }

    pub fn verify_authorization_token(
        token: &str,
        api_key: &str,
    ) -> Result<SpeechEngineAuthorizationClaims> {
        let token = token.trim().strip_prefix("Bearer ").unwrap_or(token.trim());
        let mut parts = token.split('.');
        let header = parts
            .next()
            .ok_or_else(|| Error::InvalidInput("speech engine JWT is missing header".to_owned()))?;
        let payload = parts.next().ok_or_else(|| {
            Error::InvalidInput("speech engine JWT is missing payload".to_owned())
        })?;
        let signature = parts.next().ok_or_else(|| {
            Error::InvalidInput("speech engine JWT is missing signature".to_owned())
        })?;

        if parts.next().is_some() {
            return Err(Error::InvalidInput(
                "speech engine JWT has too many segments".to_owned(),
            ));
        }

        let header_bytes = URL_SAFE_NO_PAD.decode(header)?;
        let header_json: SpeechEngineJwtHeader = serde_json::from_slice(&header_bytes)?;
        if header_json.alg != "HS256" {
            return Err(Error::InvalidInput(format!(
                "unsupported speech engine JWT alg: {}",
                header_json.alg
            )));
        }

        let signing_input = format!("{header}.{payload}");
        let secret = sha2::Sha256::digest(api_key.as_bytes());
        let mut mac = HmacSha256::new_from_slice(&secret)
            .map_err(|_| Error::InvalidInput("invalid speech engine JWT secret".to_owned()))?;
        mac.update(signing_input.as_bytes());

        let signature = URL_SAFE_NO_PAD.decode(signature)?;
        mac.verify_slice(&signature).map_err(|_| {
            Error::InvalidInput("speech engine JWT signature is invalid".to_owned())
        })?;

        let payload_bytes = URL_SAFE_NO_PAD.decode(payload)?;
        let claims: SpeechEngineAuthorizationClaims = serde_json::from_slice(&payload_bytes)?;
        validate_claims(&claims)?;
        Ok(claims)
    }

    fn validate_claims(claims: &SpeechEngineAuthorizationClaims) -> Result<()> {
        if claims.iss != JWT_ISSUER {
            return Err(Error::InvalidInput(format!(
                "invalid speech engine JWT issuer: {}",
                claims.iss
            )));
        }

        if claims.sub != JWT_SUBJECT {
            return Err(Error::InvalidInput(format!(
                "invalid speech engine JWT subject: {}",
                claims.sub
            )));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::InvalidInput("system clock is before Unix epoch".to_owned()))?
            .as_secs();

        if claims.exp.saturating_add(JWT_CLOCK_SKEW_SECONDS) < now {
            return Err(Error::InvalidInput(
                "speech engine JWT has expired".to_owned(),
            ));
        }

        Ok(())
    }

    #[derive(Clone, Debug)]
    pub enum SpeechEngineInboundMessage {
        Init(SpeechEngineInit),
        UserTranscript(SpeechEngineUserTranscript),
        Ping,
        Close,
        Error(SpeechEngineProtocolError),
        Unknown(UnknownSpeechEngineMessage),
    }

    impl SpeechEngineInboundMessage {
        pub fn message_type(&self) -> &str {
            match self {
                Self::Init(_) => "init",
                Self::UserTranscript(_) => "user_transcript",
                Self::Ping => "ping",
                Self::Close => "close",
                Self::Error(_) => "error",
                Self::Unknown(message) => &message.message_type,
            }
        }

        pub fn conversation_id(&self) -> Option<&str> {
            match self {
                Self::Init(init) => Some(&init.conversation_id),
                _ => None,
            }
        }

        pub fn event_id(&self) -> Option<u64> {
            match self {
                Self::UserTranscript(transcript) => transcript.event_id,
                _ => None,
            }
        }

        pub fn transcript(&self) -> Option<&[SpeechEngineTranscriptMessage]> {
            match self {
                Self::UserTranscript(transcript) => Some(&transcript.user_transcript),
                _ => None,
            }
        }
    }

    impl<'de> Deserialize<'de> for SpeechEngineInboundMessage {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = Value::deserialize(deserializer)?;
            let message_type = value
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| de::Error::missing_field("type"))?
                .to_owned();

            match message_type.as_str() {
                "init" => deserialize_speech_engine_message(value).map(Self::Init),
                "user_transcript" => {
                    deserialize_speech_engine_message(value).map(Self::UserTranscript)
                }
                "ping" => Ok(Self::Ping),
                "close" => Ok(Self::Close),
                "error" => deserialize_speech_engine_message(value).map(Self::Error),
                unknown => unknown_speech_engine_message(unknown, value).map(Self::Unknown),
            }
        }
    }

    fn deserialize_speech_engine_message<T, E>(value: Value) -> std::result::Result<T, E>
    where
        T: serde::de::DeserializeOwned,
        E: de::Error,
    {
        serde_json::from_value(value).map_err(E::custom)
    }

    fn unknown_speech_engine_message<E>(
        message_type: &str,
        value: Value,
    ) -> std::result::Result<UnknownSpeechEngineMessage, E>
    where
        E: de::Error,
    {
        let Value::Object(mut payload) = value else {
            return Err(E::custom(
                "speech engine websocket message must be a JSON object",
            ));
        };
        payload.remove("type");

        Ok(UnknownSpeechEngineMessage {
            message_type: message_type.to_owned(),
            payload,
        })
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct SpeechEngineInit {
        pub conversation_id: String,
        #[serde(default, flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct SpeechEngineUserTranscript {
        pub user_transcript: Vec<SpeechEngineTranscriptMessage>,
        pub event_id: Option<u64>,
        #[serde(default, flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SpeechEngineTranscriptMessage {
        pub role: String,
        pub content: String,
        #[serde(default, flatten)]
        pub extra: Map<String, Value>,
    }

    impl SpeechEngineTranscriptMessage {
        pub fn user(content: impl Into<String>) -> Self {
            Self {
                role: "user".to_owned(),
                content: content.into(),
                extra: Map::new(),
            }
        }

        pub fn agent(content: impl Into<String>) -> Self {
            Self {
                role: "agent".to_owned(),
                content: content.into(),
                extra: Map::new(),
            }
        }
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct SpeechEngineProtocolError {
        pub message: String,
        #[serde(default, flatten)]
        pub extra: Map<String, Value>,
    }

    #[derive(Clone, Debug)]
    pub struct UnknownSpeechEngineMessage {
        pub message_type: String,
        pub payload: Map<String, Value>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(tag = "type")]
    pub enum SpeechEngineOutboundMessage {
        #[serde(rename = "agent_response")]
        AgentResponse {
            content: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            event_id: Option<u64>,
            is_final: bool,
            #[serde(default, flatten)]
            extra: Map<String, Value>,
        },
        #[serde(rename = "pong")]
        Pong {},
    }

    impl SpeechEngineOutboundMessage {
        pub fn agent_response(event_id: u64, content: impl Into<String>) -> Self {
            Self::AgentResponse {
                content: content.into(),
                event_id: Some(event_id),
                is_final: false,
                extra: Map::new(),
            }
        }

        pub fn agent_response_without_event_id(content: impl Into<String>, is_final: bool) -> Self {
            Self::AgentResponse {
                content: content.into(),
                event_id: None,
                is_final,
                extra: Map::new(),
            }
        }

        pub fn final_agent_response(event_id: u64) -> Self {
            Self::AgentResponse {
                content: String::new(),
                event_id: Some(event_id),
                is_final: true,
                extra: Map::new(),
            }
        }

        pub fn pong() -> Self {
            Self::Pong {}
        }
    }

    pub struct SpeechEngineUpstreamSession<S> {
        socket: S,
        closed: bool,
    }

    impl<S> SpeechEngineUpstreamSession<S>
    where
        S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin,
    {
        pub fn new(socket: S) -> Self {
            Self {
                socket,
                closed: false,
            }
        }

        pub fn is_closed(&self) -> bool {
            self.closed
        }

        pub async fn next_message(&mut self) -> Result<Option<SpeechEngineInboundMessage>> {
            while let Some(message) = self.socket.next().await {
                let message = message?;
                match message {
                    Message::Text(text) => {
                        let message = serde_json::from_str::<SpeechEngineInboundMessage>(&text)
                            .map_err(|source| WebSocketError::Decode {
                                context: INBOUND_CONTEXT,
                                source,
                                payload_preview: payload_preview(&text),
                            })?;
                        return Ok(Some(message));
                    }
                    Message::Ping(payload) => {
                        self.socket.send(Message::Pong(payload)).await?;
                    }
                    Message::Pong(_) => {}
                    Message::Close(_) => {
                        self.closed = true;
                        return Ok(None);
                    }
                    _ => {
                        return Err(WebSocketError::UnexpectedFrame {
                            context: INBOUND_CONTEXT,
                            expected: "text or close",
                            received: frame_kind(&message),
                        }
                        .into());
                    }
                }
            }

            self.closed = true;
            Ok(None)
        }

        pub async fn send_message(&mut self, message: SpeechEngineOutboundMessage) -> Result<()> {
            let text =
                serde_json::to_string(&message).map_err(|source| WebSocketError::Encode {
                    context: OUTBOUND_CONTEXT,
                    source,
                })?;
            self.socket.send(Message::Text(text.into())).await?;
            Ok(())
        }

        pub async fn send_agent_response(
            &mut self,
            event_id: u64,
            content: impl Into<String>,
        ) -> Result<()> {
            self.send_message(SpeechEngineOutboundMessage::agent_response(
                event_id, content,
            ))
            .await
        }

        pub async fn send_final_agent_response(&mut self, event_id: u64) -> Result<()> {
            self.send_message(SpeechEngineOutboundMessage::final_agent_response(event_id))
                .await
        }

        pub async fn send_pong(&mut self) -> Result<()> {
            self.send_message(SpeechEngineOutboundMessage::pong()).await
        }

        pub async fn close(&mut self) -> Result<()> {
            self.socket.send(Message::Close(None)).await?;
            self.closed = true;
            Ok(())
        }

        pub fn into_inner(self) -> S {
            self.socket
        }
    }

    fn payload_preview(payload: &str) -> String {
        const MAX_CHARS: usize = 256;
        let preview: String = payload.chars().take(MAX_CHARS).collect();
        if payload.chars().count() > MAX_CHARS {
            format!("{preview}...")
        } else {
            preview
        }
    }

    fn frame_kind(message: &Message) -> &'static str {
        match message {
            Message::Text(_) => "text",
            Message::Binary(_) => "binary",
            Message::Ping(_) => "ping",
            Message::Pong(_) => "pong",
            Message::Close(_) => "close",
            Message::Frame(_) => "frame",
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use futures_util::{SinkExt, StreamExt};
        use serde_json::json;
        use tokio::net::TcpListener;
        use tokio_tungstenite::{accept_async, connect_async};

        #[test]
        fn inbound_messages_deserialize_by_type() {
            let init: SpeechEngineInboundMessage = serde_json::from_value(json!({
                "type": "init",
                "conversation_id": "conv_123",
                "region": "eu",
            }))
            .unwrap();
            assert_eq!(init.message_type(), "init");
            assert_eq!(init.conversation_id(), Some("conv_123"));

            let transcript: SpeechEngineInboundMessage = serde_json::from_value(json!({
                "type": "user_transcript",
                "event_id": 42,
                "user_transcript": [
                    { "role": "user", "content": "hello" },
                    { "role": "agent", "content": "hi" }
                ]
            }))
            .unwrap();
            assert_eq!(transcript.message_type(), "user_transcript");
            assert_eq!(transcript.event_id(), Some(42));
            assert_eq!(transcript.transcript().unwrap()[0].content, "hello");

            let unknown: SpeechEngineInboundMessage = serde_json::from_value(json!({
                "type": "future_event",
                "payload": true
            }))
            .unwrap();
            assert_eq!(unknown.message_type(), "future_event");
            match unknown {
                SpeechEngineInboundMessage::Unknown(message) => {
                    assert_eq!(message.payload["payload"], true);
                }
                other => panic!("expected unknown message, got {other:?}"),
            }
        }

        #[test]
        fn outbound_messages_serialize_to_protocol_shape() {
            assert_eq!(
                serde_json::to_value(SpeechEngineOutboundMessage::agent_response(7, "Hello"))
                    .unwrap(),
                json!({
                    "type": "agent_response",
                    "content": "Hello",
                    "event_id": 7,
                    "is_final": false,
                })
            );
            assert_eq!(
                serde_json::to_value(SpeechEngineOutboundMessage::final_agent_response(7)).unwrap(),
                json!({
                    "type": "agent_response",
                    "content": "",
                    "event_id": 7,
                    "is_final": true,
                })
            );
            assert_eq!(
                serde_json::to_value(SpeechEngineOutboundMessage::pong()).unwrap(),
                json!({ "type": "pong" })
            );
        }

        #[test]
        fn authorization_token_verifies_signature_and_claims() {
            let api_key = "test-api-key";
            let exp = now_unix() + 300;
            let token = signed_token(api_key, exp, JWT_ISSUER, JWT_SUBJECT);

            let claims = verify_authorization_token(&token, api_key).unwrap();
            assert_eq!(claims.iss, JWT_ISSUER);
            assert_eq!(claims.sub, JWT_SUBJECT);
            assert_eq!(claims.exp, exp);

            assert!(verify_authorization_token(&token, "wrong-api-key").is_err());

            let bad_issuer = signed_token(api_key, exp, "https://example.com", JWT_SUBJECT);
            assert!(verify_authorization_token(&bad_issuer, api_key).is_err());

            let expired = signed_token(
                api_key,
                now_unix() - JWT_CLOCK_SKEW_SECONDS - 1,
                JWT_ISSUER,
                JWT_SUBJECT,
            );
            assert!(verify_authorization_token(&expired, api_key).is_err());
        }

        #[tokio::test]
        async fn upstream_session_reads_and_writes_protocol_messages() {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let ws = accept_async(stream).await.unwrap();
                let mut session = SpeechEngineUpstreamSession::new(ws);

                let message = session.next_message().await.unwrap().unwrap();
                assert_eq!(message.message_type(), "user_transcript");
                assert_eq!(message.event_id(), Some(7));

                session.send_agent_response(7, "Hello").await.unwrap();
                session.send_final_agent_response(7).await.unwrap();
                session.close().await.unwrap();
            });

            let (mut client, _) = connect_async(format!("ws://{addr}")).await.unwrap();
            client
                .send(Message::Text(
                    serde_json::to_string(&json!({
                        "type": "user_transcript",
                        "event_id": 7,
                        "user_transcript": [
                            { "role": "user", "content": "hello" }
                        ]
                    }))
                    .unwrap()
                    .into(),
                ))
                .await
                .unwrap();

            let first = client.next().await.unwrap().unwrap();
            let Message::Text(first) = first else {
                panic!("expected first text response");
            };
            assert_eq!(
                serde_json::from_str::<Value>(&first).unwrap(),
                json!({
                    "type": "agent_response",
                    "content": "Hello",
                    "event_id": 7,
                    "is_final": false,
                })
            );

            let second = client.next().await.unwrap().unwrap();
            let Message::Text(second) = second else {
                panic!("expected second text response");
            };
            assert_eq!(
                serde_json::from_str::<Value>(&second).unwrap(),
                json!({
                    "type": "agent_response",
                    "content": "",
                    "event_id": 7,
                    "is_final": true,
                })
            );

            server.await.unwrap();
        }

        fn signed_token(api_key: &str, exp: u64, iss: &str, sub: &str) -> String {
            let header = json!({ "alg": "HS256", "typ": "JWT" });
            let payload = json!({
                "iss": iss,
                "sub": sub,
                "exp": exp,
            });
            let header = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
            let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
            let signing_input = format!("{header}.{payload}");

            let secret = sha2::Sha256::digest(api_key.as_bytes());
            let mut mac = HmacSha256::new_from_slice(&secret).unwrap();
            mac.update(signing_input.as_bytes());
            let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

            format!("{signing_input}.{signature}")
        }

        fn now_unix() -> u64 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    }
}
