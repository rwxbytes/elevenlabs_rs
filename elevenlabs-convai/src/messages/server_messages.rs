use super::*;

/// See [Server Messages](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#server-to-client-messages)
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    AgentCorrection(AgentCorrection),
    AgentResponse(AgentResponse),
    Audio(Audio),
    ClientToolCall(ClientToolCall),
    ConversationInitiationMetadata(ConversationInitiationMetadata),
    Interruption(Interruption),
    McpConnectionStatus(McpConnectionStatus),
    McpToolCall(McpToolCall),
    Ping(Ping),
    TentativeAgent(TentativeAgentResponse),
    TurnProbability(TurnProbability),
    UserTranscript(UserTranscript),
    VadScore(VadScore),
}

impl ServerMessage {
    /// Indicates if the message is an agent correction response
    pub fn is_agent_correction(&self) -> bool {
        matches!(*self, ServerMessage::AgentCorrection(_))
    }

    /// If the `ServerMessage` is an `AgentCorrection`, then it returns it, otherwise it returns `None`
    pub fn as_agent_correction(&self) -> Option<&AgentCorrection> {
        match self {
            ServerMessage::AgentCorrection(agent_correction) => Some(agent_correction),
            _ => None,
        }
    }

    /// Indicates if the message is an agent response
    pub fn is_agent_response(&self) -> bool {
        matches!(*self, ServerMessage::AgentResponse(_))
    }

    /// If the `ServerMessage` is an `AgentResponse`, then it returns it, otherwise it returns `None`
    pub fn as_agent_response(&self) -> Option<&AgentResponse> {
        match self {
            ServerMessage::AgentResponse(agent) => Some(agent),
            _ => None,
        }
    }

    /// Indicates if the message is an audio response
    pub fn is_audio(&self) -> bool {
        matches!(*self, ServerMessage::Audio(_))
    }

    /// If the `ServerMessage` is an `Audio`, then it returns it, otherwise it returns `None`
    pub fn as_audio(&self) -> Option<&Audio> {
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

    /// Indicates if the message is a conversation initiation metadata response
    pub fn is_init_metadata(&self) -> bool {
        matches!(*self, ServerMessage::ConversationInitiationMetadata(_))
    }

    /// If the `ServerMessage` is an `ConversationInitiationMetadata`, then it returns it, otherwise it returns `None`
    pub fn as_init_metadata(&self) -> Option<&ConversationInitiationMetadata> {
        match self {
            ServerMessage::ConversationInitiationMetadata(init_metadata) => Some(init_metadata),
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

    /// Indicates if the message is a MCP connection status response
    pub fn is_mcp_connection_status(&self) -> bool {
        matches!(*self, ServerMessage::McpConnectionStatus(_))
    }

    /// If the `ServerMessage` is a `McpConnectionStatus`, then it returns it, otherwise it returns `None`
    pub fn as_mcp_connection_status(&self) -> Option<&McpConnectionStatus> {
        match self {
            ServerMessage::McpConnectionStatus(mcp_connection_status) => Some(mcp_connection_status),
            _ => None,
        }
    }

    /// Indicates if the message is a MCP tool call response
    pub fn is_mcp_tool_call(&self) -> bool {
        matches!(*self, ServerMessage::McpToolCall(_))
    }

    /// If the `ServerMessage` is a `McpToolCall`, then it returns it, otherwise it returns `None`
    pub fn as_mcp_tool_call(&self) -> Option<&McpToolCall> {
        match self {
            ServerMessage::McpToolCall(mcp_tool_call) => Some(mcp_tool_call),
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

    /// If the `ServerMessage` is a `TentativeAgentResponse`, then it returns it, otherwise it returns `None`
    pub fn as_tentative_agent(&self) -> Option<&TentativeAgentResponse> {
        match self {
            ServerMessage::TentativeAgent(tentative_agent) => Some(tentative_agent),
            _ => None,
        }
    }

    /// Indicates if the message is a Turn Probability response
    pub fn is_turn_probability(&self) -> bool {
        matches!(*self, ServerMessage::TurnProbability(_))
    }

    /// If the `ServerMessage` is a `TurnProbability`, then it returns it, otherwise it returns `None`
    pub fn as_turn_probability(&self) -> Option<&TurnProbability> {
        match self {
            ServerMessage::TurnProbability(turn_probability) => Some(turn_probability),
            _ => None,
        }
    }

    /// Indicates if the message is a user transcription response
    pub fn is_user_transcript(&self) -> bool {
        matches!(*self, ServerMessage::UserTranscript(_))
    }

    /// If the `ServerMessage` is a `UserTranscript`, then it returns it, otherwise it returns `None`
    pub fn as_user_transcript(&self) -> Option<&UserTranscript> {
        match self {
            ServerMessage::UserTranscript(transcription) => Some(transcription),
            _ => None,
        }
    }

    /// Indicates if the message is a VAD Score response
    pub fn is_vad_score(&self) -> bool {
        matches!(*self, ServerMessage::VadScore(_))
    }

    /// If the `ServerMessage` is a `VadScore`, then it returns it, otherwise it returns `None`
    pub fn as_vad_score(&self) -> Option<&VadScore> {
        match self {
            ServerMessage::VadScore(vad_score) => Some(vad_score),
            _ => None,
        }
    }
}


impl TryFrom<&str> for ServerMessage {
    type Error = ConvAIError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let response: ServerMessage = serde_json::from_str(text)?;
        Ok(response)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentCorrection {
    pub r#type: String,
    pub agent_response_correction_event: AgentResponseCorrectionEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentResponseCorrectionEvent {
    pub corrected_agent_response: String,
    pub original_agent_response: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentResponse {
    pub r#type: String,
    pub agent_response_event: AgentResponseEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentResponseEvent {
    pub agent_response: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Audio {
    pub r#type: String,
    pub audio_event: AudioEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioEvent {
    pub audio_base_64: String,
    pub event_id: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToolCall {
    pub r#type: String,
    pub client_tool_call: ClientTool,
}

impl ClientToolCall {
    pub fn id(&self) -> &str {
        &self.client_tool_call.tool_call_id
    }
    pub fn name(&self) -> &str {
        &self.client_tool_call.tool_name
    }
    pub fn parameters(&self) -> &serde_json::Value {
        &self.client_tool_call.parameters
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientTool {
    pub tool_name: String,
    pub tool_call_id: String,
    pub parameters: serde_json::Value,
}

/// See [Conversation Initiation Metadata API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#conversation_initiation_metadata)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationInitiationMetadata {
    pub r#type: String,
    pub conversation_initiation_metadata_event: ConversationInitiationMetadataEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationInitiationMetadataEvent {
    pub conversation_id: String,
    pub agent_output_audio_format: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Interruption {
    pub r#type: String,
    pub interruption_event: InterruptionEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InterruptionEvent {
    pub event_id: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpConnectionStatus {
    pub r#type: String,
    pub mcp_connection_status: McpConnectionStatusEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpConnectionStatusEvent {
    pub integrations: Vec<IntegrationStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntegrationStatus {
    pub integration_id: String,
    pub integration_type: String,
    pub is_connected: bool,
    pub tool_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpToolCall {
    pub r#type: String,
    pub mcp_tool_call: McpToolCallEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpToolCallEvent {
    pub service_id: String,
    pub tool_call_id: String,
    pub tool_name: String,
    pub tool_description: Option<String>,
    pub parameters: serde_json::Value,
    pub timestamp: String,
    pub state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ping {
    pub r#type: String,
    pub ping_event: PingEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PingEvent {
    pub event_id: u32,
    pub ping_ms: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TentativeAgentResponse {
    pub r#type: String,
    pub tentative_agent_response_internal_event: TentativeAgentResponseInternalEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TentativeAgentResponseInternalEvent {
    pub tentative_agent_response: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TurnProbability {
    pub r#type: String,
    pub turn_probability_internal_event: TurnProbabilityInternalEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TurnProbabilityInternalEvent {
    pub turn_probability: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserTranscript {
    pub r#type: String,
    pub user_transcription_event: UserTranscriptionEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserTranscriptionEvent {
    pub user_transcript: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VadScore {
    pub r#type: String,
    pub vad_score_internal_event: VadScoreInternalEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VadScoreInternalEvent {
    pub vad_score: f32,
}