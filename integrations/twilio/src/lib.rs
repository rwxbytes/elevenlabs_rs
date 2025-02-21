pub use async_trait::async_trait;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{FromRef, FromRequest, FromRequestParts, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Request, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
pub use elevenlabs_convai::error::ConvAIError;
pub use elevenlabs_convai::messages::client_messages::{
    AgentOverrideData, ConversationInitiationClientData, OverrideData, PromptOverrideData,
};
pub use elevenlabs_convai::messages::server_messages::{Audio, ConversationInitiationMetadata};
pub use elevenlabs_convai::{client::AgentWebSocket, messages::server_messages::ServerMessage};
use futures_util::{SinkExt, StreamExt};
pub use rusty_twilio::endpoints::accounts::*;
pub use rusty_twilio::endpoints::applications::*;
pub use rusty_twilio::endpoints::voice::call::TwimlSrc::Twiml;
pub use rusty_twilio::endpoints::voice::{call::*, stream::*};
pub use rusty_twilio::error::TwilioError;
pub use rusty_twilio::twiml::voice::StreamBuilder;
pub use rusty_twilio::twiml::voice::VoiceResponse;
pub use rusty_twilio::TwilioClient;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::ops::ControlFlow;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, info};
use url::Url;

#[derive(Error, Debug)]
pub enum Error {
    #[error("agent websocket error")]
    AgentWebSocket(#[from] ConvAIError),
    #[error("twilio client error")]
    TwilioClient(#[from] TwilioError),
    #[error("axum error")]
    Axum(#[from] axum::Error),
    #[error("serde error")]
    Serde(#[from] serde_json::Error),
    #[error("unexpected websocket message")]
    FailedToReceiveStartMessage,
    #[error("env var error {0}")]
    EnvVar(#[from] std::env::VarError),
}

#[async_trait]
pub trait TelephonyAgent: Send + Sync {
    fn agent_ws(&self) -> Arc<Mutex<AgentWebSocket>>;
    fn twilio_client(&self) -> TwilioClient;

    fn number(&self) -> &str;

    fn on_server_message(&self) -> Option<Box<dyn FnMut(ServerMessage) + Send>> {
        None
    }

    fn on_twilio_message(&self) -> Option<Box<dyn FnMut(TwilioMessage) + Send>> {
        None
    }

    async fn on_gather(
        &self,
        gather: Gather,
    ) -> Option<Result<Response<String>, (StatusCode, String)>> {
        let _ = gather;
        None
    }

    async fn on_outbound_call_req(
        &self,
        outbound_call_req: OutboundCallRequest,
    ) -> Option<Result<Response<String>, (StatusCode, String)>> {
        let _ = outbound_call_req;
        None
    }

    //fn set_outbound_call_twilml_src(&self) -> Option<TwimlSrc> {
    //    None
    //}

}

pub trait HasTelephonyAgent: Send + Sync {
    type Agent: TelephonyAgent;

    fn telephony_agent(&self) -> &Self::Agent;
}

trait AudioBase64 {
    fn audio_base_64(&self) -> &str;
}

impl AudioBase64 for Audio {
    fn audio_base_64(&self) -> &str {
        &self.audio_event.audio_base_64
    }
}

impl AudioBase64 for MediaMessage {
    fn audio_base_64(&self) -> &str {
        &self.media.payload
    }
}

pub trait ToTwilio {
    fn to_twilio(self) -> Result<Message, Error>;
}

impl<T, U> ToTwilio for (T, U)
where
    T: AudioBase64,
    U: Into<String>,
{
    fn to_twilio(self) -> Result<Message, Error> {
        let (audio, stream_sid) = self;
        let media_msg = MediaMessage::new(stream_sid.into(), audio.audio_base_64());
        let json = serde_json::to_string(&media_msg)?;
        Ok(Message::Text(json.into()))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutboundCallRequest {
    // TODO: Validate phone number
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
}

pub async fn initiate_outbound_call(outbound_call_req: OutboundCallFromHost) -> impl IntoResponse {
    let call_response = outbound_call_req.call_response;
    info!(
        "initiated outbound call to: {}, call sid {}",
        &call_response.to, &call_response.sid
    );

    Json(json!({
        "success": true,
        "message": "outbound call initiated",
        "callSid": call_response.sid
    }))
}

pub struct OutboundCallFromHost {
    call_response: CallResponse,
}

impl<S> FromRequest<S> for OutboundCallFromHost
where
    //OutboundAgent: FromRef<S>,
    S: HasTelephonyAgent + Send + Sync,
{
    // TODO: implement Rejection
    type Rejection = JsonRejection;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let host = req
            .headers()
            .get("host")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        //let mut state = OutboundAgent::from_ref(state);
        let Json(payload) = Json::<OutboundCallRequest>::from_request(req, &state).await?;
        let agent = state.telephony_agent();
        let from = agent.number().to_string();
        let to = payload.number;
        let twilio_client = agent.twilio_client();
        let agent_ws = state.agent_ws();

        {
            let mut guard = agent_ws.lock().await;
            let mut overrides = OverrideData::default();
            let mut agent_override = AgentOverrideData::default();

            if let Some(first_message) = payload.first_message {
                info!("Overriding first message");
                agent_override = agent_override.override_first_message(first_message);
            }

            if let Some(prompt) = payload.prompt {
                info!("Overriding prompt");
                agent_override = agent_override.with_prompt_override_data(
                    PromptOverrideData::default().override_prompt(prompt),
                );
            }

            overrides = overrides.with_agent_override_data(agent_override);
            let mut init_data = ConversationInitiationClientData::default();
            init_data.with_override_data(overrides);

            guard.with_conversation_initiation_client_data(init_data);
            info!("Set conversation initiation client data");
        }

        // TODO: path ought not be hardcoded
        let url = format!("https://{}/outbound-call-twiml", host);

        let create_call_body = CreateCallBody::new(to, from, TwimlSrc::Url(url));

        let create_call = CreateCall::new(twilio_client.account_sid(), create_call_body);

        let call_response = twilio_client
            .hit(create_call)
            .await
            .expect("Failed to create call");
        info!("created call");

        Ok(OutboundCallFromHost { call_response })
    }
}

//pub async fn return_connect_stream() -> impl IntoResponse {
//    let twiml = VoiceResponse::new()
//        .connect(Stream::new("stream-id"))
//        .to_string()
//        .unwrap();
//    info!("[TwiML] Generated Response : {}", &twiml);
//}

pub async fn handle_outbound_call_twiml<S>(State(state): State<S>) -> impl IntoResponse
where
    S: HasTelephonyAgent + Send + Sync,
{
    let mut agent = state.telephony_agent();

    //let host = "dbd2-86-18-8-153.ngrok-free.app";
    //// TODO: user may set ws_path with '/' prefix, so we should handle this
    //let url = format!("wss://{}/{}", host, state.ws_path);

    let twiml = VoiceResponse::new()
        .connect(url)
        .to_http_response()
        .expect("Failed to create TwiML");
    info!("sent connect TwiML");
    twiml
}

async fn handle_twilio_message(
    msg: Message,
    agent_ws: &Arc<Mutex<AgentWebSocket>>,
    tx: &tokio::sync::mpsc::UnboundedSender<String>,
) -> ControlFlow<()> {
    match msg {
        Message::Text(txt) => {
            // Handle potential JSON parsing error
            let msg = match serde_json::from_str(txt.as_str()) {
                Ok(parsed_msg) => parsed_msg,
                Err(e) => {
                    error!("failed to parse Twilio message: {}", e);
                    return ControlFlow::Break(());
                }
            };

            match msg {
                TwilioMessage::Media(media_msg) => {
                    let payload = media_msg.media.payload;
                    if tx.send(payload).is_err() {
                        error!("failed to send Twilio payload to agent websocket");
                        return ControlFlow::Break(());
                    }
                }
                TwilioMessage::Stop(_) => {
                    match agent_ws.lock().await.end_session().await {
                        Ok(_) => {
                            info!("twilio message: stop, agent websocket session ended");
                        }
                        Err(e) => {
                            error!("failed to end agent websocket session: {}", e);
                        }
                    }
                    return ControlFlow::Break(());
                }
                TwilioMessage::Mark(_) => {
                    unimplemented!("mark message")
                }
                TwilioMessage::Dtmf(_) => {
                    unimplemented!("dtmf message")
                }
                _ => {}
            }
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Continue(()),
    }
}

// TODO: join the threads spawned in this function ?
pub async fn handle_phone_call<S>(
    mut socket: WebSocket,
    State(state): State<S>,
) -> Result<(), Error>
where
    S: HasTelephonyAgent, //OutboundAgent: FromRef<S>,
                          //S: Send + Sync,
                          //S: TelephonyAgent + Send + Sync + 'static,
{
    if let Some(Ok(msg)) = socket.next().await {
        let msg: ConnectedMessage = serde_json::from_str(msg.to_text()?)?;
        info!("Connected message: {:?}", msg);
    }

    let msg = socket
        .next()
        .await
        .ok_or(Error::FailedToReceiveStartMessage)??;

    let msg: StartMessage = serde_json::from_str(msg.to_text()?)?;
    let stream_sid = msg.stream_sid;
    info!("Start message metadata: {:?}", msg.start);

    //let state = OutboundAgent::from_ref(&state);

    let (twilio_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_rx = UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = socket.split();

    let agent = state.telephony_agent();

    // Spawn task for incoming Twilio messages.
    //let agent_ws = Arc::clone(&state.agent_ws());
    let agent_ws = agent.agent_ws();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = twilio_stream.next().await {
            if handle_twilio_message(msg, &agent_ws, &twilio_tx)
                .await
                .is_break()
            {
                break;
            }
        }
    });

    let mut cb = agent.on_server_message();

    //let agent_ws_for_convo = Arc::clone(&state.agent_ws());
    let agent_ws_for_convo = agent.agent_ws();
    // TODO: trace within this block
    tokio::spawn(async move {
        let mut convai_stream = agent_ws_for_convo
            .lock()
            .await
            .start_session(twilio_rx)
            .await?;

        while let Some(msg_result) = convai_stream.next().await {
            let server_msg = msg_result?;

            if let Some(cb) = cb.as_mut() {
                cb(server_msg.clone());
            }

            match server_msg {
                ServerMessage::Audio(audio) => {
                    twilio_sink.send((audio, &stream_sid).to_twilio()?).await?;
                }
                ServerMessage::Interruption(_interrupt) => {
                    let clear_msg = ClearMessage::new(&stream_sid);
                    let json = serde_json::to_string(&clear_msg)?;
                    twilio_sink.send(Message::Text(json.into())).await?;
                }
                _ => {}
            }
        }
        Ok::<(), Error>(())
    });
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Gather {
    pub digits: String,
    pub call_sid: String,
    pub from: String,
    pub speech_results: Option<String>,
    pub confidence: Option<String>,
}

pub struct GatherExtractor {
    connect_stream: Response<String>,
}

impl<S> FromRequest<S> for GatherExtractor
where
    S: HasTelephonyAgent,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(payload) = Form::<Gather>::from_request(req, &state)
            .await
            .map_err(|e| (e.status(), e.body_text()))?;
        let mut agent = state.telephony_agent();
        let voice_response = match agent.on_gather(payload).await {
            Some(Ok(voice_response)) => voice_response,
            Some(Err(err)) => return Err(err),
            None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "No response".to_string())),
        };

        Ok(GatherExtractor {
            connect_stream: voice_response,
        })
    }
}

pub async fn gather_action(gather: GatherExtractor) -> impl IntoResponse {
    gather.connect_stream
    //gather
    //    .connect_stream
    //    .to_http_response()
    //    .expect("Failed to create TwiML")
}
