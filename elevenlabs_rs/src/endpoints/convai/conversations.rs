//! Conversations endpoints

use super::*;
use crate::endpoints::convai::agents::{DynamicVar, LiteralJsonSchema};
use crate::endpoints::convai::knowledge_base::EmbeddingModel;
use std::collections::HashMap;
use std::string::ToString;
use strum::Display;

/// Get all conversations of agents that user owns. With option to restrict to a specific agent.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::{
///     CallSuccessful, GetConversations, GetConversationsQuery,
/// };
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let query = GetConversationsQuery::default()
///         //.with_agent_id("agent_id")
///         .with_page_size(10)
///         .with_call_successful(CallSuccessful::Failure);
///
///     let endpoint = GetConversations::with_query(query);
///
///     let resp = client.hit(endpoint).await?;
///
///     println!("{:?}", resp);
///
///     Ok(())
/// }
/// ```
/// See [Get Conversations API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/get-conversations)
#[derive(Clone, Debug, Default, Serialize)]
pub struct GetConversations {
    query: Option<GetConversationsQuery>,
}

impl crate::endpoints::sealed::Sealed for GetConversations {}

impl ElevenLabsEndpoint for GetConversations {
    const PATH: &'static str = "/v1/convai/conversations";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetConversationsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

impl GetConversations {
    pub fn with_query(query: GetConversationsQuery) -> Self {
        Self { query: Some(query) }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetConversationsQuery {
    params: QueryValues,
}

impl GetConversationsQuery {
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.params.push(("agent_id", agent_id.into()));
        self
    }

    pub fn with_call_successful(mut self, call_successful: CallSuccessful) -> Self {
        self.params
            .push(("call_successful", call_successful.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetConversationsResponse {
    pub conversations: Vec<Conversation>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Conversation {
    pub agent_id: String,
    pub agent_name: Option<String>,
    pub conversation_id: String,
    pub start_time_unix_secs: u64,
    pub call_duration_secs: u32,
    pub message_count: u32,
    pub status: ConvoStatus,
    pub call_successful: CallSuccessful,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConvoStatus {
    Done,
    Processing,
}

impl ConvoStatus {
    pub fn is_done(&self) -> bool {
        matches!(*self, ConvoStatus::Done)
    }
    pub fn is_processing(&self) -> bool {
        matches!(*self, ConvoStatus::Processing)
    }
}

#[derive(Clone, Debug, Display, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CallSuccessful {
    #[strum(to_string = "failure")]
    Failure,
    #[strum(to_string = "success")]
    Success,
    #[strum(to_string = "unknown")]
    Unknown,
}

impl CallSuccessful {
    pub fn is_failure(&self) -> bool {
        matches!(*self, CallSuccessful::Failure)
    }
    pub fn is_success(&self) -> bool {
        matches!(*self, CallSuccessful::Success)
    }
    pub fn is_unknown(&self) -> bool {
        matches!(*self, CallSuccessful::Unknown)
    }
}

/// Get the details of a particular conversation
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::GetConversationDetails;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint = GetConversationDetails::new("conversation_id");
///     let resp = client.hit(endpoint).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See [Get Conversation Details API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/get-conversation)
#[derive(Clone, Debug)]
pub struct GetConversationDetails {
    conversation_id: String,
}

impl GetConversationDetails {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetConversationDetails {}

impl ElevenLabsEndpoint for GetConversationDetails {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetConversationDetailsResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetConversationDetailsResponse {
    pub agent_id: String,
    pub conversation_id: String,
    pub status: ConvoStatus,
    pub transcript: Option<Vec<Transcript>>,
    pub metadata: Option<Metadata>,
    pub analysis: Option<Analysis>,
    pub conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Analysis {
    pub call_successful: CallSuccessful,
    pub data_collection_results: Option<HashMap<String, DataCollectionResult>>,
    pub evaluation_criteria_results: Option<HashMap<String, EvaluationResult>>,
    pub transcript_summary: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataCollectionResult {
    pub data_collection_id: String,
    pub json_schema: Option<LiteralJsonSchema>,
    pub value: Option<Value>,
    pub rationale: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EvaluationResult {
    pub criteria_id: String,
    pub result: CallSuccessful,
    pub rationale: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    pub start_time_unix_secs: u64,
    pub call_duration_secs: u32,
    pub cost: Option<u32>,
    pub deletion_settings: Option<DeletionSettings>,
    pub feedback: Option<ConvoMetadataFeedback>,
    pub authorization_method: Option<AuthorizationMethod>,
    pub charging: Option<Charging>,
    pub termination_reason: Option<String>,
    pub phone_call: Option<PhoneCall>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum PhoneCall {
    Twilio {
        r#type: String,
        agent_number: String,
        call_sid: String,
        direction: Direction,
        external_number: String,
        phone_number_id: String,
        stream_sid: String,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Inbound,
    Outbound,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeletionSettings {
    pub deletion_time_unix_secs: Option<u64>,
    pub deleted_logs_at_time_unix_secs: Option<u64>,
    pub deleted_audio_at_time_unix_secs: Option<u64>,
    pub deleted_transcript_at_time_unix_secs: Option<u64>,
    pub delete_transcript_and_pii: Option<bool>,
    pub delete_audio: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConvoMetadataFeedback {
    pub overall_score: Option<Score>,
    pub likes: Option<u32>,
    pub dislikes: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationMethod {
    Public,
    AuthorizationHeader,
    SignedUrl,
    ShareableLink,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Charging {
    pub dev_discount: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Transcript {
    pub role: Role,
    pub message: Option<String>,
    pub time_in_call_secs: u32,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
    pub feedback: Option<TranscriptFeedback>,
    pub conversation_turn_metrics: Option<ConversationTurnMetrics>,
    pub rag_retrieval_info: Option<RAGRetrievalInfo>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RAGRetrievalInfo {
    pub chunks: Vec<Chunk>,
    pub embedding_model: EmbeddingModel,
    pub retrieval_query: String,
    pub rag_latency_secs: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub document_id: String,
    pub vector_distance: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConversationTurnMetrics {
    pub metrics: Option<HashMap<String, Value>>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Agent,
    User,
}
#[derive(Clone, Debug, Deserialize)]
pub struct ToolCall {
    pub request_id: String,
    pub tool_name: String,
    pub params_as_json: String,
    pub tool_has_been_called: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ToolResult {
    pub request_id: String,
    pub tool_name: String,
    pub result_value: String,
    pub is_error: bool,
    pub tool_has_been_called: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TranscriptFeedback {
    pub score: Score,
    pub time_in_call_secs: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Score {
    Like,
    Dislike,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConversationInitiationClientData {
    pub conversation_config_override: Option<ConfigOverrideData>,
    pub custom_llm_extra_body: Option<HashMap<String, Value>>,
    pub dynamic_variables: Option<HashMap<String, DynamicVar>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigOverrideData {
    pub agent: Option<AgentOverrideData>,
    pub tts: Option<TTSOverrideData>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AgentOverrideData {
    pub prompt: Option<PromptOverrideData>,
    pub first_message: Option<String>,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PromptOverrideData {
    pub prompt: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TTSOverrideData {
    pub voice_id: Option<String>,
}

/// Delete a particular conversation
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::DeleteConversation;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = DeleteConversation::new("conversation_id");
///    let _ = client.hit(endpoint).await?;
///    Ok(())
/// }
/// ```
/// See [Delete Conversation API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/delete-conversation)
#[derive(Clone, Debug)]
pub struct DeleteConversation {
    conversation_id: String,
}

impl DeleteConversation {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteConversation {}

impl ElevenLabsEndpoint for DeleteConversation {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

/// Get the audio recording of a particular conversation
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::GetConversationAudio;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = GetConversationAudio::new("conversation_id");
///    let bytes = client.hit(endpoint).await?;
///    play(bytes)?;
///    Ok(())
/// }
/// ```
/// See [Get Conversation Audio API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/get-conversation-audio)
#[derive(Clone, Debug, Serialize)]
pub struct GetConversationAudio {
    conversation_id: String,
}

impl GetConversationAudio {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetConversationAudio {}

impl ElevenLabsEndpoint for GetConversationAudio {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/audio";

    const METHOD: Method = Method::GET;

    type ResponseBody = Bytes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// Get a signed url to start a conversation with an agent with an agent that requires authorization
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::GetSignedUrl;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = GetSignedUrl::new("agent_id");
///    let resp = client.hit(endpoint).await?;
///    println!("{}", resp.signed_url);
///   Ok(())
/// }
/// ```
/// See [Get Signed URL API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/get-signed-url)
#[derive(Clone, Debug, Serialize)]
pub struct GetSignedUrl {
    query: GetSignedUrlQuery,
}

impl GetSignedUrl {
    pub fn new(agent_id: impl Into<String>) -> Self {
        GetSignedUrl {
            query: GetSignedUrlQuery::new(agent_id),
        }
    }

    pub fn with_query(mut self, query: GetSignedUrlQuery) -> Self {
        self.query = query;
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GetSignedUrlQuery {
    params: QueryValues,
}

impl GetSignedUrlQuery {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            params: vec![("agent_id", agent_id.into())],
        }
    }

    pub fn with_include_conversation_id(mut self, include_conversation_id: bool) -> Self {
        self.params.push((
            "include_conversation_id",
            include_conversation_id.to_string(),
        ));
        self
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.params.push(("branch_id", branch_id.into()));
        self
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.params.push(("environment", environment.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetSignedUrl {}

impl ElevenLabsEndpoint for GetSignedUrl {
    const PATH: &'static str = "/v1/convai/conversation/get-signed-url";

    const METHOD: Method = Method::GET;

    type ResponseBody = SignedUrlResponse;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SignedUrlResponse {
    pub signed_url: String,
}

/// Send the feedback for the given conversation
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::{
///  SendConversationFeedback, SendConversationFeedbackBody, Score};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let body = SendConversationFeedbackBody::new(Score::Like);
///    let endpoint = SendConversationFeedback::new("conversation_id", body);
///    let resp = client.hit(endpoint).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See [Send Conversation Feedback API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/post-conversation-feedback)
#[derive(Clone, Debug)]
pub struct SendConversationFeedback {
    conversation_id: String,
    body: SendConversationFeedbackBody,
}

impl SendConversationFeedback {
    pub fn new(conversation_id: impl Into<String>, body: SendConversationFeedbackBody) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            body,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendConversationFeedbackBody {
    pub feedback: Score,
}

impl SendConversationFeedbackBody {
    pub fn new(feedback: Score) -> Self {
        Self { feedback }
    }
}

impl crate::endpoints::sealed::Sealed for SendConversationFeedback {}

impl ElevenLabsEndpoint for SendConversationFeedback {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/feedback";

    const METHOD: Method = Method::POST;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

impl TryInto<RequestBody> for &SendConversationFeedbackBody {
    type Error = crate::error::Error;

    fn try_into(self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(self)?))
    }
}

/// Handle an outbound call via Twilio
///
/// # Example
///
/// ```no_run
///
/// use elevenlabs_rs::endpoints::convai::conversations::{
///    OutboundCallViaTwilio, OutboundCallViaTwilioBody,
/// };
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let body = OutboundCallViaTwilioBody::new(
///       "agent_id",
///       "agent_phone_number_id",
///       "to_number",
///    );
///    let endpoint = OutboundCallViaTwilio::new(body);
///    let resp = client.hit(endpoint).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See [Outbound Call Via Twilio API reference](https://elevenlabs.io/docs/api-reference/conversations/twilio-outbound-call)
#[derive(Clone, Debug)]
pub struct OutboundCallViaTwilio {
    body: OutboundCallViaTwilioBody,
}

impl OutboundCallViaTwilio {
    pub fn new(body: OutboundCallViaTwilioBody) -> Self {
        Self { body }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OutboundCallViaTwilioBody {
    pub agent_id: String,
    pub agent_phone_number_id: String,
    pub to_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}

impl OutboundCallViaTwilioBody {
    pub fn new(
        agent_id: impl Into<String>,
        agent_phone_number_id: impl Into<String>,
        to_number: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            agent_phone_number_id: agent_phone_number_id.into(),
            to_number: to_number.into(),
            conversation_initiation_client_data: None,
        }
    }

    pub fn with_conversation_initiation_client_data(
        mut self,
        data: ConversationInitiationClientData,
    ) -> Self {
        self.conversation_initiation_client_data = Some(data);
        self
    }
}

impl crate::endpoints::sealed::Sealed for OutboundCallViaTwilio {}

impl ElevenLabsEndpoint for OutboundCallViaTwilio {
    const PATH: &'static str = "/v1/convai/twilio/outbound-call";

    const METHOD: Method = Method::POST;

    type ResponseBody = OutboundCallViaTwilioResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct OutboundCallViaTwilioResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "callSid")]
    pub call_sid: String,
}

impl IntoIterator for GetConversationsResponse {
    type Item = Conversation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.conversations.into_iter()
    }
}

impl<'a> IntoIterator for &'a GetConversationsResponse {
    type Item = &'a Conversation;
    type IntoIter = std::slice::Iter<'a, Conversation>;

    fn into_iter(self) -> Self::IntoIter {
        self.conversations.iter()
    }
}

impl IntoIterator for GetConversationDetailsResponse {
    type Item = Transcript;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.transcript.unwrap_or_default().into_iter()
    }
}

impl<'a> IntoIterator for &'a GetConversationDetailsResponse {
    type Item = &'a Transcript;
    type IntoIter = std::slice::Iter<'a, Transcript>;

    fn into_iter(self) -> Self::IntoIter {
        self.transcript.as_deref().unwrap_or_default().iter()
    }
}

/// Get a WebRTC token to start a conversation with an agent.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::GetWebRtcToken;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let resp = client.hit(GetWebRtcToken::new("agent_id")).await?;
///     println!("{}", resp.token);
///     Ok(())
/// }
/// ```
/// See [Get WebRTC Token API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/get-webrtc-token)
#[derive(Clone, Debug)]
pub struct GetWebRtcToken {
    query: WebRtcTokenQuery,
}

impl GetWebRtcToken {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            query: WebRtcTokenQuery::new(agent_id),
        }
    }

    pub fn with_query(mut self, query: WebRtcTokenQuery) -> Self {
        self.query = query;
        self
    }
}

/// Query parameters for [`GetWebRtcToken`].
#[derive(Clone, Debug)]
pub struct WebRtcTokenQuery {
    params: QueryValues,
}

impl WebRtcTokenQuery {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            params: vec![("agent_id", agent_id.into())],
        }
    }

    pub fn with_participant_name(mut self, participant_name: impl Into<String>) -> Self {
        self.params
            .push(("participant_name", participant_name.into()));
        self
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.params.push(("branch_id", branch_id.into()));
        self
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.params.push(("environment", environment.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetWebRtcToken {}

impl ElevenLabsEndpoint for GetWebRtcToken {
    const PATH: &'static str = "/v1/convai/conversation/token";

    const METHOD: Method = Method::GET;

    type ResponseBody = WebRtcTokenResponse;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`GetWebRtcToken`].
#[derive(Clone, Debug, Deserialize)]
pub struct WebRtcTokenResponse {
    pub token: String,
}

/// The direction of a telephony call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TelephonyDirection {
    Inbound,
    Outbound,
}

/// Register an inbound Twilio call and return the TwiML to drive it.
///
/// The response is the raw TwiML XML document.
///
/// See [Register Twilio Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/twilio/register-call)
#[derive(Clone, Debug)]
pub struct RegisterTwilioCall {
    body: RegisterTwilioCallBody,
}

impl RegisterTwilioCall {
    pub fn new(body: RegisterTwilioCallBody) -> Self {
        Self { body }
    }
}

/// Register-Twilio-call body.
#[derive(Clone, Debug, Serialize)]
pub struct RegisterTwilioCallBody {
    agent_id: String,
    from_number: String,
    to_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<TelephonyDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}

impl RegisterTwilioCallBody {
    pub fn new(
        agent_id: impl Into<String>,
        from_number: impl Into<String>,
        to_number: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            from_number: from_number.into(),
            to_number: to_number.into(),
            direction: None,
            conversation_initiation_client_data: None,
        }
    }

    pub fn with_direction(mut self, direction: TelephonyDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn with_conversation_initiation_client_data(
        mut self,
        data: ConversationInitiationClientData,
    ) -> Self {
        self.conversation_initiation_client_data = Some(data);
        self
    }
}

impl crate::endpoints::sealed::Sealed for RegisterTwilioCall {}

impl ElevenLabsEndpoint for RegisterTwilioCall {
    const PATH: &'static str = "/v1/convai/twilio/register-call";

    const METHOD: Method = Method::POST;

    type ResponseBody = String;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.text().await?)
    }
}

/// Get the live (in-progress) conversation count, optionally for a single agent.
///
/// See [Get Live Count API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/analytics/live-count)
#[derive(Clone, Debug, Default)]
pub struct GetLiveCount {
    agent_id: Option<String>,
}

impl GetLiveCount {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetLiveCount {}

impl ElevenLabsEndpoint for GetLiveCount {
    const PATH: &'static str = "/v1/convai/analytics/live-count";

    const METHOD: Method = Method::GET;

    type ResponseBody = LiveCountResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.agent_id
            .as_ref()
            .map(|agent_id| vec![("agent_id", agent_id.clone())])
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`GetLiveCount`].
#[derive(Clone, Debug, Deserialize)]
pub struct LiveCountResponse {
    pub count: i64,
}

/// List the end users that have conversed with the workspace's agents.
///
/// See [Get Conversation Users API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/users/list)
#[derive(Clone, Debug, Default)]
pub struct GetConversationUsers {
    query: Option<ConversationUsersQuery>,
}

impl GetConversationUsers {
    pub fn with_query(mut self, query: ConversationUsersQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetConversationUsers`].
#[derive(Clone, Debug, Default)]
pub struct ConversationUsersQuery {
    params: QueryValues,
}

impl ConversationUsersQuery {
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.params.push(("agent_id", agent_id.into()));
        self
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.params.push(("branch_id", branch_id.into()));
        self
    }

    pub fn with_call_start_before_unix(mut self, call_start_before_unix: i64) -> Self {
        self.params
            .push(("call_start_before_unix", call_start_before_unix.to_string()));
        self
    }

    pub fn with_call_start_after_unix(mut self, call_start_after_unix: i64) -> Self {
        self.params
            .push(("call_start_after_unix", call_start_after_unix.to_string()));
        self
    }

    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.params.push(("search", search.into()));
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
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

impl crate::endpoints::sealed::Sealed for GetConversationUsers {}

impl ElevenLabsEndpoint for GetConversationUsers {
    const PATH: &'static str = "/v1/convai/users";

    const METHOD: Method = Method::GET;

    type ResponseBody = ConversationUsersPage;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of conversation users. Each user entry is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct ConversationUsersPage {
    pub users: Vec<Value>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// A page of conversation-message search results. Each result is preserved as
/// raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct MessagesSearchResponse {
    pub results: Vec<Value>,
    pub meta: Option<Value>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Semantic ("smart") search across conversation messages.
///
/// See [Smart Search API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/smart-search)
#[derive(Clone, Debug)]
pub struct SmartSearchConversationMessages {
    query: ConversationMessagesSearchQuery,
}

impl SmartSearchConversationMessages {
    pub fn new(text_query: impl Into<String>) -> Self {
        Self {
            query: ConversationMessagesSearchQuery::new(text_query),
        }
    }

    pub fn with_query(mut self, query: ConversationMessagesSearchQuery) -> Self {
        self.query = query;
        self
    }
}

/// Query parameters for [`SmartSearchConversationMessages`] and
/// [`TextSearchConversationMessages`].
///
/// The text-search endpoint accepts many additional filters; use
/// [`with_param`](Self::with_param) to set any not covered by a dedicated method.
#[derive(Clone, Debug)]
pub struct ConversationMessagesSearchQuery {
    params: QueryValues,
}

impl ConversationMessagesSearchQuery {
    pub fn new(text_query: impl Into<String>) -> Self {
        Self {
            params: vec![("text_query", text_query.into())],
        }
    }

    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.params.push(("agent_id", agent_id.into()));
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.params.push(("branch_id", branch_id.into()));
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.params.push(("user_id", user_id.into()));
        self
    }

    pub fn with_sort_by(mut self, sort_by: impl Into<String>) -> Self {
        self.params.push(("sort_by", sort_by.into()));
        self
    }

    /// Set an arbitrary query parameter (e.g. a text-search filter).
    pub fn with_param(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.params.push((key, value.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for SmartSearchConversationMessages {}

impl ElevenLabsEndpoint for SmartSearchConversationMessages {
    const PATH: &'static str = "/v1/convai/conversations/messages/smart-search";

    const METHOD: Method = Method::GET;

    type ResponseBody = MessagesSearchResponse;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Full-text search across conversation messages, with rich filters.
///
/// See [Text Search API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/text-search)
#[derive(Clone, Debug)]
pub struct TextSearchConversationMessages {
    query: ConversationMessagesSearchQuery,
}

impl TextSearchConversationMessages {
    pub fn new(text_query: impl Into<String>) -> Self {
        Self {
            query: ConversationMessagesSearchQuery::new(text_query),
        }
    }

    pub fn with_query(mut self, query: ConversationMessagesSearchQuery) -> Self {
        self.query = query;
        self
    }
}

impl crate::endpoints::sealed::Sealed for TextSearchConversationMessages {}

impl ElevenLabsEndpoint for TextSearchConversationMessages {
    const PATH: &'static str = "/v1/convai/conversations/messages/text-search";

    const METHOD: Method = Method::GET;

    type ResponseBody = MessagesSearchResponse;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Run analysis for a conversation. Returns the updated conversation details.
///
/// See [Run Analysis API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/run-analysis)
#[derive(Clone, Debug)]
pub struct RunConversationAnalysis {
    conversation_id: String,
}

impl RunConversationAnalysis {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for RunConversationAnalysis {}

impl ElevenLabsEndpoint for RunConversationAnalysis {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/analysis/run";

    const METHOD: Method = Method::POST;

    type ResponseBody = GetConversationDetailsResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Run a named evaluation for a conversation. Returns the updated conversation
/// details.
///
/// See [Run Evaluation API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/run-evaluation)
#[derive(Clone, Debug)]
pub struct RunConversationEvaluation {
    conversation_id: String,
    body: RunConversationEvaluationBody,
}

impl RunConversationEvaluation {
    pub fn new(conversation_id: impl Into<String>, body: RunConversationEvaluationBody) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            body,
        }
    }
}

/// Body for [`RunConversationEvaluation`].
#[derive(Clone, Debug, Serialize)]
pub struct RunConversationEvaluationBody {
    evaluation_id: String,
    /// The analysis scope, e.g. `"conversation"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
}

impl RunConversationEvaluationBody {
    pub fn new(evaluation_id: impl Into<String>) -> Self {
        Self {
            evaluation_id: evaluation_id.into(),
            scope: None,
        }
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for RunConversationEvaluation {}

impl ElevenLabsEndpoint for RunConversationEvaluation {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/analysis/evaluations/run";

    const METHOD: Method = Method::POST;

    type ResponseBody = GetConversationDetailsResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Upload a file to a conversation.
///
/// See [Upload File API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/upload-file)
#[derive(Clone, Debug)]
pub struct UploadConversationFile {
    conversation_id: String,
    file: crate::shared::FilePart,
}

impl UploadConversationFile {
    pub fn new(
        conversation_id: impl Into<String>,
        file: impl Into<crate::shared::FilePart>,
    ) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            file: file.into(),
        }
    }

    pub fn from_bytes(
        conversation_id: impl Into<String>,
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(
            conversation_id,
            crate::shared::FilePart::bytes(file_name, mime, bytes),
        )
    }
}

impl crate::endpoints::sealed::Sealed for UploadConversationFile {}

impl ElevenLabsEndpoint for UploadConversationFile {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/files";

    const METHOD: Method = Method::POST;

    type ResponseBody = ConversationFileUploadResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        let part = self
            .file
            .clone()
            .into_part(Some("application/octet-stream".to_owned()))?;
        Ok(RequestBody::Multipart(Form::new().part("file", part)))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`UploadConversationFile`] and [`DeleteConversationFile`].
#[derive(Clone, Debug, Deserialize)]
pub struct ConversationFileUploadResponse {
    pub file_id: String,
}

/// Delete a previously uploaded conversation file.
///
/// See [Delete File API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/delete-file)
#[derive(Clone, Debug)]
pub struct DeleteConversationFile {
    conversation_id: String,
    file_id: String,
}

impl DeleteConversationFile {
    pub fn new(conversation_id: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            file_id: file_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteConversationFile {}

impl ElevenLabsEndpoint for DeleteConversationFile {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/files/:file_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ConversationFileUploadResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.conversation_id.and_param(PathParam::ConversationID),
            self.file_id.and_param(PathParam::FileID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Get the SIP log messages for a conversation.
///
/// See [Get Conversation SIP Messages API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/get-sip-messages)
#[derive(Clone, Debug)]
pub struct GetConversationSipMessages {
    conversation_id: String,
    query: Option<crate::endpoints::convai::phone_numbers::SipMessagesQuery>,
}

impl GetConversationSipMessages {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            query: None,
        }
    }

    pub fn with_query(
        mut self,
        query: crate::endpoints::convai::phone_numbers::SipMessagesQuery,
    ) -> Self {
        self.query = Some(query);
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetConversationSipMessages {}

impl ElevenLabsEndpoint for GetConversationSipMessages {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/sip-messages";

    const METHOD: Method = Method::GET;

    type ResponseBody = crate::endpoints::convai::phone_numbers::GetSipMessagesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.clone().into_params())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Assign tags to a conversation.
///
/// See [Assign Tags API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/assign-tags)
#[derive(Clone, Debug)]
pub struct AssignConversationTags {
    conversation_id: String,
    tag_ids: Vec<String>,
}

impl AssignConversationTags {
    pub fn new<I, S>(conversation_id: impl Into<String>, tag_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            conversation_id: conversation_id.into(),
            tag_ids: tag_ids.into_iter().map(Into::into).collect(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for AssignConversationTags {}

impl ElevenLabsEndpoint for AssignConversationTags {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/tags";

    const METHOD: Method = Method::POST;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.conversation_id.and_param(PathParam::ConversationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "tag_ids": self.tag_ids }),
        ))
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

/// Unassign a single tag from a conversation.
///
/// See [Unassign Tag API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/conversations/unassign-tag)
#[derive(Clone, Debug)]
pub struct UnassignConversationTag {
    conversation_id: String,
    tag_id: String,
}

impl UnassignConversationTag {
    pub fn new(conversation_id: impl Into<String>, tag_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            tag_id: tag_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for UnassignConversationTag {}

impl ElevenLabsEndpoint for UnassignConversationTag {
    const PATH: &'static str = "/v1/convai/conversations/:conversation_id/tags/:tag_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.conversation_id.and_param(PathParam::ConversationID),
            self.tag_id.and_param(PathParam::TagID),
        ]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}
