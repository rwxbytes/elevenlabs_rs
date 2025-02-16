use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use elevenlabs_twilio::{
    AgentWebSocket, CreateCall, CreateCallBody, OutboundAgent, ServerMessage, StatusCallbackEvent,
    TelephonyAgent, TwilioClient, TwimlSrc,
};
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::env::var;
use std::future::Future;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, span, Level};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("environment variable not set: {0}")]
    EnvVarError(String),

    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("websocket error: {0}")]
    WebSocketError(#[from] axum::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("stream SID not found")]
    StreamSidNotFound,

    #[error("twilio message parse error: {0}")]
    TwilioMessageParseError(String),

    #[error("tokio join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    //#[error("conversational_ai error: {0}")]
    //ConversationalError(#[from] ConvAIError),
    #[error("send error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<String>),
}

type Result<T> = std::result::Result<T, AppError>;

#[derive(Clone)]
pub struct AppState {
    agent: Arc<Mutex<OutboundAgent>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let ngrok_url = "https://c8dc-86-18-8-153.ngrok-free.app";
    let ngrok_ws = "wss://c8dc-86-18-8-153.ngrok-free.app";

    let agent_ws = AgentWebSocket::from_env().unwrap();
    let twilio_client = TwilioClient::from_env().unwrap();
    let outbound_agent = OutboundAgent::new(agent_ws, twilio_client)
        .set_twiml_src(&format!("{}/outbound-call", ngrok_url))
        .unwrap()
        .set_twiml_for_connection(&format!("{}/ws", ngrok_ws))
        .unwrap();

    //let body = CreateCallBody::new("to", "from", TwimlSrc::Url("url".to_string()))
    //    .with_status_callback("status_callback".to_string())
    //    .with_status_callback_event(StatusCallbackEvent::Initiated)
    //    .with_status_callback_event(StatusCallbackEvent::Ringing)
    //    .with_status_callback_event(StatusCallbackEvent::Completed)
    //    .with_status_callback_event(StatusCallbackEvent::Answered);

    //dbg!(&body);

    let state = AppState {
        agent: Arc::new(Mutex::new(outbound_agent)),
    };

    let app = Router::new()
        .route("/ring", post(ring))
        .route("/outbound-call", post(twiml))
        .route("/cb", post(status_callback))
        .route("/cbe", post(status_callback_event))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct Ring {
    to: String,
    from: String,
    url: String,
}

async fn ring(State(state): State<AppState>, Json(payload): Json<Ring>) -> impl IntoResponse {
    let body = CreateCallBody::new(payload.to, payload.from, TwimlSrc::Url(payload.url));

    let _call_resp = state
        .agent
        .lock()
        .await
        .ring(body)
        .await
        .expect("Failed to ring");
    "Ringing"
}

async fn status_callback(body: String) -> impl IntoResponse {
    info!("Status Callback: {}", body);
    StatusCode::OK
}

async fn status_callback_event(body: String) -> impl IntoResponse {
    info!("Status Callback Event: {}", body);
    StatusCode::OK
}

// TODO: get calle data
async fn twiml(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let twiml = agent.get_twiml_for_connection().unwrap();
    twiml
}

async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(state, socket))
}

async fn handle_socket(state: AppState, socket: WebSocket) {
    let span = span!(Level::INFO, "handle_socket");
    let _enter = span.enter();

    //
    //let callback = |msg: ServerMessage| match msg {
    //    ServerMessage::AgentResponse(msg) => {
    //        info!(
    //            "received agent response: {}",
    //            msg.agent_response_event.agent_response
    //        );
    //    }
    //    ServerMessage::UserTranscript(msg) => {
    //        info!(
    //            "received user transcript: {}",
    //            msg.user_transcription_event.user_transcript
    //        );
    //    }
    //    _ => {}
    //};

    //let callback: Option<Box<dyn FnMut(ServerMessage) + Send>> = Some(Box::new(callback));

    match state.agent.lock().await.talk(socket).await {
        Ok(_) => info!("phone call started"),
        Err(e) => error!("Error: {:?}", e),
    }
}
