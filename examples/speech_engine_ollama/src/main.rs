//! Speech Engine chatbot backed by Rig and Ollama.
//!
//! The microphone is captured in the browser by the ElevenLabs client SDK.
//! The Rust server owns the token endpoint, Speech Engine upstream WebSocket,
//! JWT verification, transcript handling, and Rig/Ollama agent call.

use std::{env, net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use elevenlabs_rs::{
    endpoints::genai::speech_engine::{
        ws::{
            verify_authorization_token, SpeechEngineInboundMessage, SpeechEngineOutboundMessage,
            SpeechEngineTranscriptMessage, AUTHORIZATION_HEADER,
        },
        CreateSpeechEngine, CreateSpeechEngineBody,
    },
    ElevenLabsClient, Method,
};
use rig_core::{
    client::{CompletionClient, Nothing, ProviderClient},
    completion::Prompt,
    providers::ollama,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::{debug, error, info, warn};

const DEFAULT_BIND: &str = "127.0.0.1:3001";
const DEFAULT_OLLAMA_MODEL: &str = "llama3.2:latest";
const SYSTEM_PROMPT: &str = "You are a concise voice assistant. Keep answers brief, natural, \
and easy to speak aloud. Ask one short follow-up question when it helps.";

#[derive(Clone)]
struct AppState {
    elevenlabs: ElevenLabsClient,
    api_key: Arc<str>,
    speech_engine_id: Arc<str>,
    ollama_model: Arc<str>,
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl AppError {
    fn new(error: impl Into<anyhow::Error>) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("{:#}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("server error: {}", self.0),
        )
            .into_response()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ConversationTokenResponse {
    token: String,
    #[serde(default, flatten)]
    extra: Map<String, Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "speech_engine_ollama=info,tower_http=info".to_owned()),
        )
        .init();

    let api_key = env::var("ELEVENLABS_API_KEY").context("ELEVENLABS_API_KEY is required")?;
    let elevenlabs = ElevenLabsClient::new(api_key.clone());
    let speech_engine_id = load_or_create_speech_engine(&elevenlabs).await?;
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_OLLAMA_MODEL.to_owned());
    let bind = env::var("SPEECH_ENGINE_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_owned());
    let addr: SocketAddr = bind
        .parse()
        .with_context(|| format!("invalid SPEECH_ENGINE_BIND value: {bind}"))?;

    let state = Arc::new(AppState {
        elevenlabs,
        api_key: Arc::from(api_key),
        speech_engine_id: Arc::from(speech_engine_id),
        ollama_model: Arc::from(ollama_model),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/token", get(conversation_token))
        .route("/ws", get(speech_engine_ws))
        .with_state(state.clone());

    println!("Speech Engine Ollama example");
    println!("  local UI: http://{addr}");
    println!("  upstream WebSocket path: /ws");
    println!("  speech engine id: {}", state.speech_engine_id);
    println!("  ollama model: {}", state.ollama_model);
    println!();
    println!("Open the local UI, click Start conversation, and grant microphone access.");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            info!("shutting down");
        })
        .await?;

    Ok(())
}

async fn load_or_create_speech_engine(client: &ElevenLabsClient) -> Result<String> {
    if let Ok(id) = env::var("ELEVENLABS_SPEECH_ENGINE_ID") {
        return Ok(id);
    }

    let ws_url = env::var("ELEVENLABS_SPEECH_ENGINE_WS_URL").context(
        "set ELEVENLABS_SPEECH_ENGINE_ID for an existing engine, or \
ELEVENLABS_SPEECH_ENGINE_WS_URL to create one",
    )?;

    let body = CreateSpeechEngineBody::new(ws_url)
        .with_name("Rust Ollama Speech Engine")
        .with_tags(["rust", "ollama", "example"]);

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

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn conversation_token(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<ConversationTokenResponse>, AppError> {
    let token = state
        .elevenlabs
        .raw(Method::GET, "/v1/convai/conversation/token")
        .query("agent_id", state.speech_engine_id.as_ref())
        .send_json::<ConversationTokenResponse>()
        .await
        .map_err(AppError::new)?;

    Ok(Json(token))
}

async fn speech_engine_ws(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
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

    if let Err(error) = verify_authorization_token(token, state.api_key.as_ref()) {
        warn!("rejected Speech Engine websocket: {error}");
        return (StatusCode::UNAUTHORIZED, "invalid Speech Engine token").into_response();
    }

    ws.on_upgrade(move |socket| handle_speech_engine_socket(socket, state))
}

async fn handle_speech_engine_socket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Some(frame) = socket.recv().await {
        let frame = match frame {
            Ok(frame) => frame,
            Err(error) => {
                warn!("websocket receive error: {error}");
                break;
            }
        };

        let result = match frame {
            Message::Text(text) => handle_text_message(&mut socket, &state, text.as_str()).await,
            Message::Ping(payload) => socket
                .send(Message::Pong(payload))
                .await
                .map_err(Into::into),
            Message::Close(_) => break,
            Message::Pong(_) => Ok(()),
            Message::Binary(_) => {
                warn!("ignoring unexpected binary Speech Engine frame");
                Ok(())
            }
        };

        if let Err(error) = result {
            error!("{error:#}");
            let _ = socket.send(Message::Close(None)).await;
            break;
        }
    }

    debug!("Speech Engine websocket closed");
}

async fn handle_text_message(socket: &mut WebSocket, state: &AppState, text: &str) -> Result<()> {
    let message: SpeechEngineInboundMessage =
        serde_json::from_str(text).context("failed to decode Speech Engine message")?;

    match message {
        SpeechEngineInboundMessage::Init(init) => {
            info!(
                "Speech Engine conversation started: {}",
                init.conversation_id
            );
        }
        SpeechEngineInboundMessage::UserTranscript(transcript) => {
            respond_to_transcript(
                socket,
                state,
                transcript.user_transcript,
                transcript.event_id,
            )
            .await?;
        }
        SpeechEngineInboundMessage::Ping => {
            send_speech_engine_message(socket, SpeechEngineOutboundMessage::pong()).await?;
        }
        SpeechEngineInboundMessage::Close => {
            info!("Speech Engine requested close");
            socket.send(Message::Close(None)).await?;
        }
        SpeechEngineInboundMessage::Error(error) => {
            warn!("Speech Engine protocol error: {}", error.message);
        }
        SpeechEngineInboundMessage::Unknown(unknown) => {
            debug!(
                "unknown Speech Engine message type: {}",
                unknown.message_type
            );
        }
    }

    Ok(())
}

async fn respond_to_transcript(
    socket: &mut WebSocket,
    state: &AppState,
    transcript: Vec<SpeechEngineTranscriptMessage>,
    event_id: Option<u64>,
) -> Result<()> {
    let Some(user_text) = latest_user_text(&transcript) else {
        debug!("transcript did not contain user text");
        return Ok(());
    };

    info!("user: {user_text}");
    let prompt = build_prompt(&transcript);
    let answer = generate_answer(state, prompt)
        .await
        .unwrap_or_else(|error| {
            error!("Ollama generation failed: {error:#}");
            "I had trouble thinking through that. Please try again.".to_owned()
        });
    info!("agent: {answer}");

    for chunk in speakable_chunks(&answer) {
        send_agent_response(socket, event_id, chunk, false).await?;
    }
    send_agent_response(socket, event_id, "", true).await?;

    Ok(())
}

async fn generate_answer(state: &AppState, prompt: String) -> Result<String> {
    let ollama = ollama::Client::from_env()
        .unwrap_or_else(|_| ollama::Client::new(Nothing).expect("Ollama client should build"));
    let agent = ollama
        .agent(state.ollama_model.as_ref())
        .preamble(SYSTEM_PROMPT)
        .temperature(0.6)
        .build();

    agent.prompt(prompt).await.context("Ollama prompt failed")
}

async fn send_agent_response(
    socket: &mut WebSocket,
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
    send_speech_engine_message(socket, message).await
}

async fn send_speech_engine_message(
    socket: &mut WebSocket,
    message: SpeechEngineOutboundMessage,
) -> Result<()> {
    let text = serde_json::to_string(&message)?;
    socket.send(Message::Text(text.into())).await?;
    Ok(())
}

fn latest_user_text(transcript: &[SpeechEngineTranscriptMessage]) -> Option<&str> {
    transcript
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| message.content.trim())
        .filter(|content| !content.is_empty())
}

fn build_prompt(transcript: &[SpeechEngineTranscriptMessage]) -> String {
    let mut prompt = String::from(
        "Use this voice conversation transcript. Reply only to the latest user message.\n\n",
    );

    let start = transcript.len().saturating_sub(12);
    for message in &transcript[start..] {
        let role = match message.role.as_str() {
            "agent" => "assistant",
            other => other,
        };
        prompt.push_str(role);
        prompt.push_str(": ");
        prompt.push_str(message.content.trim());
        prompt.push('\n');
    }

    prompt
}

fn speakable_chunks(answer: &str) -> impl Iterator<Item = String> + '_ {
    answer
        .split_inclusive(['.', '!', '?'])
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .map(str::to_owned)
}

const INDEX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Speech Engine Ollama</title>
  <style>
    :root { color-scheme: light dark; font-family: system-ui, sans-serif; }
    body { max-width: 720px; margin: 48px auto; padding: 0 20px; line-height: 1.5; }
    button { padding: 10px 14px; margin-right: 8px; font: inherit; }
    #log { margin-top: 24px; padding: 16px; border: 1px solid #8885; min-height: 160px; white-space: pre-wrap; }
  </style>
</head>
<body>
  <h1>Speech Engine Ollama</h1>
  <p>Browser microphone in, ElevenLabs ASR/TTS, + Rig/Ollama agent in the middle.</p>
  <p>Status: <strong id="status">idle</strong></p>
  <button id="start">Start conversation</button>
  <button id="stop" disabled>End conversation</button>
  <div id="log"></div>

  <script type="module">
    import { Conversation } from "https://esm.sh/@elevenlabs/client";

    const status = document.querySelector("#status");
    const log = document.querySelector("#log");
    const start = document.querySelector("#start");
    const stop = document.querySelector("#stop");
    let conversation = null;

    function write(line) {
      log.textContent += `${new Date().toLocaleTimeString()}  ${line}\n`;
    }

    async function getToken() {
      const response = await fetch("/api/token");
      if (!response.ok) {
        throw new Error(await response.text());
      }
      const data = await response.json();
      return data.token;
    }

    start.addEventListener("click", async () => {
      try {
        start.disabled = true;
        status.textContent = "requesting microphone";
        await navigator.mediaDevices.getUserMedia({ audio: true });

        status.textContent = "requesting token";
        const token = await getToken();

        status.textContent = "connecting";
        conversation = await Conversation.startSession({
          conversationToken: token,
          onConnect: () => {
            status.textContent = "connected";
            stop.disabled = false;
            write("connected");
          },
          onDisconnect: () => {
            status.textContent = "disconnected";
            start.disabled = false;
            stop.disabled = true;
            write("disconnected");
          },
          onMessage: (message) => write(JSON.stringify(message)),
          onError: (error) => write(`error: ${error.message ?? error}`),
        });
      } catch (error) {
        status.textContent = "error";
        start.disabled = false;
        stop.disabled = true;
        write(`error: ${error.message ?? error}`);
      }
    });

    stop.addEventListener("click", async () => {
      stop.disabled = true;
      if (conversation) {
        await conversation.endSession();
        conversation = null;
      }
    });
  </script>
</body>
</html>
"##;
