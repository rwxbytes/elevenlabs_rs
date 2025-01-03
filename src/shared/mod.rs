use serde::{Deserialize, Serialize, Serializer};
pub mod identifiers;
pub mod response_bodies {
    use serde::Deserialize;
    #[derive(Clone, Debug, Deserialize)]
    pub struct StatusResponseBody {
        pub status: String,
    }
}

#[allow(dead_code)]
pub mod query_params {
    #[derive(Clone, Debug)]
    pub enum Latency {
        /// Default latency
        None = 0,
        ///  normal latency optimizations (about 50% of possible latency improvement of option 3)
        Normal = 1,
        /// strong latency optimizations (about 75% of possible latency improvement of option 3)
        Strong = 2,
        /// max latency optimizations
        Max = 3,
        /// max latency optimizations, but also with text normalizer turned off for even more latency
        /// savings (the best latency, but can mispronounce e.g. numbers and dates)
        MaxBest = 4,
    }

    /// See Elevenlabs documentation on [supported output formats](https://help.elevenlabs.io/hc/en-us/articles/15754340124305-What-audio-formats-do-you-support).
    #[derive(Clone, Debug)]
    pub enum OutputFormat {
        Mp3_22050Hz32kbps,
        Mp3_44100Hz32kbps,
        Mp3_44100Hz64kbps,
        Mp3_44100Hz96kbps,
        Mp3_44100Hz192kbps,
        Pcm16000Hz,
        Pcm22050Hz,
        Pcm24000Hz,
        Pcm44100Hz,
        MuLaw8000Hz,
    }
    impl OutputFormat {
        pub(crate) fn to_query(&self) -> &str {
            match self {
                OutputFormat::Pcm16000Hz => "pcm_16000",
                OutputFormat::Pcm22050Hz => "pcm_22050",
                OutputFormat::Pcm24000Hz => "pcm_24000",
                OutputFormat::Pcm44100Hz => "pcm_44100",
                OutputFormat::Mp3_22050Hz32kbps => "mp3_22050_32",
                OutputFormat::Mp3_44100Hz32kbps => "mp3_44100_32",
                OutputFormat::Mp3_44100Hz64kbps => "mp3_44100_64",
                OutputFormat::Mp3_44100Hz96kbps => "mp3_44100_96",
                OutputFormat::Mp3_44100Hz192kbps => "mp3_44100_192",
                OutputFormat::MuLaw8000Hz => "ulaw_8000",
            }
        }
    }
}

/// Voice settings
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VoiceSettings {
    pub similarity_boost: f32,
    pub stability: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_speaker_boost: Option<bool>,
}

impl VoiceSettings {
    pub fn new(stability: f32, similarity: f32) -> Self {
        VoiceSettings {
            similarity_boost: similarity,
            stability,
            style: None,
            use_speaker_boost: None,
        }
    }
    pub fn with_similarity_boost(mut self, similarity_boost: f32) -> Self {
        self.similarity_boost = similarity_boost;
        self
    }
    pub fn with_stability(mut self, stability: f32) -> Self {
        self.stability = stability;
        self
    }
    pub fn with_style(mut self, style: f32) -> Self {
        self.style = Some(style);
        self
    }
    pub fn with_use_speaker_boost(mut self, use_speaker_boost: bool) -> Self {
        self.use_speaker_boost = Some(use_speaker_boost);
        self
    }
}

impl Default for VoiceSettings {
    fn default() -> Self {
        VoiceSettings {
            similarity_boost: 0.75,
            stability: 0.5,
            style: Some(0.5),
            use_speaker_boost: Some(true),
        }
    }
}

/// Voice category
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VoiceCategory {
    Generated,
    Cloned,
    Premade,
    Professional,
    Famous,
    HighQuality,
}

/// Age
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Age {
    Young,
    MiddleAged,
    Old,
}

impl Age {
    pub fn as_str(&self) -> &str {
        match self {
            Age::Young => "young",
            Age::MiddleAged => "middle_aged",
            Age::Old => "old",
        }
    }
}

/// Language
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
    pub fn to_code<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
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
    pub fn from_code<'de, D>(deserializer: D) -> std::result::Result<Self, D::Error>
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

//impl std::fmt::Display for Language {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match self {
//            Language::Arabic => write!(f, "Arabic"),
//        }
