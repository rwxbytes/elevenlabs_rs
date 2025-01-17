use super::*;

/// See [Server Messages](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#server-to-client-messages)
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    AgentResponse(AgentResponse),
    Audio(Audio),
    ClientToolCall(ClientToolCall),
    ClientToolResult(ClientToolResult),
    ConversationInitiationMetadata(ConversationInitiationMetadata),
    Interruption(Interruption),
    Ping(Ping),
    UserTranscript(UserTranscript),
    NotDocumented(serde_json::Value),
}

impl ServerMessage {
    /// Indicates if the message is an agent response
    pub fn is_agent(&self) -> bool {
        matches!(*self, ServerMessage::AgentResponse(_))
    }

    /// If the `ServerMessage` is an `AgentResponse`, then it returns it, otherwise it returns `None`
    pub fn as_agent(&self) -> Option<&AgentResponse> {
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

    /// Indicates if the message is a user transcription response
    pub fn is_transcription(&self) -> bool {
        matches!(*self, ServerMessage::UserTranscript(_))
    }

    /// If the `ServerMessage` is a `UserTranscript`, then it returns it, otherwise it returns `None`
    pub fn as_transcription(&self) -> Option<&UserTranscript> {
        match self {
            ServerMessage::UserTranscript(transcription) => Some(transcription),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientTool {
    pub tool_name: String,
    pub tool_call_id: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToolResult {
    pub r#type: String,
    pub client_tool_id: String,
    pub result: String,
    pub is_error: bool,
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
pub struct UserTranscript {
    pub r#type: String,
    pub user_transcription_event: UserTranscriptionEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserTranscriptionEvent {
    pub user_transcript: String,
}