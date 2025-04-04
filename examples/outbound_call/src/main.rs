use axum::http::StatusCode;
use axum::{
    extract::FromRef,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use elevenlabs_twilio::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct AppState {
    telephony_state: TelephonyState,
}

impl FromRef<AppState> for TelephonyState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.telephony_state.clone()
    }
}

static WS_URL: &str = "wss://your-ngrok/ws";
static TWIML_CONNECT_URL: &str = "https://your-ngrok/twiml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let agent = Arc::new(Mutex::new(AgentWebSocket::from_env()?));
    let tc = Arc::new(TwilioClient::from_env()?);

    let sub_state = TelephonyState::new("outbound_agent".to_string(), agent, tc)?;

    let app_state = AppState {
        telephony_state: sub_state,
    };

    let router = Router::new()
        .route("/ring", post(outbound_call_handler))
        .route("/twiml", post(twiml_handler))
        .route("/ws", get(agent_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserRequest {
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
}

impl From<UserRequest> for ConversationInitiationClientData {
    fn from(user_request: UserRequest) -> Self {
        let mut init = ConversationInitiationClientData::default();
        let mut overrides = OverrideData::default();

        if let Some(prompt) = user_request.prompt {
            let msg = AgentOverrideData::default().override_first_message(prompt);
            overrides = overrides.with_agent_override_data(msg);
        }
        if let Some(first_message) = user_request.first_message {
            let msg = AgentOverrideData::default().override_first_message(first_message);
            overrides = overrides.with_agent_override_data(msg);
        }
        init.with_override_data(overrides);
        init
    }
}

async fn outbound_call_handler(
    outbound_call: OutboundCall<Json<UserRequest>>,
) -> impl IntoResponse {
    let user_request = outbound_call.as_inner().0.clone();
    let number = user_request.number.clone();
    let f = move || ConversationInitiationClientData::from(user_request);

    let resp = outbound_call
        .ring_and_config(&number, TWIML_CONNECT_URL, f)
        .await
        .unwrap();

    Json(json!({
        "success": true,
        "message": "outbound call initiated",
        "callSid": resp.sid,
    }))
}

async fn twiml_handler(params: TwilioParams) -> impl IntoResponse {
    match params.connect(WS_URL) {
        Ok(twilml) => twilml.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn agent_handler(mut agent: TelephonyAgent) -> Response {
    if let Err(e) = agent.set_agent_ws("outbound_agent").await {
        error!("Error setting agent websocket for outbound agent: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let cb = |msg: ServerMessage| match msg {
        ServerMessage::AgentResponse(inner) => {
            println!(
                "agent response: {}",
                inner.agent_response_event.agent_response
            );
        }
        ServerMessage::UserTranscript(inner) => {
            println!(
                "user transcript: {}",
                inner.user_transcription_event.user_transcript
            );
        }
        _ => {}
    };

    agent.server_message_cb = Some(Box::new(cb));

    match agent.handle_phone_call().await {
        Ok(response) => {
            info!("WebSocket upgrade response generated");
            response
        }
        Err(e) => {
            error!("Error handling phone call: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
