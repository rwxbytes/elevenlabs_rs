use crate::db_types::{revisiting_customer, Customer, DB_SETUP};
use crate::twilio::{ClearMessage, MediaMessage, StartMessage, TwilioMessage};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use chrono::Utc;
use elevenlabs_convai::client::ElevenLabsAgentClient;
use elevenlabs_convai::messages::client_messages::{
    AgentOverrideData, ConversationInitiationClientData, OverrideData, PromptOverrideData,
};
use elevenlabs_convai::messages::server_messages::ServerMessage;
use elevenlabs_rs::endpoints::convai::agents::DynamicVar;
use futures_util::{SinkExt, StreamExt};
use prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

mod agent;
mod db_types;
mod handlers;
mod prelude;
mod twilio;

#[derive(Clone)]
pub struct AppState {
    caller: Arc<Mutex<Option<twilio::Caller>>>,
    client: Arc<Mutex<ElevenLabsAgentClient>>,
    revisiting_customer: Arc<Mutex<Option<Customer>>>,
    db: Surreal<Client>,
    ngrok_url: String,
}

// TODO: Update the NGROK_URL to your current ngrok URL
const NGROK_URL: &str = "https://yoursubdomain.ngrok-free.app";

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let db = Surreal::new::<Ws>("localhost:8000").await?;

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("ns").use_db("db").await?;

    let _resp = db.query(DB_SETUP).await?;

    let apikey = std::env::var("ELEVENLABS_API_KEY")?;
    let agent = agent::create_agent(NGROK_URL).await?;
    let client = ElevenLabsAgentClient::new(apikey, agent.agent_id);

    let app_state = AppState {
        caller: Arc::new(Mutex::new(None)),
        client: Arc::new(Mutex::new(client)),
        revisiting_customer: Arc::new(Mutex::new(None)),
        ngrok_url: NGROK_URL.to_string(),
        db,
    };

    let app = Router::new()
        .route("/inbound-call", post(handlers::twiml))
        .route("/tables", post(handlers::list_available_tables))
        .route("/reservation", post(handlers::create_reservation))
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give Rustalicious a call");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(state, socket))
}

async fn handle_socket(state: AppState, mut socket: WebSocket) {
    let caller = state.caller.lock().await.clone().unwrap();

    let mut dyn_vars = HashMap::new();
    let dyn_var = DynamicVar::new_string(Utc::now().to_rfc2822());
    dyn_vars.insert("datetime".to_string(), dyn_var);
    let init_client_data =
        ConversationInitiationClientData::default().with_dynamic_variables(dyn_vars);

    state
        .client
        .lock()
        .await
        .with_conversation_initiation_client_data(init_client_data);

    if let Some(customer) = revisiting_customer(&state.db, &caller.caller).await {
        let dyn_var = DynamicVar::new_string(&customer.name);
        let mut client = state.client.lock().await;
        let init_client_data = client.init_data_mut().unwrap();

        init_client_data
            .dynamic_variables
            .as_mut()
            .unwrap()
            .insert("customer".to_string(), dyn_var);

        let prompt_override_data = PromptOverrideData::default()
            .override_prompt(agent::ATTITUDE_TOWARDS_REVISITING_CUSTOMER);

        let agent_override = AgentOverrideData::default()
            .override_first_message(agent::FIRST_MSG_FOR_REVISITING_CUSTOMER)
            .with_prompt_override_data(prompt_override_data);

        let override_data = OverrideData::default().with_agent_override_data(agent_override);

        init_client_data.with_override_data(override_data);

        *state.revisiting_customer.lock().await = Some(customer);
    }

    let client = Arc::clone(&state.client);
    let client_2 = Arc::clone(&state.client);

    // Skip connected message
    socket.next().await;

    // Get stream_sid
    let stream_sid = match socket.next().await {
        Some(Ok(Message::Text(msg))) => {
            let start_msg = serde_json::from_str::<StartMessage>(&msg).unwrap();
            start_msg.stream_sid
        }
        _ => panic!("Expected start message with stream sid"),
    };

    let (twilio_payload_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_payload_rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = socket.split();

    let twilio_task: JoinHandle<Result<(), AppError>> = tokio::spawn(async move {
        while let Some(Ok(msg)) = twilio_stream.next().await {
            let txt = msg.to_text()?;
            let twilio_msg = TwilioMessage::try_from(txt)?;
            match twilio_msg {
                TwilioMessage::Media(media_msg) => {
                    let payload = media_msg.media.payload;
                    if twilio_payload_tx.send(payload).is_err() {
                        println!("Receptionist has ended the call");
                        break;
                    }
                }
                TwilioMessage::Stop(_) => {
                    client_2.lock().await.stop_conversation().await?;
                    println!("Caller has ended the call");
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    });

    let el_task: JoinHandle<Result<(), AppError>> = tokio::spawn(async move {
        let mut convai_stream = client
            .lock()
            .await
            .start_conversation(twilio_payload_rx)
            .await?;

        while let Some(msg_result) = convai_stream.next().await {
            let server_msg = msg_result.unwrap();
            match server_msg {
                ServerMessage::Audio(audio) => {
                    let payload = audio.audio_event.audio_base_64;
                    let media_msg = MediaMessage::new(&stream_sid, &payload);
                    let json = serde_json::to_string(&media_msg)?;
                    twilio_sink.send(Message::Text(json.into())).await?;
                }
                ServerMessage::Interruption(_) => {
                    let clear_msg = ClearMessage::new(&stream_sid);
                    let json = serde_json::to_string(&clear_msg)?;
                    twilio_sink.send(Message::Text(json.into())).await?;
                }
                _ => {}
            }
        }
        Ok(())
    });

    tokio::select! {
        _ = twilio_task => {
            println!("Twilio task ended");
        }
        _ = el_task => {
            println!("Elevenlabs task ended");
        }
    }
}
