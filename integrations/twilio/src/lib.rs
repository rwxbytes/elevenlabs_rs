use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{FromRequest, Query, State};
use axum::http::{HeaderMap, Request, Uri};
use axum::response::IntoResponse;
use axum::Json;
use elevenlabs_convai::error::ConvAIError;
use elevenlabs_convai::messages::client_messages::{
    AgentOverrideData, ConversationInitiationClientData, OverrideData, PromptOverrideData,
};
use elevenlabs_convai::messages::server_messages::{Audio, ConversationInitiationMetadata};
pub use elevenlabs_convai::{client::AgentWebSocket, messages::server_messages::ServerMessage};
use futures_util::{SinkExt, StreamExt};
pub use rusty_twilio::endpoints::accounts::*;
pub use rusty_twilio::endpoints::applications::*;
use rusty_twilio::endpoints::voice::call::TwimlSrc::Twiml;
pub use rusty_twilio::endpoints::voice::{call::*, stream::*};
pub use rusty_twilio::error::TwilioError;
use rusty_twilio::twiml::voice::StreamBuilder;
pub use rusty_twilio::twiml::voice::VoiceResponse;
pub use rusty_twilio::TwilioClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::ops::ControlFlow;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
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
}

//#[async_trait]
pub trait TelephonyAgent {
    fn agent_ws(&self) -> Arc<Mutex<AgentWebSocket>>;
    fn twilio_client(&self) -> TwilioClient;

    fn number(&self) -> &str;

    fn set_outbound_call_url_twiml(&self) -> &str;
    fn server_message_callback(&self) -> Option<Box<dyn FnMut(ServerMessage) + Send>> {
        None
    }

    fn twilio_message_callback() -> Option<Box<dyn FnMut(TwilioMessage) + Send>> {
        None
    }

    //fn set_outbound_call_twilml_src(&self) -> Option<TwimlSrc> {
    //    None
    //}

    //fn custom_response(&self) -> Option<impl IntoResponse>;
}

pub trait AudioBase64 {
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
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
}


pub async fn initiate_outbound_call<S>(
    State(state): State<S>,
    headers: HeaderMap,
    Json(payload): Json<OutboundCallRequest>,
) -> impl IntoResponse
where
    S: TelephonyAgent + Send + Sync + 'static,
{
    let number = state.number();
    let twilio_client = state.twilio_client();
    // TODO: set query params for prompt and first_message
    //let url = state.set_outbound_call_url_twiml();
    let host = headers.get("host").unwrap().to_str().unwrap();
    let mut url =
        Url::parse(&format!("https://{}/outbound-call-twiml", host,)).expect("Failed to parse url");
    url.set_query(Some(&format!(
        "prompt={}&first_message={}",
        payload.prompt.unwrap_or_default(),
        payload.first_message.unwrap_or_default()
    )));

    let create_call_body =
        CreateCallBody::new(payload.number, number, TwimlSrc::Url(url.to_string()));
    let resp = twilio_client
        .hit(CreateCall::new(
            twilio_client.account_sid(),
            create_call_body,
        ))
        .await
        .unwrap();

    Json(json!({
        "success": true,
        "message": "outbound call initiated",
        "callSid": resp.sid
    }))
}

pub async fn handle_outbound_call_twiml<S>(
    State(state): State<S>,
    Query(mut params): Query<HashMap<String, String>>,
) -> impl IntoResponse
where
    S: TelephonyAgent + Send + Sync + 'static,
{
    let prompt = params.remove("prompt").unwrap_or_default();
    let first_message = params.remove("first_message").unwrap_or_default();
    let url = state.set_outbound_call_url_twiml();
    let mut builder = StreamBuilder::new()
        .url(url)
        .expect("Failed to create stream noun");

    if !prompt.is_empty() {
        builder = builder.parameter("prompt", prompt);
    }

    if !first_message.is_empty() {
        builder = builder.parameter("first_message", first_message);
    }

    let stream_noun = builder.build().expect("Failed to build stream noun");

    let twiml = VoiceResponse::new()
        .connect(stream_noun)
        .to_string()
        .unwrap();
    info!("[TwiML] Generated Response : {}", &twiml);
    // TODO: make application/xml content type
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

pub async fn handle_phone_call<S>(
    mut socket: WebSocket,
    State(state): State<S>,
) -> Result<(), Error>
where
    S: TelephonyAgent + Send + Sync + 'static,
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
    let stream_sid = msg.start.stream_sid;
    let params = msg.start.custom_parameters;

    let mut overrides = OverrideData::default();
    let mut agent_override = AgentOverrideData::default();

    if let Some(prompt) = params.get("prompt") {
        agent_override = agent_override.with_prompt_override_data(
            PromptOverrideData::default().override_prompt(prompt.to_string()),
        );
    }

    if let Some(first_message) = params.get("first_message") {
        agent_override = agent_override.override_first_message(first_message.to_string());
    }

    overrides = overrides.with_agent_override_data(agent_override);

    let mut init_data = ConversationInitiationClientData::default();
    init_data.with_override_data(overrides);

    let mut agent_ws = state.agent_ws();

    let mut guard = agent_ws.lock().await;
    guard.with_conversation_initiation_client_data(init_data);

    let (twilio_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_rx = UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = socket.split();

    // Spawn task for incoming Twilio messages.
    let agent_ws = Arc::clone(&state.agent_ws());
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

    let mut cb = state.server_message_callback();

    let agent_ws_for_convo = Arc::clone(&state.agent_ws());
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
