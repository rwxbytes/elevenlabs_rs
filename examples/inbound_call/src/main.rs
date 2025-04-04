use std::sync::Arc;
use axum::http::StatusCode;
use axum::{extract::FromRef, response::IntoResponse, routing::post, Router};
use tokio::sync::Mutex;
use elevenlabs_twilio::{AgentWebSocket, ConversationInitiationClientData, Personalization, PostCall, TelephonyState, TwilioClient};
use tracing::info;

#[derive(Debug, Clone)]
struct AppState {
    telephony_state: TelephonyState,
}

impl FromRef<AppState> for TelephonyState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.telephony_state.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let agent = Arc::new(Mutex::new(AgentWebSocket::from_env()?));
    let tc = Arc::new(TwilioClient::from_env()?);

    let sub_state = TelephonyState::new("inbound_agent".to_string(), agent, tc)?;

    let app_state = AppState {
        telephony_state: sub_state,
    };

    // Change routes according to your Conversational AI workspace settings
    let app = Router::new()
        // See Personalization https://elevenlabs.io/docs/conversational-ai/guides/twilio/dynamic-calls
        .route("/inbound-call", post(native_inbound_call))
        // See Post-call webhook https://elevenlabs.io/docs/conversational-ai/workflows/post-call-webhooks
        .route("/post_call", post(post_call_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn native_inbound_call(data: Personalization) -> impl IntoResponse {
    info!("inbound call from {}", data.caller_id);
    axum::Json(ConversationInitiationClientData::default())
}

async fn post_call_handler(post_call: PostCall) -> impl IntoResponse {
    let summary = post_call.summary();

    match summary {
        Some(summary) => println!("Summary: {:?}", summary),
        None => println!("No summary available"),
    }

    (StatusCode::OK, "Webhook received")
}
