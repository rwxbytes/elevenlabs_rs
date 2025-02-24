pub use async_trait::async_trait;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{
    FromRef, FromRequest, FromRequestParts, Query, Request, State, WebSocketUpgrade,
};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode, Uri};
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
use rusty_twilio::twiml::voice::StreamNounBuilder;
pub use rusty_twilio::twiml::voice::*;
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

#[derive(Clone, Debug)]
pub struct WebSocketStreamManager {
    agent_ws: Arc<Mutex<AgentWebSocket>>,
    twilio_client: TwilioClient,
}

impl WebSocketStreamManager {
    pub fn new(agent_ws: Arc<Mutex<AgentWebSocket>>, twilio_client: TwilioClient) -> Self {
        WebSocketStreamManager {
            agent_ws,
            twilio_client,
        }
    }
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
pub async fn handle_phone_call<S>(mut socket: WebSocket, state: &S) -> Result<(), Error>
where
    //S: HasTelephonyAgent,
    WebSocketStreamManager: FromRef<S>,
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

    let state = WebSocketStreamManager::from_ref(&state);

    let (twilio_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_rx = UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = socket.split();

    let agent_ws = Arc::clone(&state.agent_ws);

    // Spawn task for incoming Twilio messages.
    //let agent_ws = Arc::clone(&state.agent_ws());
    //let agent_ws = agent.agent_ws();
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

    //let mut cb = agent.on_server_message();

    //let agent_ws_for_convo = Arc::clone(&state.agent_ws());
    //let agent_ws_for_convo = agent.agent_ws();
    let agent_ws_for_convo = Arc::clone(&state.agent_ws);
    // TODO: trace within this block
    tokio::spawn(async move {
        let mut convai_stream = agent_ws_for_convo
            .lock()
            .await
            .start_session(twilio_rx)
            .await?;

        while let Some(msg_result) = convai_stream.next().await {
            let server_msg = msg_result?;

            //if let Some(cb) = cb.as_mut() {
            //    cb(server_msg.clone());
            //}

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

pub struct OverrideExtractor<F = DefaultCallBack> {
    connect_stream: Response<String>,
    callback: F,
    agent_ws: Arc<Mutex<AgentWebSocket>>,
    request: Request,
    call_response: Option<CallResponse>,
}

impl<S> FromRequest<S> for OverrideExtractor
where
    WebSocketStreamManager: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let manager = WebSocketStreamManager::from_ref(state);

        let voice_response = VoiceResponse::new()
            .connect("wss://531f-86-18-8-153.ngrok-free.app/ws")
            .to_http_response()
            .expect("Failed to create TwiML");

        Ok(OverrideExtractor {
            connect_stream: voice_response,
            callback: DefaultCallBack,
            agent_ws: Arc::clone(&manager.agent_ws),
            request: req,
            call_response: None,
        })
    }
}

impl<F> OverrideExtractor<F> {
    pub async fn on_req<C, Fut>(self, callback: C) -> Response<String>
    where
        C: FnOnce(Request) -> Fut + Send + 'static,
        Fut: Future<Output = ConversationInitiationClientData> + Send + 'static,
    {
        let init_client_data = callback(self.request).await;
        let mut agent = self.agent_ws.lock().await;
        agent.with_conversation_initiation_client_data(init_client_data);
        self.connect_stream
    }
}

pub struct DefaultCallBack;

impl DefaultCallBack {
    async fn default_callback(_req: Request) -> ConversationInitiationClientData {
        ConversationInitiationClientData::default()
    }
}

trait Direction {}

#[derive(Debug, Default)]
pub struct Inbound;

#[derive(Debug, Default)]
pub struct Outbound;

impl Direction for Inbound {}

impl Direction for Outbound {}

pub type InboundAgent<E> = TelephonyAgent<E, Inbound>;

pub type OutboundAgent<E> = TelephonyAgent<E, Outbound>;

pub struct TelephonyAgent<E, D: Direction> {
    direction: D,
    inner_extractor: E,
    agent_ws: Arc<Mutex<AgentWebSocket>>,
    twilio_client: TwilioClient,
    to: String,
    from: String,
    stream_noun_builder: StreamNounBuilder,
    twiml: Option<Response<String>>,
}

impl<S, E, D: Direction + Default> FromRequest<S> for TelephonyAgent<E, D>
where
    WebSocketStreamManager: FromRef<S>,
    E: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = E::Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let manager = WebSocketStreamManager::from_ref(state);
        let extractor = E::from_request(req, state).await?;

        let agent_ws = Arc::clone(&manager.agent_ws);
        let twilio_client = manager.twilio_client;
        // TODO: panics if number is None, `TwilioClient::from_env()` does not enforce a number
        let from = twilio_client.number().unwrap().to_string();
        let to = "1234".to_string();
        let stream_noun_builder = StreamNounBuilder::new();
        let twiml = None;

        Ok(TelephonyAgent {
            direction: Default::default(),
            inner_extractor: extractor,
            agent_ws,
            twilio_client,
            from,
            to,
            stream_noun_builder,
            twiml,
        })
    }
}

impl<S, E, D: Direction + Default> FromRequestParts<S> for TelephonyAgent<E, D>
where
    WebSocketStreamManager: FromRef<S>,
    E: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = E::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let manager = WebSocketStreamManager::from_ref(state);
        let extractor = E::from_request_parts(parts, state).await?;

        let agent_ws = Arc::clone(&manager.agent_ws);
        let twilio_client = manager.twilio_client;
        let from = twilio_client.number().unwrap().to_string();
        let to = "1234".to_string();
        let stream_noun_builder = StreamNounBuilder::new();
        let twiml = None;

        Ok(TelephonyAgent {
            direction: Default::default(),
            inner_extractor: extractor,
            agent_ws,
            twilio_client,
            from,
            to,
            stream_noun_builder,
            twiml,
        })
    }
}

impl<E, D: Direction> TelephonyAgent<E, D> {
    pub fn build_twiml(mut self) -> Result<Self, Error> {
        let stream_noun = self.stream_noun_builder.clone().build()?;
        let twiml = VoiceResponse::new()
            .connect(stream_noun)
            .to_http_response()?;
        self.twiml = Some(twiml);
        Ok(self)
    }
    pub fn set_ws_url(mut self, ws_url: impl Into<String>) -> Result<Self, Error> {
        self.stream_noun_builder = self.stream_noun_builder.url(ws_url.into())?;
        Ok(self)
    }

    pub fn set_status_callback(
        mut self,
        status_callback: impl Into<String>,
    ) -> Result<Self, Error> {
        self.stream_noun_builder = self
            .stream_noun_builder
            .status_callback(status_callback.into())?;
        Ok(self)
    }

    pub fn set_status_callback_method(mut self, method: impl Into<String>) -> Self {
        self.stream_noun_builder = self
            .stream_noun_builder
            .status_callback_method(method.into());
        self
    }

    pub fn set_custom_parameter(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.stream_noun_builder = self.stream_noun_builder.parameter(key.into(), value.into());
        self
    }
}

impl<E> TelephonyAgent<E, Inbound> {
    pub fn answer(self) -> Option<Response<String>> {
        self.twiml
    }

    pub fn answer_if<P>(self, predicate: P) -> Option<Response<String>>
    where
        P: FnOnce(E) -> bool,
    {
        if predicate(self.inner_extractor) {
            self.twiml
        } else {
            None
        }
    }
}

impl<E> TelephonyAgent<E, Outbound> {
    pub fn replace_from(self, from: &str) -> Self {
        TelephonyAgent {
            from: from.to_string(),
            ..self
        }
    }

    pub fn replace_to(self, to: &str) -> Self {
        TelephonyAgent {
            to: to.to_string(),
            ..self
        }
    }
    pub async fn ring_and_then<F, T>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce(CallResponse) -> T,
    {
        let from = self.from;
        let to = &self.to;
        let url = "https://532e-86-18-8-153.ngrok-free.app/twiml";
        let create_call_body = CreateCallBody::new(to, from, TwimlSrc::url(url));
        let create_call = CreateCall::new(self.twilio_client.account_sid(), create_call_body);
        let call_response = self.twilio_client.hit(create_call).await?;
        Ok(f(call_response))
    }
    pub async fn ring_from(self, from: &str) -> Result<CallResponse, Error> {
        let to = &self.from;
        let url = "https://f359-86-18-8-153.ngrok-free.app/ring";
        let create_call_body = CreateCallBody::new(to, from, TwimlSrc::url(url));
        let create_call = CreateCall::new(self.twilio_client.account_sid(), create_call_body);
        let resp = self.twilio_client.hit(create_call).await?;
        Ok(resp)
    }
    pub async fn ring_if<P>(self, predicate: P) -> Option<Result<CallResponse, Error>>
    where
        P: FnOnce(E) -> bool,
    {
        if predicate(self.inner_extractor) {
            let from = self.twilio_client.number().unwrap();
            let to = &self.from.to_string();
            let url = "https://f359-86-18-8-153.ngrok-free.app/ring";
            let create_call_body = CreateCallBody::new(to, from, TwimlSrc::url(url));
            let create_call = CreateCall::new(self.twilio_client.account_sid(), create_call_body);
            Some(Ok(self.twilio_client.hit(create_call).await.unwrap()))
        } else {
            None
        }
    }

    pub async fn ring_to(self, to: &str) -> Result<CallResponse, Error> {
        let from = self.twilio_client.number().unwrap();
        let url = "https://f359-86-18-8-153.ngrok-free.app/ring";
        let create_call_body = CreateCallBody::new(to, from, TwimlSrc::url(url));
        let create_call = CreateCall::new(self.twilio_client.account_sid(), create_call_body);
        let resp = self.twilio_client.hit(create_call).await?;
        Ok(resp)
    }
}
