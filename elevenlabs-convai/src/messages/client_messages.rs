use super::*;
use elevenlabs_rs::endpoints::convai::agents::Overrides;

const PONG: &str = "pong";
const CONVERSATION_INITIATION_CLIENT_DATA: &str = "conversation_initiation_client_data";
const CLIENT_TOOL_RESULT: &str = "client_tool_result";

/// An enum for new types of individual client messages.
#[derive(Clone, Debug)]
pub enum ClientMessage {
    /// A new type of `UserAudioChunk`
    UserAudioChunk(UserAudioChunk),
    /// A new type of `Pong`
    Pong(Pong),
    /// A new type of `ConversationInitiationClientData`
    ConversationInitiationClientData(ConversationInitiationClientData),
    /// A new type of `ClientToolResult`
    ClientToolResult(ClientToolResult),
}

/// See [User Audio Chunks API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#user-audio-chunk)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserAudioChunk {
    pub user_audio_chunk: String,
}

impl UserAudioChunk {
    /// Constructs a `UserAudioChunk` to be sent to the websocket server.
    ///
    /// `audio_chunk` must be base 64 encoded.
    pub fn new(audio_chunk: impl Into<String>) -> Self {
        UserAudioChunk {
            user_audio_chunk: audio_chunk.into(),
        }
    }
}

impl TryFrom<UserAudioChunk> for Message {
    type Error = ConvAIError;
    fn try_from(chunk: UserAudioChunk) -> Result<Self, Self::Error> {
        Ok(Message::Text(serde_json::to_string(&chunk)?))
    }
}

/// See [Pong Message API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#pong-message)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pong {
    r#type: String,
    pub event_id: u32,
}

impl Pong {
    /// Constructs a `Pong`.
    ///
    /// The `event_id` must match the one received in the `Ping` message.
    pub fn new(event_id: u32) -> Self {
        Pong {
            r#type: PONG.to_string(),
            event_id,
        }
    }
}

impl TryFrom<Pong> for Message {
    type Error = ConvAIError;
    fn try_from(pong: Pong) -> Result<Self, Self::Error> {
        Ok(Message::Text(serde_json::to_string(&pong)?))
    }
}

/// An optional first websocket message to the server
/// to override agent configuration and/or provide additional LLM configuration parameters.
///
/// See [Dynamic Conversation](https://elevenlabs.io/docs/conversational-ai/customization/conversation-configuration)
#[derive(Debug, Clone, Serialize)]
pub struct ConversationInitiationClientData {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_config_override: Option<Overrides>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_llm_extra_body: Option<ExtraBody>,
}

/// Additional LLM configuration parameters
#[derive(Debug, Clone, Serialize, Default)]
pub struct ExtraBody {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl ExtraBody {
    /// Constructs a new `ExtraBody` with the given `temperature` and `max_tokens`.
    pub fn new(temperature: f32, max_tokens: u32) -> Self {
        ExtraBody {
            temperature: Some(temperature),
            max_tokens: Some(max_tokens),
        }
    }

    /// Sets the temperature of the `ExtraBody`.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the maximum tokens of the `ExtraBody`.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

impl ConversationInitiationClientData {
    /// Sets the `Overrides` of the `ConversationInitiationClientData`.
    pub fn with_overrides(mut self, overrides: Overrides) -> Self {
        self.conversation_config_override = Some(overrides);
        self
    }
    /// Sets the `ExtraBody` of the `ConversationInitiationClientData`.
    pub fn with_custom_llm_extra_body(mut self, extra_body: ExtraBody) -> Self {
        self.custom_llm_extra_body = Some(extra_body);
        self
    }
}

impl Default for ConversationInitiationClientData {
    fn default() -> Self {
        ConversationInitiationClientData {
            r#type: CONVERSATION_INITIATION_CLIENT_DATA.to_string(),
            conversation_config_override: None,
            custom_llm_extra_body: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToolResult {
    r#type: String,
    pub client_tool_id: Option<String>,
    pub result: Option<String>,
    pub is_error: Option<bool>
}

impl Default for ClientToolResult {
    fn default() -> Self {
        ClientToolResult {
            r#type: CLIENT_TOOL_RESULT.to_string(),
            client_tool_id: None,
            result: None,
            is_error: None
        }
    }
}

impl ClientToolResult {
    pub fn with_client_tool_id(mut self, client_tool_id: String) -> Self {
        self.client_tool_id = Some(client_tool_id);
        self
    }
    pub fn with_result(mut self, result: String) -> Self {
        self.result = Some(result);
        self
    }
    pub fn with_is_error(mut self, is_error: bool) -> Self {
        self.is_error = Some(is_error);
        self
    }
}



impl TryFrom<ConversationInitiationClientData> for Message {
    type Error = ConvAIError;
    fn try_from(data: ConversationInitiationClientData) -> Result<Self, Self::Error> {
        Ok(Message::Text(serde_json::to_string(&data)?))
    }
}