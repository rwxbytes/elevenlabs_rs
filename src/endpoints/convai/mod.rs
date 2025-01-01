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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum Language {
    Arabic,
    Bulgarian,
    Chinese,
    Croatian,
    Czech,
    Danish,
    Dutch,
    #[default]
    English,
    Finnish,
    French,
    German,
    Greek,
    Hindi,
    Hungarian,
    Indonesian,
    Italian,
    Japanese,
    Korean,
    Malay,
    Norwegian,
    Polish,
    Portuguese,
    Romanian,
    Russian,
    Slovak,
    Spanish,
    Swedish,
    Tamil,
    Turkish,
    Ukrainian,
    Vietnamese,
}

impl Language {
    fn to_code<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Language::Arabic => serializer.serialize_unit_variant("Language", 0, "ar"),
            Language::Bulgarian => serializer.serialize_unit_variant("Language", 1, "bg"),
            Language::Chinese => serializer.serialize_unit_variant("Language", 2, "zh"),
            Language::Croatian => serializer.serialize_unit_variant("Language", 3, "hr"),
            Language::Czech => serializer.serialize_unit_variant("Language", 4, "cs"),
            Language::Danish => serializer.serialize_unit_variant("Language", 5, "da"),
            Language::Dutch => serializer.serialize_unit_variant("Language", 6, "nl"),
            Language::English => serializer.serialize_unit_variant("Language", 7, "en"),
            Language::Finnish => serializer.serialize_unit_variant("Language", 8, "fi"),
            Language::French => serializer.serialize_unit_variant("Language", 9, "fr"),
            Language::German => serializer.serialize_unit_variant("Language", 10, "de"),
            Language::Greek => serializer.serialize_unit_variant("Language", 11, "el"),
            Language::Hindi => serializer.serialize_unit_variant("Language", 12, "hi"),
            Language::Hungarian => serializer.serialize_unit_variant("Language", 13, "hu"),
            Language::Indonesian => serializer.serialize_unit_variant("Language", 14, "id"),
            Language::Italian => serializer.serialize_unit_variant("Language", 15, "it"),
            Language::Japanese => serializer.serialize_unit_variant("Language", 16, "ja"),
            Language::Korean => serializer.serialize_unit_variant("Language", 17, "ko"),
            Language::Malay => serializer.serialize_unit_variant("Language", 18, "ms"),
            Language::Norwegian => serializer.serialize_unit_variant("Language", 19, "no"),
            Language::Polish => serializer.serialize_unit_variant("Language", 20, "pl"),
            Language::Portuguese => serializer.serialize_unit_variant("Language", 21, "pt"),
            Language::Romanian => serializer.serialize_unit_variant("Language", 22, "ro"),
            Language::Russian => serializer.serialize_unit_variant("Language", 23, "ru"),
            Language::Slovak => serializer.serialize_unit_variant("Language", 24, "sk"),
            Language::Spanish => serializer.serialize_unit_variant("Language", 25, "es"),
            Language::Swedish => serializer.serialize_unit_variant("Language", 26, "sv"),
            Language::Tamil => serializer.serialize_unit_variant("Language", 27, "ta"),
            Language::Turkish => serializer.serialize_unit_variant("Language", 28, "tr"),
            Language::Ukrainian => serializer.serialize_unit_variant("Language", 29, "uk"),
            Language::Vietnamese => serializer.serialize_unit_variant("Language", 30, "vi"),
        }
    }
    fn from_code<'de, D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let code: &str = serde::Deserialize::deserialize(deserializer)?;
        match code {
            "ar" => Ok(Language::Arabic),
            "bg" => Ok(Language::Bulgarian),
            "zh" => Ok(Language::Chinese),
            "hr" => Ok(Language::Croatian),
            "cs" => Ok(Language::Czech),
            "da" => Ok(Language::Danish),
            "nl" => Ok(Language::Dutch),
            "en" => Ok(Language::English),
            "fi" => Ok(Language::Finnish),
            "fr" => Ok(Language::French),
            "de" => Ok(Language::German),
            "el" => Ok(Language::Greek),
            "hi" => Ok(Language::Hindi),
            "hu" => Ok(Language::Hungarian),
            "id" => Ok(Language::Indonesian),
            "it" => Ok(Language::Italian),
            "ja" => Ok(Language::Japanese),
            "ko" => Ok(Language::Korean),
            "ms" => Ok(Language::Malay),
            "no" => Ok(Language::Norwegian),
            "pl" => Ok(Language::Polish),
            "pt" => Ok(Language::Portuguese),
            "ro" => Ok(Language::Romanian),
            "ru" => Ok(Language::Russian),
            "sk" => Ok(Language::Slovak),
            "es" => Ok(Language::Spanish),
            "sv" => Ok(Language::Swedish),
            "ta" => Ok(Language::Tamil),
            "tr" => Ok(Language::Turkish),
            "uk" => Ok(Language::Ukrainian),
            "vi" => Ok(Language::Vietnamese),
            _ => Err(serde::de::Error::custom("language code unexpected")),
        }
    }
}
