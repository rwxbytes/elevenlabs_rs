//#![deny(missing_docs)]
use crate::conversational_ai::error::ElevenLabsConversationalError;
use crate::endpoints::*;
//use crate::utils::audio_helpers::UpmixMonoToStereo;
use super::Result;

/// See Elevenlabs' docs on [server messages](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#server-to-client-messages)
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    Agent(AgentResponse),
    AgentCorrection(AgentCorrection),
    Audio(AudioResponse),
    ClientToolCall(ClientToolCall),
    ClientToolResult(ClientToolResult),
    InitMetadata(ConversationInitiationMetadata),
    Interruption(Interruption),
    Ping(Ping),
    TentativeAgent(TentativeAgentResponse),
    TurnProbability(TurnProbability),
    Transcription(UserTranscription),
    VadScore(VadScore),
}

impl ServerMessage {
    /// Indicates if the message is an agent response
    pub fn is_agent(&self) -> bool {
        matches!(*self, ServerMessage::Agent(_))
    }

    /// If the `ServerMessage` is an `AgentResponse`, then it returns it, otherwise it returns `None`
    pub fn as_agent(&self) -> Option<&AgentResponse> {
        match self {
            ServerMessage::Agent(agent) => Some(agent),
            _ => None,
        }
    }

    /// Indicates if the message is an audio response
    pub fn is_audio(&self) -> bool {
        matches!(*self, ServerMessage::Audio(_))
    }

    /// If the `ServerMessage` is an `AudioResponse`, then it returns it, otherwise it returns `None`
    pub fn as_audio(&self) -> Option<&AudioResponse> {
        match self {
            ServerMessage::Audio(audio) => Some(audio),
            _ => None,
        }
    }

    /// Indicates if the message is a client tool call response
    pub fn is_client_tool_call(&self) -> bool {
        matches!(*self, ServerMessage::ClientToolCall(_))
    }

    /// If the `ServerMessage` is a `ClientToolCall`, then it returns it, otherwise it returns `None`
    pub fn as_client_tool_call(&self) -> Option<&ClientToolCall> {
        match self {
            ServerMessage::ClientToolCall(client_tool_call) => Some(client_tool_call),
            _ => None,
        }
    }

    /// Indicates if the message is a client tool result response
    pub fn is_client_tool_result(&self) -> bool {
        matches!(*self, ServerMessage::ClientToolResult(_))
    }

    /// If the `ServerMessage` is a `ClientToolResult`, then it returns it, otherwise it returns `None`
    pub fn as_client_tool_result(&self) -> Option<&ClientToolResult> {
        match self {
            ServerMessage::ClientToolResult(client_tool_result) => Some(client_tool_result),
            _ => None,
        }
    }

    /// Indicates if the message is a conversation initiation metadata response
    pub fn is_init_metadata(&self) -> bool {
        matches!(*self, ServerMessage::InitMetadata(_))
    }

    /// If the `ServerMessage` is an `InitMetadata`, then it returns it, otherwise it returns `None`
    pub fn as_init_metadata(&self) -> Option<&ConversationInitiationMetadata> {
        match self {
            ServerMessage::InitMetadata(init_metadata) => Some(init_metadata),
            _ => None,
        }
    }

    /// Indicates if the message is an interruption response
    pub fn is_interruption(&self) -> bool {
        matches!(*self, ServerMessage::Interruption(_))
    }

    /// If the `ServerMessage` is an `Interruption`, then it returns it, otherwise it returns `None`
    pub fn as_interruption(&self) -> Option<&Interruption> {
        match self {
            ServerMessage::Interruption(interruption) => Some(interruption),
            _ => None,
        }
    }

    /// Indicates if the message is a ping response
    pub fn is_ping(&self) -> bool {
        matches!(*self, ServerMessage::Ping(_))
    }

    /// If the `ServerMessage` is a `Ping`, then it returns it, otherwise it returns `None`
    pub fn as_ping(&self) -> Option<&Ping> {
        match self {
            ServerMessage::Ping(ping) => Some(ping),
            _ => None,
        }
    }

    /// Indicates if the message is a tentative agent response
    pub fn is_tentative_agent(&self) -> bool {
        matches!(*self, ServerMessage::TentativeAgent(_))
    }

    /// If the `ServerMessage` is a `TentativeAgent`, then it returns it, otherwise it returns `None`
    pub fn as_tentative_agent(&self) -> Option<&TentativeAgentResponse> {
        match self {
            ServerMessage::TentativeAgent(tentative_agent) => Some(tentative_agent),
            _ => None,
        }
    }

    /// Indicates if the message is a user transcription response
    pub fn is_transcription(&self) -> bool {
        matches!(*self, ServerMessage::Transcription(_))
    }

    /// If the `ServerMessage` is a `Transcription`, then it returns it, otherwise it returns `None`
    pub fn as_transcription(&self) -> Option<&UserTranscription> {
        match self {
            ServerMessage::Transcription(transcription) => Some(transcription),
            _ => None,
        }
    }
}

impl TryFrom<&str> for ServerMessage {
    type Error = ElevenLabsConversationalError;

    fn try_from(text: &str) -> std::result::Result<Self, Self::Error> {
        let response: ServerMessage = serde_json::from_str(text)?;
        Ok(response)
    }
}

/// see Elevenlabs' docs on [agent responses](https://elevenlabs.io/docs/api-reference/conversational-ai#agent-response)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentResponse {
     r#type: String,
     agent_response_event: AgentResponseEvent,
}

impl AgentResponse {
    pub fn event(&self) -> &AgentResponseEvent {
        &self.agent_response_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentResponseEvent {
     agent_response: String,
}

impl AgentResponse {
    pub fn response(&self) -> &str {
        &self.agent_response_event.agent_response
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentCorrection {
     r#type: String,
     agent_response_correction_event: AgentResponseCorrectionEvent,
}

impl AgentCorrection {
    pub fn event(&self) -> &AgentResponseCorrectionEvent {
        &self.agent_response_correction_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentResponseCorrectionEvent {
    corrected_agent_response: String,
    original_agent_response: String,
}

impl AgentCorrection {
    pub fn corrected_response(&self) -> &str {
        &self.agent_response_correction_event.corrected_agent_response
    }
    pub fn original_response(&self) -> &str {
        &self.agent_response_correction_event.original_agent_response
    }
}

/// see Elevenlabs' docs on [audio responses](https://elevenlabs.io/docs/api-reference/conversational-ai#audio-response)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioResponse {
     r#type: String,
     audio_event: AudioEvent,
}

impl AudioResponse {
    pub fn event(&self) -> &AudioEvent {
        &self.audio_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioEvent {
     audio_base_64: String,
     event_id: u32,
}

impl AudioEvent {
    pub fn base_64(&self) -> &str {
        &self.audio_base_64
    }
    pub fn id(&self) -> u32 {
        self.event_id
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // TODO: impl decode error for mod error
        Ok(BASE64_STANDARD.decode(&self.audio_base_64).expect("Failed to decode base64"))
    }
}

//impl UpmixMonoToStereo for AudioEvent {
//    fn upmix_mono_to_stereo(&self) -> Vec<i16> {
//        let mono_samples = crate::utils::audio_helpers::decode_base64_pcm(self.base_64()).unwrap();
//        let mut stereo_samples = Vec::with_capacity(mono_samples.len() * 2);
//        for &sample in &mono_samples {
//            stereo_samples.push(sample);
//            stereo_samples.push(sample);
//        }
//        stereo_samples
//    }
//}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToolCall {
     r#type: String,
     client_tool_call: ClientTool,
}

impl ClientToolCall {
    pub fn client_tool(&self) -> &ClientTool {
        &self.client_tool_call
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientTool {
    tool_name: String,
    tool_call_id: String,
    parameters: Value,
}

impl ClientTool {
    pub fn name(&self) -> &str {
        &self.tool_name
    }
    pub fn id(&self) -> &str {
        &self.tool_call_id
    }
    pub fn parameters(&self) -> &Value {
        &self.parameters
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToolResult {
    r#type: String,
    client_tool_id: String,
    result: String,
    is_error: bool,
}

impl ClientToolResult {
    pub fn id(&self) -> &str {
        &self.client_tool_id
    }
    pub fn result(&self) -> &str {
        &self.result
    }
    pub fn is_error(&self) -> bool {
        self.is_error
    }
}

/// see Elevenlabs' docs on [conversation initiation metadata](https://elevenlabs.io/docs/api-reference/conversational-ai#conversation-initiation-metadata)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationInitiationMetadata {
     r#type: String,
     conversation_initiation_metadata_event: ConversationInitiationMetadataEvent,
}

impl ConversationInitiationMetadata {
    pub fn event(&self) -> &ConversationInitiationMetadataEvent {
        &self.conversation_initiation_metadata_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationInitiationMetadataEvent {
     conversation_id: String,
     agent_output_audio_format: String,
}

impl ConversationInitiationMetadata {
    pub fn id(&self) -> &str {
        &self.conversation_initiation_metadata_event.conversation_id
    }
    pub fn audio_format(&self) -> &str {
        &self
            .conversation_initiation_metadata_event
            .agent_output_audio_format
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Interruption {
     r#type: String,
     interruption_event: InterruptionEvent,
}

impl Interruption {
    pub fn event(&self) -> &InterruptionEvent {
        &self.interruption_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InterruptionEvent {
    pub event_id: u32,
}

impl InterruptionEvent {
    pub fn id(&self) -> u32 {
        self.event_id
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ping {
     r#type: String,
     ping_event: PingEvent,
}

impl Ping {
    pub fn event(&self) -> &PingEvent {
        &self.ping_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PingEvent {
     event_id: u32,
     ping_ms: Option<u32>,
}

impl Ping {
    pub fn id(&self) -> u32 {
        self.ping_event.event_id
    }
    pub fn ms(&self) -> Option<u32> {
        self.ping_event.ping_ms
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TentativeAgentResponse {
     r#type: String,
     tentative_agent_response_internal_event: TentativeAgentResponseInternalEvent,
}

impl TentativeAgentResponse {
    pub fn event(&self) -> &TentativeAgentResponseInternalEvent {
        &self.tentative_agent_response_internal_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TentativeAgentResponseInternalEvent {
    tentative_agent_response: String,
}

impl TentativeAgentResponse {
    pub fn response(&self) -> &str {
        &self.tentative_agent_response_internal_event.tentative_agent_response
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TurnProbability {
     r#type: String,
     turn_probability_internal_event: TurnProbabilityInternalEvent,
}

impl TurnProbability {
    pub fn event(&self) -> &TurnProbabilityInternalEvent {
        &self.turn_probability_internal_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TurnProbabilityInternalEvent {
    turn_probability: f32,
}

impl TurnProbability {
    pub fn probability(&self) -> f32 {
        self.turn_probability_internal_event.turn_probability
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserTranscription {
     r#type: String,
     user_transcription_event: UserTranscriptionEvent,
}

impl UserTranscription {
    pub fn event(&self) -> &UserTranscriptionEvent {
        &self.user_transcription_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserTranscriptionEvent {
    user_transcript: String,
}

impl UserTranscription {
    pub fn transcript(&self) -> &str {
        &self.user_transcription_event.user_transcript
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VadScore {
     r#type: String,
     vad_score_internal_event: VadScoreInternalEvent,
}

impl VadScore {
    pub fn event(&self) -> &VadScoreInternalEvent {
        &self.vad_score_internal_event
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VadScoreInternalEvent {
    vad_score: f32,
}

impl VadScore {
    pub fn score(&self) -> f32 {
        self.vad_score_internal_event.vad_score
    }
}
