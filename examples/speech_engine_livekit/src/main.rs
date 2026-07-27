//! Native LiveKit workbench for ElevenLabs Speech Engine backed by Rig/Ollama.
//!
//! The example runs three pieces in one binary:
//!
//! - `/ws` is the Speech Engine brain WebSocket. ElevenLabs connects here,
//!   sends transcripts, and receives streamed Rig/Ollama responses.
//! - a LiveKit bridge participant forwards room audio to the Speech Engine
//!   conversation WebSocket and publishes Speech Engine audio back to the room.
//! - an egui desktop participant joins that same room, publishes the system
//!   microphone with CPAL, plays agent audio with CPAL, and shows latency state.

use std::{
    collections::{HashMap, VecDeque},
    env,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        mpsc as std_mpsc, Arc, Mutex, RwLock,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{
        ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, Stream, StreamConfig,
};
use eframe::egui;
use elevenlabs_rs::{
    endpoints::{
        convai::conversations::GetSignedUrl,
        genai::speech_engine::{
            ws::{
                verify_authorization_token, SpeechEngineInboundMessage,
                SpeechEngineOutboundMessage, SpeechEngineTranscriptMessage, AUTHORIZATION_HEADER,
            },
            CreateSpeechEngine, CreateSpeechEngineBody, GetSpeechEngine, SpeechEngineAsrConfig,
            SpeechEngineBackgroundSoundConfig, SpeechEngineCallLimits, SpeechEngineConfig,
            SpeechEngineConversationConfig, SpeechEngineConversationHistoryRedactionConfig,
            SpeechEngineDictionaryLocator, SpeechEngineFileInputConfig, SpeechEngineOverrides,
            SpeechEnginePrivacyConfig, SpeechEngineRequestHeaderValue, SpeechEngineResponse,
            SpeechEngineSuggestedAudioTag, SpeechEngineSupportedVoice, SpeechEngineTtsConfig,
            SpeechEngineTurnConfig, UpdateSpeechEngine, UpdateSpeechEngineBody,
        },
    },
    ElevenLabsClient,
};
use futures_util::{SinkExt, StreamExt};
use livekit::{
    options::TrackPublishOptions,
    prelude::{RemoteAudioTrack, RemoteTrack, Room, RoomEvent, RoomOptions, TrackSource},
    track::{LocalAudioTrack, LocalTrack},
    webrtc::{
        audio_source::native::NativeAudioSource,
        audio_stream::native::NativeAudioStream,
        prelude::{AudioFrame, AudioSourceOptions, RtcAudioSource},
    },
};
use livekit_api::access_token::{AccessToken, VideoGrants};
use rig_core::{
    agent::MultiTurnStreamItem,
    client::{CompletionClient, Nothing, ProviderClient},
    completion::Message,
    providers::ollama,
    streaming::{StreamedAssistantContent, StreamingChat},
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};
use tokio::{runtime::Runtime, sync::mpsc, sync::oneshot, task::JoinHandle};
use tokio_tungstenite::{connect_async, tungstenite::Message as TungsteniteMessage};
use tracing::{debug, error, warn};

const DEFAULT_BIND: &str = "127.0.0.1:3003";
const DEFAULT_ROOM: &str = "elevenlabs-rust-livekit";
const DEFAULT_OLLAMA_MODEL: &str = "llama3.2:latest";
const BRIDGE_IDENTITY: &str = "elevenlabs-speech-engine-bridge";
const USER_IDENTITY_PREFIX: &str = "elevenlabs-rust-desktop";
const USER_INPUT_RATE: u32 = 16_000;
const DEFAULT_AGENT_OUTPUT_RATE: u32 = 24_000;
const MONO: u32 = 1;
const AGENT_AUDIO_QUEUE_MS: u32 = 100;
const MAX_LOG_LINES: usize = 500;
const MAX_TRANSCRIPT_ENTRIES: usize = 400;
const MAX_AUDIO_BUFFER_SAMPLES: usize = 96_000;
const OUTPUT_START_BUFFER_MS: usize = 80;
const DEFAULT_SYSTEM_PROMPT: &str = "You are Luna, a concise voice assistant. Keep answers brief, \
natural, and easy to speak aloud. Ask one short follow-up question only when it helps.";
const REQUIRED_CLIENT_EVENTS: [&str; 6] = [
    "audio",
    "user_transcript",
    "agent_response",
    "agent_response_correction",
    "agent_response_complete",
    "interruption",
];
const SPEECH_ENGINE_OUTPUT_FORMATS: [&str; 7] = [
    "pcm_8000",
    "pcm_16000",
    "pcm_22050",
    "pcm_24000",
    "pcm_44100",
    "pcm_48000",
    "ulaw_8000",
];
const SPEECH_ENGINE_TTS_MODELS: [&str; 6] = [
    "eleven_turbo_v2",
    "eleven_turbo_v2_5",
    "eleven_flash_v2",
    "eleven_flash_v2_5",
    "eleven_multilingual_v2",
    "eleven_v3_conversational",
];
const ALL_CLIENT_EVENTS: [&str; 24] = [
    "conversation_initiation_metadata",
    "asr_initiation_metadata",
    "ping",
    "audio",
    "interruption",
    "user_transcript",
    "tentative_user_transcript",
    "agent_response",
    "agent_response_correction",
    "client_tool_call",
    "mcp_tool_call",
    "mcp_connection_status",
    "agent_tool_request",
    "agent_tool_response",
    "agent_tool_response_full_payload",
    "agent_response_metadata",
    "vad_score",
    "agent_chat_response_part",
    "client_error",
    "guardrail_triggered",
    "dtmf_request",
    "agent_response_complete",
    "internal_turn_probability",
    "internal_tentative_agent_response",
];
const DEFAULT_MONITORING_EVENTS: [&str; 3] = [
    "user_transcript",
    "agent_response",
    "agent_response_correction",
];
const BACKGROUND_SOUND_PRESETS: [&str; 9] = [
    "office2",
    "office1",
    "restaurant",
    "city",
    "typing",
    "elevator1",
    "elevator2",
    "elevator3",
    "elevator4",
];

type SharedOutputBuffer = Arc<Mutex<OutputBuffer>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "speech_engine_livekit=info".to_owned()),
        )
        .init();

    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("failed to build Tokio runtime")?,
    );
    let (ui_tx, ui_rx) = std_mpsc::channel();
    let setup = runtime.block_on(setup(ui_tx.clone()))?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([1040.0, 680.0])
            .with_app_id("elevenlabs-speech-engine-workbench"),
        ..Default::default()
    };
    let app = SpeechEngineWorkbench::new(runtime, setup, ui_tx, ui_rx);

    eframe::run_native(
        "Speech Engine LiveKit Workbench",
        native_options,
        Box::new(move |_cc| Ok(Box::new(app))),
    )
    .map_err(|error| anyhow!("failed to run egui app: {error}"))?;

    Ok(())
}

async fn setup(ui_tx: std_mpsc::Sender<UiEvent>) -> Result<Setup> {
    let telemetry = Telemetry::new(ui_tx);
    let api_key = required_env("ELEVENLABS_API_KEY")?;
    let elevenlabs = ElevenLabsClient::new(api_key.clone());
    let preferred_output_format = preferred_speech_engine_output_format();
    let agent_output_rate = parse_speech_engine_output_rate(&preferred_output_format)
        .unwrap_or(DEFAULT_AGENT_OUTPUT_RATE);
    let speech_engine_id =
        load_or_create_speech_engine(&elevenlabs, &preferred_output_format).await?;
    let selected_model =
        SharedText::new(env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_OLLAMA_MODEL.into()));
    let system_prompt = SharedText::new(
        env::var("RIG_SYSTEM_PROMPT").unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_owned()),
    );
    let playback = PlaybackControl::default();

    let config = Config {
        elevenlabs_api_key: Arc::from(api_key),
        speech_engine_id: SharedText::new(speech_engine_id),
        livekit_url: Arc::from(required_env("LIVEKIT_URL")?),
        livekit_api_key: Arc::from(required_env("LIVEKIT_API_KEY")?),
        livekit_api_secret: Arc::from(required_env("LIVEKIT_API_SECRET")?),
        livekit_room: Arc::from(env::var("LIVEKIT_ROOM").unwrap_or_else(|_| DEFAULT_ROOM.into())),
        selected_model,
        system_prompt,
        agent_output_rate: SharedRate::new(agent_output_rate),
        agent_output_format: SharedText::new(preferred_output_format),
        verify_speech_engine_auth: env_bool("SPEECH_ENGINE_VERIFY_AUTH", false),
    };
    let bind = env::var("LIVEKIT_EXAMPLE_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_owned());
    let bind_addr: SocketAddr = bind
        .parse()
        .with_context(|| format!("invalid LIVEKIT_EXAMPLE_BIND value: {bind}"))?;

    let state = AppState {
        config,
        elevenlabs,
        interruptions: InterruptionState::default(),
        telemetry,
        playback,
    };

    let mut tasks = Vec::new();
    tasks.push(spawn_brain_server(state.clone(), bind_addr).await?);
    tasks.push(tokio::spawn({
        let state = state.clone();
        async move {
            if let Err(error) = run_livekit_bridge(state.clone()).await {
                state
                    .telemetry
                    .error(format!("LiveKit bridge stopped: {error:#}"));
            }
        }
    }));

    Ok(Setup {
        state,
        bind_addr,
        tasks,
    })
}

async fn spawn_brain_server(state: AppState, bind_addr: SocketAddr) -> Result<JoinHandle<()>> {
    let app = Router::new()
        .route("/ws", get(speech_engine_ws))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("failed to bind Speech Engine brain server at {bind_addr}"))?;
    let actual_addr = listener.local_addr()?;
    state.telemetry.log(format!(
        "brain WebSocket listening at ws://{actual_addr}/ws"
    ));

    Ok(tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            state
                .telemetry
                .error(format!("brain WebSocket server stopped: {error:#}"));
        }
    }))
}

#[derive(Clone)]
struct Config {
    elevenlabs_api_key: Arc<str>,
    speech_engine_id: SharedText,
    livekit_url: Arc<str>,
    livekit_api_key: Arc<str>,
    livekit_api_secret: Arc<str>,
    livekit_room: Arc<str>,
    selected_model: SharedText,
    system_prompt: SharedText,
    agent_output_rate: SharedRate,
    agent_output_format: SharedText,
    verify_speech_engine_auth: bool,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    elevenlabs: ElevenLabsClient,
    interruptions: InterruptionState,
    telemetry: Telemetry,
    playback: PlaybackControl,
}

struct Setup {
    state: AppState,
    bind_addr: SocketAddr,
    tasks: Vec<JoinHandle<()>>,
}

#[derive(Clone, Default)]
struct InterruptionState {
    generation: Arc<AtomicU64>,
}

impl InterruptionState {
    fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    fn mark_interrupted(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn should_play_generation(&self, generation: u64) -> bool {
        self.current_generation() == generation
    }
}

#[derive(Clone)]
struct SharedText {
    inner: Arc<RwLock<String>>,
}

impl SharedText {
    fn new(model: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(model)),
        }
    }

    fn get(&self) -> String {
        self.inner
            .read()
            .map(|model| model.clone())
            .unwrap_or_else(|_| DEFAULT_OLLAMA_MODEL.to_owned())
    }

    fn set(&self, model: impl Into<String>) {
        if let Ok(mut current) = self.inner.write() {
            *current = model.into();
        }
    }
}

#[derive(Clone)]
struct SharedRate {
    inner: Arc<AtomicU32>,
}

impl SharedRate {
    fn new(rate: u32) -> Self {
        Self {
            inner: Arc::new(AtomicU32::new(rate)),
        }
    }

    fn get(&self) -> u32 {
        self.inner.load(Ordering::SeqCst)
    }

    fn set(&self, rate: u32) {
        self.inner.store(rate, Ordering::SeqCst);
    }
}

#[derive(Clone)]
struct PlaybackControl {
    buffer: Arc<Mutex<Option<SharedOutputBuffer>>>,
    gain: Arc<Mutex<f32>>,
}

impl Default for PlaybackControl {
    fn default() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(None)),
            gain: Arc::new(Mutex::new(1.0)),
        }
    }
}

impl PlaybackControl {
    fn set_buffer(&self, buffer: SharedOutputBuffer) {
        if let Ok(mut current) = self.buffer.lock() {
            *current = Some(buffer);
        }
    }

    fn clear_buffer(&self) {
        let buffer = self
            .buffer
            .lock()
            .ok()
            .and_then(|current| current.as_ref().cloned());

        if let Some(buffer) = buffer {
            if let Ok(mut output) = buffer.lock() {
                output.samples.clear();
                output.started = false;
            }
        }
    }

    fn gain(&self) -> f32 {
        self.gain.lock().map(|gain| *gain).unwrap_or(1.0)
    }

    fn set_gain(&self, gain: f32) {
        if let Ok(mut current) = self.gain.lock() {
            *current = gain.clamp(0.05, 1.5);
        }
    }
}

#[derive(Clone)]
struct Telemetry {
    tx: std_mpsc::Sender<UiEvent>,
}

impl Telemetry {
    fn new(tx: std_mpsc::Sender<UiEvent>) -> Self {
        Self { tx }
    }

    fn emit(&self, event: UiEvent) {
        let _ = self.tx.send(event);
    }

    fn log(&self, message: impl Into<String>) {
        self.emit(UiEvent::Log {
            level: "info",
            message: message.into(),
        });
    }

    fn warn(&self, message: impl Into<String>) {
        self.emit(UiEvent::Log {
            level: "warn",
            message: message.into(),
        });
    }

    fn error(&self, message: impl Into<String>) {
        self.emit(UiEvent::Log {
            level: "error",
            message: message.into(),
        });
    }

    fn latency(&self, label: impl Into<String>, elapsed: Duration) {
        self.emit(UiEvent::Latency {
            label: label.into(),
            millis: elapsed.as_millis(),
        });
    }
}

struct AgentAudioChunk {
    generation: u64,
    pcm: Vec<u8>,
}

enum SpeechConnectionOutcome {
    Reconnect,
    AudioChannelClosed,
}

#[derive(Clone, Debug)]
enum UiEvent {
    Log {
        level: &'static str,
        message: String,
    },
    ModelsLoaded(Vec<String>),
    ModelsError(String),
    ParticipantConnected(bool),
    UserTranscript {
        event_id: Option<u64>,
        text: String,
    },
    AgentResponse(String),
    Latency {
        label: String,
        millis: u128,
    },
    Interruption {
        generation: u64,
    },
    SpeechEngineLoaded {
        engine: Box<SpeechEngineResponse>,
        operation: &'static str,
    },
}

#[derive(Clone)]
struct LogLine {
    level: &'static str,
    time: String,
    message: String,
}

#[derive(Clone)]
struct LatencyMetric {
    label: String,
    millis: u128,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Speaker {
    User,
    Agent,
    System,
}

#[derive(Clone)]
struct TranscriptEntry {
    speaker: Speaker,
    time: String,
    text: String,
}

#[derive(Default)]
struct OutputBuffer {
    samples: VecDeque<i16>,
    started: bool,
    start_threshold_samples: usize,
}

#[derive(Clone, PartialEq)]
struct SpeechEngineForm {
    ws_url: String,
    request_headers_json: String,
    name: String,
    language: String,
    tags: String,
    asr_provider: String,
    asr_quality: String,
    user_input_audio_format: String,
    keywords: String,
    tts_model_id: String,
    voice_id: String,
    supported_voices_json: String,
    agent_output_audio_format: String,
    expressive_mode: bool,
    suggested_audio_tags_json: String,
    stability: f32,
    speed: f32,
    similarity_boost: f32,
    text_normalisation_type: String,
    pronunciation_dictionary_locators_json: String,
    enable_phoneme_tags: bool,
    audio_filter: String,
    turn_timeout: f32,
    initial_wait_time_enabled: bool,
    initial_wait_time: f32,
    silence_end_call_timeout: f32,
    turn_mode: String,
    turn_eagerness: String,
    spelling_patience: String,
    speculative_turn: bool,
    retranscribe_on_turn_timeout: bool,
    turn_model: String,
    interruption_ignore_terms: String,
    interruption_ignore_term_languages: String,
    transcribe_on_disabled_interruptions: bool,
    text_only: bool,
    max_duration_seconds: u32,
    client_events: Vec<String>,
    file_input_enabled: bool,
    max_files_per_conversation: u32,
    monitoring_enabled: bool,
    monitoring_events: Vec<String>,
    background_sound_source_type: String,
    background_sound_source_id: String,
    background_sound_volume: f32,
    background_sound_crossfade_loop: bool,
    source_attribution: bool,
    record_voice: bool,
    retention_days: i32,
    delete_transcript_and_pii: bool,
    delete_audio: bool,
    apply_to_existing_conversations: bool,
    zero_retention_mode: bool,
    conversation_history_redaction_enabled: bool,
    conversation_history_redaction_entities: String,
    daily_limit: i32,
    agent_concurrency_limit: i32,
    bursting_enabled: bool,
    allow_first_message_override: bool,
}

impl SpeechEngineForm {
    fn from_env(preferred_output_format: impl Into<String>) -> Self {
        let preferred_output_format = preferred_output_format.into();
        Self {
            ws_url: env::var("ELEVENLABS_SPEECH_ENGINE_WS_URL").unwrap_or_default(),
            request_headers_json: "{}".to_owned(),
            name: "Rust LiveKit Rig Speech Engine".to_owned(),
            language: "en".to_owned(),
            tags: "rust,livekit,rig,ollama".to_owned(),
            asr_provider: "scribe_realtime".to_owned(),
            asr_quality: "high".to_owned(),
            user_input_audio_format: format!("pcm_{USER_INPUT_RATE}"),
            keywords: String::new(),
            tts_model_id: "eleven_flash_v2".to_owned(),
            voice_id: String::new(),
            supported_voices_json: "[]".to_owned(),
            agent_output_audio_format: env::var("ELEVENLABS_SPEECH_ENGINE_TTS_OUTPUT_FORMAT")
                .unwrap_or(preferred_output_format),
            expressive_mode: false,
            suggested_audio_tags_json: "[]".to_owned(),
            stability: 0.5,
            speed: 1.0,
            similarity_boost: 0.75,
            text_normalisation_type: "system_prompt".to_owned(),
            pronunciation_dictionary_locators_json: "[]".to_owned(),
            enable_phoneme_tags: true,
            audio_filter: String::new(),
            turn_timeout: 7.0,
            initial_wait_time_enabled: false,
            initial_wait_time: 7.0,
            silence_end_call_timeout: 60.0,
            turn_mode: "turn".to_owned(),
            turn_eagerness: "normal".to_owned(),
            spelling_patience: "auto".to_owned(),
            speculative_turn: false,
            retranscribe_on_turn_timeout: false,
            turn_model: "turn_v3".to_owned(),
            interruption_ignore_terms: String::new(),
            interruption_ignore_term_languages: String::new(),
            transcribe_on_disabled_interruptions: false,
            text_only: false,
            max_duration_seconds: 600,
            client_events: REQUIRED_CLIENT_EVENTS
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            file_input_enabled: true,
            max_files_per_conversation: 10,
            monitoring_enabled: false,
            monitoring_events: DEFAULT_MONITORING_EVENTS
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            background_sound_source_type: String::new(),
            background_sound_source_id: String::new(),
            background_sound_volume: 0.6,
            background_sound_crossfade_loop: false,
            source_attribution: false,
            record_voice: false,
            retention_days: -1,
            delete_transcript_and_pii: false,
            delete_audio: false,
            apply_to_existing_conversations: false,
            zero_retention_mode: false,
            conversation_history_redaction_enabled: false,
            conversation_history_redaction_entities: String::new(),
            daily_limit: 100_000,
            agent_concurrency_limit: -1,
            bursting_enabled: true,
            allow_first_message_override: false,
        }
    }

    fn from_response(engine: &SpeechEngineResponse) -> Self {
        let output_format = engine
            .tts
            .agent_output_audio_format
            .clone()
            .unwrap_or_else(|| format!("pcm_{DEFAULT_AGENT_OUTPUT_RATE}"));
        let mut form = Self::from_env(output_format);
        form.ws_url.clone_from(&engine.speech_engine.ws_url);
        form.request_headers_json = pretty_json(
            &engine
                .speech_engine
                .request_headers
                .clone()
                .unwrap_or_default(),
        );
        form.name.clone_from(&engine.name);
        form.language.clone_from(&engine.language);
        form.tags = engine.tags.join(", ");

        form.asr_provider = engine.asr.provider.clone().unwrap_or_default();
        form.asr_quality = engine.asr.quality.clone().unwrap_or_default();
        form.user_input_audio_format = engine
            .asr
            .user_input_audio_format
            .clone()
            .unwrap_or_else(|| format!("pcm_{USER_INPUT_RATE}"));
        form.keywords = engine.asr.keywords.clone().unwrap_or_default().join(", ");

        form.tts_model_id = engine.tts.model_id.clone().unwrap_or_default();
        form.voice_id = engine.tts.voice_id.clone().unwrap_or_default();
        form.supported_voices_json =
            pretty_json(&engine.tts.supported_voices.clone().unwrap_or_default());
        form.agent_output_audio_format = engine
            .tts
            .agent_output_audio_format
            .clone()
            .unwrap_or_else(|| format!("pcm_{DEFAULT_AGENT_OUTPUT_RATE}"));
        form.expressive_mode = engine.tts.expressive_mode.unwrap_or(false);
        form.suggested_audio_tags_json =
            pretty_json(&engine.tts.suggested_audio_tags.clone().unwrap_or_default());
        form.stability = engine.tts.stability.unwrap_or(0.5);
        form.speed = engine.tts.speed.unwrap_or(1.0);
        form.similarity_boost = engine.tts.similarity_boost.unwrap_or(0.75);
        form.text_normalisation_type = engine
            .tts
            .text_normalisation_type
            .clone()
            .unwrap_or_default();
        form.pronunciation_dictionary_locators_json = pretty_json(
            &engine
                .tts
                .pronunciation_dictionary_locators
                .clone()
                .unwrap_or_default(),
        );
        form.enable_phoneme_tags = engine.tts.enable_phoneme_tags.unwrap_or(false);
        form.audio_filter = engine.tts.audio_filter.clone().unwrap_or_default();

        form.turn_timeout = engine.turn.turn_timeout.unwrap_or(7.0);
        form.initial_wait_time_enabled = engine.turn.initial_wait_time.is_some();
        form.initial_wait_time = engine.turn.initial_wait_time.unwrap_or(form.turn_timeout);
        form.silence_end_call_timeout = engine.turn.silence_end_call_timeout.unwrap_or(-1.0);
        form.turn_mode = engine
            .turn
            .mode
            .clone()
            .unwrap_or_else(|| "turn".to_owned());
        form.turn_eagerness = engine
            .turn
            .turn_eagerness
            .clone()
            .unwrap_or_else(|| "normal".to_owned());
        form.spelling_patience = engine
            .turn
            .spelling_patience
            .clone()
            .unwrap_or_else(|| "auto".to_owned());
        form.speculative_turn = engine.turn.speculative_turn.unwrap_or(false);
        form.retranscribe_on_turn_timeout =
            engine.turn.retranscribe_on_turn_timeout.unwrap_or(false);
        form.turn_model = engine
            .turn
            .turn_model
            .clone()
            .unwrap_or_else(|| "turn_v3".to_owned());
        form.interruption_ignore_terms = engine
            .turn
            .interruption_ignore_terms
            .clone()
            .unwrap_or_default()
            .join(", ");
        form.interruption_ignore_term_languages = engine
            .turn
            .interruption_ignore_term_languages
            .clone()
            .unwrap_or_default()
            .join(", ");
        form.transcribe_on_disabled_interruptions = engine
            .turn
            .transcribe_on_disabled_interruptions
            .unwrap_or(false);

        form.text_only = engine.conversation.text_only.unwrap_or(false);
        form.max_duration_seconds = engine.conversation.max_duration_seconds.unwrap_or(600);
        form.client_events = engine
            .conversation
            .client_events
            .clone()
            .unwrap_or_else(|| {
                REQUIRED_CLIENT_EVENTS
                    .into_iter()
                    .map(ToOwned::to_owned)
                    .collect()
            });
        if let Some(file_input) = &engine.conversation.file_input {
            form.file_input_enabled = file_input.enabled.unwrap_or(false);
            form.max_files_per_conversation = file_input.max_files_per_conversation.unwrap_or(10);
        }
        form.monitoring_enabled = engine.conversation.monitoring_enabled.unwrap_or(false);
        form.monitoring_events = engine
            .conversation
            .monitoring_events
            .clone()
            .unwrap_or_default();
        if let Some(background) = &engine.conversation.background_sound {
            form.background_sound_source_type = background.source_type.clone().unwrap_or_default();
            form.background_sound_source_id = background.source_id.clone().unwrap_or_default();
            form.background_sound_volume = background.volume.unwrap_or(0.6);
            form.background_sound_crossfade_loop = background.crossfade_loop.unwrap_or(false);
        }
        form.source_attribution = engine.conversation.source_attribution.unwrap_or(false);

        form.record_voice = engine.privacy.record_voice.unwrap_or(false);
        form.retention_days = engine.privacy.retention_days.unwrap_or(-1);
        form.delete_transcript_and_pii = engine.privacy.delete_transcript_and_pii.unwrap_or(false);
        form.delete_audio = engine.privacy.delete_audio.unwrap_or(false);
        form.apply_to_existing_conversations = engine
            .privacy
            .apply_to_existing_conversations
            .unwrap_or(false);
        form.zero_retention_mode = engine.privacy.zero_retention_mode.unwrap_or(false);
        if let Some(redaction) = &engine.privacy.conversation_history_redaction {
            form.conversation_history_redaction_enabled = redaction.enabled.unwrap_or(false);
            form.conversation_history_redaction_entities =
                redaction.entities.clone().unwrap_or_default().join(", ");
        }

        form.daily_limit = engine.call_limits.daily_limit.unwrap_or(100_000);
        form.agent_concurrency_limit = engine.call_limits.agent_concurrency_limit.unwrap_or(-1);
        form.bursting_enabled = engine.call_limits.bursting_enabled.unwrap_or(false);
        form.allow_first_message_override = engine.overrides.first_message.unwrap_or(false);
        form
    }

    fn build_body(&self) -> Result<CreateSpeechEngineBody> {
        let ws_url = self.ws_url.trim();
        if ws_url.is_empty() {
            return Err(anyhow!("Speech Engine ws_url is required"));
        }

        let request_headers: HashMap<String, SpeechEngineRequestHeaderValue> =
            parse_json_config("request headers", &self.request_headers_json)?;
        let mut speech_engine = SpeechEngineConfig::new(ws_url.to_owned());
        for (name, value) in request_headers {
            if name.trim().is_empty() {
                return Err(anyhow!("request header names cannot be empty"));
            }
            speech_engine = speech_engine.with_request_header(name, value);
        }

        let mut body = CreateSpeechEngineBody::new(ws_url.to_owned())
            .with_speech_engine(speech_engine)
            .with_name(non_empty_or(&self.name, "Rust LiveKit Rig Speech Engine"))
            .with_asr(self.asr_config())
            .with_tts(self.tts_config()?)
            .with_turn(self.turn_config())
            .with_conversation(self.conversation_config())
            .with_privacy(self.privacy_config())
            .with_call_limits(self.call_limits())
            .with_overrides(
                SpeechEngineOverrides::default().first_message(self.allow_first_message_override),
            );

        let tags = split_csv(&self.tags);
        if !tags.is_empty() {
            body = body.with_tags(tags);
        }

        if !self.language.trim().is_empty() {
            body = body.with_language(self.language.trim().to_owned());
        }

        Ok(body)
    }

    fn build_update_body(&self) -> Result<UpdateSpeechEngineBody> {
        let create = self.build_body()?;
        Ok(UpdateSpeechEngineBody {
            name: create.name,
            speech_engine: Some(create.speech_engine),
            asr: create.asr,
            tts: create.tts,
            turn: create.turn,
            conversation: create.conversation,
            privacy: create.privacy,
            call_limits: create.call_limits,
            language: create.language,
            tags: create.tags,
            overrides: create.overrides,
        })
    }

    fn asr_config(&self) -> SpeechEngineAsrConfig {
        let mut config =
            SpeechEngineAsrConfig::default().with_user_input_audio_format(non_empty_or(
                &self.user_input_audio_format,
                &format!("pcm_{USER_INPUT_RATE}"),
            ));
        if !self.asr_provider.trim().is_empty() {
            config = config.with_provider(self.asr_provider.trim().to_owned());
        }
        if !self.asr_quality.trim().is_empty() {
            config = config.with_quality(self.asr_quality.trim().to_owned());
        }
        let keywords = split_csv(&self.keywords);
        if !keywords.is_empty() {
            config = config.with_keywords(keywords);
        }
        config
    }

    fn tts_config(&self) -> Result<SpeechEngineTtsConfig> {
        let supported_voices: Vec<SpeechEngineSupportedVoice> =
            parse_json_config("supported voices", &self.supported_voices_json)?;
        let suggested_audio_tags: Vec<SpeechEngineSuggestedAudioTag> =
            parse_json_config("suggested audio tags", &self.suggested_audio_tags_json)?;
        let pronunciation_dictionaries: Vec<SpeechEngineDictionaryLocator> = parse_json_config(
            "pronunciation dictionary locators",
            &self.pronunciation_dictionary_locators_json,
        )?;

        let mut config = SpeechEngineTtsConfig::default()
            .with_agent_output_audio_format(non_empty_or(
                &self.agent_output_audio_format,
                &format!("pcm_{DEFAULT_AGENT_OUTPUT_RATE}"),
            ))
            .expressive_mode(self.expressive_mode)
            .with_supported_voices(supported_voices)
            .with_suggested_audio_tags(suggested_audio_tags)
            .with_stability(self.stability)
            .with_speed(self.speed)
            .with_similarity_boost(self.similarity_boost)
            .with_pronunciation_dictionary_locators(pronunciation_dictionaries)
            .enable_phoneme_tags(self.enable_phoneme_tags);
        if !self.tts_model_id.trim().is_empty() {
            config = config.with_model_id(self.tts_model_id.trim().to_owned());
        }
        if !self.voice_id.trim().is_empty() {
            config = config.with_voice_id(self.voice_id.trim().to_owned());
        }
        if !self.text_normalisation_type.trim().is_empty() {
            config =
                config.with_text_normalisation_type(self.text_normalisation_type.trim().to_owned());
        }
        if !self.audio_filter.trim().is_empty() {
            config = config.with_audio_filter(self.audio_filter.trim().to_owned());
        }
        Ok(config)
    }

    fn turn_config(&self) -> SpeechEngineTurnConfig {
        let mut config = SpeechEngineTurnConfig::default()
            .with_turn_timeout(self.turn_timeout)
            .with_silence_end_call_timeout(self.silence_end_call_timeout)
            .with_mode(non_empty_or(&self.turn_mode, "turn"))
            .with_turn_eagerness(non_empty_or(&self.turn_eagerness, "normal"))
            .with_spelling_patience(non_empty_or(&self.spelling_patience, "auto"))
            .speculative_turn(self.speculative_turn)
            .retranscribe_on_turn_timeout(self.retranscribe_on_turn_timeout)
            .with_turn_model(non_empty_or(&self.turn_model, "turn_v3"))
            .with_interruption_ignore_terms(split_csv(&self.interruption_ignore_terms))
            .with_interruption_ignore_term_languages(split_csv(
                &self.interruption_ignore_term_languages,
            ))
            .transcribe_on_disabled_interruptions(self.transcribe_on_disabled_interruptions);
        if self.initial_wait_time_enabled {
            config = config.with_initial_wait_time(self.initial_wait_time);
        }
        if !self.turn_eagerness.trim().is_empty() {
            config = config.with_turn_eagerness(self.turn_eagerness.trim().to_owned());
        }
        config
    }

    fn conversation_config(&self) -> SpeechEngineConversationConfig {
        let mut client_events = self.client_events.clone();
        for required in REQUIRED_CLIENT_EVENTS {
            if !client_events.iter().any(|event| event == required) {
                client_events.push(required.to_owned());
            }
        }

        let mut background = SpeechEngineBackgroundSoundConfig::default()
            .with_volume(self.background_sound_volume)
            .crossfade_loop(self.background_sound_crossfade_loop);
        if !self.background_sound_source_type.trim().is_empty() {
            background =
                background.with_source_type(self.background_sound_source_type.trim().to_owned());
        }
        if !self.background_sound_source_id.trim().is_empty() {
            background =
                background.with_source_id(self.background_sound_source_id.trim().to_owned());
        }

        SpeechEngineConversationConfig::default()
            .text_only(self.text_only)
            .with_max_duration_seconds(self.max_duration_seconds)
            .with_client_events(client_events)
            .with_file_input(
                SpeechEngineFileInputConfig::default()
                    .enabled(self.file_input_enabled)
                    .with_max_files_per_conversation(self.max_files_per_conversation),
            )
            .monitoring_enabled(self.monitoring_enabled)
            .with_monitoring_events(self.monitoring_events.clone())
            .with_background_sound(background)
            .source_attribution(self.source_attribution)
    }

    fn privacy_config(&self) -> SpeechEnginePrivacyConfig {
        SpeechEnginePrivacyConfig::default()
            .record_voice(self.record_voice)
            .with_retention_days(self.retention_days)
            .delete_transcript_and_pii(self.delete_transcript_and_pii)
            .delete_audio(self.delete_audio)
            .apply_to_existing_conversations(self.apply_to_existing_conversations)
            .zero_retention_mode(self.zero_retention_mode)
            .with_conversation_history_redaction(
                SpeechEngineConversationHistoryRedactionConfig::default()
                    .enabled(self.conversation_history_redaction_enabled)
                    .with_entities(split_csv(&self.conversation_history_redaction_entities)),
            )
    }

    fn call_limits(&self) -> SpeechEngineCallLimits {
        SpeechEngineCallLimits::default()
            .with_daily_limit(self.daily_limit)
            .with_agent_concurrency_limit(self.agent_concurrency_limit)
            .bursting_enabled(self.bursting_enabled)
    }
}

fn parse_json_config<T>(label: &str, json: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(json).with_context(|| format!("invalid {label} JSON"))
}

fn pretty_json<T>(value: &T) -> String
where
    T: serde::Serialize,
{
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".to_owned())
}

struct ParticipantTask {
    stop_tx: Option<oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkbenchTab {
    Conversation,
    SpeechEngine,
    Events,
}

/// Sections of the Speech Engine configuration, shown one at a time so the
/// form never turns into one endless scroll.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EngineSection {
    General,
    Asr,
    Tts,
    Turn,
    Conversation,
    ClientEvents,
    Privacy,
    Limits,
    Advanced,
}

impl EngineSection {
    const ALL: [Self; 9] = [
        Self::General,
        Self::Asr,
        Self::Tts,
        Self::Turn,
        Self::Conversation,
        Self::ClientEvents,
        Self::Privacy,
        Self::Limits,
        Self::Advanced,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Asr => "Speech to text",
            Self::Tts => "Voice",
            Self::Turn => "Turn taking",
            Self::Conversation => "Conversation",
            Self::ClientEvents => "Client events",
            Self::Privacy => "Privacy",
            Self::Limits => "Limits",
            Self::Advanced => "Advanced JSON",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LogFilter {
    All,
    Info,
    Warn,
    Error,
}

impl LogFilter {
    fn accepts(self, level: &str) -> bool {
        match self {
            Self::All => true,
            Self::Info => level == "info",
            Self::Warn => level == "warn",
            Self::Error => level == "error",
        }
    }
}

struct SpeechEngineWorkbench {
    runtime: Arc<Runtime>,
    state: AppState,
    bind_addr: SocketAddr,
    background_tasks: Vec<JoinHandle<()>>,
    ui_tx: std_mpsc::Sender<UiEvent>,
    ui_rx: std_mpsc::Receiver<UiEvent>,
    logs: VecDeque<LogLine>,
    latencies: VecDeque<LatencyMetric>,
    transcript: VecDeque<TranscriptEntry>,
    models: Vec<String>,
    model_input: String,
    system_prompt_input: String,
    speech_engine_form: SpeechEngineForm,
    loaded_speech_engine_form: SpeechEngineForm,
    active_tab: WorkbenchTab,
    engine_section: EngineSection,
    log_filter: LogFilter,
    log_query: String,
    dark_mode: bool,
    applied_theme: Option<bool>,
    output_gain: f32,
    participant: Option<ParticipantTask>,
    participant_connected: bool,
    participant_connecting: bool,
    interruptions: u64,
}

impl SpeechEngineWorkbench {
    fn new(
        runtime: Arc<Runtime>,
        setup: Setup,
        ui_tx: std_mpsc::Sender<UiEvent>,
        ui_rx: std_mpsc::Receiver<UiEvent>,
    ) -> Self {
        let model_input = setup.state.config.selected_model.get();
        let system_prompt_input = setup.state.config.system_prompt.get();
        let output_gain = setup.state.playback.gain();
        let speech_engine_form =
            SpeechEngineForm::from_env(setup.state.config.agent_output_format.get());
        let app = Self {
            runtime,
            state: setup.state,
            bind_addr: setup.bind_addr,
            background_tasks: setup.tasks,
            ui_tx,
            ui_rx,
            logs: VecDeque::new(),
            latencies: VecDeque::new(),
            transcript: VecDeque::new(),
            models: Vec::new(),
            model_input,
            system_prompt_input,
            loaded_speech_engine_form: speech_engine_form.clone(),
            speech_engine_form,
            active_tab: WorkbenchTab::Conversation,
            engine_section: EngineSection::General,
            log_filter: LogFilter::All,
            log_query: String::new(),
            dark_mode: true,
            applied_theme: None,
            output_gain,
            participant: None,
            participant_connected: false,
            participant_connecting: false,
            interruptions: 0,
        };
        app.refresh_models();
        app.load_selected_speech_engine();
        app
    }

    fn refresh_models(&self) {
        let tx = self.ui_tx.clone();
        self.runtime.spawn(async move {
            match fetch_ollama_models().await {
                Ok(models) => {
                    let _ = tx.send(UiEvent::ModelsLoaded(models));
                }
                Err(error) => {
                    let _ = tx.send(UiEvent::ModelsError(format!("{error:#}")));
                }
            }
        });
    }

    fn load_selected_speech_engine(&self) {
        let client = self.state.elevenlabs.clone();
        let speech_engine_id = self.state.config.speech_engine_id.get();
        let telemetry = self.state.telemetry.clone();
        self.runtime.spawn(async move {
            telemetry.log(format!(
                "loading Speech Engine configuration for {speech_engine_id}"
            ));
            match client.hit(GetSpeechEngine::new(speech_engine_id)).await {
                Ok(engine) => telemetry.emit(UiEvent::SpeechEngineLoaded {
                    engine: Box::new(engine),
                    operation: "loaded",
                }),
                Err(error) => telemetry.error(format!(
                    "failed to load Speech Engine configuration: {error:#}"
                )),
            }
        });
    }

    fn connect_participant(&mut self) {
        if self.participant.is_some() || self.participant_connecting {
            return;
        }

        self.participant_connecting = true;
        self.push_log("info", "connecting native LiveKit participant");

        let (stop_tx, stop_rx) = oneshot::channel();
        let state = self.state.clone();
        let thread = std::thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    state.telemetry.error(format!(
                        "failed to build desktop participant runtime: {error:#}"
                    ));
                    state.telemetry.emit(UiEvent::ParticipantConnected(false));
                    return;
                }
            };

            let result = runtime.block_on(run_desktop_participant(state.clone(), stop_rx));
            if let Err(error) = result {
                state
                    .telemetry
                    .error(format!("desktop participant stopped: {error:#}"));
            }
            state.telemetry.emit(UiEvent::ParticipantConnected(false));
        });

        self.participant = Some(ParticipantTask {
            stop_tx: Some(stop_tx),
            thread: Some(thread),
        });
    }

    fn disconnect_participant(&mut self) {
        if let Some(mut participant) = self.participant.take() {
            if let Some(stop_tx) = participant.stop_tx.take() {
                let _ = stop_tx.send(());
            }
            if let Some(thread) = participant.thread.take() {
                std::thread::spawn(move || {
                    let _ = thread.join();
                });
            }
        }
        self.participant_connected = false;
        self.participant_connecting = false;
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.ui_rx.try_recv() {
            match event {
                UiEvent::Log { level, message } => self.push_log(level, message),
                UiEvent::ModelsLoaded(models) => {
                    self.models = models;
                    self.push_log(
                        "info",
                        format!("loaded {} Ollama model(s)", self.models.len()),
                    );
                }
                UiEvent::ModelsError(error) => {
                    self.push_log("warn", format!("failed to load Ollama models: {error}"));
                }
                UiEvent::ParticipantConnected(connected) => {
                    self.participant_connected = connected;
                    self.participant_connecting = false;
                    if !connected {
                        self.participant = None;
                    }
                }
                UiEvent::UserTranscript { event_id, text } => {
                    self.push_transcript(Speaker::User, text.clone());
                    self.push_log("info", format!("user event_id={event_id:?}: {text}"));
                }
                UiEvent::AgentResponse(text) => {
                    self.push_transcript(Speaker::Agent, text.clone());
                    self.push_log("info", format!("agent: {text}"));
                }
                UiEvent::Latency { label, millis } => {
                    self.latencies.push_front(LatencyMetric { label, millis });
                    while self.latencies.len() > 24 {
                        self.latencies.pop_back();
                    }
                }
                UiEvent::Interruption { generation } => {
                    self.interruptions = generation;
                    self.push_transcript(Speaker::System, "user interrupted the agent".to_owned());
                    self.push_log(
                        "info",
                        format!("interruption cleared agent audio generation {generation}"),
                    );
                }
                UiEvent::SpeechEngineLoaded { engine, operation } => {
                    self.apply_speech_engine_response(*engine, operation);
                }
            }
        }
    }

    fn apply_speech_engine_response(
        &mut self,
        engine: SpeechEngineResponse,
        operation: &'static str,
    ) {
        let previous_rate = self.state.config.agent_output_rate.get();
        if let Some(output_format) = engine.tts.agent_output_audio_format.as_deref() {
            if let Some(output_rate) = parse_speech_engine_output_rate(output_format) {
                self.state.config.agent_output_rate.set(output_rate);
                self.state
                    .config
                    .agent_output_format
                    .set(output_format.to_owned());
                if output_rate != previous_rate {
                    self.push_log(
                        "warn",
                        format!(
                            "Speech Engine output is {output_rate} Hz but the active LiveKit bridge was created at {previous_rate} Hz; restart the workbench before connecting"
                        ),
                    );
                }
            }
        }

        self.state
            .config
            .speech_engine_id
            .set(engine.speech_engine_id.clone());
        self.speech_engine_form = SpeechEngineForm::from_response(&engine);
        self.loaded_speech_engine_form = self.speech_engine_form.clone();
        self.push_log(
            "info",
            format!(
                "{operation} Speech Engine configuration {}",
                engine.speech_engine_id
            ),
        );
    }

    fn push_log(&mut self, level: &'static str, message: impl Into<String>) {
        self.logs.push_back(LogLine {
            level,
            time: clock_time(),
            message: message.into(),
        });
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }

    fn push_transcript(&mut self, speaker: Speaker, text: String) {
        self.transcript.push_back(TranscriptEntry {
            speaker,
            time: clock_time(),
            text,
        });
        while self.transcript.len() > MAX_TRANSCRIPT_ENTRIES {
            self.transcript.pop_front();
        }
    }

    fn apply_model(&mut self) {
        let model = self.model_input.trim();
        if model.is_empty() {
            self.push_log("warn", "model cannot be empty");
            return;
        }
        if !self.models.is_empty() && !self.models.iter().any(|known| known == model) {
            self.push_log(
                "warn",
                format!("Ollama model `{model}` is not in the local model list"),
            );
            return;
        }
        self.state.config.selected_model.set(model.to_owned());
        self.push_log("info", format!("selected Ollama model `{model}`"));
    }

    fn apply_system_prompt(&mut self) {
        let prompt = self.system_prompt_input.trim();
        if prompt.is_empty() {
            self.push_log("warn", "system prompt cannot be empty");
            return;
        }
        self.state.config.system_prompt.set(prompt.to_owned());
        self.push_log("info", "updated Rig agent system prompt");
    }

    fn create_speech_engine_from_form(&mut self) {
        if !self.validate_speech_engine_audio_config() {
            return;
        }
        let body = match self.speech_engine_form.build_body() {
            Ok(body) => body,
            Err(error) => {
                self.push_log("error", format!("{error:#}"));
                return;
            }
        };

        let client = self.state.elevenlabs.clone();
        let telemetry = self.state.telemetry.clone();
        self.runtime.spawn(async move {
            telemetry.log("creating Speech Engine from UI configuration");
            match client.hit(CreateSpeechEngine::new(body)).await {
                Ok(engine) => telemetry.emit(UiEvent::SpeechEngineLoaded {
                    engine: Box::new(engine),
                    operation: "created",
                }),
                Err(error) => telemetry.error(format!("failed to create Speech Engine: {error:#}")),
            }
        });
    }

    fn update_speech_engine_from_form(&mut self) {
        if !self.validate_speech_engine_audio_config() {
            return;
        }
        let body = match self.speech_engine_form.build_update_body() {
            Ok(body) => body,
            Err(error) => {
                self.push_log("error", format!("{error:#}"));
                return;
            }
        };

        let client = self.state.elevenlabs.clone();
        let speech_engine_id = self.state.config.speech_engine_id.get();
        let telemetry = self.state.telemetry.clone();
        self.runtime.spawn(async move {
            telemetry.log(format!(
                "updating Speech Engine configuration {speech_engine_id}"
            ));
            match client
                .hit(UpdateSpeechEngine::new(speech_engine_id, body))
                .await
            {
                Ok(engine) => telemetry.emit(UiEvent::SpeechEngineLoaded {
                    engine: Box::new(engine),
                    operation: "updated",
                }),
                Err(error) => telemetry.error(format!("failed to update Speech Engine: {error:#}")),
            }
        });
    }

    fn validate_speech_engine_audio_config(&mut self) -> bool {
        let output_format = self.speech_engine_form.agent_output_audio_format.trim();
        let Some(output_rate) = parse_speech_engine_output_rate(output_format) else {
            self.push_log(
                "error",
                format!(
                    "unsupported workbench output format `{output_format}`; use pcm_8000, pcm_16000, pcm_22050, pcm_24000, pcm_44100, pcm_48000, or ulaw_8000"
                ),
            );
            return false;
        };

        let active_rate = self.state.config.agent_output_rate.get();
        if output_rate != active_rate {
            self.push_log(
                "warn",
                format!(
                    "new engine output rate is {output_rate} Hz but the active LiveKit bridge was created at {active_rate} Hz; restart this example after selecting the engine"
                ),
            );
        }

        let input_format = self.speech_engine_form.user_input_audio_format.trim();
        if input_format != format!("pcm_{USER_INPUT_RATE}") {
            self.push_log(
                "warn",
                format!(
                    "Speech Engine input format is `{input_format}` but the LiveKit bridge publishes pcm_{USER_INPUT_RATE}"
                ),
            );
        }
        true
    }

    // ------------------------------------------------------------------
    // Derived UI state
    // ------------------------------------------------------------------

    fn palette(&self) -> Palette {
        if self.dark_mode {
            Palette::DARK
        } else {
            Palette::LIGHT
        }
    }

    fn participant_status(&self, palette: &Palette) -> (egui::Color32, &'static str) {
        if self.participant_connected {
            (palette.success, "connected")
        } else if self.participant_connecting {
            (palette.warn, "connecting")
        } else {
            (palette.text_faint, "disconnected")
        }
    }

    /// Most recent measurement whose label contains `needle`.
    fn latest_latency(&self, needle: &str) -> Option<u128> {
        self.latencies
            .iter()
            .find(|metric| metric.label.contains(needle))
            .map(|metric| metric.millis)
    }

    fn engine_form_is_dirty(&self) -> bool {
        self.speech_engine_form != self.loaded_speech_engine_form
    }

    /// `(info, warn, error)` counts across the retained log window.
    fn log_counts(&self) -> (usize, usize, usize) {
        let mut counts = (0, 0, 0);
        for line in &self.logs {
            match line.level {
                "warn" => counts.1 += 1,
                "error" => counts.2 += 1,
                _ => counts.0 += 1,
            }
        }
        counts
    }

    // ------------------------------------------------------------------
    // Chrome
    // ------------------------------------------------------------------

    fn show_top_bar(&mut self, ctx: &egui::Context, palette: &Palette) {
        let (dot, status) = self.participant_status(palette);
        egui::TopBottomPanel::top("workbench_top")
            .exact_height(58.0)
            .frame(
                egui::Frame::NONE
                    .fill(palette.surface)
                    .inner_margin(egui::Margin::symmetric(16, 0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    brand_mark(ui, palette);
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Speech Engine Workbench")
                                .size(15.0)
                                .strong()
                                .color(palette.text),
                        );
                        ui.label(
                            egui::RichText::new("ElevenLabs · LiveKit · Rig · Ollama")
                                .size(10.5)
                                .color(palette.text_faint),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.participant_connected || self.participant_connecting {
                            if ghost_button(ui, palette, "Disconnect", 112.0, true).clicked() {
                                self.disconnect_participant();
                            }
                        } else if primary_button(ui, palette, "Connect", 112.0, true).clicked() {
                            self.connect_participant();
                        }

                        ui.add_space(2.0);
                        let icon = if self.dark_mode { "☀" } else { "☾" };
                        if ui
                            .add(egui::Button::new(icon).min_size(egui::vec2(30.0, 28.0)))
                            .on_hover_text("Toggle light and dark theme")
                            .clicked()
                        {
                            self.dark_mode = !self.dark_mode;
                        }

                        ui.add_space(6.0);
                        status_chip(ui, palette, dot, status);
                    });
                });
            });
    }

    fn show_status_bar(&self, ctx: &egui::Context, palette: &Palette) {
        let (_, status) = self.participant_status(palette);
        let (_, warnings, errors) = self.log_counts();
        egui::TopBottomPanel::bottom("workbench_status")
            .exact_height(26.0)
            .frame(
                egui::Frame::NONE
                    .fill(palette.surface)
                    .inner_margin(egui::Margin::symmetric(14, 0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    status_text(ui, palette.text_dim, format!("ws://{}/ws", self.bind_addr));
                    status_text(ui, palette.border_strong, "│".to_owned());
                    status_text(
                        ui,
                        palette.text_dim,
                        format!("room {}", self.state.config.livekit_room),
                    );
                    status_text(ui, palette.border_strong, "│".to_owned());
                    status_text(ui, palette.text_dim, format!("participant {status}"));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        status_text(
                            ui,
                            palette.text_faint,
                            format!("{} events", self.logs.len()),
                        );
                        if errors > 0 {
                            status_text(ui, palette.error, format!("{errors} errors"));
                        }
                        if warnings > 0 {
                            status_text(ui, palette.warn, format!("{warnings} warnings"));
                        }
                    });
                });
            });
    }

    fn show_side_panel(&mut self, ctx: &egui::Context, palette: &Palette) {
        egui::SidePanel::left("workbench_controls")
            .resizable(true)
            .default_width(330.0)
            .width_range(300.0..=440.0)
            .frame(
                egui::Frame::NONE
                    .fill(palette.sidebar)
                    .inner_margin(egui::Margin::symmetric(12, 12)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("sidebar_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.show_session_card(ui, palette);
                        ui.add_space(10.0);
                        self.show_audio_card(ui, palette);
                        ui.add_space(10.0);
                        self.show_brain_card(ui, palette);
                        ui.add_space(4.0);
                    });
            });
    }

    fn show_session_card(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let (dot, status) = self.participant_status(palette);
        let local_ws = format!("ws://{}/ws", self.bind_addr);
        let livekit_url = self.state.config.livekit_url.to_string();
        let room = self.state.config.livekit_room.to_string();
        let engine_id = self.state.config.speech_engine_id.get();

        card(ui, palette, "Session", |ui| {
            ui.horizontal(|ui| {
                status_dot(ui, dot);
                ui.label(
                    egui::RichText::new(status)
                        .size(12.5)
                        .strong()
                        .color(palette.text),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("desktop participant")
                            .size(10.5)
                            .color(palette.text_faint),
                    );
                });
            });

            ui.add_space(10.0);
            info_row(ui, palette, "ws", &local_ws);
            info_row(ui, palette, "LiveKit", &livekit_url);
            info_row(ui, palette, "Room", &room);
            info_row(ui, palette, "Engine", &engine_id);

            ui.add_space(12.0);
            let spacing = ui.spacing().item_spacing.x;
            let button_width = ((ui.available_width() - spacing) / 2.0).max(80.0);
            ui.horizontal(|ui| {
                let idle = !self.participant_connected && !self.participant_connecting;
                if primary_button(ui, palette, "Connect", button_width, idle).clicked() {
                    self.connect_participant();
                }
                let live = self.participant_connected || self.participant_connecting;
                if ghost_button(ui, palette, "Disconnect", button_width, live).clicked() {
                    self.disconnect_participant();
                }
            });
        });
    }

    fn show_audio_card(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let output_format = self.state.config.agent_output_format.get();
        let output_rate = self.state.config.agent_output_rate.get();
        let connected = self.participant_connected;

        card(ui, palette, "Audio", |ui| {
            ui.horizontal_wrapped(|ui| {
                pill(ui, output_format, palette.text_dim, palette.surface_alt);
                pill(
                    ui,
                    format!("{output_rate} Hz"),
                    palette.text_dim,
                    palette.surface_alt,
                );
                pill(
                    ui,
                    format!("in pcm_{USER_INPUT_RATE}"),
                    palette.text_dim,
                    palette.surface_alt,
                );
                if connected {
                    pill(
                        ui,
                        "live",
                        palette.success,
                        palette.success.gamma_multiply(0.18),
                    );
                }
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Playback gain")
                        .size(12.0)
                        .color(palette.text_dim),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(
                            (self.output_gain - 1.0).abs() > f32::EPSILON,
                            egui::Button::new(
                                egui::RichText::new("reset")
                                    .size(10.5)
                                    .color(palette.text_dim),
                            )
                            .small(),
                        )
                        .clicked()
                    {
                        self.output_gain = 1.0;
                        self.state.playback.set_gain(1.0);
                    }
                    ui.label(
                        egui::RichText::new(format!("{:.2}×", self.output_gain))
                            .size(12.0)
                            .family(egui::FontFamily::Monospace)
                            .color(palette.text),
                    );
                });
            });

            let slider_width = ui.available_width();
            ui.spacing_mut().slider_width = slider_width;
            if ui
                .add(egui::Slider::new(&mut self.output_gain, 0.05..=1.5).show_value(false))
                .changed()
            {
                self.state.playback.set_gain(self.output_gain);
            }

            ui.add_space(8.0);
            hint(
                ui,
                palette,
                "Local speaker multiplier. Keep it at 1.00× unless playback clips or is too quiet.",
            );
            ui.add_space(4.0);
            hint(
                ui,
                palette,
                "Wear headphones for clean barge-in — CPAL has no WebRTC echo cancellation.",
            );
        });
    }

    fn show_brain_card(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let active_model = self.state.config.selected_model.get();
        let active_prompt = self.state.config.system_prompt.get();

        card(ui, palette, "Brain", |ui| {
            info_row(ui, palette, "Model", &active_model);

            ui.add_space(10.0);
            let refresh_width = 30.0;
            let combo_width =
                (ui.available_width() - refresh_width - ui.spacing().item_spacing.x).max(90.0);
            ui.horizontal(|ui| {
                let models = self.models.clone();
                egui::ComboBox::from_id_salt("ollama_model_picker")
                    .width(combo_width)
                    .selected_text(if self.model_input.is_empty() {
                        "pick a local model".to_owned()
                    } else {
                        self.model_input.clone()
                    })
                    .show_ui(ui, |ui| {
                        if models.is_empty() {
                            ui.label(
                                egui::RichText::new("no models reported by Ollama")
                                    .size(11.5)
                                    .color(palette.text_faint),
                            );
                        }
                        for model in models {
                            ui.selectable_value(&mut self.model_input, model.clone(), model);
                        }
                    });
                if ui
                    .add(egui::Button::new("⟳").min_size(egui::vec2(refresh_width, 24.0)))
                    .on_hover_text("Refresh the list of local Ollama models")
                    .clicked()
                {
                    self.refresh_models();
                }
            });

            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.model_input)
                    .hint_text("or type a model id")
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(8.0);
            let model_dirty = self.model_input.trim() != active_model;
            let full_width = ui.available_width();
            if primary_button(ui, palette, "Apply model", full_width, model_dirty).clicked() {
                self.apply_model();
            }

            ui.add_space(6.0);
            ui.hyperlink_to(
                egui::RichText::new("Browse the Ollama registry ↗")
                    .size(11.0)
                    .color(palette.accent),
                "https://ollama.com/search",
            );

            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("SYSTEM PROMPT")
                    .size(10.5)
                    .strong()
                    .extra_letter_spacing(0.6)
                    .color(palette.text_faint),
            );
            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::multiline(&mut self.system_prompt_input)
                    .desired_rows(6)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(8.0);
            let prompt_dirty = self.system_prompt_input.trim() != active_prompt;
            let full_width = ui.available_width();
            if primary_button(ui, palette, "Apply prompt", full_width, prompt_dirty).clicked() {
                self.apply_system_prompt();
            }
        });
    }

    // ------------------------------------------------------------------
    // Conversation tab
    // ------------------------------------------------------------------

    fn show_conversation_tab(&mut self, ui: &mut egui::Ui) {
        let palette = self.palette();
        let first_token = self.latest_latency("first LLM token");
        let full_reply = self.latest_latency("final LLM text");
        let turns = self
            .transcript
            .iter()
            .filter(|entry| entry.speaker == Speaker::Agent)
            .count();

        let spacing = ui.spacing().item_spacing.x;
        let tile_width = ((ui.available_width() - spacing * 3.0) / 4.0).max(110.0);
        ui.horizontal(|ui| {
            stat_tile(
                ui,
                &palette,
                tile_width,
                "Time to first token",
                &format_millis(first_token),
                latency_color(&palette, first_token, 900),
            );
            stat_tile(
                ui,
                &palette,
                tile_width,
                "Time to full reply",
                &format_millis(full_reply),
                latency_color(&palette, full_reply, 2_500),
            );
            stat_tile(
                ui,
                &palette,
                tile_width,
                "Agent turns",
                &turns.to_string(),
                palette.text,
            );
            stat_tile(
                ui,
                &palette,
                tile_width,
                "Interruptions",
                &self.interruptions.to_string(),
                if self.interruptions > 0 {
                    palette.warn
                } else {
                    palette.text
                },
            );
        });

        ui.add_space(12.0);

        let body_height = ui.available_height();
        let side_width = (ui.available_width() * 0.32).clamp(200.0, 280.0);
        let transcript_width = (ui.available_width() - side_width - spacing).max(240.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(transcript_width, body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.show_transcript_card(ui, &palette, body_height),
            );
            ui.allocate_ui_with_layout(
                egui::vec2(side_width, body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.show_latency_card(ui, &palette, body_height),
            );
        });
    }

    fn show_transcript_card(&self, ui: &mut egui::Ui, palette: &Palette, height: f32) {
        card_sized(ui, palette, "Transcript", height, |ui| {
            if self.transcript.is_empty() {
                empty_state(
                    ui,
                    palette,
                    "No turns yet",
                    "Connect the desktop participant and start speaking. User transcripts, agent replies and barge-ins land here.",
                );
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("transcript_scroll")
                .stick_to_bottom(true)
                .max_height((height - 46.0).max(80.0))
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for entry in &self.transcript {
                        transcript_row(ui, palette, entry);
                    }
                });
        });
    }

    fn show_latency_card(&self, ui: &mut egui::Ui, palette: &Palette, height: f32) {
        card_sized(ui, palette, "Turn latency", height, |ui| {
            if self.latencies.is_empty() {
                empty_state(
                    ui,
                    palette,
                    "Nothing measured",
                    "Latency is recorded for every turn once the agent starts replying.",
                );
                return;
            }
            let worst = self
                .latencies
                .iter()
                .map(|metric| metric.millis)
                .max()
                .unwrap_or(1)
                .max(1);
            egui::ScrollArea::vertical()
                .id_salt("latency_scroll")
                .max_height((height - 46.0).max(80.0))
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for metric in &self.latencies {
                        ui.label(
                            egui::RichText::new(shorten_latency_label(&metric.label))
                                .size(11.0)
                                .color(palette.text_dim),
                        );
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(58.0, 14.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{} ms", metric.millis))
                                            .size(11.0)
                                            .family(egui::FontFamily::Monospace)
                                            .color(palette.text),
                                    );
                                },
                            );
                            meter(
                                ui,
                                palette,
                                metric.millis as f32 / worst as f32,
                                latency_color(palette, Some(metric.millis), 1_500),
                            );
                        });
                        ui.add_space(8.0);
                    }
                });
        });
    }

    // ------------------------------------------------------------------
    // Speech Engine tab
    // ------------------------------------------------------------------

    fn show_speech_engine_tab(&mut self, ui: &mut egui::Ui) {
        let palette = self.palette();
        self.show_engine_header(ui, &palette);
        ui.add_space(12.0);

        let body_height = ui.available_height();
        let nav_width = 168.0;
        let spacing = ui.spacing().item_spacing.x;
        let content_width = (ui.available_width() - nav_width - spacing).max(280.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(nav_width, body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.show_engine_nav(ui, &palette),
            );
            ui.allocate_ui_with_layout(
                egui::vec2(content_width, body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("speech_engine_section_scroll")
                        .max_height(body_height)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| match self.engine_section {
                            EngineSection::General => self.show_engine_general(ui, &palette),
                            EngineSection::Asr => self.show_engine_asr(ui, &palette),
                            EngineSection::Tts => self.show_engine_tts(ui, &palette),
                            EngineSection::Turn => self.show_engine_turn(ui, &palette),
                            EngineSection::Conversation => {
                                self.show_engine_conversation(ui, &palette)
                            }
                            EngineSection::ClientEvents => {
                                self.show_engine_client_events(ui, &palette)
                            }
                            EngineSection::Privacy => self.show_engine_privacy(ui, &palette),
                            EngineSection::Limits => self.show_engine_limits(ui, &palette),
                            EngineSection::Advanced => self.show_engine_advanced(ui, &palette),
                        });
                },
            );
        });
    }

    fn show_engine_header(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let dirty = self.engine_form_is_dirty();
        let engine_id = self.state.config.speech_engine_id.get();
        let name = if self.speech_engine_form.name.trim().is_empty() {
            "Untitled Speech Engine".to_owned()
        } else {
            self.speech_engine_form.name.clone()
        };
        let width = ui.available_width();

        egui::Frame::NONE
            .fill(palette.surface)
            .stroke(egui::Stroke::new(1.0_f32, palette.border))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_min_width(width - 28.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(name)
                                .size(14.0)
                                .strong()
                                .color(palette.text),
                        );
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&engine_id)
                                    .size(11.0)
                                    .family(egui::FontFamily::Monospace)
                                    .color(palette.text_faint),
                            );
                            if dirty {
                                pill(
                                    ui,
                                    "unsaved changes",
                                    palette.warn,
                                    palette.warn.gamma_multiply(0.18),
                                );
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if primary_button(ui, palette, "Save changes", 122.0, dirty).clicked() {
                            self.update_speech_engine_from_form();
                        }
                        if ghost_button(ui, palette, "Create copy", 108.0, true).clicked() {
                            self.create_speech_engine_from_form();
                        }
                        if ghost_button(ui, palette, "Reload", 82.0, true)
                            .on_hover_text("Discard local edits and refetch this engine")
                            .clicked()
                        {
                            self.load_selected_speech_engine();
                        }
                    });
                });
            });
    }

    fn show_engine_nav(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let width = ui.available_width();
        egui::Frame::NONE
            .fill(palette.surface)
            .stroke(egui::Stroke::new(1.0_f32, palette.border))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.set_min_width(width - 12.0);
                ui.spacing_mut().item_spacing.y = 2.0;
                for section in EngineSection::ALL {
                    if nav_item(ui, palette, section.title(), self.engine_section == section) {
                        self.engine_section = section;
                    }
                }
            });
    }

    fn show_engine_general(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        card(ui, palette, "Identity", |ui| {
            form_grid(ui, "engine_general_grid", |ui| {
                field(
                    ui,
                    palette,
                    "ws URL",
                    "Publicly reachable URL that ElevenLabs dials for LLM responses. Expose the local server (for example through an ngrok tunnel) and paste that URL here.",
                    |ui| {
                        ui.text_edit_singleline(&mut self.speech_engine_form.ws_url);
                    },
                );
                field(
                    ui,
                    palette,
                    "Name",
                    "Shown in the ElevenLabs dashboard.",
                    |ui| {
                        ui.text_edit_singleline(&mut self.speech_engine_form.name);
                    },
                );
                field(
                    ui,
                    palette,
                    "Language",
                    "ISO 639-1 code used as the default for transcription and synthesis.",
                    |ui| {
                        ui.text_edit_singleline(&mut self.speech_engine_form.language);
                    },
                );
                field(ui, palette, "Tags", "Comma separated labels.", |ui| {
                    ui.text_edit_singleline(&mut self.speech_engine_form.tags);
                });
            });
        });

        ui.add_space(10.0);
        json_editor(
            ui,
            palette,
            "Upstream request headers",
            "Extra headers sent with the outbound WebSocket handshake. Must be a JSON object.",
            5,
            &mut self.speech_engine_form.request_headers_json,
        );
    }

    fn show_engine_asr(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let bridge_format = format!("pcm_{USER_INPUT_RATE}");
        let mismatch = self.speech_engine_form.user_input_audio_format.trim() != bridge_format;

        card(ui, palette, "Speech to text", |ui| {
            form_grid(ui, "engine_asr_grid", |ui| {
                field(ui, palette, "Provider", "Transcription backend.", |ui| {
                    combo_str(
                        ui,
                        "speech_engine_asr_provider",
                        &mut self.speech_engine_form.asr_provider,
                        &["scribe_realtime", "elevenlabs"],
                    );
                });
                field(
                    ui,
                    palette,
                    "Quality",
                    "Transcription quality tier.",
                    |ui| {
                        combo_str(
                            ui,
                            "speech_engine_asr_quality",
                            &mut self.speech_engine_form.asr_quality,
                            &["high"],
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Input format",
                    "Audio format ElevenLabs expects on the conversation socket.",
                    |ui| {
                        combo_str(
                            ui,
                            "speech_engine_input_format",
                            &mut self.speech_engine_form.user_input_audio_format,
                            &SPEECH_ENGINE_OUTPUT_FORMATS,
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Keywords",
                    "Comma separated bias words that improve recognition of names and jargon.",
                    |ui| {
                        ui.text_edit_singleline(&mut self.speech_engine_form.keywords);
                    },
                );
            });

            if mismatch {
                ui.add_space(10.0);
                notice(
                    ui,
                    palette,
                    palette.warn,
                    &format!(
                        "The LiveKit bridge publishes {bridge_format}. Anything else will be rejected or transcribed as noise."
                    ),
                );
            }
        });
    }

    fn show_engine_tts(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let active_rate = self.state.config.agent_output_rate.get();
        let selected_rate = parse_speech_engine_output_rate(
            self.speech_engine_form.agent_output_audio_format.trim(),
        );

        card(ui, palette, "Voice", |ui| {
            form_grid(ui, "engine_tts_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Model",
                    "Pick a known model or type any model id the account has access to.",
                    |ui| {
                        ui.vertical(|ui| {
                            combo_str(
                                ui,
                                "speech_engine_tts_model",
                                &mut self.speech_engine_form.tts_model_id,
                                &SPEECH_ENGINE_TTS_MODELS,
                            );
                            ui.text_edit_singleline(&mut self.speech_engine_form.tts_model_id);
                        });
                    },
                );
                field(
                    ui,
                    palette,
                    "Voice id",
                    "Leave empty to use the account default voice.",
                    |ui| {
                        ui.text_edit_singleline(&mut self.speech_engine_form.voice_id);
                    },
                );
                field(
                    ui,
                    palette,
                    "Output format",
                    "Audio the engine streams back. The workbench resamples to the LiveKit track rate.",
                    |ui| {
                        combo_str(
                            ui,
                            "speech_engine_output_format",
                            &mut self.speech_engine_form.agent_output_audio_format,
                            &SPEECH_ENGINE_OUTPUT_FORMATS,
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Text normalisation",
                    "How numbers, dates and symbols are expanded before synthesis.",
                    |ui| {
                        combo_str(
                            ui,
                            "speech_engine_text_normalisation",
                            &mut self.speech_engine_form.text_normalisation_type,
                            &["", "system_prompt", "elevenlabs"],
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Audio filter",
                    "Optional post-processing filter name.",
                    |ui| {
                        ui.text_edit_singleline(&mut self.speech_engine_form.audio_filter);
                    },
                );
            });

            if selected_rate.is_some_and(|rate| rate != active_rate) {
                ui.add_space(10.0);
                notice(
                    ui,
                    palette,
                    palette.warn,
                    &format!(
                        "The LiveKit bridge was created at {active_rate} Hz. Restart the workbench after saving so the publisher matches the new output format."
                    ),
                );
            }
            if selected_rate.is_none() {
                ui.add_space(10.0);
                notice(
                    ui,
                    palette,
                    palette.error,
                    "Unsupported output format. Use one of the pcm_* rates or ulaw_8000.",
                );
            }
        });

        ui.add_space(10.0);
        card(ui, palette, "Delivery", |ui| {
            form_grid(ui, "engine_tts_delivery_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Stability",
                    "Low values are more expressive, high values are more consistent.",
                    |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.speech_engine_form.stability, 0.0..=1.0)
                                .fixed_decimals(2),
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Similarity",
                    "How closely synthesis tracks the reference voice.",
                    |ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut self.speech_engine_form.similarity_boost,
                                0.0..=1.0,
                            )
                            .fixed_decimals(2),
                        );
                    },
                );
                field(ui, palette, "Speed", "Playback rate multiplier.", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.speech_engine_form.speed, 0.7..=1.2)
                            .fixed_decimals(2),
                    );
                });
            });

            ui.add_space(8.0);
            ui.checkbox(
                &mut self.speech_engine_form.expressive_mode,
                "Expressive mode",
            );
            ui.checkbox(
                &mut self.speech_engine_form.enable_phoneme_tags,
                "Enable phoneme tags",
            );
        });
    }

    fn show_engine_turn(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        card(ui, palette, "Timing", |ui| {
            form_grid(ui, "engine_turn_timing_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Turn timeout",
                    "How long the engine waits for more speech before handing the turn over.",
                    |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.speech_engine_form.turn_timeout)
                                .range(0.0..=3_600.0)
                                .speed(0.1)
                                .suffix(" s"),
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Initial wait",
                    "Optional grace period before the first turn is evaluated.",
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.speech_engine_form.initial_wait_time_enabled, "");
                            ui.add_enabled(
                                self.speech_engine_form.initial_wait_time_enabled,
                                egui::DragValue::new(
                                    &mut self.speech_engine_form.initial_wait_time,
                                )
                                .range(0.0..=3_600.0)
                                .speed(0.1)
                                .suffix(" s"),
                            );
                        });
                    },
                );
                field(
                    ui,
                    palette,
                    "Silence ends call",
                    "Hang up after this much silence. Use -1 to disable.",
                    |ui| {
                        ui.add(
                            egui::DragValue::new(
                                &mut self.speech_engine_form.silence_end_call_timeout,
                            )
                            .range(-1.0..=86_400.0)
                            .speed(0.5)
                            .suffix(" s"),
                        );
                    },
                );
            });
        });

        ui.add_space(10.0);
        card(ui, palette, "Detection", |ui| {
            form_grid(ui, "engine_turn_detection_grid", |ui| {
                field(ui, palette, "Mode", "Turn detection mode name.", |ui| {
                    ui.text_edit_singleline(&mut self.speech_engine_form.turn_mode);
                });
                field(
                    ui,
                    palette,
                    "Eagerness",
                    "How quickly the engine assumes the user finished speaking.",
                    |ui| {
                        combo_str(
                            ui,
                            "speech_engine_turn_eagerness",
                            &mut self.speech_engine_form.turn_eagerness,
                            &["patient", "normal", "eager"],
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Spelling patience",
                    "Extra tolerance while the user spells something out.",
                    |ui| {
                        combo_str(
                            ui,
                            "speech_engine_spelling_patience",
                            &mut self.speech_engine_form.spelling_patience,
                            &["auto", "off"],
                        );
                    },
                );
                field(ui, palette, "Turn model", "Turn detection model.", |ui| {
                    combo_str(
                        ui,
                        "speech_engine_turn_model",
                        &mut self.speech_engine_form.turn_model,
                        &["turn_v2", "turn_v3"],
                    );
                });
            });

            ui.add_space(8.0);
            ui.checkbox(
                &mut self.speech_engine_form.speculative_turn,
                "Speculative turn",
            );
            ui.checkbox(
                &mut self.speech_engine_form.retranscribe_on_turn_timeout,
                "Retranscribe on turn timeout",
            );
            ui.checkbox(
                &mut self.speech_engine_form.transcribe_on_disabled_interruptions,
                "Transcribe during disabled interruptions",
            );
        });

        ui.add_space(10.0);
        card(ui, palette, "Interruptions", |ui| {
            form_grid(ui, "engine_turn_interruption_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Ignore terms",
                    "Comma separated filler words that should not count as a barge-in.",
                    |ui| {
                        ui.text_edit_singleline(
                            &mut self.speech_engine_form.interruption_ignore_terms,
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Term languages",
                    "Comma separated language codes the ignore terms apply to.",
                    |ui| {
                        ui.text_edit_singleline(
                            &mut self.speech_engine_form.interruption_ignore_term_languages,
                        );
                    },
                );
            });
        });
    }

    fn show_engine_conversation(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        card(ui, palette, "Session", |ui| {
            ui.checkbox(&mut self.speech_engine_form.text_only, "Text only");
            ui.add_space(8.0);
            form_grid(ui, "engine_conversation_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Maximum duration",
                    "Hard cap on a single conversation.",
                    |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.speech_engine_form.max_duration_seconds)
                                .range(0..=86_400)
                                .suffix(" s"),
                        );
                    },
                );
            });
        });

        ui.add_space(10.0);
        card(ui, palette, "File input", |ui| {
            ui.checkbox(
                &mut self.speech_engine_form.file_input_enabled,
                "Allow file uploads",
            );
            ui.add_space(8.0);
            ui.add_enabled_ui(self.speech_engine_form.file_input_enabled, |ui| {
                form_grid(ui, "engine_file_input_grid", |ui| {
                    field(
                        ui,
                        palette,
                        "Files per conversation",
                        "Upload cap per conversation.",
                        |ui| {
                            ui.add(
                                egui::DragValue::new(
                                    &mut self.speech_engine_form.max_files_per_conversation,
                                )
                                .range(1..=10),
                            );
                        },
                    );
                });
            });
        });

        ui.add_space(10.0);
        card(ui, palette, "Ambience", |ui| {
            let selected = if self
                .speech_engine_form
                .background_sound_source_id
                .is_empty()
            {
                "none".to_owned()
            } else {
                self.speech_engine_form.background_sound_source_id.clone()
            };
            form_grid(ui, "engine_background_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Background preset",
                    "Ambient bed mixed under the agent voice.",
                    |ui| {
                        let width = ui.spacing().combo_width;
                        egui::ComboBox::from_id_salt("speech_engine_background_sound")
                            .width(width)
                            .selected_text(selected)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(
                                        self.speech_engine_form
                                            .background_sound_source_id
                                            .is_empty(),
                                        "none",
                                    )
                                    .clicked()
                                {
                                    self.speech_engine_form.background_sound_source_type.clear();
                                    self.speech_engine_form.background_sound_source_id.clear();
                                }
                                for preset in BACKGROUND_SOUND_PRESETS {
                                    if ui
                                        .selectable_label(
                                            self.speech_engine_form.background_sound_source_id
                                                == preset,
                                            preset,
                                        )
                                        .clicked()
                                    {
                                        self.speech_engine_form.background_sound_source_type =
                                            "preset".to_owned();
                                        self.speech_engine_form.background_sound_source_id =
                                            preset.to_owned();
                                    }
                                }
                            });
                    },
                );
                field(
                    ui,
                    palette,
                    "Custom source id",
                    "Overrides the preset with an uploaded asset id.",
                    |ui| {
                        ui.text_edit_singleline(
                            &mut self.speech_engine_form.background_sound_source_id,
                        );
                    },
                );
                field(ui, palette, "Volume", "Relative ambience level.", |ui| {
                    ui.add(
                        egui::Slider::new(
                            &mut self.speech_engine_form.background_sound_volume,
                            0.01..=1.0,
                        )
                        .fixed_decimals(2),
                    );
                });
            });

            ui.add_space(8.0);
            ui.checkbox(
                &mut self.speech_engine_form.background_sound_crossfade_loop,
                "Crossfade the loop",
            );
            ui.checkbox(
                &mut self.speech_engine_form.source_attribution,
                "Send source attribution",
            );
        });
    }

    fn show_engine_client_events(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        card(ui, palette, "Client events", |ui| {
            hint(
                ui,
                palette,
                "Events streamed to the conversation socket. The workbench needs the highlighted ones, so they stay locked on.",
            );
            ui.add_space(10.0);
            show_event_selector(
                ui,
                palette,
                "speech_engine_client_events",
                &mut self.speech_engine_form.client_events,
                &REQUIRED_CLIENT_EVENTS,
            );
        });

        ui.add_space(10.0);
        card(ui, palette, "Monitoring", |ui| {
            ui.checkbox(
                &mut self.speech_engine_form.monitoring_enabled,
                "Forward events to monitoring",
            );
            ui.add_space(10.0);
            ui.add_enabled_ui(self.speech_engine_form.monitoring_enabled, |ui| {
                show_event_selector(
                    ui,
                    palette,
                    "speech_engine_monitoring_events",
                    &mut self.speech_engine_form.monitoring_events,
                    &[],
                );
            });
        });
    }

    fn show_engine_privacy(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        card(ui, palette, "Retention", |ui| {
            ui.checkbox(&mut self.speech_engine_form.record_voice, "Record voice");
            ui.add_space(8.0);
            form_grid(ui, "engine_privacy_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Retention days",
                    "How long recordings are kept. Use -1 for the account default.",
                    |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.speech_engine_form.retention_days)
                                .range(-1..=3_650),
                        );
                    },
                );
            });
        });

        ui.add_space(10.0);
        card(ui, palette, "Deletion", |ui| {
            ui.checkbox(
                &mut self.speech_engine_form.delete_transcript_and_pii,
                "Delete transcripts and PII",
            );
            ui.checkbox(&mut self.speech_engine_form.delete_audio, "Delete audio");
            ui.checkbox(
                &mut self.speech_engine_form.apply_to_existing_conversations,
                "Apply to existing conversations",
            );
            ui.checkbox(
                &mut self.speech_engine_form.zero_retention_mode,
                "Zero retention mode",
            );
        });

        ui.add_space(10.0);
        card(ui, palette, "Redaction", |ui| {
            ui.checkbox(
                &mut self
                    .speech_engine_form
                    .conversation_history_redaction_enabled,
                "Redact conversation history",
            );
            ui.add_space(8.0);
            ui.add_enabled_ui(
                self.speech_engine_form
                    .conversation_history_redaction_enabled,
                |ui| {
                    form_grid(ui, "engine_redaction_grid", |ui| {
                        field(
                            ui,
                            palette,
                            "Entities",
                            "Comma separated entity types to strip from stored history.",
                            |ui| {
                                ui.text_edit_singleline(
                                    &mut self
                                        .speech_engine_form
                                        .conversation_history_redaction_entities,
                                );
                            },
                        );
                    });
                },
            );
        });
    }

    fn show_engine_limits(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        card(ui, palette, "Call limits", |ui| {
            form_grid(ui, "engine_limits_grid", |ui| {
                field(
                    ui,
                    palette,
                    "Daily limit",
                    "Maximum conversations per day. Use -1 for unlimited.",
                    |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.speech_engine_form.daily_limit)
                                .range(-1..=10_000_000),
                        );
                    },
                );
                field(
                    ui,
                    palette,
                    "Concurrency limit",
                    "Maximum simultaneous conversations. Use -1 for unlimited.",
                    |ui| {
                        ui.add(
                            egui::DragValue::new(
                                &mut self.speech_engine_form.agent_concurrency_limit,
                            )
                            .range(-1..=100_000),
                        );
                    },
                );
            });

            ui.add_space(8.0);
            ui.checkbox(
                &mut self.speech_engine_form.bursting_enabled,
                "Allow bursting above the concurrency limit",
            );
        });

        ui.add_space(10.0);
        card(ui, palette, "Overrides", |ui| {
            ui.checkbox(
                &mut self.speech_engine_form.allow_first_message_override,
                "Allow the client to override the first message",
            );
        });
    }

    fn show_engine_advanced(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        json_editor(
            ui,
            palette,
            "Supported voices",
            "Array of voice objects offered to the agent at runtime.",
            6,
            &mut self.speech_engine_form.supported_voices_json,
        );
        ui.add_space(10.0);
        json_editor(
            ui,
            palette,
            "Suggested audio tags",
            "Array of audio tags the engine may inject for expressive speech.",
            5,
            &mut self.speech_engine_form.suggested_audio_tags_json,
        );
        ui.add_space(10.0);
        json_editor(
            ui,
            palette,
            "Pronunciation dictionaries",
            "Array of dictionary locators applied before synthesis.",
            5,
            &mut self
                .speech_engine_form
                .pronunciation_dictionary_locators_json,
        );
    }

    // ------------------------------------------------------------------
    // Events tab
    // ------------------------------------------------------------------

    fn show_events_tab(&mut self, ui: &mut egui::Ui) {
        let palette = self.palette();
        let (_, warnings, errors) = self.log_counts();
        let mut clear_requested = false;
        let mut copy_requested = false;

        let width = ui.available_width();
        egui::Frame::NONE
            .fill(palette.surface)
            .stroke(egui::Stroke::new(1.0_f32, palette.border))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_min_width(width - 24.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.log_query)
                            .hint_text("filter messages")
                            .desired_width(170.0),
                    );
                    ui.add_space(4.0);
                    let mut filter = self.log_filter;
                    segmented(
                        ui,
                        &palette,
                        &mut filter,
                        48.0,
                        &[
                            (LogFilter::All, "All"),
                            (LogFilter::Info, "Info"),
                            (LogFilter::Warn, "Warn"),
                            (LogFilter::Error, "Error"),
                        ],
                    );
                    self.log_filter = filter;

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new("Clear")
                                    .size(11.5)
                                    .color(palette.text_dim),
                            ))
                            .clicked()
                        {
                            clear_requested = true;
                        }
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new("Copy")
                                    .size(11.5)
                                    .color(palette.text_dim),
                            ))
                            .on_hover_text("Copy the filtered log to the clipboard")
                            .clicked()
                        {
                            copy_requested = true;
                        }
                        ui.add_space(4.0);
                        if errors > 0 {
                            pill(
                                ui,
                                format!("{errors} errors"),
                                palette.error,
                                palette.error.gamma_multiply(0.18),
                            );
                        }
                        if warnings > 0 {
                            pill(
                                ui,
                                format!("{warnings} warnings"),
                                palette.warn,
                                palette.warn.gamma_multiply(0.18),
                            );
                        }
                    });
                });
            });

        ui.add_space(12.0);

        let query = self.log_query.trim().to_lowercase();
        let filter = self.log_filter;
        let matches = |line: &LogLine| {
            filter.accepts(line.level)
                && (query.is_empty() || line.message.to_lowercase().contains(&query))
        };

        if copy_requested {
            let dump = self
                .logs
                .iter()
                .filter(|line| matches(line))
                .map(|line| format!("{} {:>5} {}", line.time, line.level, line.message))
                .collect::<Vec<_>>()
                .join("\n");
            ui.ctx().copy_text(dump);
        }

        let height = ui.available_height();
        card_sized(ui, &palette, "Event log", height, |ui| {
            let visible = self.logs.iter().filter(|line| matches(line)).count();
            if visible == 0 {
                empty_state(
                    ui,
                    &palette,
                    if self.logs.is_empty() {
                        "No events yet"
                    } else {
                        "No matching events"
                    },
                    "Telemetry from the brain server, the LiveKit bridge and the desktop participant is collected here.",
                );
                return;
            }
            let row_width = ui.available_width();
            egui::ScrollArea::vertical()
                .id_salt("event_log_scroll")
                .stick_to_bottom(true)
                .max_height((height - 46.0).max(80.0))
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for (index, line) in self.logs.iter().filter(|line| matches(line)).enumerate() {
                        log_row(ui, &palette, line, index % 2 == 1, row_width);
                    }
                });
        });

        if clear_requested {
            self.logs.clear();
        }
    }
}

// ----------------------------------------------------------------------
// Theme
// ----------------------------------------------------------------------

/// Flat colour set the whole workbench is drawn from.
#[derive(Clone, Copy)]
struct Palette {
    canvas: egui::Color32,
    sidebar: egui::Color32,
    surface: egui::Color32,
    surface_alt: egui::Color32,
    input: egui::Color32,
    border: egui::Color32,
    border_strong: egui::Color32,
    text: egui::Color32,
    text_dim: egui::Color32,
    text_faint: egui::Color32,
    accent: egui::Color32,
    accent_hover: egui::Color32,
    on_accent: egui::Color32,
    success: egui::Color32,
    warn: egui::Color32,
    error: egui::Color32,
    info: egui::Color32,
}

impl Palette {
    const DARK: Self = Self {
        canvas: egui::Color32::from_rgb(0x0D, 0x11, 0x17),
        sidebar: egui::Color32::from_rgb(0x11, 0x16, 0x1D),
        surface: egui::Color32::from_rgb(0x16, 0x1B, 0x22),
        surface_alt: egui::Color32::from_rgb(0x1C, 0x22, 0x2B),
        input: egui::Color32::from_rgb(0x0B, 0x0F, 0x14),
        border: egui::Color32::from_rgb(0x2A, 0x31, 0x3B),
        border_strong: egui::Color32::from_rgb(0x3A, 0x42, 0x4E),
        text: egui::Color32::from_rgb(0xE6, 0xED, 0xF3),
        text_dim: egui::Color32::from_rgb(0x9B, 0xA6, 0xB2),
        text_faint: egui::Color32::from_rgb(0x6E, 0x77, 0x81),
        accent: egui::Color32::from_rgb(0x2F, 0x81, 0xF7),
        accent_hover: egui::Color32::from_rgb(0x4C, 0x93, 0xF8),
        on_accent: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        success: egui::Color32::from_rgb(0x3F, 0xB9, 0x50),
        warn: egui::Color32::from_rgb(0xD2, 0x99, 0x22),
        error: egui::Color32::from_rgb(0xF8, 0x51, 0x49),
        info: egui::Color32::from_rgb(0x58, 0xA6, 0xFF),
    };

    const LIGHT: Self = Self {
        canvas: egui::Color32::from_rgb(0xF3, 0xF5, 0xF8),
        sidebar: egui::Color32::from_rgb(0xEC, 0xEF, 0xF3),
        surface: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        surface_alt: egui::Color32::from_rgb(0xF0, 0xF3, 0xF7),
        input: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        border: egui::Color32::from_rgb(0xD8, 0xDE, 0xE6),
        border_strong: egui::Color32::from_rgb(0xBF, 0xC7, 0xD1),
        text: egui::Color32::from_rgb(0x1B, 0x22, 0x29),
        text_dim: egui::Color32::from_rgb(0x5A, 0x66, 0x73),
        text_faint: egui::Color32::from_rgb(0x88, 0x92, 0xA0),
        accent: egui::Color32::from_rgb(0x09, 0x69, 0xDA),
        accent_hover: egui::Color32::from_rgb(0x21, 0x7B, 0xE6),
        on_accent: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        success: egui::Color32::from_rgb(0x1A, 0x7F, 0x37),
        warn: egui::Color32::from_rgb(0x9A, 0x67, 0x00),
        error: egui::Color32::from_rgb(0xCF, 0x22, 0x2E),
        info: egui::Color32::from_rgb(0x09, 0x69, 0xDA),
    };
}

fn apply_theme(ctx: &egui::Context, dark: bool) {
    let palette = if dark { Palette::DARK } else { Palette::LIGHT };
    let theme = if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    };
    ctx.set_theme(if dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });

    let mut style = (*ctx.style()).clone();

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.indent = 16.0;
    style.spacing.interact_size.y = 24.0;
    style.spacing.slider_width = 150.0;
    style.spacing.combo_width = 170.0;
    style.spacing.text_edit_width = 220.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = false;
    style.spacing.tooltip_width = 340.0;

    style.text_styles = [
        (
            egui::TextStyle::Heading,
            egui::FontId::new(17.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Body,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Button,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Small,
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Monospace,
            egui::FontId::new(12.0, egui::FontFamily::Monospace),
        ),
    ]
    .into();

    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = palette.canvas;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.input;
    visuals.faint_bg_color = palette.surface_alt;
    visuals.override_text_color = Some(palette.text);
    visuals.hyperlink_color = palette.accent;
    visuals.window_stroke = egui::Stroke::new(1.0_f32, palette.border);
    visuals.window_corner_radius = egui::CornerRadius::same(10);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.striped = false;
    visuals.slider_trailing_fill = true;
    visuals.selection = egui::style::Selection {
        bg_fill: palette.accent.gamma_multiply(0.35),
        stroke: egui::Stroke::new(1.0_f32, palette.accent),
    };

    let radius = egui::CornerRadius::same(7);
    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.noninteractive.weak_bg_fill = palette.surface;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, palette.border);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, palette.text_dim);
    visuals.widgets.noninteractive.corner_radius = radius;

    visuals.widgets.inactive.bg_fill = palette.surface_alt;
    visuals.widgets.inactive.weak_bg_fill = palette.surface_alt;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, palette.border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);
    visuals.widgets.inactive.corner_radius = radius;
    visuals.widgets.inactive.expansion = 0.0;

    visuals.widgets.hovered.bg_fill = palette.surface_alt;
    visuals.widgets.hovered.weak_bg_fill = palette.surface_alt;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, palette.border_strong);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);
    visuals.widgets.hovered.corner_radius = radius;
    visuals.widgets.hovered.expansion = 0.0;

    visuals.widgets.active.bg_fill = palette.accent.gamma_multiply(0.30);
    visuals.widgets.active.weak_bg_fill = palette.accent.gamma_multiply(0.30);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, palette.accent);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);
    visuals.widgets.active.corner_radius = radius;
    visuals.widgets.active.expansion = 0.0;

    visuals.widgets.open.bg_fill = palette.surface_alt;
    visuals.widgets.open.weak_bg_fill = palette.surface_alt;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, palette.accent);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);
    visuals.widgets.open.corner_radius = radius;

    style.visuals = visuals;
    ctx.set_style_of(theme, style);
}

// ----------------------------------------------------------------------
// Reusable widgets
// ----------------------------------------------------------------------

fn card<R>(
    ui: &mut egui::Ui,
    palette: &Palette,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    card_sized(ui, palette, title, 0.0, add_contents)
}

fn card_sized<R>(
    ui: &mut egui::Ui,
    palette: &Palette,
    title: &str,
    min_height: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let width = ui.available_width();
    egui::Frame::NONE
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(width - 24.0);
            if min_height > 0.0 {
                ui.set_min_height(min_height - 20.0);
            }
            if !title.is_empty() {
                ui.label(
                    egui::RichText::new(title.to_uppercase())
                        .size(10.5)
                        .strong()
                        .extra_letter_spacing(0.6)
                        .color(palette.text_faint),
                );
                ui.add_space(8.0);
            }
            add_contents(ui)
        })
        .inner
}

/// Two column label/value grid used by every configuration section.
fn form_grid(ui: &mut egui::Ui, id: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width();
    let control_width = (width - 210.0).clamp(150.0, 340.0);
    ui.spacing_mut().text_edit_width = control_width;
    ui.spacing_mut().combo_width = control_width;
    ui.spacing_mut().slider_width = (control_width - 70.0).max(90.0);
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([18.0, 10.0])
        .min_col_width(160.0)
        .show(ui, |ui| add_contents(ui));
}

fn field(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: &str,
    help: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(12.0)
                .color(palette.text_dim),
        );
        if !help.is_empty() {
            ui.label(
                egui::RichText::new("?")
                    .size(10.0)
                    .strong()
                    .color(palette.text_faint),
            )
            .on_hover_text(help);
        }
    });
    add_contents(ui);
    ui.end_row();
}

fn combo_str(ui: &mut egui::Ui, id: &'static str, value: &mut String, options: &[&str]) {
    let width = ui.spacing().combo_width;
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .selected_text(if value.is_empty() {
            "default".to_owned()
        } else {
            value.clone()
        })
        .show_ui(ui, |ui| {
            for option in options {
                let label = if option.is_empty() {
                    "default"
                } else {
                    *option
                };
                ui.selectable_value(value, (*option).to_owned(), label);
            }
        });
}

/// JSON text area with live validation and a pretty-print button.
fn json_editor(
    ui: &mut egui::Ui,
    palette: &Palette,
    title: &str,
    help: &str,
    rows: usize,
    value: &mut String,
) {
    card(ui, palette, title, |ui| {
        hint(ui, palette, help);
        ui.add_space(8.0);
        ui.add(
            egui::TextEdit::multiline(value)
                .desired_width(f32::INFINITY)
                .desired_rows(rows)
                .font(egui::TextStyle::Monospace),
        );
        ui.add_space(8.0);

        let parsed = serde_json::from_str::<Value>(value);
        ui.horizontal(|ui| {
            match &parsed {
                Ok(_) => pill(
                    ui,
                    "valid JSON",
                    palette.success,
                    palette.success.gamma_multiply(0.18),
                ),
                Err(error) => pill(
                    ui,
                    format!("invalid JSON — {error}"),
                    palette.error,
                    palette.error.gamma_multiply(0.18),
                ),
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        parsed.is_ok(),
                        egui::Button::new(
                            egui::RichText::new("Format")
                                .size(11.5)
                                .color(palette.text_dim),
                        ),
                    )
                    .clicked()
                {
                    if let Ok(parsed) = &parsed {
                        *value = pretty_json(parsed);
                    }
                }
            });
        });
    });
}

fn pill(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    foreground: egui::Color32,
    background: egui::Color32,
) -> egui::Response {
    egui::Frame::NONE
        .fill(background)
        .corner_radius(egui::CornerRadius::same(255))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text.into())
                    .size(10.5)
                    .color(foreground),
            );
        })
        .response
}

fn status_chip(ui: &mut egui::Ui, palette: &Palette, color: egui::Color32, text: &str) {
    egui::Frame::NONE
        .fill(palette.surface_alt)
        .corner_radius(egui::CornerRadius::same(255))
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                status_dot(ui, color);
                ui.label(egui::RichText::new(text).size(11.0).color(palette.text));
            });
        });
}

fn status_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

fn status_text(ui: &mut egui::Ui, color: egui::Color32, text: String) {
    ui.label(
        egui::RichText::new(text)
            .size(10.5)
            .family(egui::FontFamily::Monospace)
            .color(color),
    );
}

fn info_row(ui: &mut egui::Ui, palette: &Palette, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(72.0, 18.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_max_width(72.0);
                ui.label(
                    egui::RichText::new(label)
                        .size(11.5)
                        .color(palette.text_faint),
                );
            },
        );
        let response = ui.add(
            egui::Label::new(
                egui::RichText::new(value)
                    .size(11.5)
                    .family(egui::FontFamily::Monospace)
                    .color(palette.text),
            )
            .truncate()
            .sense(egui::Sense::click()),
        );
        if response.clicked() {
            ui.ctx().copy_text(value.to_owned());
        }
        response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(format!("{value}\n\nClick to copy"));
    });
}

fn hint(ui: &mut egui::Ui, palette: &Palette, text: &str) {
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .size(11.0)
                .color(palette.text_faint),
        )
        .wrap(),
    );
}

fn notice(ui: &mut egui::Ui, palette: &Palette, color: egui::Color32, text: &str) {
    egui::Frame::NONE
        .fill(color.gamma_multiply(0.14))
        .stroke(egui::Stroke::new(1.0_f32, color.gamma_multiply(0.45)))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_min_width(width);
            ui.add(
                egui::Label::new(egui::RichText::new(text).size(11.5).color(palette.text)).wrap(),
            );
        });
}

fn empty_state(ui: &mut egui::Ui, palette: &Palette, title: &str, body: &str) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(title)
                .size(13.0)
                .strong()
                .color(palette.text_dim),
        );
        ui.add_space(6.0);
        ui.set_max_width(360.0);
        ui.add(
            egui::Label::new(
                egui::RichText::new(body)
                    .size(11.5)
                    .color(palette.text_faint),
            )
            .wrap(),
        );
    });
    ui.add_space(24.0);
}

fn stat_tile(
    ui: &mut egui::Ui,
    palette: &Palette,
    width: f32,
    label: &str,
    value: &str,
    accent: egui::Color32,
) {
    egui::Frame::NONE
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(width - 24.0);
            ui.set_max_width(width - 24.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(label.to_uppercase())
                        .size(9.5)
                        .strong()
                        .extra_letter_spacing(0.6)
                        .color(palette.text_faint),
                );
                ui.add_space(2.0);
                ui.label(egui::RichText::new(value).size(20.0).strong().color(accent));
            });
        });
}

fn meter(ui: &mut egui::Ui, palette: &Palette, fraction: f32, color: egui::Color32) {
    let width = ui.available_width().max(24.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 6.0), egui::Sense::hover());
    let radius = egui::CornerRadius::same(3);
    ui.painter().rect_filled(rect, radius, palette.surface_alt);
    let filled = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(rect.width() * fraction.clamp(0.0, 1.0), rect.height()),
    );
    ui.painter().rect_filled(filled, radius, color);
}

fn brand_mark(ui: &mut egui::Ui, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(8), palette.accent);
    let painter = ui.painter();
    let bars = [0.32_f32, 0.66, 1.0, 0.58, 0.28];
    let bar_width = 2.0;
    let gap = 3.0;
    let total = bars.len() as f32 * bar_width + (bars.len() - 1) as f32 * gap;
    let mut x = rect.center().x - total / 2.0 + bar_width / 2.0;
    for bar in bars {
        let half = rect.height() * 0.26 * bar + 1.5;
        painter.line_segment(
            [
                egui::pos2(x, rect.center().y - half),
                egui::pos2(x, rect.center().y + half),
            ],
            egui::Stroke::new(bar_width, palette.on_accent),
        );
        x += bar_width + gap;
    }
}

fn primary_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    let (rest, hover) = if enabled {
        (palette.accent, palette.accent_hover)
    } else {
        (palette.surface_alt, palette.surface_alt)
    };
    let text = if enabled {
        palette.on_accent
    } else {
        palette.text_faint
    };
    ui.scope(|ui| {
        paint_button_states(ui, rest, hover, egui::Stroke::NONE);
        ui.add_enabled(
            enabled,
            egui::Button::new(egui::RichText::new(label).size(12.5).strong().color(text))
                .corner_radius(egui::CornerRadius::same(7))
                .min_size(egui::vec2(width, 28.0)),
        )
    })
    .inner
}

fn ghost_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    let text = if enabled {
        palette.text
    } else {
        palette.text_faint
    };
    ui.scope(|ui| {
        paint_button_states(
            ui,
            egui::Color32::TRANSPARENT,
            palette.surface_alt,
            egui::Stroke::new(1.0_f32, palette.border_strong),
        );
        ui.add_enabled(
            enabled,
            egui::Button::new(egui::RichText::new(label).size(12.5).color(text))
                .corner_radius(egui::CornerRadius::same(7))
                .min_size(egui::vec2(width, 28.0)),
        )
    })
    .inner
}

/// Pins a button's fill and outline across every interaction state so hover and
/// press still read, unlike a hard-coded [`egui::Button::fill`].
fn paint_button_states(
    ui: &mut egui::Ui,
    rest: egui::Color32,
    hover: egui::Color32,
    stroke: egui::Stroke,
) {
    let widgets = &mut ui.style_mut().visuals.widgets;
    for state in [
        &mut widgets.noninteractive,
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
    ] {
        state.weak_bg_fill = rest;
        state.bg_stroke = stroke;
        state.expansion = 0.0;
    }
    widgets.hovered.weak_bg_fill = hover;
    widgets.active.weak_bg_fill = hover;
}

/// Pill shaped tab strip. `segment_width` is a minimum, so longer labels still fit.
fn segmented<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    palette: &Palette,
    current: &mut T,
    segment_width: f32,
    options: &[(T, &str)],
) {
    egui::Frame::NONE
        .fill(palette.surface_alt)
        .corner_radius(egui::CornerRadius::same(9))
        .inner_margin(egui::Margin::same(3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (value, label) in options {
                    let selected = *current == *value;
                    let text = egui::RichText::new(*label).size(12.5).color(if selected {
                        palette.text
                    } else {
                        palette.text_dim
                    });
                    let button = egui::Button::new(text)
                        .fill(if selected {
                            palette.surface
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .stroke(if selected {
                            egui::Stroke::new(1.0_f32, palette.border)
                        } else {
                            egui::Stroke::NONE
                        })
                        .corner_radius(egui::CornerRadius::same(7))
                        .min_size(egui::vec2(segment_width, 24.0));
                    if ui.add(button).clicked() {
                        *current = *value;
                    }
                }
            });
        });
}

/// Left hand navigation entry inside the Speech Engine tab.
fn nav_item(ui: &mut egui::Ui, palette: &Palette, label: &str, selected: bool) -> bool {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), egui::Sense::click());
    let radius = egui::CornerRadius::same(7);
    if selected {
        ui.painter()
            .rect_filled(rect, radius, palette.accent.gamma_multiply(0.16));
        let bar = egui::Rect::from_min_size(
            rect.left_top() + egui::vec2(0.0, 6.0),
            egui::vec2(2.5, rect.height() - 12.0),
        );
        ui.painter()
            .rect_filled(bar, egui::CornerRadius::same(2), palette.accent);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, radius, palette.surface_alt);
    }
    ui.painter().text(
        rect.left_center() + egui::vec2(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::new(12.5, egui::FontFamily::Proportional),
        if selected {
            palette.text
        } else {
            palette.text_dim
        },
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

fn transcript_row(ui: &mut egui::Ui, palette: &Palette, entry: &TranscriptEntry) {
    let (label, accent) = match entry.speaker {
        Speaker::User => ("you", palette.info),
        Speaker::Agent => ("agent", palette.accent),
        Speaker::System => ("system", palette.warn),
    };
    let tinted = entry.speaker == Speaker::Agent;
    egui::Frame::NONE
        .fill(if tinted {
            palette.surface_alt
        } else {
            egui::Color32::TRANSPARENT
        })
        .stroke(egui::Stroke::new(
            1.0_f32,
            if tinted {
                palette.border
            } else {
                egui::Color32::TRANSPARENT
            },
        ))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_min_width(width);
            ui.horizontal(|ui| {
                pill(ui, label, accent, accent.gamma_multiply(0.18));
                ui.label(
                    egui::RichText::new(&entry.time)
                        .size(10.0)
                        .family(egui::FontFamily::Monospace)
                        .color(palette.text_faint),
                );
            });
            ui.add_space(3.0);
            ui.add(
                egui::Label::new(egui::RichText::new(&entry.text).size(12.5).color(
                    if entry.speaker == Speaker::System {
                        palette.text_dim
                    } else {
                        palette.text
                    },
                ))
                .wrap(),
            );
        });
    ui.add_space(6.0);
}

fn log_row(ui: &mut egui::Ui, palette: &Palette, line: &LogLine, striped: bool, width: f32) {
    let color = log_level_color(palette, line.level);
    egui::Frame::NONE
        .fill(if striped {
            palette.surface_alt.gamma_multiply(0.6)
        } else {
            egui::Color32::TRANSPARENT
        })
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui| {
            ui.set_min_width(width - 12.0);
            ui.horizontal_top(|ui| {
                ui.label(
                    egui::RichText::new(&line.time)
                        .size(10.5)
                        .family(egui::FontFamily::Monospace)
                        .color(palette.text_faint),
                )
                .on_hover_text("UTC");
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(44.0, 15.0), egui::Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(4),
                    color.gamma_multiply(0.18),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    line.level,
                    egui::FontId::new(9.5, egui::FontFamily::Monospace),
                    color,
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&line.message)
                            .size(12.0)
                            .color(palette.text),
                    )
                    .wrap(),
                );
            });
        });
}

fn log_level_color(palette: &Palette, level: &str) -> egui::Color32 {
    match level {
        "error" => palette.error,
        "warn" => palette.warn,
        _ => palette.info,
    }
}

fn latency_color(palette: &Palette, value: Option<u128>, budget: u128) -> egui::Color32 {
    match value {
        None => palette.text_dim,
        Some(millis) if millis <= budget => palette.success,
        Some(millis) if millis <= budget * 2 => palette.warn,
        Some(_) => palette.error,
    }
}

fn format_millis(value: Option<u128>) -> String {
    match value {
        Some(millis) if millis >= 1_000 => format!("{:.2} s", millis as f64 / 1_000.0),
        Some(millis) => format!("{millis} ms"),
        None => "—".to_owned(),
    }
}

fn shorten_latency_label(label: &str) -> &str {
    match label {
        "transcript to first LLM token" => "first token",
        "transcript to final LLM text" => "full reply",
        other => other,
    }
}

/// `HH:MM:SS` in UTC. Good enough to correlate events without pulling in a
/// date-time dependency.
fn clock_time() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    let day = seconds % 86_400;
    format!(
        "{:02}:{:02}:{:02}",
        day / 3_600,
        (day % 3_600) / 60,
        day % 60
    )
}

fn show_event_selector(
    ui: &mut egui::Ui,
    palette: &Palette,
    id: &'static str,
    selected: &mut Vec<String>,
    required: &[&str],
) {
    let enabled = ALL_CLIENT_EVENTS
        .iter()
        .filter(|event| required.contains(event) || selected.iter().any(|name| name == *event))
        .count();

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{enabled} of {} enabled", ALL_CLIENT_EVENTS.len()))
                .size(11.0)
                .color(palette.text_dim),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("none")
                        .size(11.0)
                        .color(palette.text_dim),
                ))
                .clicked()
            {
                selected.retain(|event| required.contains(&event.as_str()));
            }
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("all")
                        .size(11.0)
                        .color(palette.text_dim),
                ))
                .clicked()
            {
                *selected = ALL_CLIENT_EVENTS
                    .into_iter()
                    .map(ToOwned::to_owned)
                    .collect();
            }
        });
    });

    ui.add_space(10.0);
    egui::Grid::new(id)
        .num_columns(3)
        .spacing([14.0, 5.0])
        .show(ui, |ui| {
            for (index, event) in ALL_CLIENT_EVENTS.into_iter().enumerate() {
                let is_required = required.contains(&event);
                let mut checked = is_required || selected.iter().any(|name| name == event);
                let label = egui::RichText::new(event).size(11.5).color(if is_required {
                    palette.accent
                } else {
                    palette.text
                });
                let response =
                    ui.add_enabled(!is_required, egui::Checkbox::new(&mut checked, label));
                if is_required {
                    response.clone().on_hover_text(
                        "The workbench subscribes to this event; it cannot be turned off here.",
                    );
                }
                if response.changed() {
                    if checked {
                        selected.push(event.to_owned());
                    } else {
                        selected.retain(|name| name != event);
                    }
                }
                if index % 3 == 2 {
                    ui.end_row();
                }
            }
        });
}

impl eframe::App for SpeechEngineWorkbench {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.applied_theme != Some(self.dark_mode) {
            apply_theme(ctx, self.dark_mode);
            self.applied_theme = Some(self.dark_mode);
        }

        self.drain_events();
        ctx.request_repaint_after(Duration::from_millis(100));

        let palette = self.palette();
        self.show_top_bar(ctx, &palette);
        self.show_status_bar(ctx, &palette);
        self.show_side_panel(ctx, &palette);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(palette.canvas)
                    .inner_margin(egui::Margin::symmetric(16, 14)),
            )
            .show(ctx, |ui| {
                let mut tab = self.active_tab;
                ui.horizontal(|ui| {
                    segmented(
                        ui,
                        &palette,
                        &mut tab,
                        108.0,
                        &[
                            (WorkbenchTab::Conversation, "Conversation"),
                            (WorkbenchTab::SpeechEngine, "Speech Engine"),
                            (WorkbenchTab::Events, "Events"),
                        ],
                    );
                });
                self.active_tab = tab;

                ui.add_space(12.0);
                match self.active_tab {
                    WorkbenchTab::Conversation => self.show_conversation_tab(ui),
                    WorkbenchTab::SpeechEngine => self.show_speech_engine_tab(ui),
                    WorkbenchTab::Events => self.show_events_tab(ui),
                }
            });
    }
}

impl Drop for SpeechEngineWorkbench {
    fn drop(&mut self) {
        self.disconnect_participant();
        for task in self.background_tasks.drain(..) {
            task.abort();
        }
    }
}

async fn load_or_create_speech_engine(
    client: &ElevenLabsClient,
    preferred_output_format: &str,
) -> Result<String> {
    if let Ok(id) = env::var("ELEVENLABS_SPEECH_ENGINE_ID") {
        return Ok(id);
    }

    let ws_url = env::var("ELEVENLABS_SPEECH_ENGINE_WS_URL").context(
        "set ELEVENLABS_SPEECH_ENGINE_ID for an existing engine, or \
ELEVENLABS_SPEECH_ENGINE_WS_URL to create one",
    )?;

    let mut form = SpeechEngineForm::from_env(preferred_output_format);
    form.ws_url = ws_url;
    let body = form.build_body()?;

    let engine = client
        .hit(CreateSpeechEngine::new(body))
        .await
        .context("failed to create Speech Engine")?;

    println!(
        "Created Speech Engine {}. Set ELEVENLABS_SPEECH_ENGINE_ID to reuse it.",
        engine.speech_engine_id
    );

    Ok(engine.speech_engine_id)
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .as_deref()
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

fn non_empty_or(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn preferred_speech_engine_output_format() -> String {
    if let Ok(format) = env::var("ELEVENLABS_SPEECH_ENGINE_TTS_OUTPUT_FORMAT") {
        return format;
    }

    let preferred_rate = default_output_sample_rate()
        .and_then(nearest_supported_pcm_rate)
        .unwrap_or(DEFAULT_AGENT_OUTPUT_RATE);
    format!("pcm_{preferred_rate}")
}

fn default_output_sample_rate() -> Option<u32> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;
    Some(config.sample_rate().0)
}

fn nearest_supported_pcm_rate(rate: u32) -> Option<u32> {
    const SUPPORTED: [u32; 6] = [8_000, 16_000, 22_050, 24_000, 44_100, 48_000];
    SUPPORTED
        .into_iter()
        .min_by_key(|supported| supported.abs_diff(rate))
}

fn parse_speech_engine_output_rate(format: &str) -> Option<u32> {
    match format.trim() {
        "pcm_8000" | "ulaw_8000" => Some(8_000),
        "pcm_16000" => Some(16_000),
        "pcm_22050" => Some(22_050),
        "pcm_24000" => Some(24_000),
        "pcm_44100" => Some(44_100),
        "pcm_48000" => Some(48_000),
        _ => None,
    }
}

async fn speech_engine_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if state.config.verify_speech_engine_auth {
        let Some(token) = headers
            .get(AUTHORIZATION_HEADER)
            .and_then(|value| value.to_str().ok())
        else {
            return (
                StatusCode::UNAUTHORIZED,
                format!("missing {AUTHORIZATION_HEADER} header"),
            )
                .into_response();
        };

        if let Err(error) =
            verify_authorization_token(token, state.config.elevenlabs_api_key.as_ref())
        {
            warn!("rejected Speech Engine websocket: {error}");
            return (StatusCode::UNAUTHORIZED, "invalid Speech Engine token").into_response();
        }
    }

    ws.on_upgrade(move |socket| handle_speech_engine_socket(socket, state))
}

async fn handle_speech_engine_socket(socket: WebSocket, state: AppState) {
    let (mut socket_tx, mut socket_rx) = socket.split();
    let (outbound_tx, mut outbound_rx) = mpsc::channel::<AxumMessage>(32);
    let writer = tokio::spawn(async move {
        while let Some(message) = outbound_rx.recv().await {
            socket_tx.send(message).await?;
        }
        Result::<()>::Ok(())
    });

    let mut current_event_id = None;
    let mut current_response: Option<JoinHandle<()>> = None;

    while let Some(frame) = socket_rx.next().await {
        let frame = match frame {
            Ok(frame) => frame,
            Err(error) => {
                warn!("websocket receive error: {error}");
                break;
            }
        };

        let result = match frame {
            AxumMessage::Text(text) => {
                handle_speech_engine_text(
                    state.clone(),
                    &text,
                    outbound_tx.clone(),
                    &mut current_event_id,
                    &mut current_response,
                )
                .await
            }
            AxumMessage::Ping(payload) => outbound_tx
                .send(AxumMessage::Pong(payload))
                .await
                .map_err(Into::into)
                .map(|_| true),
            AxumMessage::Close(_) => break,
            AxumMessage::Pong(_) => Ok(true),
            AxumMessage::Binary(_) => {
                warn!("ignoring unexpected binary Speech Engine brain frame");
                Ok(true)
            }
        };

        match result {
            Ok(true) => {}
            Ok(false) => break,
            Err(error) => {
                error!("{error:#}");
                state.telemetry.error(format!("{error:#}"));
                let _ = outbound_tx.send(AxumMessage::Close(None)).await;
                break;
            }
        }
    }

    if let Some(response) = current_response {
        response.abort();
    }
    drop(outbound_tx);

    match writer.await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            warn!("Speech Engine brain writer ended with error: {error:#}");
        }
        Err(error) => {
            warn!("Speech Engine brain writer task failed: {error}");
        }
    }

    debug!("Speech Engine brain WebSocket closed");
}

async fn handle_speech_engine_text(
    state: AppState,
    text: &str,
    outbound_tx: mpsc::Sender<AxumMessage>,
    current_event_id: &mut Option<u64>,
    current_response: &mut Option<JoinHandle<()>>,
) -> Result<bool> {
    let message: SpeechEngineInboundMessage =
        serde_json::from_str(text).context("failed to decode Speech Engine message")?;

    match message {
        SpeechEngineInboundMessage::Init(init) => {
            state.telemetry.log(format!(
                "Speech Engine brain conversation started: {}",
                init.conversation_id
            ));
        }
        SpeechEngineInboundMessage::UserTranscript(transcript) => {
            let event_id = transcript.event_id;
            if is_stale_or_duplicate_event(*current_event_id, event_id) {
                debug!(
                    "ignoring stale or duplicate Speech Engine transcript event_id={event_id:?}"
                );
                return Ok(true);
            }

            if let Some(response) = current_response.take() {
                if !response.is_finished() {
                    state.telemetry.log(format!(
                        "cancelling in-flight agent response for newer transcript event_id={event_id:?}"
                    ));
                    response.abort();
                }
            }

            if let Some(event_id) = event_id {
                *current_event_id = Some(event_id);
            }

            let response_tx = outbound_tx.clone();
            let response_state = state.clone();
            *current_response = Some(tokio::spawn(async move {
                let fallback_tx = response_tx.clone();
                let fallback_state = response_state.clone();
                if let Err(error) = respond_to_transcript(
                    response_tx,
                    response_state,
                    transcript.user_transcript,
                    event_id,
                )
                .await
                {
                    error!("{error:#}");
                    fallback_state.telemetry.error(format!(
                        "agent response failed for event_id={event_id:?}: {error:#}"
                    ));
                    let _ = send_agent_response(
                        &fallback_tx,
                        event_id,
                        "I hit a local model error. Check the Ollama model selection and logs.",
                        false,
                    )
                    .await;
                    let _ = send_agent_response(&fallback_tx, event_id, "", true).await;
                }
            }));
        }
        SpeechEngineInboundMessage::Ping => {
            send_brain_message(&outbound_tx, SpeechEngineOutboundMessage::pong()).await?;
        }
        SpeechEngineInboundMessage::Close => {
            state.telemetry.log("Speech Engine requested brain close");
            let _ = outbound_tx.send(AxumMessage::Close(None)).await;
            return Ok(false);
        }
        SpeechEngineInboundMessage::Error(error) => {
            state
                .telemetry
                .warn(format!("Speech Engine protocol error: {}", error.message));
        }
        SpeechEngineInboundMessage::Unknown(unknown) => {
            debug!("unknown brain message type: {}", unknown.message_type);
        }
    }

    Ok(true)
}

async fn respond_to_transcript(
    outbound_tx: mpsc::Sender<AxumMessage>,
    state: AppState,
    transcript: Vec<SpeechEngineTranscriptMessage>,
    event_id: Option<u64>,
) -> Result<()> {
    let started_at = Instant::now();
    let Some((latest_user_index, user_text)) = latest_user_turn(&transcript) else {
        debug!("transcript did not contain user text");
        return Ok(());
    };

    if !is_actionable_user_text(user_text) {
        state.telemetry.log(format!(
            "ignoring non-actionable transcript event_id={event_id:?}: {user_text}"
        ));
        return Ok(());
    }

    state.telemetry.emit(UiEvent::UserTranscript {
        event_id,
        text: user_text.to_owned(),
    });
    let chat_history = rig_chat_history(&transcript[..latest_user_index]);
    let ollama = match ollama::Client::from_env() {
        Ok(client) => client,
        Err(_) => {
            ollama::Client::new(Nothing).context("failed to create the default Ollama client")?
        }
    };
    let model = state.config.selected_model.get();
    let system_prompt = state.config.system_prompt.get();
    let agent = ollama
        .agent(model.as_str())
        .preamble(&system_prompt)
        .temperature(0.6)
        .build();

    let mut stream = agent.stream_chat(user_text.to_owned(), &chat_history).await;
    let mut sent_text = false;
    let mut sent_first_token = false;
    let mut full_response = String::new();
    let mut final_response = None;

    while let Some(item) = stream.next().await {
        match item.context("Ollama stream failed")? {
            MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text)) => {
                if text.text.is_empty() {
                    continue;
                }

                if !sent_first_token {
                    sent_first_token = true;
                    state
                        .telemetry
                        .latency("transcript to first LLM token", started_at.elapsed());
                }

                full_response.push_str(&text.text);
                sent_text = true;
                send_agent_response(&outbound_tx, event_id, text.text, false).await?;
            }
            MultiTurnStreamItem::FinalResponse(response) => {
                final_response = Some(response.response().to_owned());
            }
            _ => {}
        }
    }

    if !sent_text {
        let fallback = final_response
            .filter(|response| !response.is_empty())
            .unwrap_or_else(|| "I had trouble thinking through that. Please try again.".to_owned());
        full_response.push_str(&fallback);
        send_agent_response(&outbound_tx, event_id, fallback, false).await?;
    }

    state.telemetry.emit(UiEvent::AgentResponse(full_response));
    state
        .telemetry
        .latency("transcript to final LLM text", started_at.elapsed());
    send_agent_response(&outbound_tx, event_id, "", true).await?;

    Ok(())
}

async fn send_agent_response(
    outbound_tx: &mpsc::Sender<AxumMessage>,
    event_id: Option<u64>,
    content: impl Into<String>,
    is_final: bool,
) -> Result<()> {
    let content = content.into();
    let message = match (event_id, is_final) {
        (Some(event_id), true) => SpeechEngineOutboundMessage::final_agent_response(event_id),
        (Some(event_id), false) => SpeechEngineOutboundMessage::agent_response(event_id, content),
        (None, _) => {
            SpeechEngineOutboundMessage::agent_response_without_event_id(content, is_final)
        }
    };
    send_brain_message(outbound_tx, message).await
}

async fn send_brain_message(
    outbound_tx: &mpsc::Sender<AxumMessage>,
    message: SpeechEngineOutboundMessage,
) -> Result<()> {
    let text = serde_json::to_string(&message)?;
    outbound_tx.send(AxumMessage::Text(text.into())).await?;
    Ok(())
}

fn is_stale_or_duplicate_event(current: Option<u64>, incoming: Option<u64>) -> bool {
    matches!((current, incoming), (Some(current), Some(incoming)) if incoming <= current)
}

fn latest_user_turn(transcript: &[SpeechEngineTranscriptMessage]) -> Option<(usize, &str)> {
    transcript
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, message)| {
            let content = message.content.trim();
            (message.role == "user" && !content.is_empty()).then_some((index, content))
        })
}

fn is_actionable_user_text(text: &str) -> bool {
    text.chars().any(|ch| ch.is_alphanumeric())
}

fn rig_chat_history(transcript: &[SpeechEngineTranscriptMessage]) -> Vec<Message> {
    let start = transcript.len().saturating_sub(12);
    transcript[start..]
        .iter()
        .filter_map(|message| {
            let content = message.content.trim();
            if content.is_empty() {
                return None;
            }

            match message.role.as_str() {
                "user" => Some(Message::user(content.to_owned())),
                "agent" | "assistant" => Some(Message::assistant(content.to_owned())),
                "system" => Some(Message::system(content.to_owned())),
                other => {
                    debug!("ignoring unsupported transcript role: {other}");
                    None
                }
            }
        })
        .collect()
}

async fn run_livekit_bridge(state: AppState) -> Result<()> {
    let bridge_token = create_livekit_token(
        state.config.livekit_api_key.as_ref(),
        state.config.livekit_api_secret.as_ref(),
        BRIDGE_IDENTITY,
        state.config.livekit_room.as_ref(),
        true,
    )?;
    let (room, mut room_events) = Room::connect(
        state.config.livekit_url.as_ref(),
        &bridge_token,
        RoomOptions::default(),
    )
    .await
    .context("failed to connect bridge participant to LiveKit room")?;
    let room = Arc::new(room);
    let agent_output_rate = state.config.agent_output_rate.get();

    let agent_audio_source = NativeAudioSource::new(
        AudioSourceOptions::default(),
        agent_output_rate,
        MONO,
        AGENT_AUDIO_QUEUE_MS,
    );
    publish_audio_track(&room, &agent_audio_source, "elevenlabs-agent").await?;

    let (audio_tx, mut audio_rx) = mpsc::channel::<String>(128);
    state.telemetry.log(format!(
        "bridge connected to room `{}` as `{}`",
        state.config.livekit_room, BRIDGE_IDENTITY
    ));

    let room_telemetry = state.telemetry.clone();
    let room_events_task = tokio::spawn(async move {
        while let Some(room_event) = room_events.recv().await {
            handle_bridge_room_event(room_event, audio_tx.clone(), &room_telemetry);
        }
        room_telemetry.warn("LiveKit room event stream ended");
    });

    loop {
        state
            .telemetry
            .log("waiting for desktop participant audio before opening Speech Engine conversation");
        let Some(first_audio) = audio_rx.recv().await else {
            state
                .telemetry
                .warn("LiveKit audio channel closed; stopping bridge");
            break;
        };

        match run_speech_engine_conversation(
            state.clone(),
            agent_audio_source.clone(),
            agent_output_rate,
            &mut audio_rx,
            first_audio,
        )
        .await
        {
            Ok(SpeechConnectionOutcome::Reconnect) => {
                state
                    .telemetry
                    .warn("Speech Engine conversation disconnected; reconnecting");
            }
            Ok(SpeechConnectionOutcome::AudioChannelClosed) => {
                state
                    .telemetry
                    .warn("LiveKit audio channel closed; stopping bridge");
                break;
            }
            Err(error) => {
                state.telemetry.warn(format!(
                    "Speech Engine conversation failed: {error:#}; reconnecting"
                ));
            }
        }

        tokio::time::sleep(Duration::from_millis(750)).await;
    }

    room_events_task.abort();
    let _ = room.close().await;

    Ok(())
}

async fn run_speech_engine_conversation(
    state: AppState,
    agent_audio_source: NativeAudioSource,
    agent_output_rate: u32,
    audio_rx: &mut mpsc::Receiver<String>,
    first_audio: String,
) -> Result<SpeechConnectionOutcome> {
    let speech_engine_id = state.config.speech_engine_id.get();
    let signed_url = state
        .elevenlabs
        .hit(GetSignedUrl::new(speech_engine_id.clone()))
        .await
        .context("failed to get Speech Engine conversation signed URL")?
        .signed_url;
    let (speech_ws, _) = connect_async(signed_url)
        .await
        .context("failed to connect Speech Engine conversation WebSocket")?;
    let (mut speech_write, mut speech_read) = speech_ws.split();

    let (speech_tx, mut speech_rx) = mpsc::channel::<TungsteniteMessage>(256);
    let (agent_audio_tx, mut agent_audio_rx) = mpsc::channel::<AgentAudioChunk>(128);
    let agent_output_format = state.config.agent_output_format.get();

    let writer = tokio::spawn(async move {
        while let Some(message) = speech_rx.recv().await {
            speech_write.send(message).await?;
        }
        Result::<()>::Ok(())
    });

    let reader_speech_tx = speech_tx.clone();
    let reader_audio_tx = agent_audio_tx.clone();
    let reader_source = agent_audio_source.clone();
    let reader_interruptions = state.interruptions.clone();
    let reader_playback = state.playback.clone();
    let reader_telemetry = state.telemetry.clone();
    let reader = tokio::spawn(async move {
        while let Some(message) = speech_read.next().await {
            handle_conversation_message(
                message?,
                &reader_speech_tx,
                &reader_audio_tx,
                &reader_source,
                &reader_interruptions,
                &reader_playback,
                &reader_telemetry,
            )
            .await?;
        }
        Result::<()>::Ok(())
    });
    let mut reader = Box::pin(reader);

    let playback_source = agent_audio_source.clone();
    let playback_interruptions = state.interruptions.clone();
    let playback = tokio::spawn(async move {
        while let Some(chunk) = agent_audio_rx.recv().await {
            if !playback_interruptions.should_play_generation(chunk.generation) {
                debug!(
                    "dropping stale agent audio chunk for generation {}",
                    chunk.generation
                );
                continue;
            }

            capture_audio_bytes(
                &playback_source,
                &chunk.pcm,
                agent_output_rate,
                &agent_output_format,
            )
            .await?;
        }
        Result::<()>::Ok(())
    });

    speech_tx
        .send(TungsteniteMessage::Text(
            json!({ "type": "conversation_initiation_client_data" })
                .to_string()
                .into(),
        ))
        .await?;

    state.telemetry.log(format!(
        "Speech Engine conversation WebSocket connected for {speech_engine_id}"
    ));

    speech_tx
        .send(TungsteniteMessage::Text(
            json!({ "user_audio_chunk": first_audio })
                .to_string()
                .into(),
        ))
        .await?;

    let outcome = loop {
        tokio::select! {
            audio = audio_rx.recv() => {
                let Some(audio) = audio else {
                    break SpeechConnectionOutcome::AudioChannelClosed;
                };
                if speech_tx
                    .send(TungsteniteMessage::Text(
                        json!({ "user_audio_chunk": audio }).to_string().into(),
                    ))
                    .await
                    .is_err()
                {
                    break SpeechConnectionOutcome::Reconnect;
                }
            }
            result = &mut reader => {
                match result {
                    Ok(Ok(())) => state.telemetry.warn("Speech Engine conversation WebSocket ended"),
                    Ok(Err(error)) => state.telemetry.warn(format!("Speech Engine conversation reader ended: {error:#}")),
                    Err(error) => state.telemetry.warn(format!("Speech Engine conversation reader task failed: {error}")),
                }
                break SpeechConnectionOutcome::Reconnect;
            }
        }
    };

    drop(agent_audio_tx);
    drop(speech_tx);

    match writer.await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => state.telemetry.warn(format!(
            "Speech Engine conversation writer ended: {error:#}"
        )),
        Err(error) => state.telemetry.warn(format!(
            "Speech Engine conversation writer task failed: {error}"
        )),
    }
    match playback.await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => state
            .telemetry
            .warn(format!("Speech Engine playback task ended: {error:#}")),
        Err(error) => state
            .telemetry
            .warn(format!("Speech Engine playback task failed: {error}")),
    }

    Ok(outcome)
}

fn handle_bridge_room_event(
    event: RoomEvent,
    audio_tx: mpsc::Sender<String>,
    telemetry: &Telemetry,
) {
    match event {
        RoomEvent::Connected { .. } => {
            telemetry.log("LiveKit bridge connected");
        }
        RoomEvent::TrackSubscribed {
            track, participant, ..
        } => {
            let identity = participant.identity().to_string();
            if identity == BRIDGE_IDENTITY {
                return;
            }

            let RemoteTrack::Audio(track) = track else {
                return;
            };

            telemetry.log(format!("bridge subscribed to audio from `{identity}`"));
            let telemetry = telemetry.clone();
            tokio::spawn(async move {
                if let Err(error) = pump_livekit_audio_to_speech_engine(track, audio_tx).await {
                    telemetry.warn(format!("audio pump for `{identity}` ended: {error:#}"));
                }
            });
        }
        RoomEvent::Disconnected { reason } => {
            telemetry.warn(format!("LiveKit bridge disconnected: {reason:?}"));
        }
        _ => {}
    }
}

async fn run_desktop_participant(
    state: AppState,
    mut stop_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let identity = format!("{USER_IDENTITY_PREFIX}-{}", now_millis());
    let token = create_livekit_token(
        state.config.livekit_api_key.as_ref(),
        state.config.livekit_api_secret.as_ref(),
        &identity,
        state.config.livekit_room.as_ref(),
        false,
    )?;

    let (room, mut room_events) = Room::connect(
        state.config.livekit_url.as_ref(),
        &token,
        RoomOptions::default(),
    )
    .await
    .context("failed to connect desktop participant to LiveKit room")?;
    let room = Arc::new(room);
    state.telemetry.emit(UiEvent::ParticipantConnected(true));
    state
        .telemetry
        .log(format!("desktop participant connected as `{identity}`"));
    let agent_output_rate = state.config.agent_output_rate.get();

    let output = build_output_stream(state.playback.clone())
        .context("failed to build speaker output stream")?;
    let (speaker_tx, speaker_rx) = mpsc::channel::<Vec<i16>>(128);
    let output_task = spawn_output_buffer_task(
        speaker_rx,
        agent_output_rate,
        output.buffer.clone(),
        output.sample_rate,
        output.channels,
    );
    state.playback.set_buffer(output.buffer.clone());
    output
        .stream
        .play()
        .context("failed to start speaker stream")?;

    let mic_source = NativeAudioSource::new(
        AudioSourceOptions::default(),
        USER_INPUT_RATE,
        MONO,
        AGENT_AUDIO_QUEUE_MS,
    );
    publish_audio_track(&room, &mic_source, "desktop-microphone").await?;

    let (mic_tx, mic_rx) = mpsc::channel::<Vec<i16>>(64);
    let input = build_input_stream(mic_tx).context("failed to build microphone input stream")?;
    let mic_task = tokio::spawn(pump_cpal_audio_to_livekit(
        mic_rx,
        mic_source,
        input.sample_rate,
    ));
    input
        .stream
        .play()
        .context("failed to start microphone stream")?;

    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                state.telemetry.log("desktop participant disconnect requested");
                break;
            }
            room_event = room_events.recv() => {
                let Some(room_event) = room_event else {
                    state.telemetry.warn("desktop participant room events ended");
                    break;
                };
                handle_desktop_room_event(
                    room_event,
                    speaker_tx.clone(),
                    &state.telemetry,
                    agent_output_rate,
                );
            }
        }
    }

    mic_task.abort();
    output_task.abort();
    let _ = room.close().await;
    state.playback.clear_buffer();
    Ok(())
}

fn handle_desktop_room_event(
    event: RoomEvent,
    speaker_tx: mpsc::Sender<Vec<i16>>,
    telemetry: &Telemetry,
    agent_output_rate: u32,
) {
    match event {
        RoomEvent::Connected { .. } => {
            telemetry.log("desktop participant joined LiveKit room");
        }
        RoomEvent::TrackSubscribed {
            track, participant, ..
        } => {
            let identity = participant.identity().to_string();
            if identity != BRIDGE_IDENTITY {
                return;
            }

            let RemoteTrack::Audio(track) = track else {
                return;
            };

            telemetry.log("desktop participant subscribed to agent audio");
            let telemetry = telemetry.clone();
            tokio::spawn(async move {
                if let Err(error) =
                    pump_agent_audio_to_speaker(track, speaker_tx, agent_output_rate).await
                {
                    telemetry.warn(format!("speaker audio pump ended: {error:#}"));
                }
            });
        }
        RoomEvent::Disconnected { reason } => {
            telemetry.warn(format!("desktop participant disconnected: {reason:?}"));
        }
        _ => {}
    }
}

async fn pump_livekit_audio_to_speech_engine(
    track: RemoteAudioTrack,
    audio_tx: mpsc::Sender<String>,
) -> Result<()> {
    let mut stream = NativeAudioStream::new(track.rtc_track(), USER_INPUT_RATE as i32, MONO as i32);

    while let Some(frame) = stream.next().await {
        let payload = encode_pcm_i16_le(&frame.data);
        if audio_tx.send(payload).await.is_err() {
            break;
        }
    }

    Ok(())
}

async fn pump_agent_audio_to_speaker(
    track: RemoteAudioTrack,
    speaker_tx: mpsc::Sender<Vec<i16>>,
    agent_output_rate: u32,
) -> Result<()> {
    let mut stream =
        NativeAudioStream::new(track.rtc_track(), agent_output_rate as i32, MONO as i32);

    while let Some(frame) = stream.next().await {
        if frame.data.is_empty() {
            continue;
        }
        if speaker_tx.send(frame.data.to_vec()).await.is_err() {
            break;
        }
    }

    Ok(())
}

async fn publish_audio_track(room: &Room, source: &NativeAudioSource, name: &str) -> Result<()> {
    let track = LocalAudioTrack::create_audio_track(name, RtcAudioSource::Native(source.clone()));
    room.local_participant()
        .publish_track(
            LocalTrack::Audio(track),
            TrackPublishOptions {
                source: TrackSource::Microphone,
                ..Default::default()
            },
        )
        .await?;
    Ok(())
}

async fn pump_cpal_audio_to_livekit(
    mut mic_rx: mpsc::Receiver<Vec<i16>>,
    source: NativeAudioSource,
    input_sample_rate: u32,
) -> Result<()> {
    let mut resampler = StreamingLinearResampler::new(input_sample_rate, USER_INPUT_RATE);
    while let Some(samples) = mic_rx.recv().await {
        let samples = resampler.process(&samples);
        if samples.is_empty() {
            continue;
        }

        let frame = AudioFrame {
            data: samples.as_slice().into(),
            sample_rate: USER_INPUT_RATE,
            num_channels: MONO,
            samples_per_channel: samples.len() as u32 / MONO,
        };
        source.capture_frame(&frame).await?;
    }

    Ok(())
}

async fn handle_conversation_message(
    message: TungsteniteMessage,
    speech_tx: &mpsc::Sender<TungsteniteMessage>,
    agent_audio_tx: &mpsc::Sender<AgentAudioChunk>,
    source: &NativeAudioSource,
    interruptions: &InterruptionState,
    playback: &PlaybackControl,
    telemetry: &Telemetry,
) -> Result<()> {
    match message {
        TungsteniteMessage::Text(text) => {
            handle_conversation_text(
                &text,
                speech_tx,
                agent_audio_tx,
                source,
                interruptions,
                playback,
                telemetry,
            )
            .await?;
        }
        TungsteniteMessage::Ping(payload) => {
            speech_tx
                .send(TungsteniteMessage::Pong(payload))
                .await
                .context("failed to send WebSocket pong")?;
        }
        TungsteniteMessage::Pong(_) => {}
        TungsteniteMessage::Close(frame) => {
            telemetry.warn(format!(
                "Speech Engine conversation WebSocket closed: {frame:?}"
            ));
        }
        _ => {}
    }
    Ok(())
}

async fn handle_conversation_text(
    text: &str,
    speech_tx: &mpsc::Sender<TungsteniteMessage>,
    agent_audio_tx: &mpsc::Sender<AgentAudioChunk>,
    source: &NativeAudioSource,
    interruptions: &InterruptionState,
    playback: &PlaybackControl,
    telemetry: &Telemetry,
) -> Result<()> {
    let event: Value = serde_json::from_str(text).context("failed to decode conversation event")?;
    match event.get("type").and_then(Value::as_str) {
        Some("audio") => {
            let Some(audio) = event
                .get("audio_event")
                .and_then(|event| event.get("audio_base_64"))
                .and_then(Value::as_str)
            else {
                return Ok(());
            };
            let pcm = BASE64_STANDARD
                .decode(audio)
                .context("failed to decode Speech Engine audio")?;
            agent_audio_tx
                .send(AgentAudioChunk {
                    generation: interruptions.current_generation(),
                    pcm,
                })
                .await?;
        }
        Some("interruption") => {
            let generation = interruptions.mark_interrupted();
            source.clear_buffer();
            playback.clear_buffer();
            telemetry.emit(UiEvent::Interruption { generation });
        }
        Some("conversation_initiation_metadata") => {
            telemetry.log("Speech Engine conversation initialized");
        }
        Some("user_transcript") => {
            if let Some(transcript) = event
                .get("user_transcription_event")
                .and_then(|event| event.get("user_transcript"))
                .and_then(Value::as_str)
            {
                telemetry.log(format!("conversation transcript: {transcript}"));
            }
        }
        Some("agent_response") => {
            if let Some(response) = event
                .get("agent_response_event")
                .and_then(|event| event.get("agent_response"))
                .and_then(Value::as_str)
            {
                telemetry.log(format!("conversation agent: {response}"));
            }
        }
        Some("ping") => {
            let event_id = event
                .get("ping_event")
                .and_then(|event| event.get("event_id"))
                .cloned()
                .unwrap_or(Value::Null);
            speech_tx
                .send(TungsteniteMessage::Text(
                    json!({ "type": "pong", "event_id": event_id })
                        .to_string()
                        .into(),
                ))
                .await
                .context("failed to send protocol pong")?;
        }
        Some(other) => {
            debug!("unknown conversation event: {other}");
        }
        None => {}
    }

    Ok(())
}

async fn capture_audio_bytes(
    source: &NativeAudioSource,
    audio: &[u8],
    agent_output_rate: u32,
    agent_output_format: &str,
) -> Result<()> {
    let samples = decode_agent_audio(audio, agent_output_format)?;
    if samples.is_empty() {
        return Ok(());
    }

    let frame_samples = (agent_output_rate as usize / 100) * MONO as usize;
    for chunk in samples.chunks(frame_samples.max(1)) {
        let frame = AudioFrame {
            data: chunk.into(),
            sample_rate: agent_output_rate,
            num_channels: MONO,
            samples_per_channel: chunk.len() as u32 / MONO,
        };
        source.capture_frame(&frame).await?;
    }
    Ok(())
}

fn decode_agent_audio(audio: &[u8], format: &str) -> Result<Vec<i16>> {
    match format.trim() {
        "ulaw_8000" => Ok(audio.iter().map(|sample| decode_mulaw(*sample)).collect()),
        format if format.starts_with("pcm_") => Ok(decode_pcm_i16_le(audio)),
        other => Err(anyhow!("unsupported Speech Engine output format `{other}`")),
    }
}

fn decode_mulaw(sample: u8) -> i16 {
    let sample = !sample;
    let sign = sample & 0x80;
    let exponent = (sample >> 4) & 0x07;
    let mantissa = sample & 0x0f;
    let magnitude = (((mantissa as i32) << 3) + 0x84) << exponent;
    let magnitude = magnitude - 0x84;

    if sign != 0 {
        -magnitude as i16
    } else {
        magnitude as i16
    }
}

fn build_input_stream(audio_tx: mpsc::Sender<Vec<i16>>) -> Result<InputAudio> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("no default input device available")?;
    let supported = device
        .default_input_config()
        .context("failed to read default input config")?;
    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels() as usize;
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();

    let stream = match sample_format {
        SampleFormat::F32 => {
            let tx = audio_tx.clone();
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let _ = tx.try_send(downmix_f32_to_i16(data, channels));
                },
                input_error,
                None,
            )?
        }
        SampleFormat::I16 => {
            let tx = audio_tx.clone();
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let _ = tx.try_send(downmix_i16(data, channels));
                },
                input_error,
                None,
            )?
        }
        SampleFormat::U16 => {
            let tx = audio_tx.clone();
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let _ = tx.try_send(downmix_u16_to_i16(data, channels));
                },
                input_error,
                None,
            )?
        }
        other => return Err(anyhow!("unsupported input sample format: {other:?}")),
    };

    Ok(InputAudio {
        stream,
        sample_rate,
    })
}

fn build_output_stream(playback: PlaybackControl) -> Result<OutputAudio> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("no default output device available")?;
    let supported = device
        .default_output_config()
        .context("failed to read default output config")?;
    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels() as usize;
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();
    let buffer = Arc::new(Mutex::new(OutputBuffer {
        samples: VecDeque::new(),
        started: false,
        start_threshold_samples: (sample_rate as usize * channels * OUTPUT_START_BUFFER_MS) / 1000,
    }));
    let gain = playback.gain.clone();
    playback.set_buffer(buffer.clone());

    let stream = match sample_format {
        SampleFormat::F32 => {
            let buffer = buffer.clone();
            let gain = gain.clone();
            device.build_output_stream(
                &config,
                move |output: &mut [f32], _| fill_output_f32(output, &buffer, &gain),
                output_error,
                None,
            )?
        }
        SampleFormat::I16 => {
            let buffer = buffer.clone();
            let gain = gain.clone();
            device.build_output_stream(
                &config,
                move |output: &mut [i16], _| fill_output_i16(output, &buffer, &gain),
                output_error,
                None,
            )?
        }
        SampleFormat::U16 => {
            let buffer = buffer.clone();
            let gain = gain.clone();
            device.build_output_stream(
                &config,
                move |output: &mut [u16], _| fill_output_u16(output, &buffer, &gain),
                output_error,
                None,
            )?
        }
        other => return Err(anyhow!("unsupported output sample format: {other:?}")),
    };

    Ok(OutputAudio {
        stream,
        sample_rate,
        channels,
        buffer,
    })
}

fn spawn_output_buffer_task(
    mut speaker_rx: mpsc::Receiver<Vec<i16>>,
    agent_output_rate: u32,
    buffer: SharedOutputBuffer,
    output_sample_rate: u32,
    output_channels: usize,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut resampler = StreamingLinearResampler::new(agent_output_rate, output_sample_rate);
        while let Some(samples) = speaker_rx.recv().await {
            let samples = resampler.process(&samples);
            let samples = upmix_mono(samples, output_channels);
            let Ok(mut buffer) = buffer.lock() else {
                break;
            };

            if buffer.samples.len() > MAX_AUDIO_BUFFER_SAMPLES {
                let overflow = buffer.samples.len() - MAX_AUDIO_BUFFER_SAMPLES;
                buffer.samples.drain(..overflow);
            }
            buffer.samples.extend(samples);
        }
    })
}

struct InputAudio {
    stream: Stream,
    sample_rate: u32,
}

struct OutputAudio {
    stream: Stream,
    sample_rate: u32,
    channels: usize,
    buffer: SharedOutputBuffer,
}

fn input_error(error: cpal::StreamError) {
    eprintln!("input stream error: {error}");
}

fn output_error(error: cpal::StreamError) {
    eprintln!("output stream error: {error}");
}

fn downmix_f32_to_i16(data: &[f32], channels: usize) -> Vec<i16> {
    data.chunks(channels)
        .map(|frame| {
            let mono = frame.iter().copied().sum::<f32>() / frame.len() as f32;
            (mono.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
        })
        .collect()
}

fn downmix_i16(data: &[i16], channels: usize) -> Vec<i16> {
    data.chunks(channels)
        .map(|frame| {
            let sum = frame.iter().map(|sample| *sample as i32).sum::<i32>();
            (sum / frame.len() as i32) as i16
        })
        .collect()
}

fn downmix_u16_to_i16(data: &[u16], channels: usize) -> Vec<i16> {
    data.chunks(channels)
        .map(|frame| {
            let sum = frame
                .iter()
                .map(|sample| *sample as i32 - 32_768)
                .sum::<i32>();
            (sum / frame.len() as i32) as i16
        })
        .collect()
}

fn fill_output_f32(output: &mut [f32], buffer: &SharedOutputBuffer, gain: &Arc<Mutex<f32>>) {
    let Ok(mut buffer) = buffer.lock() else {
        output.fill(0.0);
        return;
    };
    if !ensure_output_started(&mut buffer, output.len()) {
        output.fill(0.0);
        return;
    }
    let gain = gain.lock().map(|gain| *gain).unwrap_or(1.0);

    for sample in output {
        *sample = buffer
            .samples
            .pop_front()
            .map(|sample| (sample as f32 / i16::MAX as f32 * gain).clamp(-1.0, 1.0))
            .unwrap_or(0.0);
    }
    if buffer.samples.is_empty() {
        buffer.started = false;
    }
}

fn fill_output_i16(output: &mut [i16], buffer: &SharedOutputBuffer, gain: &Arc<Mutex<f32>>) {
    let Ok(mut buffer) = buffer.lock() else {
        output.fill(0);
        return;
    };
    if !ensure_output_started(&mut buffer, output.len()) {
        output.fill(0);
        return;
    }
    let gain = gain.lock().map(|gain| *gain).unwrap_or(1.0);

    for sample in output {
        *sample = buffer
            .samples
            .pop_front()
            .map(|sample| apply_gain_i16(sample, gain))
            .unwrap_or(0);
    }
    if buffer.samples.is_empty() {
        buffer.started = false;
    }
}

fn fill_output_u16(output: &mut [u16], buffer: &SharedOutputBuffer, gain: &Arc<Mutex<f32>>) {
    let Ok(mut buffer) = buffer.lock() else {
        output.fill(32_768);
        return;
    };
    if !ensure_output_started(&mut buffer, output.len()) {
        output.fill(32_768);
        return;
    }
    let gain = gain.lock().map(|gain| *gain).unwrap_or(1.0);

    for sample in output {
        let value = buffer
            .samples
            .pop_front()
            .map(|sample| apply_gain_i16(sample, gain) as i32 + 32_768)
            .unwrap_or(32_768);
        *sample = value.clamp(0, u16::MAX as i32) as u16;
    }
    if buffer.samples.is_empty() {
        buffer.started = false;
    }
}

fn apply_gain_i16(sample: i16, gain: f32) -> i16 {
    (sample as f32 * gain)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

fn ensure_output_started(buffer: &mut OutputBuffer, callback_len: usize) -> bool {
    if buffer.started {
        return true;
    }

    let start_threshold = callback_len.max(buffer.start_threshold_samples);
    if buffer.samples.len() >= start_threshold {
        buffer.started = true;
        true
    } else {
        false
    }
}

fn upmix_mono(samples: Vec<i16>, channels: usize) -> Vec<i16> {
    if channels <= 1 {
        return samples;
    }

    samples
        .into_iter()
        .flat_map(|sample| std::iter::repeat_n(sample, channels))
        .collect()
}

struct StreamingLinearResampler {
    source_rate: u32,
    target_rate: u32,
    pending: Vec<i16>,
    position: f64,
}

impl StreamingLinearResampler {
    fn new(source_rate: u32, target_rate: u32) -> Self {
        Self {
            source_rate,
            target_rate,
            pending: Vec::new(),
            position: 0.0,
        }
    }

    fn process(&mut self, samples: &[i16]) -> Vec<i16> {
        if samples.is_empty() {
            return Vec::new();
        }

        if self.source_rate == self.target_rate {
            return samples.to_vec();
        }

        self.pending.extend_from_slice(samples);
        if self.pending.len() < 2 {
            return Vec::new();
        }

        let step = self.source_rate as f64 / self.target_rate as f64;
        let mut output = Vec::with_capacity(
            ((self.pending.len() as f64 - self.position) / step).max(0.0) as usize,
        );

        while self.position + 1.0 < self.pending.len() as f64 {
            let left = self.position.floor() as usize;
            let right = left + 1;
            let fraction = self.position - left as f64;
            let left_sample = self.pending[left] as f64;
            let right_sample = self.pending[right] as f64;
            let sample = left_sample + (right_sample - left_sample) * fraction;
            output.push(sample.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16);
            self.position += step;
        }

        let drain = self.position.floor() as usize;
        if drain > 0 {
            self.pending.drain(..drain);
            self.position -= drain as f64;
        }

        output
    }
}

fn create_livekit_token(
    api_key: &str,
    api_secret: &str,
    identity: &str,
    room: &str,
    room_create: bool,
) -> Result<String> {
    Ok(AccessToken::with_api_key(api_key, api_secret)
        .with_identity(identity)
        .with_name(identity)
        .with_grants(VideoGrants {
            room_create,
            room_join: true,
            room: room.to_owned(),
            can_publish: true,
            can_subscribe: true,
            can_publish_data: true,
            ..Default::default()
        })
        .to_jwt()?)
}

fn encode_pcm_i16_le(samples: &[i16]) -> String {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    BASE64_STANDARD.encode(bytes)
}

fn decode_pcm_i16_le(bytes: &[u8]) -> Vec<i16> {
    bytes
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

async fn fetch_ollama_models() -> Result<Vec<String>> {
    let host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://127.0.0.1:11434".to_owned());
    let url = format!("{}/api/tags", host.trim_end_matches('/'));
    let response: OllamaTagsResponse = reqwest::get(url).await?.error_for_status()?.json().await?;
    let mut models = response
        .models
        .into_iter()
        .map(|model| model.name)
        .collect::<Vec<_>>();
    models.sort();
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_builds_complete_speech_engine_configuration() {
        let mut form = SpeechEngineForm::from_env("pcm_48000");
        form.ws_url = "wss://example.test/ws".to_owned();
        form.agent_output_audio_format = "pcm_48000".to_owned();
        form.request_headers_json = serde_json::json!({
            "X-Static": "value",
            "X-Secret": { "secret_id": "secret_123" },
            "X-Dynamic": { "variable_name": "tenant_id" }
        })
        .to_string();
        form.supported_voices_json = serde_json::json!([{
            "label": "Support",
            "voice_id": "voice_123",
            "language": "en"
        }])
        .to_string();
        form.suggested_audio_tags_json = serde_json::json!([{
            "tag": "happy",
            "description": "Positive response"
        }])
        .to_string();
        form.pronunciation_dictionary_locators_json = serde_json::json!([{
            "pronunciation_dictionary_id": "dict_123",
            "version_id": "version_456"
        }])
        .to_string();
        form.client_events.clear();
        form.background_sound_source_type = "preset".to_owned();
        form.background_sound_source_id = "office1".to_owned();
        form.conversation_history_redaction_enabled = true;
        form.conversation_history_redaction_entities = "name, email_address".to_owned();
        form.allow_first_message_override = true;

        let value = serde_json::to_value(form.build_body().unwrap()).unwrap();
        assert_eq!(
            value["speech_engine"]["request_headers"]["X-Dynamic"]["variable_name"],
            "tenant_id"
        );
        assert_eq!(value["tts"]["supported_voices"][0]["voice_id"], "voice_123");
        assert_eq!(
            value["tts"]["pronunciation_dictionary_locators"][0]["version_id"],
            "version_456"
        );
        assert_eq!(
            value["conversation"]["background_sound"]["source_id"],
            "office1"
        );
        assert_eq!(
            value["privacy"]["conversation_history_redaction"]["entities"][1],
            "email_address"
        );
        assert_eq!(value["overrides"]["first_message"], true);
        for required in REQUIRED_CLIENT_EVENTS {
            assert!(value["conversation"]["client_events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event == required));
        }
    }

    #[test]
    fn form_loads_new_nested_fields_from_response() {
        let response: SpeechEngineResponse = serde_json::from_value(serde_json::json!({
            "speech_engine_id": "seng_test",
            "name": "Loaded engine",
            "speech_engine": {
                "ws_url": "wss://example.test/ws",
                "request_headers": {}
            },
            "asr": {},
            "tts": {
                "agent_output_audio_format": "pcm_24000"
            },
            "turn": {
                "interruption_ignore_term_languages": ["en", "fr"]
            },
            "conversation": {
                "file_input": {
                    "enabled": true,
                    "max_files_per_conversation": 7
                },
                "background_sound": {
                    "source_type": "preset",
                    "source_id": "city",
                    "volume": 0.4,
                    "crossfade_loop": true
                }
            },
            "privacy": {
                "conversation_history_redaction": {
                    "enabled": true,
                    "entities": ["name"]
                }
            },
            "call_limits": {},
            "language": "en",
            "tags": ["loaded"],
            "overrides": {
                "first_message": true
            },
            "metadata": {
                "created_at_unix_secs": 1,
                "updated_at_unix_secs": 2
            },
            "access_info": null
        }))
        .unwrap();

        let form = SpeechEngineForm::from_response(&response);
        assert_eq!(form.agent_output_audio_format, "pcm_24000");
        assert_eq!(form.interruption_ignore_term_languages, "en, fr");
        assert_eq!(form.max_files_per_conversation, 7);
        assert_eq!(form.background_sound_source_id, "city");
        assert!(form.conversation_history_redaction_enabled);
        assert!(form.allow_first_message_override);
    }

    #[test]
    fn invalid_advanced_json_is_rejected_before_api_call() {
        let mut form = SpeechEngineForm::from_env("pcm_24000");
        form.ws_url = "wss://example.test/ws".to_owned();
        form.supported_voices_json = "[not-json]".to_owned();
        let error = form.build_body().unwrap_err().to_string();
        assert!(error.contains("invalid supported voices JSON"));
    }
}
