//#![deny(missing_docs)]
use crate::shared::Language;
use super::{Deserialize, ElevenLabsConversationalError, Message, Result, Serialize};

const PONG: &str = "pong";
const CONVERSATION_INITIATION_CLIENT_DATA: &str = "conversation_initiation_client_data";


/// An enum for new types of individual client messages.
#[derive(Clone, Debug)]
pub enum ClientMessage {
    /// A new type of `UserAudioChunk`
    UserAudioChunk(UserAudioChunk),
    /// A new type of `Pong`
    Pong(Pong),
    /// A new type of `ConversationInitiationClientData`
    ConversationInitiationClientData(ConversationInitiationClientData),
}

/// Base64 encoded audio chunk from the user to the server
///
/// Notes:
///
/// - Audio Format Requirements:
///   - PCM 16-bit mono format
///   - Base64 encoded
///   - Sample rate of 16,000 Hz
///
/// - Recommended Chunk Duration:
///   - Send audio chunks approximately every 250 milliseconds (0.25 seconds)
///   - This equates to chunks of about 4,000 samples at a 16,000 Hz sample rate
///
/// - Optimizing Latency and Efficiency:
///   - Balance Latency and Efficiency: Sending audio chunks every 250 milliseconds offers a good trade-off between responsiveness and network overhead.
///   - Adjust Based on Needs:
///     - Lower Latency Requirements: Decrease the chunk duration to send smaller chunks more frequently.
///     - Higher Efficiency Requirements: Increase the chunk duration to send larger chunks less frequently.
///     - Network Conditions: Adapt the chunk size if you experience network constraints or variability.
///
/// see Elevenlabs' docs on [user audio chunks](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#user-audio-chunk)
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
    type Error = ElevenLabsConversationalError;
    fn try_from(chunk: UserAudioChunk) -> Result<Self> {
        Ok(Message::Text(serde_json::to_string(&chunk)?))
    }
}

/// see Elevenlabs' docs on [pong messages](https://elevenlabs.io/docs/conversational-ai/api-reference/websocket#pong-message)
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
    type Error = ElevenLabsConversationalError;
    fn try_from(pong: Pong) -> Result<Self> {
        Ok(Message::Text(serde_json::to_string(&pong)?))
    }
}

/// An optional first websocket message to the server
/// to override agent configuration and/or provide additional LLM configuration parameters
#[derive(Debug, Clone, Serialize)]
pub struct ConversationInitiationClientData {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_config_override: Option<Overrides>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_llm_extra_body: Option<ExtraBody>,
}

/// Override settings for conversation behavior
///
/// See ElevenLabs' docs on [dynamic conversation](https://elevenlabs.io/docs/conversational-ai/customization/conversation-configuration)
#[derive(Debug, Clone, Serialize, Default)]
pub struct Overrides {
    /// Override settings for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentOverrides>,
    /// Override settings for the TTS voice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<TTSOverrides>,
}

/// Override settings for the agent
#[derive(Debug, Clone, Serialize, Default)]
pub struct AgentOverrides {
    /// Override the agent's system prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<PromptOverride>,
    /// Override the agent's first message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
    /// Override the agent's language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
}

/// Override the agent's system prompt
#[derive(Debug, Clone, Serialize)]
pub struct PromptOverride {
    pub prompt: String,
}

impl PromptOverride {
    /// Constructs a new `PromptOverride` with the given `prompt`.
    pub fn new(prompt: impl Into<String>) -> Self {
        PromptOverride {
            prompt: prompt.into(),
        }
    }
}

/// Override the agent's TTS voice
#[derive(Debug, Clone, Serialize,)]
pub struct TTSOverrides {
    pub voice_id: String,
}

impl TTSOverrides {
    /// Constructs a new `TTSOverrides` with the given `voice_id`.
    pub fn new(voice_id: impl Into<String>) -> Self {
        TTSOverrides {
            voice_id: voice_id.into(),
        }
    }
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

impl TryFrom<ConversationInitiationClientData> for Message {
    type Error = ElevenLabsConversationalError;
    fn try_from(data: ConversationInitiationClientData) -> Result<Self> {
        Ok(Message::Text(serde_json::to_string(&data)?))
    }
}