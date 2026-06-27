use super::{ElevenLabsEndpoint, RequestBody};
use crate::endpoints::admin::audio_native::{
    GetAudioNativeProjectSettings, UpdateAudioNativeContentFromUrl,
    UpdateAudioNativeContentFromUrlBody, UpdateAudioNativeProjectContent,
    UpdateAudioNativeProjectContentBody,
};
use crate::endpoints::admin::auth_connections::{
    CreateAuthConnection, CreateBearerAuth, DeleteAuthConnection, JwtAlgorithm,
    ListAuthConnections, UpdateAuthConnection, UpdateOAuth2Jwt,
};
use crate::endpoints::admin::history::{GetGeneratedItems, GetHistoryItem, HistoryQuery};
use crate::endpoints::admin::pronunciation::{
    AddDictionaryFromRules, AddDictionaryFromRulesBody, AddRules, AddRulesBody, GetDictionaries,
    GetDictionariesQuery, Rule, SetRules, SetRulesBody, UpdateDictionary, UpdateDictionaryBody,
    WorkspaceAccess,
};
use crate::endpoints::admin::pvc_voices::{
    AddPvcVoiceSamples, AddPvcVoiceSamplesBody, CreatePvcVoice, CreatePvcVoiceBody,
    DeletePvcVoiceSample, EditPvcVoice, EditPvcVoiceBody, GetPvcSampleAudio,
    GetPvcSampleAudioQuery, GetPvcSampleWaveform, GetPvcVoiceCaptcha, GetSeparatedSpeakerAudio,
    GetSpeakerSeparationStatus, RequestPvcManualVerification, RequestPvcManualVerificationBody,
    RunPvcTraining, RunPvcTrainingBody, StartSpeakerSeparation, UpdatePvcVoiceSample,
    UpdatePvcVoiceSampleBody, VerifyPvcVoiceCaptcha,
};
use crate::endpoints::admin::voice::{
    AddVoice, GetVoice, GetVoices, GetVoicesQuery, ListSimilarVoices, ListSimilarVoicesBody,
    VoiceBody, VoiceType,
};
use crate::endpoints::admin::workspace::{
    AddMemberToGroup, AuditLogsQuery, CreateWorkspaceWebhook, DeleteWorkspaceWebhook,
    GetWorkspaceAuditLogs, GetWorkspaceGroups, GetWorkspaceWebhooks, InviteUsers, InviteUsersBody,
    RemoveMemberFromGroup, SearchWorkspaceGroups, SeatType, UpdateWorkspaceWebhook,
    UpdateWorkspaceWebhookBody, WebhookHmacSettings,
};
use crate::endpoints::convai::agent_management::{
    CreateAgentBranch, CreateAgentBranchBody, CreateAgentDeployments, CreateAgentDraft,
    DeleteAgentDraft, DuplicateAgent, GetAgentBranch, GetAgentSummaries, GetAgentTopics,
    GetAgentVersion, GetMergePreview, GetRebasePreview, ListAgentBranches, ListAgentBranchesQuery,
    MergeAgentBranch, MergeAgentBranchBody, RebaseAgentBranch, RunAgentTests, SimulateConversation,
    SimulateConversationStream, UpdateAgentBranch, UpdateAgentBranchBody,
};
use crate::endpoints::convai::agent_testing::{
    AgentTestsQuery, BulkMoveTests, BulkMoveTestsBody, CreateAgentTest, CreateAgentTestFolder,
    CreateAgentTestFolderBody, DeleteAgentTest, DeleteAgentTestFolder, GetAgentTest,
    GetAgentTestFolder, GetAgentTestSummaries, ListAgentTests, UpdateAgentTest,
    UpdateAgentTestFolder,
};
use crate::endpoints::convai::agents::GetAgentKnowledgeBaseSize;
use crate::endpoints::convai::agents::{
    AgentQuery, ApiSchema, ConvAIModel, ConversationConfig, CreateAgent, CreateAgentBody,
    TTSConfig, WebHook, LLM,
};
use crate::endpoints::convai::batch_calling::{
    CancelBatchCall, DeleteBatchCall, GetBatchCall, ListWorkspaceBatchCalls,
    ListWorkspaceBatchCallsQuery, OutboundCallRecipient, RetryBatchCall, SubmitBatchCall,
    SubmitBatchCallBody,
};
use crate::endpoints::convai::conversations::{
    ConversationUsersQuery, GetConversationUsers, GetLiveCount,
};
use crate::endpoints::convai::conversations::{
    GetConversations, GetConversationsQuery, GetSignedUrl, GetSignedUrlQuery, GetWebRtcToken,
    OutboundCallViaTwilio, OutboundCallViaTwilioBody, RegisterTwilioCall, RegisterTwilioCallBody,
    TelephonyDirection,
};
use crate::endpoints::convai::environment_variables::{
    CreateEnvironmentVariable, CreateEnvironmentVariableRequest, EnvironmentVariableType,
    EnvironmentVariablesQuery, GetEnvironmentVariable, ListEnvironmentVariables,
    UpdateEnvironmentVariable, UpdateEnvironmentVariableBody,
};
#[allow(deprecated)]
use crate::endpoints::convai::knowledge_base::{
    BulkMoveKnowledgeBase, BulkMoveKnowledgeBaseBody, ComputeRagIndexesBatch, CreateFileDocument,
    CreateKnowledgeBaseDoc, CreateKnowledgeBaseFolder, CreateKnowledgeBaseFolderBody,
    CreateTextDocument, CreateTextDocumentBody, CreateUrlDocument, CreateUrlDocumentBody,
    DeleteRagIndex, DocumentChunksQuery, EmbeddingModel, GetDocumentChunks,
    GetKnowledgeBaseSummaries, GetRagIndexOverview, GetSourceFileUrl, KnowledgeBaseDoc,
    MoveKnowledgeBaseEntity, RagIndexItem, RefreshDocument, SearchKnowledgeBase,
    UpdateFileDocument,
};
use crate::endpoints::convai::llm::{
    AgentLlmUsageBody, CalculateAgentLlmUsage, CalculateLlmUsage, ListLlms, LlmUsageBody,
};
use crate::endpoints::convai::mcp_servers::{
    AddMcpToolApproval, AddMcpToolApprovalBody, CreateMcpServer, CreateMcpToolConfig,
    DeleteMcpServer, DeleteMcpToolApproval, DeleteMcpToolConfig, GetMcpServer, GetMcpToolConfig,
    ListMcpServers, ListMcpTools, McpApprovalPolicy, McpServerConfig, McpServerConfigUpdate,
    McpToolApprovalPolicy, McpToolConfigCreate, McpToolConfigOverrides, UpdateMcpApprovalPolicy,
    UpdateMcpServerConfig, UpdateMcpToolConfig,
};
use crate::endpoints::convai::phone_numbers::{
    CreatePhoneNumber, CreatePhoneNumberBody, GetPhoneNumber, ListPhoneNumbers,
};
use crate::endpoints::convai::phone_numbers::{GetSipMessages, SipMessagesQuery};
use crate::endpoints::convai::tags::{
    ConversationTagsQuery, CreateConversationTag, CreateConversationTagBody, DeleteConversationTag,
    GetConversationTag, ListConversationTags, UpdateConversationTag, UpdateConversationTagBody,
};
use crate::endpoints::convai::telephony::{
    ExotelOutboundCall, OutboundCallBody, SipTrunkOutboundCall, WhatsAppOutboundCall,
    WhatsAppOutboundCallBody, WhatsAppOutboundMessage, WhatsAppOutboundMessageBody,
};
use crate::endpoints::convai::test_invocations::{
    GetTestInvocation, ListTestInvocations, ResubmitTests, ResubmitTestsBody, TestInvocationsQuery,
};
use crate::endpoints::convai::tools::{CreateTool, GetTool};
use crate::endpoints::convai::tools::{
    GetToolDependentAgents, GetToolExecutions, ToolExecutionsQuery,
};
use crate::endpoints::convai::whatsapp_accounts::{
    DeleteWhatsAppAccount, GetWhatsAppAccount, ListWhatsAppAccounts, UpdateWhatsAppAccount,
    UpdateWhatsAppAccountBody,
};
use crate::endpoints::convai::widget::{CreateWidgetAvatar, CreateWidgetAvatarBody};
use crate::endpoints::convai::workspace::{
    DashboardSettings, GetDashboardSettings, GetSecretDependencies, SecretDependencyResourceType,
    UpdateDashboardSettings,
};
use crate::endpoints::genai::audio_isolation::{
    AudioIsolation, AudioIsolationBody, AudioIsolationHistoryQuery,
    DeleteAudioIsolationHistoryItem, GetAudioIsolationHistory,
};
use crate::endpoints::genai::dubbing::{DubAVideoOrAnAudioFile, DubbingBody};
use crate::endpoints::genai::forced_alignment::{CreateForcedAlignment, CreateForcedAlignmentBody};
use crate::endpoints::genai::music::{
    ComposeMusic, ComposeMusicDetailed, CompositionPlanBody, GenerateCompositionPlan,
    MusicComposeBody, MusicModel, MusicQuery, SeparateStems, StemSeparationBody, StemVariation,
    StreamMusic, UploadMusic, UploadMusicBody, VideoToMusic, VideoToMusicBody,
};
use crate::endpoints::genai::speech_engine::{
    CreateSpeechEngine, CreateSpeechEngineBody, DeleteSpeechEngine, GetSpeechEngine,
    ListSpeechEngines, ListSpeechEnginesQuery, SpeechEngineAsrConfig, SpeechEngineConfig,
    SpeechEngineRequestHeaderValue, SpeechEngineTtsConfig, UpdateSpeechEngine,
    UpdateSpeechEngineBody,
};
use crate::endpoints::genai::speech_to_text::{
    AdditionalFormat, CreateTranscript, CreateTranscriptBody, CreateTranscriptQuery,
    DeleteTranscript, GetTranscript, Granularity, SpeechToTextModel,
};
use crate::endpoints::genai::text_to_dialogue::{
    DialogueInput, TextToDialogue, TextToDialogueBody, TextToDialogueQuery, TextToDialogueStream,
    TextToDialogueStreamWithTimestamps, TextToDialogueWithTimestamps,
};
use crate::endpoints::genai::text_to_voice::{
    SaveVoiceFromPreview, SaveVoiceFromPreviewBody, TextToVoice, TextToVoiceBody,
    TextToVoiceDesign, TextToVoiceDesignBody, TextToVoicePreviewStream, TextToVoiceQuery,
    TextToVoiceRemix, TextToVoiceRemixBody,
};
use crate::endpoints::genai::tokens::{CreateSingleUseToken, SingleUseTokenType};
use crate::endpoints::genai::tts::{
    TextToSpeech, TextToSpeechBody, TextToSpeechQuery, TextToSpeechStream,
    TextToSpeechStreamWithTimestamps, TextToSpeechWithTimestamps,
};
use crate::endpoints::genai::voice_changer::{VoiceChanger, VoiceChangerBody};
use crate::{FilePart, Model, OutputFormat};
use reqwest::Method;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn assert_endpoint<E: ElevenLabsEndpoint>(endpoint: &E, method: Method, expected_url: &str) {
    assert_eq!(E::METHOD, method);
    assert_eq!(endpoint.url().as_str(), expected_url);
}

async fn json_body<E: ElevenLabsEndpoint>(endpoint: &E) -> Value {
    match endpoint.request_body().await.unwrap() {
        RequestBody::Json(value) => value,
        other => panic!("expected JSON body, got {other:?}"),
    }
}

async fn assert_multipart_body<E: ElevenLabsEndpoint>(endpoint: &E) {
    match endpoint.request_body().await.unwrap() {
        RequestBody::Multipart(_) => {}
        other => panic!("expected multipart body, got {other:?}"),
    }
}

#[test]
fn open_model_and_format_values_round_trip() {
    assert_eq!(OutputFormat::Mp3_24000Hz48kbps.to_string(), "mp3_24000_48");
    assert_eq!(OutputFormat::Pcm48000Hz.to_string(), "pcm_48000");
    assert_eq!(OutputFormat::ALaw8000Hz.to_string(), "alaw_8000");
    assert_eq!(
        OutputFormat::custom("future_codec").to_string(),
        "future_codec"
    );

    let model_id: String = Model::custom("eleven_future").into();
    assert_eq!(model_id, "eleven_future");

    assert_eq!(
        serde_json::to_value(ConvAIModel::custom("eleven_future")).unwrap(),
        json!("eleven_future")
    );
    assert_eq!(
        serde_json::from_value::<ConvAIModel>(json!("eleven_future")).unwrap(),
        ConvAIModel::custom("eleven_future")
    );
    assert_eq!(
        serde_json::from_value::<ConvAIModel>(json!("eleven_flash_v2")).unwrap(),
        ConvAIModel::ElevenFlashV2
    );

    assert_eq!(
        serde_json::to_value(LLM::other("provider-new-model")).unwrap(),
        json!("provider-new-model")
    );
    assert_eq!(
        serde_json::from_value::<LLM>(json!("provider-new-model")).unwrap(),
        LLM::other("provider-new-model")
    );
    assert_eq!(
        serde_json::from_value::<LLM>(json!("gpt-4o")).unwrap(),
        LLM::Gpt4o
    );

    assert_eq!(
        serde_json::to_value(EmbeddingModel::custom("embedding-next")).unwrap(),
        json!("embedding-next")
    );
    assert_eq!(
        serde_json::from_value::<EmbeddingModel>(json!("embedding-next")).unwrap(),
        EmbeddingModel::custom("embedding-next")
    );
    assert_eq!(
        serde_json::from_value::<EmbeddingModel>(json!("e5_mistral_7b_instruct")).unwrap(),
        EmbeddingModel::E5Mistral7BInstruct
    );
}

#[tokio::test]
async fn tts_endpoints_encode_paths_and_propagate_queries() {
    let query = TextToSpeechQuery::default()
        .with_output_format(OutputFormat::Mp3_24000Hz48kbps)
        .with_logging(false);
    let body = TextToSpeechBody::new("hello").with_model_id(Model::custom("eleven_v3"));

    let endpoint = TextToSpeech::new("voice/id", body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-speech/voice%2Fid?output_format=mp3_24000_48&enable_logging=false",
    );
    assert_eq!(json_body(&endpoint).await["model_id"], "eleven_v3");

    let endpoint = TextToSpeechStream::new("voice/id", body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-speech/voice%2Fid/stream?output_format=mp3_24000_48&enable_logging=false",
    );

    let endpoint =
        TextToSpeechWithTimestamps::new("voice/id", body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-speech/voice%2Fid/with-timestamps?output_format=mp3_24000_48&enable_logging=false",
    );

    let endpoint = TextToSpeechStreamWithTimestamps::new("voice/id", body).with_query(query);
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-speech/voice%2Fid/stream/with-timestamps?output_format=mp3_24000_48&enable_logging=false",
    );
}

#[tokio::test]
async fn text_to_dialogue_endpoints_share_query_and_body_shape() {
    let query = TextToDialogueQuery::default()
        .with_output_format(OutputFormat::ALaw8000Hz)
        .with_logging(false);
    let body = TextToDialogueBody::new(vec![DialogueInput::new("hello", "voice/id")])
        .with_model_id(Model::custom("eleven_v3"))
        .with_language_code("en");

    let endpoint = TextToDialogue::new(body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-dialogue?output_format=alaw_8000&enable_logging=false",
    );
    let body_json = json_body(&endpoint).await;
    assert_eq!(body_json["model_id"], "eleven_v3");
    assert_eq!(body_json["inputs"][0]["voice_id"], "voice/id");

    let endpoint = TextToDialogueStream::new(body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-dialogue/stream?output_format=alaw_8000&enable_logging=false",
    );

    let endpoint = TextToDialogueWithTimestamps::new(body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-dialogue/with-timestamps?output_format=alaw_8000&enable_logging=false",
    );

    let endpoint = TextToDialogueStreamWithTimestamps::new(body).with_query(query);
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-dialogue/stream/with-timestamps?output_format=alaw_8000&enable_logging=false",
    );
}

#[tokio::test]
async fn music_endpoints_encode_paths_and_bodies() {
    let query = MusicQuery::default().with_output_format(OutputFormat::Mp3_44100Hz128kbps);
    let body = MusicComposeBody::from_prompt("an upbeat track").with_music_length_ms(30_000);

    let endpoint = ComposeMusic::new(body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/music?output_format=mp3_44100_128",
    );
    let body_json = json_body(&endpoint).await;
    assert_eq!(body_json["prompt"], "an upbeat track");
    assert_eq!(body_json["model_id"], "music_v1");

    let endpoint = StreamMusic::new(body.clone()).with_query(query.clone());
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/music/stream?output_format=mp3_44100_128",
    );

    let endpoint = ComposeMusicDetailed::new(body).with_timestamps(true);
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/music/detailed",
    );
    assert_eq!(json_body(&endpoint).await["with_timestamps"], true);

    let endpoint = GenerateCompositionPlan::new(
        CompositionPlanBody::new("a three-part synthwave track").with_model(MusicModel::MusicV2),
    );
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/music/plan",
    );
    assert_eq!(json_body(&endpoint).await["model_id"], "music_v2");

    let endpoint = SeparateStems::new(
        StemSeparationBody::from_bytes("song.mp3", "audio/mpeg", b"fake audio".to_vec())
            .with_stem_variation(StemVariation::TwoStems),
    );
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/music/stem-separation",
    );
    assert_multipart_body(&endpoint).await;

    let endpoint = UploadMusic::new(UploadMusicBody::from_bytes(
        "song.mp3",
        "audio/mpeg",
        b"fake audio".to_vec(),
    ));
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/music/upload",
    );
    assert_multipart_body(&endpoint).await;

    let endpoint = VideoToMusic::new(
        VideoToMusicBody::new([FilePart::bytes(
            "clip.mp4",
            "video/mp4",
            b"fake video".to_vec(),
        )])
        .with_description("A tense cinematic score")
        .with_tags(["cinematic", "suspense"]),
    )
    .with_query(MusicQuery::default().with_output_format(OutputFormat::Mp3_44100Hz128kbps));
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/music/video-to-music?output_format=mp3_44100_128",
    );
    assert_multipart_body(&endpoint).await;
}

#[tokio::test]
async fn speech_to_text_exposes_query_and_builds_multipart_body() {
    let file = TempFile::new("mp3", b"fake audio");
    let body = CreateTranscriptBody::new(SpeechToTextModel::ScribeV2, file.path_str())
        .with_language_code("en")
        .with_tag_audio_events(true)
        .with_timestamps_granularity(Granularity::Character)
        .with_additional_formats(vec![AdditionalFormat::new_srt()]);
    let endpoint = CreateTranscript::new(body)
        .with_query(CreateTranscriptQuery::default().enable_logging(false));

    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/speech-to-text?enable_logging=false",
    );
    assert_multipart_body(&endpoint).await;

    let get_transcript = GetTranscript::new("transcript/id");
    assert_endpoint(
        &get_transcript,
        Method::GET,
        "https://api.elevenlabs.io/v1/speech-to-text/transcripts/transcript%2Fid",
    );

    let delete_transcript = DeleteTranscript::new("transcript/id");
    assert_endpoint(
        &delete_transcript,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/speech-to-text/transcripts/transcript%2Fid",
    );

    let bytes_body = CreateTranscriptBody::new(
        SpeechToTextModel::ScribeV2,
        FilePart::bytes("memory.mp3", "audio/mpeg", b"fake audio".to_vec()),
    );
    assert_multipart_body(&CreateTranscript::new(bytes_body)).await;
}

#[tokio::test]
async fn speech_core_gap_endpoints_match_openapi_shape() {
    let token = CreateSingleUseToken::new(SingleUseTokenType::realtime_scribe());
    assert_endpoint(
        &token,
        Method::POST,
        "https://api.elevenlabs.io/v1/single-use-token/realtime_scribe",
    );

    let forced_audio = TempFile::new("mp3", b"fake audio");
    let forced = CreateForcedAlignment::new(CreateForcedAlignmentBody::new(
        forced_audio.path_str(),
        "hello world",
    ));
    assert_endpoint(
        &forced,
        Method::POST,
        "https://api.elevenlabs.io/v1/forced-alignment",
    );
    assert_multipart_body(&forced).await;

    let forced = CreateForcedAlignment::new(CreateForcedAlignmentBody::from_bytes(
        "memory.mp3",
        "audio/mpeg",
        b"fake audio".to_vec(),
        "hello world",
    ));
    assert_multipart_body(&forced).await;
}

#[tokio::test]
async fn speech_engine_endpoints_match_openapi_shape() {
    let speech_engine_config = SpeechEngineConfig::new("wss://example.com/ws")
        .with_request_header("X-Trace-Id", "static-trace")
        .with_request_header(
            "X-Secret",
            SpeechEngineRequestHeaderValue::secret("secret/id"),
        );
    let create_body = CreateSpeechEngineBody::new("wss://ignored.example/ws")
        .with_speech_engine(speech_engine_config)
        .with_name("Support Engine")
        .with_language("en")
        .with_tags(["production", "support"])
        .with_asr(
            SpeechEngineAsrConfig::default()
                .with_provider("scribe_realtime")
                .with_user_input_audio_format("pcm_16000")
                .with_keywords(["ElevenLabs", "Rust"]),
        )
        .with_tts(
            SpeechEngineTtsConfig::default()
                .with_model_id("eleven_flash_v2")
                .with_voice_id("voice/id")
                .with_agent_output_audio_format("pcm_16000"),
        );
    let create = CreateSpeechEngine::new(create_body);
    assert_endpoint(
        &create,
        Method::POST,
        "https://api.elevenlabs.io/v1/speech-engine",
    );
    let body = json_body(&create).await;
    assert_eq!(body["name"], "Support Engine");
    assert_eq!(body["speech_engine"]["ws_url"], "wss://example.com/ws");
    assert_eq!(
        body["speech_engine"]["request_headers"]["X-Trace-Id"],
        "static-trace"
    );
    assert_eq!(
        body["speech_engine"]["request_headers"]["X-Secret"]["secret_id"],
        "secret/id"
    );
    assert_eq!(body["asr"]["provider"], "scribe_realtime");
    assert_eq!(body["tts"]["voice_id"], "voice/id");

    let list = ListSpeechEngines::with_query(
        ListSpeechEnginesQuery::default()
            .with_page_size(10)
            .with_search("support")
            .with_sort_direction("desc")
            .with_sort_by("created_at")
            .with_cursor("cursor/id"),
    );
    assert_endpoint(
        &list,
        Method::GET,
        "https://api.elevenlabs.io/v1/speech-engine?page_size=10&search=support&sort_direction=desc&sort_by=created_at&cursor=cursor%2Fid",
    );

    let get = GetSpeechEngine::new("seng/id");
    assert_endpoint(
        &get,
        Method::GET,
        "https://api.elevenlabs.io/v1/speech-engine/seng%2Fid",
    );

    let update = UpdateSpeechEngine::new(
        "seng/id",
        UpdateSpeechEngineBody::new()
            .with_name("Renamed")
            .with_ws_url("wss://example.com/renamed"),
    );
    assert_endpoint(
        &update,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/speech-engine/seng%2Fid",
    );
    let body = json_body(&update).await;
    assert_eq!(body["name"], "Renamed");
    assert_eq!(body["speech_engine"]["ws_url"], "wss://example.com/renamed");

    let delete = DeleteSpeechEngine::new("seng/id");
    assert_endpoint(
        &delete,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/speech-engine/seng%2Fid",
    );
}

#[tokio::test]
async fn audio_isolation_history_endpoints_encode_paths_and_queries() {
    let history = GetAudioIsolationHistory::default().with_query(
        AudioIsolationHistoryQuery::default()
            .with_page_size(10)
            .with_page(2)
            .with_search("podcast"),
    );
    assert_endpoint(
        &history,
        Method::GET,
        "https://api.elevenlabs.io/v1/audio-isolation/history?page_size=10&page=2&search=podcast",
    );

    let delete = DeleteAudioIsolationHistoryItem::new("item/id");
    assert_endpoint(
        &delete,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/audio-isolation/history/item%2Fid",
    );
}

#[tokio::test]
async fn audio_native_endpoints_encode_paths_and_bodies() {
    let from_url = UpdateAudioNativeContentFromUrl::new(
        UpdateAudioNativeContentFromUrlBody::new("https://example.com/article")
            .with_title("My Article")
            .with_author("Jane Doe"),
    );
    assert_endpoint(
        &from_url,
        Method::POST,
        "https://api.elevenlabs.io/v1/audio-native/content",
    );
    let body_json = json_body(&from_url).await;
    assert_eq!(body_json["url"], "https://example.com/article");
    assert_eq!(body_json["title"], "My Article");
    assert_eq!(body_json["author"], "Jane Doe");

    let project_content = UpdateAudioNativeProjectContent::new(
        "project/id",
        UpdateAudioNativeProjectContentBody::from_bytes(
            "article.html",
            "text/html",
            b"<html></html>".to_vec(),
        )
        .with_auto_convert(true),
    );
    assert_endpoint(
        &project_content,
        Method::POST,
        "https://api.elevenlabs.io/v1/audio-native/project%2Fid/content",
    );
    assert_multipart_body(&project_content).await;

    let settings = GetAudioNativeProjectSettings::new("project/id");
    assert_endpoint(
        &settings,
        Method::GET,
        "https://api.elevenlabs.io/v1/audio-native/project%2Fid/settings",
    );
}

#[tokio::test]
async fn speech_upload_endpoints_accept_in_memory_file_parts() {
    let audio = FilePart::bytes("memory.wav", "audio/wav", b"fake audio".to_vec());

    let isolation = AudioIsolation::new(AudioIsolationBody::new(audio.clone()));
    assert_endpoint(
        &isolation,
        Method::POST,
        "https://api.elevenlabs.io/v1/audio-isolation",
    );
    assert_multipart_body(&isolation).await;

    let voice_changer = VoiceChanger::new(
        "voice/id",
        VoiceChangerBody::new(audio).with_model_id(Model::ElevenMultilingualV2STS),
    );
    assert_endpoint(
        &voice_changer,
        Method::POST,
        "https://api.elevenlabs.io/v1/speech-to-speech/voice%2Fid",
    );
    assert_multipart_body(&voice_changer).await;

    let dubbing = DubAVideoOrAnAudioFile::new(DubbingBody::new("es").with_file_bytes(
        "clip.mp3",
        "audio/mpeg",
        b"fake audio".to_vec(),
    ));
    assert_endpoint(
        &dubbing,
        Method::POST,
        "https://api.elevenlabs.io/v1/dubbing",
    );
    assert_multipart_body(&dubbing).await;
}

#[tokio::test]
async fn admin_endpoint_shapes_cover_voices_history_and_dictionaries() {
    let voices = GetVoices::with_query(
        GetVoicesQuery::default()
            .with_voice_type(VoiceType::Default)
            .with_page_size(2)
            .include_total_count(true),
    );
    assert_endpoint(
        &voices,
        Method::GET,
        "https://api.elevenlabs.io/v2/voices?voice_type=default&page_size=2&include_total_count=true",
    );

    let voice = GetVoice::new("voice/id");
    assert_endpoint(
        &voice,
        Method::GET,
        "https://api.elevenlabs.io/v1/voices/voice%2Fid",
    );

    let add_voice = AddVoice::new(
        VoiceBody::add(
            "Studio",
            vec![FilePart::bytes(
                "sample.mp3",
                "audio/mpeg",
                b"fake audio".to_vec(),
            )],
        )
        .with_remove_background_noise(true),
    );
    assert_endpoint(
        &add_voice,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/add",
    );
    assert_multipart_body(&add_voice).await;

    let similar_voices = ListSimilarVoices::new(
        ListSimilarVoicesBody::from_bytes("sample.mp3", "audio/mpeg", b"fake audio".to_vec())
            .with_top_k(3),
    );
    assert_endpoint(
        &similar_voices,
        Method::POST,
        "https://api.elevenlabs.io/v1/similar-voices",
    );
    assert_multipart_body(&similar_voices).await;

    let history = GetGeneratedItems::with_query(
        HistoryQuery::default()
            .with_page_size(10)
            .with_voice_id("voice/id"),
    );
    assert_endpoint(
        &history,
        Method::GET,
        "https://api.elevenlabs.io/v1/history?page_size=10&voice_id=voice%2Fid",
    );

    let history_item = GetHistoryItem::new("history/id");
    assert_endpoint(
        &history_item,
        Method::GET,
        "https://api.elevenlabs.io/v1/history/history%2Fid",
    );

    let dictionaries = GetDictionaries::with_query(
        GetDictionariesQuery::default()
            .with_page_size(5)
            .with_sort("created_at_unix"),
    );
    assert_endpoint(
        &dictionaries,
        Method::GET,
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries?page_size=5&sort=created_at_unix",
    );

    let add_rules = AddRules::new(
        "dictionary/id",
        AddRulesBody::new(vec![Rule::new_alias("TTS", "text to speech")]),
    );
    assert_endpoint(
        &add_rules,
        Method::POST,
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries/dictionary%2Fid/add-rules",
    );
    assert_eq!(
        json_body(&add_rules).await["rules"][0],
        json!({
            "type": "alias",
            "string_to_replace": "TTS",
            "alias": "text to speech",
        })
    );

    let add_from_rules = AddDictionaryFromRules::new(
        AddDictionaryFromRulesBody::new("acronyms", vec![Rule::new_alias("TTS", "text to speech")])
            .with_description("acronyms")
            .with_workspace_access(WorkspaceAccess::Editor),
    );
    assert_endpoint(
        &add_from_rules,
        Method::POST,
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries/add-from-rules",
    );
    let body = json_body(&add_from_rules).await;
    assert_eq!(body["name"], "acronyms");
    assert_eq!(body["workspace_access"], "editor");

    let set_rules = SetRules::new(
        "dictionary/id",
        SetRulesBody::new(vec![Rule::new_phoneme("Apple", "ˈæpəl", "ipa")]),
    );
    assert_endpoint(
        &set_rules,
        Method::POST,
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries/dictionary%2Fid/set-rules",
    );

    let update = UpdateDictionary::new(
        "dictionary/id",
        UpdateDictionaryBody::default()
            .with_name("Renamed")
            .with_archived(true),
    );
    assert_endpoint(
        &update,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries/dictionary%2Fid",
    );
    let body = json_body(&update).await;
    assert_eq!(body["name"], "Renamed");
    assert_eq!(body["archived"], true);
}

#[tokio::test]
async fn pvc_voice_endpoints_encode_paths_and_bodies() {
    let create = CreatePvcVoice::new(
        CreatePvcVoiceBody::new("My Voice", "en").with_description("A warm narration voice"),
    );
    assert_endpoint(
        &create,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc",
    );
    let body = json_body(&create).await;
    assert_eq!(body["name"], "My Voice");
    assert_eq!(body["language"], "en");

    let edit = EditPvcVoice::new("voice/id", EditPvcVoiceBody::default().with_name("Renamed"));
    assert_endpoint(
        &edit,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid",
    );

    let get_captcha = GetPvcVoiceCaptcha::new("voice/id");
    assert_endpoint(
        &get_captcha,
        Method::GET,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/captcha",
    );

    let verify_captcha = VerifyPvcVoiceCaptcha::new(
        "voice/id",
        FilePart::bytes("rec.mp3", "audio/mpeg", b"fake audio".to_vec()),
    );
    assert_endpoint(
        &verify_captcha,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/captcha",
    );
    assert_multipart_body(&verify_captcha).await;

    let add_samples = AddPvcVoiceSamples::new(
        "voice/id",
        AddPvcVoiceSamplesBody::new([FilePart::bytes(
            "sample.mp3",
            "audio/mpeg",
            b"fake audio".to_vec(),
        )])
        .with_remove_background_noise(true),
    );
    assert_endpoint(
        &add_samples,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples",
    );
    assert_multipart_body(&add_samples).await;

    let update_sample = UpdatePvcVoiceSample::new(
        "voice/id",
        "sample/id",
        UpdatePvcVoiceSampleBody::default()
            .with_selected_speaker_ids(["speaker_0"])
            .with_trim_start_time(0)
            .with_trim_end_time(10),
    );
    assert_endpoint(
        &update_sample,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples/sample%2Fid",
    );
    assert_eq!(
        json_body(&update_sample).await["selected_speaker_ids"][0],
        "speaker_0"
    );

    let delete_sample = DeletePvcVoiceSample::new("voice/id", "sample/id");
    assert_endpoint(
        &delete_sample,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples/sample%2Fid",
    );

    let sample_audio = GetPvcSampleAudio::new("voice/id", "sample/id")
        .with_query(GetPvcSampleAudioQuery::default().with_remove_background_noise(true));
    assert_endpoint(
        &sample_audio,
        Method::GET,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples/sample%2Fid/audio?remove_background_noise=true",
    );

    let start_separation = StartSpeakerSeparation::new("voice/id", "sample/id");
    assert_endpoint(
        &start_separation,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples/sample%2Fid/separate-speakers",
    );

    let separation_status = GetSpeakerSeparationStatus::new("voice/id", "sample/id");
    assert_endpoint(
        &separation_status,
        Method::GET,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples/sample%2Fid/speakers",
    );

    let speaker_audio = GetSeparatedSpeakerAudio::new("voice/id", "sample/id", "speaker/id");
    assert_endpoint(
        &speaker_audio,
        Method::GET,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples/sample%2Fid/speakers/speaker%2Fid/audio",
    );

    let waveform = GetPvcSampleWaveform::new("voice/id", "sample/id");
    assert_endpoint(
        &waveform,
        Method::GET,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/samples/sample%2Fid/waveform",
    );

    let train = RunPvcTraining::new("voice/id")
        .with_body(RunPvcTrainingBody::default().with_model_id("eleven_turbo_v2"));
    assert_endpoint(
        &train,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/train",
    );
    assert_eq!(json_body(&train).await["model_id"], "eleven_turbo_v2");

    let verification = RequestPvcManualVerification::new(
        "voice/id",
        RequestPvcManualVerificationBody::new([FilePart::bytes(
            "id.pdf",
            "application/pdf",
            b"fake doc".to_vec(),
        )])
        .with_extra_text("please verify"),
    );
    assert_endpoint(
        &verification,
        Method::POST,
        "https://api.elevenlabs.io/v1/voices/pvc/voice%2Fid/verification",
    );
    assert_multipart_body(&verification).await;
}

#[tokio::test]
async fn workspace_endpoints_encode_paths_and_bodies() {
    let audit_logs = GetWorkspaceAuditLogs::default().with_query(
        AuditLogsQuery::default()
            .with_limit(50)
            .with_actor_uid("user/1"),
    );
    assert_endpoint(
        &audit_logs,
        Method::GET,
        "https://api.elevenlabs.io/v1/workspace/audit-logs?limit=50&actor_uid=user%2F1",
    );

    assert_endpoint(
        &GetWorkspaceGroups,
        Method::GET,
        "https://api.elevenlabs.io/v1/workspace/groups",
    );

    let search = SearchWorkspaceGroups::new("My Group");
    assert_endpoint(
        &search,
        Method::GET,
        "https://api.elevenlabs.io/v1/workspace/groups/search?name=My+Group",
    );

    let add_member = AddMemberToGroup::new("group/id", "john@example.com");
    assert_endpoint(
        &add_member,
        Method::POST,
        "https://api.elevenlabs.io/v1/workspace/groups/group%2Fid/members",
    );
    assert_eq!(json_body(&add_member).await["email"], "john@example.com");

    let remove_member = RemoveMemberFromGroup::new("group/id", "john@example.com");
    assert_endpoint(
        &remove_member,
        Method::POST,
        "https://api.elevenlabs.io/v1/workspace/groups/group%2Fid/members/remove",
    );

    let invite = InviteUsers::new(
        InviteUsersBody::new(["a@example.com", "b@example.com"])
            .with_seat_type(SeatType::WorkspaceMember)
            .with_group_ids(["group_1"]),
    );
    assert_endpoint(
        &invite,
        Method::POST,
        "https://api.elevenlabs.io/v1/workspace/invites/add-bulk",
    );
    let body = json_body(&invite).await;
    assert_eq!(body["emails"][0], "a@example.com");
    assert_eq!(body["seat_type"], "workspace_member");

    let webhooks = GetWorkspaceWebhooks::default().with_include_usages(true);
    assert_endpoint(
        &webhooks,
        Method::GET,
        "https://api.elevenlabs.io/v1/workspace/webhooks?include_usages=true",
    );

    let create_webhook = CreateWorkspaceWebhook::new(WebhookHmacSettings::new(
        "My Webhook",
        "https://example.com/callback",
    ));
    assert_endpoint(
        &create_webhook,
        Method::POST,
        "https://api.elevenlabs.io/v1/workspace/webhooks",
    );
    let body = json_body(&create_webhook).await;
    assert_eq!(body["settings"]["auth_type"], "hmac");
    assert_eq!(
        body["settings"]["webhook_url"],
        "https://example.com/callback"
    );

    let update_webhook = UpdateWorkspaceWebhook::new(
        "webhook/id",
        UpdateWorkspaceWebhookBody::new("My Webhook", true).with_retry_enabled(true),
    );
    assert_endpoint(
        &update_webhook,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/workspace/webhooks/webhook%2Fid",
    );
    assert_eq!(json_body(&update_webhook).await["is_disabled"], true);

    let delete_webhook = DeleteWorkspaceWebhook::new("webhook/id");
    assert_endpoint(
        &delete_webhook,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/workspace/webhooks/webhook%2Fid",
    );
}

#[tokio::test]
async fn convai_agent_management_endpoints_encode_paths_and_bodies() {
    assert_endpoint(
        &GetAgentSummaries::new(["a_1", "a_2"]),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agents/summaries?agent_ids=a_1&agent_ids=a_2",
    );

    let create_branch = CreateAgentBranch::new(
        "agent/id",
        CreateAgentBranchBody::new("ver_1", "feature", "a feature branch"),
    );
    assert_endpoint(
        &create_branch,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches",
    );
    assert_eq!(
        json_body(&create_branch).await["parent_version_id"],
        "ver_1"
    );

    let list_branches = ListAgentBranches::new("agent/id").with_query(
        ListAgentBranchesQuery::default()
            .with_include_archived(true)
            .with_limit(10),
    );
    assert_endpoint(
        &list_branches,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches?include_archived=true&limit=10",
    );

    assert_endpoint(
        &GetAgentBranch::new("agent/id", "branch/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches/branch%2Fid",
    );

    let update_branch = UpdateAgentBranch::new(
        "agent/id",
        "branch/id",
        UpdateAgentBranchBody::default().with_name("renamed"),
    );
    assert_endpoint(
        &update_branch,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches/branch%2Fid",
    );
    assert_eq!(json_body(&update_branch).await["name"], "renamed");

    assert_endpoint(
        &RebaseAgentBranch::new("agent/id", "branch/id"),
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches/branch%2Fid/rebase",
    );

    assert_endpoint(
        &GetRebasePreview::new("agent/id", "branch/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches/branch%2Fid/rebase-preview",
    );

    let merge = MergeAgentBranch::new(
        "agent/id",
        "src/branch",
        "main",
        MergeAgentBranchBody::default().with_force(true),
    );
    assert_endpoint(
        &merge,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches/src%2Fbranch/merge?target_branch_id=main",
    );
    assert_eq!(json_body(&merge).await["force"], true);

    assert_endpoint(
        &GetMergePreview::new("agent/id", "src/branch", "main").with_force(true),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/branches/src%2Fbranch/merge-preview?target_branch_id=main&force=true",
    );

    let deployments = CreateAgentDeployments::new(
        "agent/id",
        std::collections::HashMap::from([("branch_1".to_string(), json!(100))]),
    );
    assert_endpoint(
        &deployments,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/deployments",
    );
    assert_eq!(
        json_body(&deployments).await["traffic_percentage_branch_id_map"]["branch_1"],
        100
    );

    let draft = CreateAgentDraft::new("agent/id", "branch/id", json!({ "name": "draft" }));
    assert_endpoint(
        &draft,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/drafts?branch_id=branch%2Fid",
    );

    assert_endpoint(
        &DeleteAgentDraft::new("agent/id", "branch/id"),
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/drafts?branch_id=branch%2Fid",
    );

    let duplicate = DuplicateAgent::new("agent/id").with_name("copy");
    assert_endpoint(
        &duplicate,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/duplicate",
    );
    assert_eq!(json_body(&duplicate).await["name"], "copy");

    assert_endpoint(
        &RunAgentTests::new("agent/id", json!({ "tests": [] })),
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/run-tests",
    );

    assert_endpoint(
        &SimulateConversation::new("agent/id", json!({ "simulation_specification": {} })),
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/simulate-conversation",
    );

    assert_endpoint(
        &SimulateConversationStream::new("agent/id", json!({})),
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/simulate-conversation/stream",
    );

    assert_endpoint(
        &GetAgentTopics::new("agent/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/topics",
    );

    assert_endpoint(
        &GetAgentVersion::new("agent/id", "ver/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/versions/ver%2Fid",
    );
}

#[tokio::test]
async fn convai_knowledge_base_endpoints_encode_paths_and_bodies() {
    let text = CreateTextDocument::new(CreateTextDocumentBody::new("hello").with_name("greeting"));
    assert_endpoint(
        &text,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/text",
    );
    assert_eq!(json_body(&text).await["text"], "hello");

    let url = CreateUrlDocument::new(
        CreateUrlDocumentBody::new("https://example.com").with_enable_auto_sync(true),
    );
    assert_endpoint(
        &url,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/url",
    );
    assert_eq!(json_body(&url).await["enable_auto_sync"], true);

    let file = CreateFileDocument::from_bytes("doc.pdf", "application/pdf", b"data".to_vec())
        .with_name("My PDF");
    assert_endpoint(
        &file,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/file",
    );
    assert_multipart_body(&file).await;

    let folder = CreateKnowledgeBaseFolder::new(CreateKnowledgeBaseFolderBody::new("Docs"));
    assert_endpoint(
        &folder,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/folder",
    );

    let bulk = BulkMoveKnowledgeBase::new(
        BulkMoveKnowledgeBaseBody::new(["doc_1", "doc_2"]).with_move_to("folder/id"),
    );
    assert_endpoint(
        &bulk,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/bulk-move",
    );
    assert_eq!(json_body(&bulk).await["document_ids"][0], "doc_1");

    let mv = MoveKnowledgeBaseEntity::new("doc/id").with_move_to("folder/id");
    assert_endpoint(
        &mv,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/doc%2Fid/move",
    );
    assert_eq!(json_body(&mv).await["move_to"], "folder/id");

    assert_endpoint(
        &GetRagIndexOverview,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/rag-index",
    );

    let batch = ComputeRagIndexesBatch::new([RagIndexItem::new(
        "doc/id",
        EmbeddingModel::E5Mistral7BInstruct,
    )]);
    assert_endpoint(
        &batch,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/rag-index",
    );
    assert_eq!(json_body(&batch).await["items"][0]["document_id"], "doc/id");

    assert_endpoint(
        &DeleteRagIndex::new("doc/id", "rag/id"),
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/doc%2Fid/rag-index/rag%2Fid",
    );

    let search = SearchKnowledgeBase::new("query text");
    assert_endpoint(
        &search,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/search?query=query+text",
    );

    let summaries = GetKnowledgeBaseSummaries::new(["doc_1", "doc_2"]);
    assert_endpoint(
        &summaries,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/summaries?document_ids=doc_1&document_ids=doc_2",
    );

    let chunks = GetDocumentChunks::new("doc/id", EmbeddingModel::E5Mistral7BInstruct).with_query(
        DocumentChunksQuery::new(EmbeddingModel::E5Mistral7BInstruct).with_page_size(20),
    );
    assert_endpoint(
        &chunks,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/doc%2Fid/chunks?embedding_model=e5_mistral_7b_instruct&page_size=20",
    );

    assert_endpoint(
        &RefreshDocument::new("doc/id"),
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/doc%2Fid/refresh",
    );

    assert_endpoint(
        &GetSourceFileUrl::new("doc/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/doc%2Fid/source-file-url",
    );

    let update_file =
        UpdateFileDocument::from_bytes("doc/id", "doc.pdf", "application/pdf", b"x".to_vec());
    assert_endpoint(
        &update_file,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/knowledge-base/doc%2Fid/update-file",
    );
    assert_multipart_body(&update_file).await;
}

#[tokio::test]
async fn convai_agent_testing_endpoints_encode_paths_and_bodies() {
    let list = ListAgentTests::default().with_query(
        AgentTestsQuery::default()
            .with_page_size(30)
            .with_include_folders(true),
    );
    assert_endpoint(
        &list,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agent-testing?page_size=30&include_folders=true",
    );

    let create = CreateAgentTest::new(json!({ "type": "llm", "name": "greeting test" }));
    assert_endpoint(
        &create,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agent-testing/create",
    );
    assert_eq!(json_body(&create).await["type"], "llm");

    assert_endpoint(
        &GetAgentTest::new("test/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agent-testing/test%2Fid",
    );

    let update = UpdateAgentTest::new("test/id", json!({ "type": "llm", "name": "renamed" }));
    assert_endpoint(
        &update,
        Method::PUT,
        "https://api.elevenlabs.io/v1/convai/agent-testing/test%2Fid",
    );
    assert_eq!(json_body(&update).await["name"], "renamed");

    assert_endpoint(
        &DeleteAgentTest::new("test/id"),
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/agent-testing/test%2Fid",
    );

    let summaries = GetAgentTestSummaries::new(["test_1", "test_2"]);
    assert_endpoint(
        &summaries,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agent-testing/summaries",
    );
    assert_eq!(json_body(&summaries).await["test_ids"][0], "test_1");

    let bulk = BulkMoveTests::new(BulkMoveTestsBody::new(["test_1"]).with_move_to("folder/id"));
    assert_endpoint(
        &bulk,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agent-testing/bulk-move",
    );
    let body = json_body(&bulk).await;
    assert_eq!(body["entity_ids"][0], "test_1");
    assert_eq!(body["move_to"], "folder/id");

    let create_folder = CreateAgentTestFolder::new(
        CreateAgentTestFolderBody::new("My Folder").with_parent_folder_id("parent/id"),
    );
    assert_endpoint(
        &create_folder,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agent-testing/folders",
    );
    assert_eq!(json_body(&create_folder).await["name"], "My Folder");

    assert_endpoint(
        &GetAgentTestFolder::new("folder/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agent-testing/folders/folder%2Fid",
    );

    let update_folder = UpdateAgentTestFolder::new("folder/id", "Renamed");
    assert_endpoint(
        &update_folder,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/agent-testing/folders/folder%2Fid",
    );
    assert_eq!(json_body(&update_folder).await["name"], "Renamed");

    let delete_folder = DeleteAgentTestFolder::new("folder/id").with_force(true);
    assert_endpoint(
        &delete_folder,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/agent-testing/folders/folder%2Fid?force=true",
    );
}

#[tokio::test]
async fn convai_mcp_server_endpoints_encode_paths_and_bodies() {
    let create = CreateMcpServer::new(
        McpServerConfig::new("My MCP", "https://mcp.example.com/sse")
            .with_description("Internal tools")
            .with_approval_policy(McpApprovalPolicy::RequireApprovalAll),
    );
    assert_endpoint(
        &create,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/mcp-servers",
    );
    let body = json_body(&create).await;
    assert_eq!(body["config"]["name"], "My MCP");
    assert_eq!(body["config"]["approval_policy"], "require_approval_all");

    assert_endpoint(
        &ListMcpServers,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/mcp-servers",
    );

    assert_endpoint(
        &GetMcpServer::new("mcp/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid",
    );

    assert_endpoint(
        &DeleteMcpServer::new("mcp/id"),
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid",
    );

    let update = UpdateMcpServerConfig::new(
        "mcp/id",
        McpServerConfigUpdate::default().with_response_timeout_secs(45),
    );
    assert_endpoint(
        &update,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid",
    );
    assert_eq!(json_body(&update).await["response_timeout_secs"], 45);

    let policy = UpdateMcpApprovalPolicy::new("mcp/id", McpApprovalPolicy::AutoApproveAll);
    assert_endpoint(
        &policy,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/approval-policy",
    );
    assert_eq!(
        json_body(&policy).await["approval_policy"],
        "auto_approve_all"
    );

    let add_approval = AddMcpToolApproval::new(
        "mcp/id",
        AddMcpToolApprovalBody::new(
            "search",
            "Search the web",
            McpToolApprovalPolicy::RequiresApproval,
        ),
    );
    assert_endpoint(
        &add_approval,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/tool-approvals",
    );
    assert_eq!(json_body(&add_approval).await["tool_name"], "search");

    assert_endpoint(
        &DeleteMcpToolApproval::new("mcp/id", "search tool"),
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/tool-approvals/search%20tool",
    );

    let create_cfg = CreateMcpToolConfig::new(
        "mcp/id",
        McpToolConfigCreate::new("search")
            .with_overrides(McpToolConfigOverrides::default().with_response_timeout_secs(30)),
    );
    assert_endpoint(
        &create_cfg,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/tool-configs",
    );
    let body = json_body(&create_cfg).await;
    assert_eq!(body["tool_name"], "search");
    assert_eq!(body["response_timeout_secs"], 30);

    assert_endpoint(
        &GetMcpToolConfig::new("mcp/id", "search"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/tool-configs/search",
    );

    assert_endpoint(
        &UpdateMcpToolConfig::new(
            "mcp/id",
            "search",
            McpToolConfigOverrides::default().with_disable_interruptions(true),
        ),
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/tool-configs/search",
    );

    assert_endpoint(
        &DeleteMcpToolConfig::new("mcp/id", "search"),
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/tool-configs/search",
    );

    assert_endpoint(
        &ListMcpTools::new("mcp/id"),
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/mcp-servers/mcp%2Fid/tools",
    );
}

#[tokio::test]
async fn convai_whatsapp_accounts_and_test_invocation_endpoints_encode_paths_and_bodies() {
    let list_accounts = ListWhatsAppAccounts::default().with_agent_id("agent/id");
    assert_endpoint(
        &list_accounts,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/whatsapp-accounts?agent_id=agent%2Fid",
    );

    let get_account = GetWhatsAppAccount::new("phone/id");
    assert_endpoint(
        &get_account,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/whatsapp-accounts/phone%2Fid",
    );

    let update_account = UpdateWhatsAppAccount::new(
        "phone/id",
        UpdateWhatsAppAccountBody::default()
            .with_assigned_agent_id("agent/id")
            .with_enable_messaging(true),
    );
    assert_endpoint(
        &update_account,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/whatsapp-accounts/phone%2Fid",
    );
    let body = json_body(&update_account).await;
    assert_eq!(body["assigned_agent_id"], "agent/id");
    assert_eq!(body["enable_messaging"], true);

    let delete_account = DeleteWhatsAppAccount::new("phone/id");
    assert_endpoint(
        &delete_account,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/whatsapp-accounts/phone%2Fid",
    );

    let list_inv = ListTestInvocations::default().with_query(
        TestInvocationsQuery::default()
            .with_agent_id("agent/id")
            .with_page_size(10),
    );
    assert_endpoint(
        &list_inv,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/test-invocations?agent_id=agent%2Fid&page_size=10",
    );

    let get_inv = GetTestInvocation::new("inv/id");
    assert_endpoint(
        &get_inv,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/test-invocations/inv%2Fid",
    );

    let resubmit = ResubmitTests::new(
        "inv/id",
        ResubmitTestsBody::new("agent/id", ["run_1", "run_2"]).with_branch_id("branch/id"),
    );
    assert_endpoint(
        &resubmit,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/test-invocations/inv%2Fid/resubmit",
    );
    let body = json_body(&resubmit).await;
    assert_eq!(body["agent_id"], "agent/id");
    assert_eq!(body["test_run_ids"][0], "run_1");
    assert_eq!(body["branch_id"], "branch/id");
}

#[tokio::test]
async fn convai_singleton_endpoints_encode_paths_and_bodies() {
    assert_endpoint(
        &ListLlms,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/llm/list",
    );

    let calc = CalculateLlmUsage::new(LlmUsageBody::new(800, 4, true));
    assert_endpoint(
        &calc,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/llm-usage/calculate",
    );
    let body = json_body(&calc).await;
    assert_eq!(body["prompt_length"], 800);
    assert_eq!(body["rag_enabled"], true);

    let agent_calc = CalculateAgentLlmUsage::new(
        "agent/id",
        AgentLlmUsageBody::default().with_prompt_length(500),
    );
    assert_endpoint(
        &agent_calc,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agent/agent%2Fid/llm-usage/calculate",
    );
    assert_eq!(json_body(&agent_calc).await["prompt_length"], 500);

    let kb_size = GetAgentKnowledgeBaseSize::new("agent/id");
    assert_endpoint(
        &kb_size,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/agent/agent%2Fid/knowledge-base/size",
    );

    let live = GetLiveCount::new().with_agent_id("agent/id");
    assert_endpoint(
        &live,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/analytics/live-count?agent_id=agent%2Fid",
    );

    let users = GetConversationUsers::default().with_query(
        ConversationUsersQuery::default()
            .with_search("acme")
            .with_page_size(20),
    );
    assert_endpoint(
        &users,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/users?search=acme&page_size=20",
    );

    let exotel = ExotelOutboundCall::new(OutboundCallBody::new(
        "agent/id",
        "phone/id",
        "+15550000000",
    ));
    assert_endpoint(
        &exotel,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/exotel/outbound-call",
    );
    assert_eq!(json_body(&exotel).await["to_number"], "+15550000000");

    let sip = SipTrunkOutboundCall::new(OutboundCallBody::new(
        "agent/id",
        "phone/id",
        "+15550000000",
    ));
    assert_endpoint(
        &sip,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/sip-trunk/outbound-call",
    );

    let wa_call = WhatsAppOutboundCall::new(WhatsAppOutboundCallBody::new(
        "agent/id",
        "wa_phone",
        "wa_user",
        "permission_template",
        "en",
    ));
    assert_endpoint(
        &wa_call,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/whatsapp/outbound-call",
    );
    assert_eq!(json_body(&wa_call).await["whatsapp_user_id"], "wa_user");

    let wa_msg = WhatsAppOutboundMessage::new(WhatsAppOutboundMessageBody::new(
        "agent/id",
        "wa_phone",
        "wa_user",
        "template",
        "en",
        [json!("param1")],
    ));
    assert_endpoint(
        &wa_msg,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/whatsapp/outbound-message",
    );
    assert_eq!(json_body(&wa_msg).await["template_params"][0], "param1");
}

#[tokio::test]
async fn convai_tags_and_environment_variable_endpoints_encode_paths_and_bodies() {
    let list_tags = ListConversationTags::default()
        .with_query(ConversationTagsQuery::default().with_page_size(20));
    assert_endpoint(
        &list_tags,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/tags?page_size=20",
    );

    let create_tag = CreateConversationTag::new(
        CreateConversationTagBody::new("VIP").with_description("Important callers"),
    );
    assert_endpoint(
        &create_tag,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/tags",
    );
    assert_eq!(json_body(&create_tag).await["title"], "VIP");

    let get_tag = GetConversationTag::new("tag/id");
    assert_endpoint(
        &get_tag,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/tags/tag%2Fid",
    );

    let update_tag = UpdateConversationTag::new(
        "tag/id",
        UpdateConversationTagBody::default().with_title("VIPs"),
    );
    assert_endpoint(
        &update_tag,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/tags/tag%2Fid",
    );
    assert_eq!(json_body(&update_tag).await["title"], "VIPs");

    let delete_tag = DeleteConversationTag::new("tag/id");
    assert_endpoint(
        &delete_tag,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/tags/tag%2Fid",
    );

    let list_env = ListEnvironmentVariables::default().with_query(
        EnvironmentVariablesQuery::default()
            .with_page_size(10)
            .with_type(EnvironmentVariableType::Secret),
    );
    assert_endpoint(
        &list_env,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/environment-variables?page_size=10&type=secret",
    );

    let create_env = CreateEnvironmentVariable::new(CreateEnvironmentVariableRequest::string(
        "API_BASE_URL",
        std::collections::HashMap::from([(
            "production".to_string(),
            "https://api.example.com".to_string(),
        )]),
    ));
    assert_endpoint(
        &create_env,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/environment-variables",
    );
    let body = json_body(&create_env).await;
    assert_eq!(body["type"], "string");
    assert_eq!(body["label"], "API_BASE_URL");
    assert_eq!(body["values"]["production"], "https://api.example.com");

    let get_env = GetEnvironmentVariable::new("env/id");
    assert_endpoint(
        &get_env,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/environment-variables/env%2Fid",
    );

    let update_env = UpdateEnvironmentVariable::new(
        "env/id",
        UpdateEnvironmentVariableBody::new(std::collections::HashMap::from([(
            "production".to_string(),
            json!("https://new.example.com"),
        )])),
    );
    assert_endpoint(
        &update_env,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/environment-variables/env%2Fid",
    );
    assert_eq!(
        json_body(&update_env).await["values"]["production"],
        "https://new.example.com"
    );
}

#[tokio::test]
async fn convai_quick_win_endpoints_encode_paths_and_bodies() {
    let sip =
        GetSipMessages::new("phone/id").with_query(SipMessagesQuery::default().with_page_size(50));
    assert_endpoint(
        &sip,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/phone-numbers/phone%2Fid/sip-messages?page_size=50",
    );

    let deps = GetSecretDependencies::new("secret/id", SecretDependencyResourceType::Tools);
    assert_endpoint(
        &deps,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/secrets/secret%2Fid/dependencies/tools",
    );

    assert_endpoint(
        &GetDashboardSettings,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/settings/dashboard",
    );

    let update_dashboard =
        UpdateDashboardSettings::new(DashboardSettings::new([json!({ "type": "call_success" })]));
    assert_endpoint(
        &update_dashboard,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/convai/settings/dashboard",
    );
    assert_eq!(
        json_body(&update_dashboard).await["charts"][0]["type"],
        "call_success"
    );

    let dependent = GetToolDependentAgents::new("tool/id");
    assert_endpoint(
        &dependent,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/tools/tool%2Fid/dependent-agents",
    );

    let executions = GetToolExecutions::new("tool/id")
        .with_query(ToolExecutionsQuery::default().with_is_error(true));
    assert_endpoint(
        &executions,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/tools/tool%2Fid/executions?is_error=true",
    );

    let signed = GetSignedUrl::new("agent/id")
        .with_query(GetSignedUrlQuery::new("agent/id").with_include_conversation_id(true));
    assert_endpoint(
        &signed,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/conversation/get-signed-url?agent_id=agent%2Fid&include_conversation_id=true",
    );

    let token = GetWebRtcToken::new("agent/id");
    assert_endpoint(
        &token,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/conversation/token?agent_id=agent%2Fid",
    );

    let register = RegisterTwilioCall::new(
        RegisterTwilioCallBody::new("agent/id", "+15550000000", "+15551111111")
            .with_direction(TelephonyDirection::Inbound),
    );
    assert_endpoint(
        &register,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/twilio/register-call",
    );
    let body = json_body(&register).await;
    assert_eq!(body["agent_id"], "agent/id");
    assert_eq!(body["direction"], "inbound");
}

#[tokio::test]
async fn batch_calling_endpoints_encode_paths_and_bodies() {
    let submit = SubmitBatchCall::new(
        SubmitBatchCallBody::new(
            "Spring campaign",
            "agent/id",
            [OutboundCallRecipient::phone_number("+15551234567")],
        )
        .with_agent_phone_number_id("phone/id")
        .with_target_concurrency_limit(5),
    );
    assert_endpoint(
        &submit,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/batch-calling/submit",
    );
    let body = json_body(&submit).await;
    assert_eq!(body["call_name"], "Spring campaign");
    assert_eq!(body["agent_id"], "agent/id");
    assert_eq!(body["recipients"][0]["phone_number"], "+15551234567");
    assert_eq!(body["target_concurrency_limit"], 5);

    let list = ListWorkspaceBatchCalls::default().with_query(
        ListWorkspaceBatchCallsQuery::default()
            .with_limit(10)
            .with_agent_id("agent/id"),
    );
    assert_endpoint(
        &list,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/batch-calling/workspace?limit=10&agent_id=agent%2Fid",
    );

    let get = GetBatchCall::new("batch/id");
    assert_endpoint(
        &get,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/batch-calling/batch%2Fid",
    );

    let cancel = CancelBatchCall::new("batch/id");
    assert_endpoint(
        &cancel,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/batch-calling/batch%2Fid/cancel",
    );

    let retry = RetryBatchCall::new("batch/id");
    assert_endpoint(
        &retry,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/batch-calling/batch%2Fid/retry",
    );

    let delete = DeleteBatchCall::new("batch/id");
    assert_endpoint(
        &delete,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/convai/batch-calling/batch%2Fid",
    );
}

#[tokio::test]
async fn auth_connection_endpoints_encode_paths_and_bodies() {
    let create = CreateAuthConnection::new(CreateBearerAuth::new("My API", "acme", "token"));
    assert_endpoint(
        &create,
        Method::POST,
        "https://api.elevenlabs.io/v1/workspace/auth-connections",
    );
    let body = json_body(&create).await;
    assert_eq!(body["auth_type"], "bearer_auth");
    assert_eq!(body["provider"], "acme");

    assert_endpoint(
        &ListAuthConnections,
        Method::GET,
        "https://api.elevenlabs.io/v1/workspace/auth-connections",
    );

    let update = UpdateAuthConnection::new(
        "auth/id",
        UpdateOAuth2Jwt::default()
            .with_issuer("issuer")
            .with_algorithm(JwtAlgorithm::Rs256),
    );
    assert_endpoint(
        &update,
        Method::PATCH,
        "https://api.elevenlabs.io/v1/workspace/auth-connections/auth%2Fid",
    );
    let body = json_body(&update).await;
    assert_eq!(body["auth_type"], "oauth2_jwt");
    assert_eq!(body["algorithm"], "RS256");

    let delete = DeleteAuthConnection::new("auth/id");
    assert_endpoint(
        &delete,
        Method::DELETE,
        "https://api.elevenlabs.io/v1/workspace/auth-connections/auth%2Fid",
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn convai_endpoint_shapes_cover_agents_conversations_tools_and_calls() {
    let agent = CreateAgent::new(CreateAgentBody::new(ConversationConfig::default()))
        .with_query(AgentQuery::default().use_tool_ids());
    assert_endpoint(
        &agent,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/create?use_tool_ids=true",
    );
    assert!(json_body(&agent).await.get("conversation_config").is_some());

    let conversations = GetConversations::with_query(
        GetConversationsQuery::default()
            .with_agent_id("agent/id")
            .with_page_size(10),
    );
    assert_endpoint(
        &conversations,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/conversations?agent_id=agent%2Fid&page_size=10",
    );

    let get_tool = GetTool::new("tool/id");
    assert_endpoint(
        &get_tool,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/tools/tool%2Fid",
    );

    let tool = CreateTool::new(WebHook::new(
        "lookup",
        "does a lookup",
        ApiSchema::new("https://example.com/lookup"),
    ));
    assert_endpoint(
        &tool,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/tools",
    );
    assert_eq!(json_body(&tool).await["tool_config"]["name"], "lookup");

    let knowledge_doc = CreateKnowledgeBaseDoc::new(KnowledgeBaseDoc::file_bytes(
        "guide.txt",
        "text/plain",
        b"hello".to_vec(),
    ));
    assert_endpoint(
        &knowledge_doc,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/knowledge-base",
    );
    assert_multipart_body(&knowledge_doc).await;

    let avatar = CreateWidgetAvatar::new(
        "agent/id",
        CreateWidgetAvatarBody::from_bytes("avatar.png", "image/png", b"fake image".to_vec()),
    );
    assert_endpoint(
        &avatar,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/agents/agent%2Fid/avatar",
    );
    assert_multipart_body(&avatar).await;

    let create_phone = CreatePhoneNumber::new(CreatePhoneNumberBody::new_twilio(
        "+15551234567",
        "main",
        "sid",
        "token",
    ));
    assert_endpoint(
        &create_phone,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/phone-numbers",
    );
    assert_eq!(json_body(&create_phone).await["provider"], "twilio");

    let list_phone_numbers = ListPhoneNumbers;
    assert_endpoint(
        &list_phone_numbers,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/phone-numbers",
    );

    let phone_number = GetPhoneNumber::new("phone/id");
    assert_endpoint(
        &phone_number,
        Method::GET,
        "https://api.elevenlabs.io/v1/convai/phone-numbers/phone%2Fid",
    );

    let outbound = OutboundCallViaTwilio::new(OutboundCallViaTwilioBody::new(
        "agent/id",
        "phone/id",
        "+15557654321",
    ));
    assert_endpoint(
        &outbound,
        Method::POST,
        "https://api.elevenlabs.io/v1/convai/twilio/outbound-call",
    );
    assert_eq!(json_body(&outbound).await["agent_id"], "agent/id");
}

#[tokio::test]
async fn text_to_voice_paths_and_queries_match_current_api_shape() {
    let body = TextToVoiceBody::new("warm narrator").with_text("A warm and grounded sample.");
    let endpoint = TextToVoice::new(body)
        .with_query(TextToVoiceQuery::default().with_output_format(OutputFormat::Pcm48000Hz));
    assert_endpoint(
        &endpoint,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-voice/create-previews?output_format=pcm_48000",
    );
    assert_eq!(
        json_body(&endpoint).await["voice_description"],
        "warm narrator"
    );

    let save = SaveVoiceFromPreview::new(SaveVoiceFromPreviewBody::new(
        "Narrator",
        "warm narrator",
        "generated_voice_id",
    ));
    assert_endpoint(
        &save,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-voice",
    );
    assert_eq!(
        json_body(&save).await["generated_voice_id"],
        "generated_voice_id"
    );

    let design_body = TextToVoiceDesignBody::new("warm narrator with a steady studio tone")
        .with_model_id("eleven_ttv_v3")
        .stream_previews(true)
        .should_enhance(true)
        .with_reference_audio_base64("ZmFrZQ==")
        .with_prompt_strength(0.25);
    let design = TextToVoiceDesign::new(design_body).with_query(
        TextToVoiceQuery::default().with_output_format(OutputFormat::Mp3_44100Hz192kbps),
    );
    assert_endpoint(
        &design,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-voice/design?output_format=mp3_44100_192",
    );
    let design_body = json_body(&design).await;
    assert_eq!(design_body["model_id"], "eleven_ttv_v3");
    assert_eq!(design_body["stream_previews"], true);

    let remix_body = TextToVoiceRemixBody::new("make the voice brighter")
        .with_auto_generated_text()
        .with_guidance_scale(2.0)
        .with_prompt_strength(0.5);
    let remix = TextToVoiceRemix::new("voice/id", remix_body)
        .with_query(TextToVoiceQuery::default().with_output_format(OutputFormat::Pcm16000Hz));
    assert_endpoint(
        &remix,
        Method::POST,
        "https://api.elevenlabs.io/v1/text-to-voice/voice%2Fid/remix?output_format=pcm_16000",
    );
    assert_eq!(json_body(&remix).await["auto_generate_text"], true);

    let preview_stream = TextToVoicePreviewStream::new("generated/id");
    assert_endpoint(
        &preview_stream,
        Method::GET,
        "https://api.elevenlabs.io/v1/text-to-voice/generated%2Fid/stream",
    );
}

#[test]
fn convai_tts_config_accepts_future_model_ids() {
    let config = TTSConfig::default().with_model_id("eleven_voice_engine_next");
    assert_eq!(
        serde_json::to_value(config).unwrap()["model_id"],
        "eleven_voice_engine_next"
    );
}

struct TempFile {
    path: PathBuf,
    path_string: String,
}

impl TempFile {
    fn new(extension: &str, bytes: &[u8]) -> Self {
        let path = temp_path(extension);
        std::fs::write(&path, bytes).unwrap();
        let path_string = path.to_string_lossy().into_owned();
        Self { path, path_string }
    }

    fn path_str(&self) -> &str {
        &self.path_string
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn temp_path(extension: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!(
        "elevenlabs_rs_endpoint_test_{}_{}.{}",
        std::process::id(),
        nanos,
        extension
    ));
    assert!(
        !Path::new(&path).exists(),
        "temporary test path unexpectedly exists"
    );
    path
}
