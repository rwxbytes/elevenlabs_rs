use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    extract::FromRef, response::{IntoResponse, Response}, routing::{get, post},
    Form,
    Json,
    Router,
};
use elevenlabs_twilio::agents::DynamicVar;
use elevenlabs_twilio::TwilioClient;
use elevenlabs_twilio::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub struct AppState {
    pub telephony_state: TelephonyState,
    pub customer_conversation: Arc<Mutex<Option<CustomerConversation>>>,
    pub conference_states: Arc<Mutex<HashMap<String, ConferenceRequestParams>>>,
    pub convo_to_call_sid: Arc<Mutex<HashMap<String, String>>>,
}

impl FromRef<AppState> for TelephonyState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.telephony_state.clone()
    }
}

static WS_URL: &str = "";
static NGROK_URL: &str = "";

const PUT_CALLER_IN_CONFERENCE: &str = "put_caller_in_conference";
const PUT_HUMAN_OPERATOR_IN_CONFERENCE: &str = "put_human_operator_in_conference";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let app_state = AppState {
        telephony_state: TelephonyState::from_env()?,
        customer_conversation: Arc::new(Mutex::new(None)),
        conference_states: Arc::new(Mutex::new(HashMap::new())),
        convo_to_call_sid: Arc::new(Mutex::new(HashMap::new())),
    };

    let router = Router::new()
        .route("/inbound-call", post(inbound_call_handler))
        .route("/ws", get(agent_handler))
        .route("/events/conference", post(conference_events_handler))
        .route("/events/participant", post(participant_events_handler))
        .route("/amd_callback", post(amd_callback_handler))
        .route("/post_call", post(post_call_handler))
        .route("/contact_staff", get(contact_staff_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}

async fn inbound_call_handler(inbound_call: InboundCall) -> impl IntoResponse {
    match inbound_call
        .answer(WS_URL)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    {
        Ok(voice_response) => voice_response,
        Err(err) => {
            error!("Error: {:?}", err);
            err.into_response()
        }
    }
}

async fn agent_handler(State(state): State<AppState>, mut agent: TelephonyAgent) -> Response {
    // TODO: set agent_id for agent handling customer calls
    let (tools_tx, mut tools_rx) = tokio::sync::mpsc::unbounded_channel();

    agent.tools_tx = Some(tools_tx);
    let agent_ws = agent.agent_ws.clone();
    let twilio_c = agent.twilio_client.clone();

    let convo_to_call_sid = state.convo_to_call_sid.clone();

    tokio::spawn(async move {
        while let Some((tool_call, call_sid, convo_id)) = tools_rx.recv().await {
            match tool_call.name() {
                PUT_CALLER_IN_CONFERENCE => {
                    let init_conf = Conference::new("Room 11");
                    let update_conf = Conference {
                        participant_label: Some("Customer".to_string()),
                        start_conference_on_enter: Some(false),
                        end_conference_on_exit: Some(true),
                        status_callback: Some(format!("{}/events/conference", NGROK_URL)),
                        status_callback_event: Some(String::from("start end join leave")),
                        ..init_conf
                    };

                    let twiml = match VoiceResponse::new().dial(update_conf).to_string() {
                        Ok(twiml) => twiml,
                        Err(e) => {
                            error!("Error creating TwiML for call transfer tool: {:?}", e);
                            let faulty_tool = ClientToolResult::new(tool_call.id())
                                .is_error(true)
                                .with_result("tool call failed".to_string());

                            if let Err(e) =
                                agent_ws.lock().await.send_tool_result(faulty_tool).await
                            {
                                error!("Error sending tool result: {:?}", e);
                            } else {
                                info!("Faulty Tool result sent successfully");
                            }
                            continue;
                        }
                    };

                    let update_call_body = UpdateCallBody::twiml(&twiml);

                    let endpoint =
                        UpdateCall::new(twilio_c.account_sid(), call_sid, update_call_body);

                    // Put caller in conference room,
                    // Twilio sends a Stop message to the agent websocket, and ends current call
                    let resp = twilio_c.hit(endpoint).await;

                    match resp {
                        Ok(r) => {
                            info!("Call updated successfully");
                            let mut convo_to_call_sid = convo_to_call_sid.lock().await;
                            if let Some(convo_id) = convo_id {
                                info!("Inserting call_sid into convo_to_call_sid");
                                convo_to_call_sid.insert(convo_id, r.sid.clone());
                            } else {
                                error!("No conversation ID found");
                            }
                        }
                        Err(e) => {
                            error!("Error updating call: {:?}", e);
                            continue;
                        }
                    };
                }
                _ => {
                    error!("Unknown tool call: {:?}", tool_call);
                }
            }
        }
    });

    agent.handle_phone_call().await
}

// TODO: end conference if adding participant fails from this handler
async fn conference_events_handler(
    State(state): State<AppState>,
    conference_event: Form<ConferenceRequestParams>,
) -> impl IntoResponse {
    info!("Conference event: {:#?}", conference_event);

    let mut conference_states = state.conference_states.lock().await;

    if let Some(call_sid) = &conference_event.call_sid {
        match conference_states.insert(call_sid.clone(), conference_event.0) {
            Some(_) => {
                info!("Conference state updated");
            }
            None => {
                info!("New conference state added");
            }
        }
    } else {
        info!(
            "conference event without call_sid {:?}",
            conference_event.friendly_name
        );
    }

    (StatusCode::OK, "Webhook received")
}

async fn participant_events_handler(
    participant_event: Form<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Participant event: {:?}", participant_event);
    (StatusCode::OK, "Webhook received")
}

async fn amd_callback_handler(amd_callback: Form<HashMap<String, String>>) -> impl IntoResponse {
    info!("AMD Callback: {:?}", amd_callback);
    (StatusCode::OK, "Webhook received")
}

async fn post_call_handler(
    State(state): State<AppState>,
    post_call: PostCall,
) -> impl IntoResponse {
    info!("Post call: {:#?}", post_call.payload);

    let convo_id = post_call.payload.data.conversation_id.clone();
    let mut convo_to_call_sid = state.convo_to_call_sid.lock().await;
    let call_sid = convo_to_call_sid.remove(&convo_id);

    if let Some(call_sid) = call_sid {
        let mut conf_state = state.conference_states.lock().await;
        if let Some(conf_state) = conf_state.get(&call_sid) {
            let conf_friendly_name = conf_state.friendly_name.clone().unwrap();
            let mut customer_conversation = state.customer_conversation.lock().await;
            *customer_conversation = Some(CustomerConversation {
                payload: post_call.payload.clone(),
                conference_friendly_name: conf_friendly_name,
            });

            let twilio_c = TwilioClient::from_env().expect("TwilioClient creation failed");
            let contact_staff_url = "";
            let stream_noun = Stream::new(contact_staff_url);
            let twiml = VoiceResponse::new()
                .connect(stream_noun)
                .to_string()
                .expect("Failed to create TwiML");

            let create_call_body = CreateCallBody {
                to: "",
                from: "",
                twiml: Some(&twiml),
                ..Default::default()
            };

            let endpoint = CreateCall::new(twilio_c.account_sid(), create_call_body);
            match twilio_c.hit(endpoint).await {
                Ok(resp) => {
                    info!("Call created successfully: {:#?}", resp);
                }
                Err(e) => {
                    error!("Error creating call: {:?}", e);
                }
            }
        }
    } else {
        error!("No call_sid found for conversation ID: {}", convo_id);
    }

    (StatusCode::OK, "Webhook received")
}

async fn contact_staff_handler(
    State(state): State<AppState>,
    mut agent: TelephonyAgent,
) -> impl IntoResponse {
    let mut agent_ws = agent.agent_ws.clone();
    agent_ws.lock().await.with_agent_id("");
    let customer_conversation = state.customer_conversation.lock().await;
    let conf_sid = customer_conversation
        .as_ref()
        .and_then(|c| Some(c.conference_friendly_name.clone()));

    if let Some(customer_conversation) = customer_conversation.clone() {
        let init_data = customer_conversation.into();
        agent_ws
            .lock()
            .await
            .with_conversation_initiation_client_data(init_data);
    } else {
        error!("No customer conversation found");
    }

    let (tools_tx, mut tools_rx) = tokio::sync::mpsc::unbounded_channel();

    agent.tools_tx = Some(tools_tx);
    let agent_ws = agent.agent_ws.clone();
    let twilio_c = agent.twilio_client.clone();

    tokio::spawn(async move {
        while let Some((tool_call, call_sid, _)) = tools_rx.recv().await {
            match tool_call.name() {
                PUT_HUMAN_OPERATOR_IN_CONFERENCE => {
                    let conf_sid = conf_sid.clone().unwrap_or_default();
                    info!("Conference sid: {}", conf_sid);

                    let init_conf = Conference::new(conf_sid);
                    let update_conf = Conference {
                        participant_label: Some("Human".to_string()),
                        start_conference_on_enter: Some(true),
                        end_conference_on_exit: Some(true),
                        ..init_conf
                    };

                    let twiml = match VoiceResponse::new().dial(update_conf).to_string() {
                        Ok(twiml) => twiml,
                        Err(e) => {
                            error!("Error creating TwiML for call transfer tool: {:?}", e);
                            let faulty_tool = ClientToolResult::new(tool_call.id())
                                .is_error(true)
                                .with_result("tool call failed".to_string());

                            if let Err(e) =
                                agent_ws.lock().await.send_tool_result(faulty_tool).await
                            {
                                error!("Error sending tool result: {:?}", e);
                            } else {
                                info!("Faulty Tool result sent successfully");
                            }
                            continue;
                        }
                    };

                    let update_call_body = UpdateCallBody::twiml(&twiml);

                    let endpoint =
                        UpdateCall::new(twilio_c.account_sid(), call_sid, update_call_body);

                    // Put caller in conference room,
                    // Twilio sends a Stop message to the agent websocket, and ends current call
                    let resp = twilio_c.hit(endpoint).await;

                    match resp {
                        Ok(r) => {
                            info!("Call updated successfully {:#?}", r);
                        }
                        Err(e) => {
                            error!("Error updating call: {:?}", e);
                            continue;
                        }
                    };
                }
                _ => {
                    error!("Unknown tool call: {:?}", tool_call);
                    let faulty_tool = ClientToolResult::new(tool_call.id())
                        .is_error(true)
                        .with_result("tool call failed".to_string());

                    if let Err(e) = agent_ws.lock().await.send_tool_result(faulty_tool).await {
                        error!("Error sending tool result: {:?}", e);
                    } else {
                        info!("Faulty Tool result sent successfully");
                    }
                }
            }
        }
    });

    agent.handle_phone_call().await
}

#[derive(Clone, Debug)]
struct CustomerConversation {
    pub payload: PostCallPayload,
    pub conference_friendly_name: String,
}
// TODO: add a similar system prompt to the call forwarding agent
// You have just been speaking to a customer, who you have put on hold as they requested to speak to a human operator.
// You are now speaking to a human operator. Your task is to brief them about the conversation you just had with the customer.
// Here is the summary of the conversation:
//
// {{summary}}
impl From<CustomerConversation> for ConversationInitiationClientData {
    fn from(customer_conversation: CustomerConversation) -> Self {
        let post_call_payload = customer_conversation.payload;

        let mut dyn_vars = HashMap::new();
        let summary = post_call_payload
            .data
            .analysis
            .clone()
            .unwrap()
            .transcript_summary;
        let dyn_summary = DynamicVar::new_string(summary);
        dyn_vars.insert("summary".to_string(), dyn_summary);
        let init_data =
            ConversationInitiationClientData::default().with_dynamic_variables(dyn_vars);

        init_data
    }
}
