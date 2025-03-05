use axum::response::Response;
use axum::routing::get;
use axum::{extract::FromRef, response::IntoResponse, routing::post, Router};
use elevenlabs_twilio::{
    AgentOverrideData, ConversationInitiationClientData, InboundCall, OverrideData, TelephonyAgent,
    WebSocketStreamManager,
};
use tracing::info;

#[derive(Debug, Clone)]
    struct AppState {
    ws_manager: WebSocketStreamManager,
}

impl FromRef<AppState> for WebSocketStreamManager {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.ws_manager.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let ws_manager = WebSocketStreamManager::from_env()?;

    let app_state = AppState { ws_manager };

    let app = Router::new()
        //.route("/inbound-call", post(inbound_call))
        //.route("/inbound-call", post(dynamic_inbound_call))
        .route("/inbound-call", post(inbound_call))
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

// TODO: some macro like init_convo!
async fn inbound_call(inbound_call: InboundCall) -> impl IntoResponse {
    let ws_url = "wss://7c4e-86-18-8-153.ngrok-free.app/ws";
    let f = async move || {
        let agent_overrride =
            AgentOverrideData::default().override_first_message("In Rust We Trust");
        let mut init = ConversationInitiationClientData::default();
        init.with_override_data(OverrideData::default().with_agent_override_data(agent_overrride));
        info!("sent conversation initiation from inbound call");
        init
    };
    inbound_call.dynamically_answer(ws_url, f).unwrap()
}

async fn answer_if(inbound_call: InboundCall) -> impl IntoResponse {
    let ws_url = "wss://02d8-86-18-8-153.ngrok-free.app/ws";
    inbound_call
        .answer_if(ws_url, |req| req.from == "some_number")
        .unwrap()
}

async fn dynamic_inbound_call(inbound_call: InboundCall) -> impl IntoResponse {
    let ws_url = "wss://02d8-86-18-8-153.ngrok-free.app/ws";
    let caller_id = inbound_call.inner_extractor.from.clone();
    let f = async move || {
        let agent_overrride =
            AgentOverrideData::default().override_first_message("In Rust We Trust");
        let mut init = ConversationInitiationClientData::default();
        init.with_override_data(OverrideData::default().with_agent_override_data(agent_overrride));
        info!("sent conversation initiation from inbound call");
        init
    };
    inbound_call.dynamically_answer(ws_url, f).unwrap()
}

async fn ws_handler(agent: TelephonyAgent) -> Response {
    agent.handle_phone_call().await
}
