use axum::extract::FromRef;
use axum::response::IntoResponse;
use axum::{extract::ws::{Message, WebSocket, WebSocketUpgrade}, extract::State, response::Response, routing::{get, post}, Form, Json, Router};
use elevenlabs_twilio::ServerMessage::{
    AgentResponse, ConversationInitiationMetadata, UserTranscript,
};
use elevenlabs_twilio::{AgentWebSocket, Error, OutboundCall, ServerMessage, TelephonyAgent, TwilioClient, TwilioMessage, TwilioRequestParams, VoiceResponse, WebSocketStreamManager};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, instrument, span, Level};

#[derive(Clone, Debug)]
pub struct AppState {
    web_socket_stream_manager: WebSocketStreamManager,
}

// TODO: put in Arc & Mutex ?
impl FromRef<AppState> for WebSocketStreamManager {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.web_socket_stream_manager.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserRequest {
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let agent_ws = Arc::new(Mutex::new(AgentWebSocket::from_env()?));
    let twilio_client = TwilioClient::from_env()?;
    let web_socket_stream_manager = WebSocketStreamManager::new(agent_ws, twilio_client);

    let app_state = AppState {
        web_socket_stream_manager,
    };

    let router = Router::new()
        .route("/ring", post(ring))
        .route("/twiml", post(handle_outbound_call_twiml))
        .route("/ws", get(handle_media_stream))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}

pub async fn handle_outbound_call_twiml(_: Form<TwilioRequestParams>) -> impl IntoResponse {
    let url = "wss://02d8-86-18-8-153.ngrok-free.app/ws";

    let twiml = VoiceResponse::new()
        .connect(url)
        .to_http_response()
        .expect("Failed to create TwiML");

    info!("sent connect TwiML");
    twiml
}

async fn ring(outbound_call: OutboundCall<Json<UserRequest>>) -> impl IntoResponse {
    let number_to_ring = outbound_call.inner_extractor.number.clone();
    let url = "https://02d8-86-18-8-153.ngrok-free.app/twiml";
    let resp = outbound_call.ring(&number_to_ring, url).await.unwrap();

    println!("ring response: {:?}", &resp);

    Json(json!({
        "success": true,
        "message": "outbound call initiated",
        "callSid": resp.sid
    }))
}

async fn handle_media_stream(mut agent: TelephonyAgent) -> Response {
    let cb = move |msg: ServerMessage| match msg {
        AgentResponse(inner) => {
            println!(
                "agent response: {:?}",
                inner.agent_response_event.agent_response
            );
        }
        UserTranscript(inner) => {
            println!(
                "user transcript: {:?}",
                inner.user_transcription_event.user_transcript
            );
        }
        _ => {}
    };

    agent.server_message_cb = Some(Box::new(cb));
    agent.handle_phone_call().await
}
