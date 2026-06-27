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
    #[strum(serialize = ":auth_connection_id")]
    AuthConnectionID,
    #[strum(serialize = ":batch_id")]
    BatchID,
    #[strum(serialize = ":chunk_id")]
    ChunkID,
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
    #[strum(serialize = ":project_id")]
    ProjectID,
    #[strum(serialize = ":pronunciation_dictionary_id")]
    PronunciationDictionaryID,
    #[strum(serialize = ":public_user_id")]
    PublicUserID,
    #[strum(serialize = ":resource_id")]
    ResourceID,
    #[strum(serialize = ":resource_type")]
    ResourceType,
    #[strum(serialize = ":sample_id")]
    SampleID,
    #[strum(serialize = ":secret_id")]
    SecretID,
    #[strum(serialize = ":speaker_id")]
    SpeakerID,
    #[strum(serialize = ":speech_engine_id")]
    SpeechEngineID,
    #[strum(serialize = ":generated_voice_id")]
    GeneratedVoiceID,
    #[strum(serialize = ":group_id")]
    GroupID,
    #[strum(serialize = ":webhook_id")]
    WebhookID,
    #[strum(serialize = ":token_type")]
    TokenType,
    #[strum(serialize = ":tool_id")]
    ToolID,
    #[strum(serialize = ":transcription_id")]
    TranscriptionID,
    #[strum(serialize = ":version_id")]
    VersionID,
    #[strum(serialize = ":voice_id")]
    VoiceID,
}
