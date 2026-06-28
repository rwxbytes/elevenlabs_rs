use super::*;
use elevenlabs_rs::endpoints::convai::agents::DynamicVar;
use std::collections::HashMap;

const PONG: &str = "pong";
const CONVERSATION_INITIATION_CLIENT_DATA: &str = "conversation_initiation_client_data";
const CLIENT_TOOL_RESULT: &str = "client_tool_result";
const CONTEXTUAL_UPDATE: &str = "contextual_update";
const FEEDBACK: &str = "feedback";
const FILE_INPUT: &str = "file_input";
const MULTIMODAL_MESSAGE: &str = "multimodal_message";
const USER_ACTIVITY: &str = "user_activity";
const USER_MESSAGE: &str = "user_message";

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
    /// A new type of `ContextualUpdate`
    ContextualUpdate(ContextualUpdate),
    Feedback(Feedback),
    UserMessage(UserMessage),
    UserActivity(UserActivity),
    MultimodalMessage(MultimodalMessage),
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
        to_text_message(&chunk)
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
        to_text_message(&pong)
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
    pub conversation_config_override: Option<OverrideData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_llm_extra_body: Option<ExtraBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_variables: Option<HashMap<String, DynamicVar>>,
}

impl ConversationInitiationClientData {
    /// Sets the `OverrideData` of the `ConversationInitiationClientData`.
    pub fn with_override_data(&mut self, overrides: OverrideData) {
        self.conversation_config_override = Some(overrides);
    }
    /// Sets the `ExtraBody` of the `ConversationInitiationClientData`.
    pub fn with_custom_llm_extra_body(&mut self, extra_body: ExtraBody) {
        self.custom_llm_extra_body = Some(extra_body);
    }

    pub fn with_dynamic_variables(
        mut self,
        dynamic_variables: HashMap<String, DynamicVar>,
    ) -> Self {
        self.dynamic_variables = Some(dynamic_variables);
        self
    }
}

impl Default for ConversationInitiationClientData {
    fn default() -> Self {
        ConversationInitiationClientData {
            r#type: CONVERSATION_INITIATION_CLIENT_DATA.to_string(),
            conversation_config_override: None,
            custom_llm_extra_body: None,
            dynamic_variables: None,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct OverrideData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentOverrideData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<TTSOverrideData>,
}

impl OverrideData {
    pub fn with_agent_override_data(mut self, agent: AgentOverrideData) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn with_tts_override_data(mut self, tts: TTSOverrideData) -> Self {
        self.tts = Some(tts);
        self
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct AgentOverrideData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<PromptOverrideData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl AgentOverrideData {
    pub fn with_prompt_override_data(mut self, prompt: PromptOverrideData) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn override_first_message(mut self, first_message: impl Into<String>) -> Self {
        self.first_message = Some(first_message.into());
        self
    }

    pub fn override_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PromptOverrideData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

impl PromptOverrideData {
    pub fn override_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct TTSOverrideData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
}

impl TTSOverrideData {
    pub fn override_voice_id(mut self, voice_id: impl Into<String>) -> Self {
        self.voice_id = Some(voice_id.into());
        self
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToolResult {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl Default for ClientToolResult {
    fn default() -> Self {
        ClientToolResult {
            r#type: CLIENT_TOOL_RESULT.to_string(),
            tool_call_id: None,
            result: None,
            is_error: None,
        }
    }
}

impl ClientToolResult {
    pub fn new(id: impl Into<String>) -> Self {
        ClientToolResult {
            r#type: CLIENT_TOOL_RESULT.to_string(),
            tool_call_id: Some(id.into()),
            result: None,
            is_error: None,
        }
    }
    pub fn with_client_tool_id(mut self, client_tool_id: String) -> Self {
        self.tool_call_id = Some(client_tool_id);
        self
    }
    pub fn with_tool_call_id(mut self, tool_call_id: impl Into<String>) -> Self {
        self.tool_call_id = Some(tool_call_id.into());
        self
    }
    pub fn with_result(mut self, result: String) -> Self {
        self.result = Some(result);
        self
    }
    pub fn has_error(mut self, is_error: bool) -> Self {
        self.is_error = Some(is_error);
        self
    }
}

impl TryFrom<ClientToolResult> for Message {
    type Error = ConvAIError;
    fn try_from(result: ClientToolResult) -> Result<Self, Self::Error> {
        to_text_message(&result)
    }
}

impl TryFrom<ConversationInitiationClientData> for Message {
    type Error = ConvAIError;
    fn try_from(data: ConversationInitiationClientData) -> Result<Self, Self::Error> {
        to_text_message(&data)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContextualUpdate {
    r#type: String,
    pub text: String,
}

impl ContextualUpdate {
    pub fn new(text: impl Into<String>) -> Self {
        ContextualUpdate {
            r#type: CONTEXTUAL_UPDATE.to_string(),
            text: text.into(),
        }
    }
}

impl TryFrom<ContextualUpdate> for Message {
    type Error = ConvAIError;
    fn try_from(update: ContextualUpdate) -> Result<Self, Self::Error> {
        to_text_message(&update)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feedback {
    r#type: String,
    pub score: String,
    pub event_id: u32,
}

impl Feedback {
    pub fn new(event_id: u32, score: impl Into<String>) -> Self {
        Self {
            r#type: FEEDBACK.to_owned(),
            score: score.into(),
            event_id,
        }
    }

    pub fn like(event_id: u32) -> Self {
        Self::new(event_id, "like")
    }

    pub fn dislike(event_id: u32) -> Self {
        Self::new(event_id, "dislike")
    }
}

impl TryFrom<Feedback> for Message {
    type Error = ConvAIError;
    fn try_from(feedback: Feedback) -> Result<Self, Self::Error> {
        to_text_message(&feedback)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserMessage {
    r#type: String,
    pub text: String,
}

impl UserMessage {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            r#type: USER_MESSAGE.to_owned(),
            text: text.into(),
        }
    }
}

impl TryFrom<UserMessage> for Message {
    type Error = ConvAIError;
    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        to_text_message(&message)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserActivity {
    r#type: String,
}

impl UserActivity {
    pub fn new() -> Self {
        Self {
            r#type: USER_ACTIVITY.to_owned(),
        }
    }
}

impl Default for UserActivity {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<UserActivity> for Message {
    type Error = ConvAIError;
    fn try_from(activity: UserActivity) -> Result<Self, Self::Error> {
        to_text_message(&activity)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultimodalMessage {
    r#type: String,
    pub text: UserMessage,
    pub file: FileInput,
}

impl MultimodalMessage {
    pub fn new(text: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            r#type: MULTIMODAL_MESSAGE.to_owned(),
            text: UserMessage::new(text),
            file: FileInput::new(file_id),
        }
    }
}

impl TryFrom<MultimodalMessage> for Message {
    type Error = ConvAIError;
    fn try_from(message: MultimodalMessage) -> Result<Self, Self::Error> {
        to_text_message(&message)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileInput {
    r#type: String,
    pub file_id: String,
}

impl FileInput {
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            r#type: FILE_INPUT.to_owned(),
            file_id: file_id.into(),
        }
    }
}

impl TryFrom<ClientMessage> for Message {
    type Error = ConvAIError;

    fn try_from(message: ClientMessage) -> Result<Self, Self::Error> {
        match message {
            ClientMessage::UserAudioChunk(message) => message.try_into(),
            ClientMessage::Pong(message) => message.try_into(),
            ClientMessage::ConversationInitiationClientData(message) => message.try_into(),
            ClientMessage::ClientToolResult(message) => message.try_into(),
            ClientMessage::ContextualUpdate(message) => message.try_into(),
            ClientMessage::Feedback(message) => message.try_into(),
            ClientMessage::UserMessage(message) => message.try_into(),
            ClientMessage::UserActivity(message) => message.try_into(),
            ClientMessage::MultimodalMessage(message) => message.try_into(),
        }
    }
}

fn to_text_message<T>(value: &T) -> Result<Message, ConvAIError>
where
    T: Serialize,
{
    Ok(Message::Text(serde_json::to_string(value)?.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn message_json(message: Message) -> Value {
        let Message::Text(text) = message else {
            panic!("expected text message");
        };
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn current_client_events_serialize_to_protocol_shapes() {
        assert_eq!(
            message_json(Message::try_from(Feedback::like(43)).unwrap()),
            json!({ "type": "feedback", "score": "like", "event_id": 43 })
        );
        assert_eq!(
            message_json(Message::try_from(UserMessage::new("upgrade me")).unwrap()),
            json!({ "type": "user_message", "text": "upgrade me" })
        );
        assert_eq!(
            message_json(Message::try_from(UserActivity::new()).unwrap()),
            json!({ "type": "user_activity" })
        );
        assert_eq!(
            message_json(
                Message::try_from(MultimodalMessage::new("What is this file?", "file_12345"))
                    .unwrap()
            ),
            json!({
                "type": "multimodal_message",
                "text": { "type": "user_message", "text": "What is this file?" },
                "file": { "type": "file_input", "file_id": "file_12345" }
            })
        );
    }
}
