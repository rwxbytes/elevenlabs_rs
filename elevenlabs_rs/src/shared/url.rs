use std::string::ToString;
use strum::IntoStaticStr;

pub(crate) trait AndPathParam {
    fn and_param(&self, id: PathParam) -> (&'static str, &str);
}

impl AndPathParam for String {
    fn and_param(&self, param: PathParam) -> (&'static str, &str) {
        (param.into(), self)
    }
}


#[allow(dead_code)]
#[derive(IntoStaticStr)]
pub(crate) enum PathParam {
    #[strum(serialize = ":agent_id")]
    AgentID,
    #[strum(serialize = ":conversation_id")]
    ConversationID,
    #[strum(serialize = ":documentation_id")]
    DocumentationID,
    #[strum(serialize = ":dubbing_id")]
    DubbingID,
    #[strum(serialize = ":history_item_id")]
    HistoryItemID,
    #[strum(serialize = ":model_id")]
    ModelID,
    #[strum(serialize = ":language_code")]
    LanguageCodeID,
    #[strum(serialize = ":phone_number_id")]
    PhoneNumberID,
    #[strum(serialize = ":pronunciation_dictionary_id")]
    PronunciationDictionaryID,
    #[strum(serialize = ":public_user_id")]
    PublicUserID,
    #[strum(serialize = ":sample_id")]
    SampleID,
    #[strum(serialize = ":tool_id")]
    ToolID,
    #[strum(serialize = ":version_id")]
    VersionID,
    #[strum(serialize = ":voice_id")]
    VoiceID,
}