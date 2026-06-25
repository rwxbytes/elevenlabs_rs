use super::{ElevenLabsEndpoint, RequestBody};
use crate::endpoints::admin::history::{GetGeneratedItems, GetHistoryItem, HistoryQuery};
use crate::endpoints::admin::pronunciation::{
    AddRules, AddRulesBody, GetDictionaries, GetDictionariesQuery, Rule,
};
use crate::endpoints::admin::voice::{
    AddVoice, GetVoice, GetVoices, GetVoicesQuery, ListSimilarVoices, ListSimilarVoicesBody,
    VoiceBody, VoiceType,
};
use crate::endpoints::convai::agents::{
    AgentQuery, ApiSchema, ConvAIModel, ConversationConfig, CreateAgent, CreateAgentBody,
    TTSConfig, WebHook, LLM,
};
use crate::endpoints::convai::conversations::{
    GetConversations, GetConversationsQuery, OutboundCallViaTwilio, OutboundCallViaTwilioBody,
};
use crate::endpoints::convai::knowledge_base::{
    CreateKnowledgeBaseDoc, EmbeddingModel, KnowledgeBaseDoc,
};
use crate::endpoints::convai::phone_numbers::{
    CreatePhoneNumber, CreatePhoneNumberBody, GetPhoneNumber, ListPhoneNumbers,
};
use crate::endpoints::convai::tools::{CreateTool, GetTool};
use crate::endpoints::convai::widget::{CreateWidgetAvatar, CreateWidgetAvatarBody};
use crate::endpoints::genai::audio_isolation::{AudioIsolation, AudioIsolationBody};
use crate::endpoints::genai::dubbing::{DubAVideoOrAnAudioFile, DubbingBody};
use crate::endpoints::genai::forced_alignment::{CreateForcedAlignment, CreateForcedAlignmentBody};
use crate::endpoints::genai::music::{
    ComposeMusic, ComposeMusicDetailed, CompositionPlanBody, GenerateCompositionPlan, MusicComposeBody,
    MusicModel, MusicQuery, SeparateStems, StemSeparationBody, StemVariation, StreamMusic,
    UploadMusic, UploadMusicBody,
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
}

#[tokio::test]
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
