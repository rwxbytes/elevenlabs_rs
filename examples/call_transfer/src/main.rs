mod handlers;
mod helpers;
mod toolkit;

use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    extract::FromRef, response::{IntoResponse, Response}, routing::{get, post},
    Form,
    Json,
    Router,
};
use dotenv::dotenv;
use elevenlabs_twilio::agents::DynamicVar;
use elevenlabs_twilio::TwilioClientExt;
use elevenlabs_twilio::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, field, info, instrument, warn, Instrument, Span};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// --- Agent Names/Types (Registered in TelephonyState) ---
const ASSESSMENT_AGENT: &str = "assessment_agent";
const WARM_TRANSFER_AGENT: &str = "warm_transfer_agent";
const WAIT_MANAGEMENT_AGENT: &str = "wait_management_agent";

// --- Agent Tool Names ---
const TOOL_PUT_CALLER_IN_CONFERENCE: &str = "put_caller_in_conference";
const TOOL_PUT_HUMAN_OPERATOR_IN_CONFERENCE: &str = "put_human_operator_in_conference";
const SYSTEM_TOOL_END_CALL: &str = "end_call";

// --- Participant Labels ---
const LABEL_CALLER: &str = "Caller";
const LABEL_PARTICIPANT_B: &str = "Participant B";
const LABEL_WAIT_MANAGER: &str = "Wait Manager";

// --- URL Paths ---
const PATH_INBOUND: &str = "/inbound-call";
const PATH_WS_CALLER: &str = "/caller";
const PATH_WS_PARTICIPANT_B: &str = "/participant_b";
const PATH_EVENTS_CONFERENCE: &str = "/events/conference";
const PATH_EVENTS_CALL: &str = "/events/call";
const PATH_POST_CALL: &str = "/post_call";
const PATH_AMD: &str = "/amd";
const PATH_WAIT_MANAGER: &str = "/wait_manager";

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub telephony_state: TelephonyState,
    pub conference_state: Arc<Mutex<ConferenceState>>,
    pub call_transfer_state: Arc<Mutex<HashMap<String, CallTransferData>>>,
    pub to_be_notified: Arc<Mutex<VecDeque<CallTransferData>>>,
}

impl FromRef<AppState> for TelephonyState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.telephony_state.clone()
    }
}
#[derive(Clone, Debug)]
pub struct CallTransferData {
    //pub caller_name: String,
    pub caller_call_sid: Option<String>,
    pub conference_name: String,
    pub retry_count: u8,
    pub max_retries: u8, // signify it as attempts instead ?
    pub participant_b_phone_number: String,
    pub warm_transfer_agent_phone_number: String,
    pub customer_summary: String,
}
impl CallTransferData {
    pub fn to_convo_initiation_client_data(&self) -> ConversationInitiationClientData {
        let mut dyn_vars = HashMap::new();
        //dyn_vars.insert(
        //    "caller_name".to_string(),
        //    DynamicVar::new_string(self.caller_name.clone()),
        //);
        dyn_vars.insert(
            "summary".to_string(),
            DynamicVar::new_string(self.customer_summary.clone()),
        );

        ConversationInitiationClientData::default().with_dynamic_variables(dyn_vars)
    }
}

#[derive(Debug, Default)]
pub struct ConferenceState {
    // Conference Name -> Conference Parameters
    conferences: HashMap<String, ConferenceData>,
}

#[derive(Clone, Debug)]
pub struct ConferenceData {
    params: Option<ConferenceRequestParams>,
    caller_call_sid: Option<String>,
}

// not needed jus use methods on the map
impl ConferenceState {
    pub async fn add_conference(&mut self, name: String, caller_call_sid: String) {
        self.conferences.insert(
            name,
            ConferenceData {
                params: None,
                caller_call_sid: Some(caller_call_sid),
            },
        );
    }

    pub async fn get_conference(&self, name: &str) -> Option<ConferenceData> {
        self.conferences.get(name).cloned()
    }

    pub async fn update_conference_params(&mut self, name: &str, params: ConferenceRequestParams) {
        if let Some(conf) = self.conferences.get_mut(name) {
            conf.params = Some(params);
        }
    }

    pub async fn remove_conference(&mut self, name: &str) {
        self.conferences.remove(name);
    }

    pub async fn get_caller_call_sid(&self, name: &str) -> Option<String> {
        self.conferences
            .get(name)
            .and_then(|data| data.caller_call_sid.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub https_ngrok_base_url: String,
    pub wss_ngrok_base_url: String,
    pub assessment_agent_id: String,
    pub assessment_agent_phone_number: String,
    pub warm_transfer_agent_id: String,
    pub warm_transfer_agent_phone_number: String,
    pub wait_management_agent_id: String,
    pub wait_management_agent_phone_number: String,
    pub caller_phone_number: String,
    pub participant_b_phone_number: String,
    pub max_call_transfer_retries: u8,
    pub participant_b_call_timeout_secs: u32,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        Config {
            https_ngrok_base_url: env::var("HTTPS_NGROK_BASE_URL")
                .expect("NGROK_BASE_URL must be set"),
            wss_ngrok_base_url: env::var("WSS_NGROK_BASE_URL")
                .expect("WSS_NGROK_BASE_URL must be set"),
            assessment_agent_id: env::var("ASSESSMENT_AGENT_ID").unwrap_or_default(),
            assessment_agent_phone_number: env::var("ASSESSMENT_AGENT_PHONE_NUMBER")
                .expect("ASSESSMENT_AGENT_PHONE_NUMBER must be set"),
            warm_transfer_agent_id: env::var("WARM_TRANSFER_AGENT_ID")
                .expect("WARM_TRANSFER_AGENT_ID must be set"),
            warm_transfer_agent_phone_number: env::var("WARM_TRANSFER_AGENT_PHONE_NUMBER")
                .expect("WARM_TRANSFER_AGENT_PHONE_NUMBER must be set"),
            wait_management_agent_id: env::var("WAIT_MANAGEMENT_AGENT_ID")
                .expect("WAIT_MANAGEMENT_AGENT_ID must be set"),
            wait_management_agent_phone_number: env::var("WAIT_MANAGEMENT_AGENT_PHONE_NUMBER")
                .expect("WAIT_MANAGEMENT_AGENT_PHONE_NUMBER must be set"),
            caller_phone_number: env::var("CALLER_PHONE_NUMBER")
                .expect("CALLER_PHONE_NUMBER must be set"),
            participant_b_phone_number: env::var("PARTICIPANT_B_PHONE_NUMBER")
                .expect("PARTICIPANT_B_PHONE_NUMBER must be set"),
            max_call_transfer_retries: env::var("MAX_CALL_TRANSFER_RETRIES")
                .unwrap_or("1".to_string())
                .parse()
                .unwrap(),
            participant_b_call_timeout_secs: env::var("PARTICIPANT_B_CALL_TIMEOUT_SECS")
                .unwrap_or("15".to_string())
                .parse()
                .unwrap(),
        }
    }
}

//#[derive(Debug)]
//pub enum AppError {
//    TelephonyError(Error),
//    Twilio(TwilioError),
//}

//impl IntoResponse for AppError {
//    fn into_response(self) -> Response {
//        match self {
//            AppError::TelephonyError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
//            AppError::Twilio(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
//        }
//    }
//}
//
//type Result<T> = std::result::Result<T, AppError>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("Tracing initialized. Starting application...");

    let config = Config::from_env();

    let elevenlabs_api_key =
        env::var("ELEVENLABS_API_KEY").expect("ELEVENLABS_API_KEY must be set");

    let twilio_client = Arc::new(TwilioClient::from_env()?);

    // Register an Agent to handle inbound calls, i.e. the callers
    let agent_ws = Arc::new(Mutex::new(AgentWebSocket::new(
        &elevenlabs_api_key,
        &config.assessment_agent_id,
    )));
    let mut telephony_state =
        TelephonyState::new(ASSESSMENT_AGENT.to_string(), agent_ws, twilio_client)?;

    // Register an Agent to handle outbound calls, i.e. to the person the caller wants to speak to
    let warm_transfer_agent =
        AgentWebSocket::new(&elevenlabs_api_key, &config.warm_transfer_agent_id);
    telephony_state
        .register_agent_ws(WARM_TRANSFER_AGENT.to_string(), warm_transfer_agent)
        .await?;

    // Register an Agent to enter the conference and notify the caller after max attempts of trying to
    // reach the person the caller wants to speak to
    let wait_management_agent =
        AgentWebSocket::new(&elevenlabs_api_key, &config.wait_management_agent_id);
    telephony_state
        .register_agent_ws(WAIT_MANAGEMENT_AGENT.to_string(), wait_management_agent)
        .await?;

    let app_state = AppState {
        config,
        telephony_state,
        conference_state: Arc::new(Mutex::new(ConferenceState::default())),
        call_transfer_state: Arc::new(Mutex::new(HashMap::new())),
        to_be_notified: Arc::new(Mutex::new(VecDeque::new())),
    };

    let router = Router::new()
        .route(PATH_INBOUND, post(handlers::events::inbound_call_handler))
        .route(PATH_WS_CALLER, get(handlers::agents::inbound_agent_handler))
        .route(
            PATH_WS_PARTICIPANT_B,
            get(handlers::agents::outbound_agent_handler),
        )
        .route(
            PATH_EVENTS_CONFERENCE,
            post(handlers::events::conference_events_handler),
        )
        .route(
            PATH_WAIT_MANAGER,
            get(handlers::agents::wait_management_agent_handler),
        )
        .route(
            PATH_EVENTS_CALL,
            post(handlers::events::participant_b_callback_handler),
        )
        .route(PATH_POST_CALL, post(handlers::events::post_call_handler))
        .route(PATH_AMD, post(handlers::events::amd_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await?;
    info!(local_addr = %listener.local_addr()?, "Listening for connections");
    axum::serve(listener, router).await?;

    Ok(())
}

#[derive(Clone, Debug)]
struct CustomerConversation {
    //pub caller_name: String,
    pub summary: String,
    pub conference_friendly_name: String,
}
// TODO: add a similar system prompt to the call forwarding agent:
//
// You have just been speaking to a caller named {{caller_name}}, who you have put on hold as they requested to speak to a human operator.
// You are now speaking to a human operator. Your task is to brief them about the conversation you just had with the customer.
// Here is the summary of the conversation:
//
// {{summary}}
impl From<CustomerConversation> for ConversationInitiationClientData {
    fn from(customer_conversation: CustomerConversation) -> Self {
        let mut dyn_vars = HashMap::new();
        //dyn_vars.insert(
        //    "caller_name".to_string(),
        //    DynamicVar::new_string(customer_conversation.caller_name),
        //);
        dyn_vars.insert(
            "summary".to_string(),
            DynamicVar::new_string(customer_conversation.summary),
        );
        dyn_vars.insert(
            "conf_name".to_string(),
            DynamicVar::new_string(customer_conversation.conference_friendly_name),
        );
        ConversationInitiationClientData::default().with_dynamic_variables(dyn_vars)
    }
}
