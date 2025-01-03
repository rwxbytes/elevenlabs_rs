pub use super::*;
use crate::endpoints::convai::agents::{ConversationConfig, CreateAgentBody, PlatformSettings};
use serde::Serializer;

pub mod agents;
pub mod conversations;
pub mod knowledge_base;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AgentID(pub(crate) String);

impl std::fmt::Display for AgentID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AgentID {
    fn from(id: String) -> Self {
        AgentID(id)
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DocumentationID(pub(crate) String);

impl std::fmt::Display for DocumentationID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientEvent {
    AgentResponse,
    AgentResponseCorrection,
    AsrInitiationMetadata,
    Audio,
    ClientToolCall,
    ConversationInitiationMetadata,
    InternalTentativeAgentResponse,
    InternalTurnProbability,
    InternalVadScore,
    Interruption,
    Ping,
    UserTranscript,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ConvAIModel {
    #[default]
    #[serde(rename = "eleven_turbo_v2")]
    ElevenTurboV2,
    #[serde(rename = "eleven_turbo_v2_5")]
    ElevenTurboV2_5,
    #[serde(rename = "eleven_flash_v2")]
    ElevenFlashV2,
    #[serde(rename = "eleven_flash_v2_5")]
    ElevenFlashV2_5,
}
