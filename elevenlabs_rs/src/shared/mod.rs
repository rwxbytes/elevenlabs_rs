#![allow(deprecated)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use serde::{Deserialize, Serialize, Serializer};
use std::string::ToString;
use serde_json::Value;
use strum::Display;
use crate::endpoints::admin::voice_library::{Gender, SharedVoiceCategory};

pub(crate) mod url;

pub mod response_bodies {
    use serde::Deserialize;
    #[derive(Clone, Debug, Deserialize)]
    pub struct StatusResponseBody {
        pub status: String,
    }
}

#[allow(dead_code)]
pub mod query_params {
    use super::*;

    //#[deprecated(since = "0.3.2")]
    //#[derive(Clone, Debug, Display)]
    //pub enum Latency {
    //    /// Default latency
    //    #[strum(to_string = "0")]
    //    None = 0,
    //    ///  normal latency optimizations (about 50% of possible latency improvement of option 3)
    //    #[strum(to_string = "1")]
    //    Normal = 1,
    //    /// strong latency optimizations (about 75% of possible latency improvement of option 3)
    //    #[strum(to_string = "2")]
    //    Strong = 2,
    //    /// max latency optimizations
    //    #[strum(to_string = "3")]
    //    Max = 3,
    //    /// max latency optimizations, but also with text normalizer turned off for even more latency
    //    /// savings (the best latency, but can mispronounce e.g. numbers and dates)
    //    #[strum(to_string = "4")]
    //    MaxBest = 4,
    //}

    /// See Elevenlabs documentation on [supported output formats](https://help.elevenlabs.io/hc/en-us/articles/15754340124305-What-audio-formats-do-you-support).
    #[derive(Clone, Debug, Display)]
    pub enum OutputFormat {
        #[strum(to_string = "mp3_22050_32")]
        Mp3_22050Hz32kbps,
        #[strum(to_string = "mp3_44100_32")]
        Mp3_44100Hz32kbps,
        #[strum(to_string = "mp3_44100_64")]
        Mp3_44100Hz64kbps,
        #[strum(to_string = "mp3_44100_96")]
        Mp3_44100Hz96kbps,
        #[strum(to_string = "mp3_44100_128")]
        Mp3_44100Hz128kbps,
        #[strum(to_string = "mp3_44100_192")]
        Mp3_44100Hz192kbps,
        #[strum(to_string = "pcm_16000")]
        Pcm16000Hz,
        #[strum(to_string = "pcm_22050")]
        Pcm22050Hz,
        #[strum(to_string = "pcm_24000")]
        Pcm24000Hz,
        #[strum(to_string = "pcm_44100")]
        Pcm44100Hz,
        #[strum(to_string = "ulaw_8000")]
        MuLaw8000Hz,
    }
}


#[derive(Clone, Debug)]
pub enum Model {
    ElevenMultilingualV2,
    #[deprecated]
    ElevenMultilingualV1,
    #[deprecated]
    ElevenEnglishV1,
    ElevenEnglishV2,
    ElevenTurboV2,
    ElevenTurboV2_5,
    ElevenMultilingualV2STS,
    ElevenFlashV2,
    ElevenFlashV2_5,
}

impl From<Model> for String {
    fn from(model: Model) -> String {
        match model {
            Model::ElevenMultilingualV2 => "eleven_multilingual_v2".to_string(),
            Model::ElevenMultilingualV1 => "eleven_multilingual_v1".to_string(),
            Model::ElevenEnglishV1 => "eleven_monolingual_v1".to_string(),
            Model::ElevenEnglishV2 => "eleven_english_sts_v2".to_string(),
            Model::ElevenTurboV2 => "eleven_turbo_v2".to_string(),
            Model::ElevenTurboV2_5 => "eleven_turbo_v2_5".to_string(),
            Model::ElevenMultilingualV2STS => "eleven_multilingual_sts_v2".to_string(),
            Model::ElevenFlashV2 => "eleven_flash_v2".to_string(),
            Model::ElevenFlashV2_5 => "eleven_flash_v2_5".to_string(),
        }
    }
}


#[derive(Clone, Debug, Default)]
pub enum DefaultVoice {
    Aria,
    Roger,
    Sarah,
    Laura,
    Charlie,
    George,
    Callum,
    River,
    Liam,
    Charlotte,
    Alice,
    Matilda,
    Will,
    Jessica,
    #[default]
    Eric,
    Chris,
    Brian,
    Daniel,
    Lily,
    Bill,
}

impl From<DefaultVoice> for String {
    fn from(id: DefaultVoice) -> String {
        match id {
            DefaultVoice::Aria => "9BWtsMINqrJLrRacOk9x".to_string(),
            DefaultVoice::Roger => "CwhRBWXzGAHq8TQ4Fs17".to_string(),
            DefaultVoice::Sarah => "EXAVITQu4vr4xnSDxMaL".to_string(),
            DefaultVoice::Laura => "FGY2WhTYpPnrIDTdsKH5".to_string(),
            DefaultVoice::Charlie => "IKne3meq5aSn9XLyUdCD".to_string(),
            DefaultVoice::George => "JBFqnCBsd6RMkjVDRZzb".to_string(),
            DefaultVoice::Callum => "N2lVS1w4EtoT3dr4eOWO".to_string(),
            DefaultVoice::River => "SAz9YHcvj6GT2YYXdXww".to_string(),
            DefaultVoice::Liam => "TX3LPaxmHKxFdv7VOQHJ".to_string(),
            DefaultVoice::Charlotte => "XB0fDUnXU5powFXDhCwa".to_string(),
            DefaultVoice::Alice => "Xb7hH8MSUJpSbSDYk0k2".to_string(),
            DefaultVoice::Matilda => "XrExE9yKIg1WjnnlVkGX".to_string(),
            DefaultVoice::Will => "bIHbv24MWmeRgasZH58o".to_string(),
            DefaultVoice::Jessica => "cgSgspJ2msm6clMCkdW9".to_string(),
            DefaultVoice::Eric => "cjVigY5qzO86Huf0OWal".to_string(),
            DefaultVoice::Chris => "iP95p4xoKVk53GoZ742B".to_string(),
            DefaultVoice::Brian => "nPczCjzI2devNBz1zQrb".to_string(),
            DefaultVoice::Daniel => "onwK4e9ZLuTAKqWW03F9".to_string(),
            DefaultVoice::Lily => "pFZP5JQG7iQjIQuC4Bku".to_string(),
            DefaultVoice::Bill => "pqHfZKP75CvOlQylNhV4".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LegacyVoice {
    Adam,
    Antoni,
    Arnold,
    Clyde,
    Dave,
    Dorothy,
    Drew,
    Domi,
    Eli,
    Emily,
    Ethan,
    Fin,
    Freya,
    Gigi,
    Giovanni,
    Glinda,
    Grace,
    Harry,
    James,
    Jessie,
    Jeremy,
    Joseph,
    Josh,
    Michael,
    Mimi,
    Nicole,
    Patrick,
    Paul,
    Rachel,
    Sam,
    Serena,
    Thomas,
}

impl From<LegacyVoice> for String {
    fn from(id: LegacyVoice) -> String {
        match id {
            LegacyVoice::Adam => "pNInz6obpgDQGcFmaJgB".to_string(),
            LegacyVoice::Antoni => "ErXwobaYiN019PkySvjV".to_string(),
            LegacyVoice::Arnold => "VR6AewLTigWG4xSOukaG".to_string(),
            LegacyVoice::Clyde => "2EiwWnXFnvU5JabPnv8n".to_string(),
            LegacyVoice::Dave => "CYw3kZ02Hs0563khs1Fj".to_string(),
            LegacyVoice::Dorothy => "ThT5KcBeYPX3keUQqHPh".to_string(),
            LegacyVoice::Drew => "29vD33N1CtxCmqQRPOHJ".to_string(),
            LegacyVoice::Domi => "AZnzlk1XvdvUeBnXmlld".to_string(),
            LegacyVoice::Eli => "MF3mGyEYCl7XYWbV9V6O".to_string(),
            LegacyVoice::Emily => "LcfcDJNUP1GQjkzn1xUU".to_string(),
            LegacyVoice::Ethan => "g5CIjZEefAph4nQFvHAz".to_string(),
            LegacyVoice::Fin => "D38z5RcWu1voky8WS1ja".to_string(),
            LegacyVoice::Freya => "jsCqWAovK2LkecY7zXl4".to_string(),
            LegacyVoice::Gigi => "jBpfuIE2acCO8z3wKNLl".to_string(),
            LegacyVoice::Giovanni => "zcAOhNBS3c14rBihAFp1".to_string(),
            LegacyVoice::Glinda => "z9fAnlkpzviPz146aGWa".to_string(),
            LegacyVoice::Grace => "oWAxZDx7w5VEj9dCyTzz".to_string(),
            LegacyVoice::Harry => "SOYHLrjzK2X1ezoPC6cr".to_string(),
            LegacyVoice::James => "ZQe5CZNOzWyzPSCn5a3c".to_string(),
            LegacyVoice::Jessie => "t0jbNlBVZ17f02VDIeMI".to_string(),
            LegacyVoice::Jeremy => "bVMeCyTHy58xNoL34h3p".to_string(),
            LegacyVoice::Joseph => "Zlb1dXrM653N07WRdFW3".to_string(),
            LegacyVoice::Josh => "TxGEqnHWrfWFTfGW9XjX".to_string(),
            LegacyVoice::Michael => "flq6f7yk4E4fJM5XTYuZ".to_string(),
            LegacyVoice::Mimi => "zrHiDhphv9ZnVXBqCLjz".to_string(),
            LegacyVoice::Nicole => "piTKgcLEGmPE4e6mEKli".to_string(),
            LegacyVoice::Patrick => "ODq5zmih8GrVes37Dizd".to_string(),
            LegacyVoice::Paul => "5Q0t7uMcjvnagumLfvZi".to_string(),
            LegacyVoice::Rachel => "21m00Tcm4TlvDq8ikWAM".to_string(),
            LegacyVoice::Sam => "yoZ06aMxZJJ28mfd3POQ".to_string(),
            LegacyVoice::Serena => "pMsXgVXv3BLzUgSXRplE".to_string(),
            LegacyVoice::Thomas => "GBv7mTt0atIp3Br8iCZE".to_string(),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}

impl VoiceSettings {
    pub fn new(stability: f32, similarity: f32) -> Self {
        VoiceSettings {
            similarity_boost: similarity,
            stability,
            style: None,
            use_speaker_boost: None,
            speed: None,
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

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
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
            speed: Some(1.0),
        }
    }
}

/// Verified language
#[derive(Clone, Debug, Deserialize)]
pub struct VerifiedLanguage {
    pub language: String,
    pub model_id: String,
    accent: Option<String>,
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

/// Voice sample
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct VoiceSample {
    pub sample_id: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub hash: Option<String>,
}

/// Shared voice
#[derive(Clone, Debug, Deserialize)]
pub struct SharedVoice {
    pub public_owner_id: String,
    pub voice_id: String,
    pub date_unix: f32,
    pub name: String,
    pub accent: String,
    pub gender: Gender,
    pub age: Age,
    pub descriptive: String,
    pub use_case: String,
    pub category: SharedVoiceCategory,
    #[serde(deserialize_with = "Language::from_code")]
    pub language: Language,
    pub description: String,
    pub preview_url: String,
    pub usage_character_count_1y: f32,
    pub usage_character_count_7d: f32,
    pub play_api_usage_character_count_1y: f32,
    pub cloned_by_count: f32,
    pub rate: f32,
    pub free_users_allowed: bool,
    pub live_moderation_enabled: bool,
    pub featured: bool,
    pub verified_language: Option<Vec<VerifiedLanguage>>,
    pub notice_period: Option<f32>,
    pub instagram_username: Option<String>,
    pub twitter_username: Option<String>,
    pub youtube_username: Option<String>,
    pub tiktok_username: Option<String>,
    pub image_url: Option<String>,
    pub is_added_by_user: Option<bool>,
}

/// Safety control
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SafetyControl {
    None,
    Ban,
    Captcha,
    CaptchaAndModeration,
    EnterpriseBan,
    EnterpriseCaptcha,
}

/// Fine-Tuning
#[derive(Clone, Debug, Deserialize)]
pub struct FineTuning {
    pub is_allowed_to_fine_tune: Option<bool>,
    pub state: Option<HashMap<String, FineTuningState>>,
    pub verification_failures: Option<Vec<String>>,
    pub verification_attempts_count: Option<u32>,
    pub manual_verification_requested: Option<bool>,
    pub language: Option<String>,
    pub progress: Option<HashMap<String, f32>>,
    pub message: Option<HashMap<String, String>>,
    pub dataset_duration_seconds: Option<u32>,
    pub verification_attempts: Option<Vec<VerificationAttempt>>,
    pub slice_ids: Option<Vec<String>>,
    pub manual_verification: Option<ManualVerification>,
    pub max_verification_attempts: Option<u32>,
    pub next_max_verification_attempts_rest_unix_ms: Option<u32>,
    pub finetuning_state: Option<Value>,
}

/// Fine-Tuning state
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FineTuningState {
    NotStarted,
    Queued,
    FineTuning,
    FineTuned,
    Failed,
    Delayed,
}

/// Verification attempt
#[derive(Clone, Debug, Deserialize)]
pub struct VerificationAttempt {
    pub text: String,
    pub date_unix: u64,
    pub accepted: bool,
    pub similarity: f32,
    pub levenshtein_distance: u32,
    pub recording: Option<Recording>,
}

/// Recording
#[derive(Clone, Debug, Deserialize)]
pub struct Recording {
    pub recording_id: String,
    pub mime_type: String,
    pub size_bytes: u32,
    pub upload_date_unix: u64,
    pub transcription: String,
}

/// Manual verification
#[derive(Clone, Debug, Deserialize)]
pub struct ManualVerification {
    pub extra_text: String,
    pub request_time_unix: u64,
    pub files: Vec<ManualVerificationFile>,
}

/// Manual verification file
#[derive(Clone, Debug, Deserialize)]
pub struct ManualVerificationFile {
    pub file_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u32,
    pub upload_date_unix: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sharing {
    pub status: Option<SharingStatus>,
    pub history_item_sample_id: Option<String>,
    pub date_unix: Option<i64>,
    pub whitelisted_emails: Option<Vec<String>>,
    pub public_owner_id: Option<String>,
    pub original_voice_id: Option<String>,
    pub financial_rewards_enabled: Option<bool>,
    pub free_users_allowed: Option<bool>,
    pub live_moderation_enabled: Option<bool>,
    pub rate: Option<f64>,
    pub notice_period: Option<i64>,
    pub disable_at_unix: Option<i64>,
    pub voice_mixing_allowed: Option<bool>,
    pub featured: Option<bool>,
    pub category: Option<VoiceCategory>,
    pub reader_app_enabled: Option<bool>,
    pub image_url: Option<String>,
    pub ban_reason: Option<String>,
    pub liked_by_count: Option<i64>,
    pub cloned_by_count: Option<i64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    pub review_status: Option<ReviewStatus>,
    pub review_message: Option<String>,
    pub enabled_in_library: Option<bool>,
    pub instagram_username: Option<String>,
    pub twitter_username: Option<String>,
    pub youtube_username: Option<String>,
    pub tiktok_username: Option<String>,
    pub moderation_check: Option<ModerationCheck>,
    pub reader_restricted_on: Option<Vec<ReaderRestrictedOn>>,
}

/// Sharing status
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharingStatus {
    Enabled,
    Disabled,
    Copied,
    CopiedDisabled,
}

/// Review status
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    NotRequested,
    Pending,
    Declined,
    Allowed,
    AllowedWithChanges,
}

/// Moderation check
#[derive(Debug, Clone, Deserialize)]
pub struct ModerationCheck {
    pub date_checked_unix: Option<u64>,
    pub name_value: Option<String>,
    pub name_check: Option<bool>,
    pub description_value: Option<String>,
    pub description_check: Option<bool>,
    pub sample_ids: Option<Vec<String>>,
    pub sample_checks: Option<Vec<f64>>,
    pub captcha_ids: Option<Vec<String>>,
    pub captcha_checks: Option<Vec<f64>>,
}

/// Reader restricted on
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReaderRestrictedOn {
    pub resource_type: ResourceType,
    pub resource_id: String,
}

/// Resource type
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Read,
    Collection
}

/// Voice Verification
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceVerification {
    pub requires_verification: bool,
    pub is_verified: bool,
    pub verification_failures: Option<Vec<String>>,
    pub verification_attempts_count: Option<u32>,
    pub language: Option<String>,
    pub verification_attempts: Option<Vec<VerificationAttempt>>,
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

/// Dictionary locator
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DictionaryLocator {
    pronunciation_dictionary_id: String,
    version_id: String,
}

impl DictionaryLocator {
    pub fn new(dictionary_id: &str, version_id: &str) -> Self {
        DictionaryLocator {
            pronunciation_dictionary_id: dictionary_id.to_string(),
            version_id: version_id.to_string(),
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
    Filipino,
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
            Language::Filipino => serializer.serialize_unit_variant("Language", 31, "fil"),
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
            "fil" => Ok(Language::Filipino),
            _ => Err(serde::de::Error::custom("language code unexpected")),
        }
    }
}
