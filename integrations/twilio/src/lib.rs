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
pub use elevenlabs_convai::convai::*;
pub use elevenlabs_convai::error::ConvAIError;
pub use elevenlabs_convai::messages::client_messages::{
    AgentOverrideData, ConversationInitiationClientData, OverrideData, PromptOverrideData,
};
pub use elevenlabs_convai::messages::server_messages::{Audio, ConversationInitiationMetadata};
pub use elevenlabs_convai::{client::AgentWebSocket, messages::server_messages::ServerMessage};
pub use elevenlabs_convai::{DefaultVoice, ElevenLabsClient, LegacyVoice};
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
use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::str::FromStr;
use std::sync::{Arc, LazyLock};
use thiserror::Error;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, info};
use url::Url;
use crate::conversations::GetConversationDetailsResponse;

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
    FailedToReceiveConnectedMessage,
    #[error("unexpected websocket message")]
    FailedToReceiveStartMessage,
    #[error("env var error {0}")]
    EnvVar(#[from] std::env::VarError),
}

#[derive(Clone, Debug)]
pub struct WebSocketStreamManager {
    agent_ws: Arc<Mutex<AgentWebSocket>>,
    twilio_client: TwilioClient,
    convo_init_rx: Arc<Mutex<Option<Receiver<ConversationInitiationClientData>>>>,
}

impl WebSocketStreamManager {
    pub fn new(agent_ws: Arc<Mutex<AgentWebSocket>>, twilio_client: TwilioClient) -> Self {
        WebSocketStreamManager {
            agent_ws,
            twilio_client,
            convo_init_rx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_env() -> Result<Self, Error> {
        let agent_ws = Arc::new(Mutex::new(AgentWebSocket::from_env()?));

        let twilio_client = match TwilioClient::from_env() {
            Ok(c) if c.number().is_some() => c,
            Ok(_) => return Err(Error::TwilioClient(TwilioError::MissingPhoneNumberEnvVar)),
            Err(e) => return Err(Error::TwilioClient(e)),
        };

        Ok(WebSocketStreamManager::new(agent_ws, twilio_client))
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

#[derive(Clone, Debug, Deserialize)]
pub struct NativeExtractor {
    pub caller_id: String,
    pub agent_id: String,
    pub called_number: String,
    pub call_sid: String,
}

pub type Personalization = Json<NativeExtractor>;

#[derive(Clone, Debug)]
pub struct InboundCall {
    pub inner_extractor: Form<TwilioRequestParams>,
    pub convo_init_rx: Arc<Mutex<Option<Receiver<ConversationInitiationClientData>>>>,
}

impl<S> FromRequest<S> for InboundCall
where
    WebSocketStreamManager: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = FormRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        info!("Inbound call request");
        let extractor = Form::<TwilioRequestParams>::from_request(req, state).await?;
        let mut manager = WebSocketStreamManager::from_ref(state);
        let convo_init_rx = manager.convo_init_rx.clone();

        Ok(InboundCall {
            inner_extractor: extractor,
            convo_init_rx,
        })
    }
}

impl InboundCall {
    pub fn answer(self, ws_url: impl Into<String>) -> Result<Response<String>, Error> {
        Ok(VoiceResponse::new()
            .connect(ws_url.into())
            .to_http_response()?)
    }

    pub fn answer_if<P>(
        self,
        ws_url: impl Into<String>,
        predicate: P,
    ) -> Result<Response<String>, Error>
    where
        P: FnOnce(&Form<TwilioRequestParams>) -> bool,
    {
        if predicate(&self.inner_extractor) {
            self.answer(ws_url)
        } else {
            Ok(VoiceResponse::new().reject().to_http_response()?)
        }
    }

    // TODO: change Output to Result as user's async function can fail
    pub fn dynamically_answer<F, Fut>(
        mut self,
        ws_url: impl Into<String>,
        f: F,
    ) -> Result<Response<String>, Error>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ConversationInitiationClientData> + Send,
    {
        tokio::spawn(async move {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            *self.convo_init_rx.lock().await = Some(rx);
            let data = f().await;
            if tx.send(data).await.is_err() {
                error!("Failed to send conversation initiation data to agent websocket");
            }
            info!("Conversation initiation data sent to agent websocket");
        });

        Ok(VoiceResponse::new()
            .connect(ws_url.into())
            .to_http_response()?)
    }
}

#[derive(Clone, Debug)]
pub struct OutboundCall<E> {
    pub inner_extractor: E,
    pub twilio_client: TwilioClient,
    pub number: String,
}

impl<S, E> FromRequest<S> for OutboundCall<E>
where
    WebSocketStreamManager: FromRef<S>,
    S: Send + Sync,
    E: FromRequest<S>,
{
    type Rejection = E::Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        info!("Outbound call request");
        let inner_extractor = E::from_request(req, state).await?;
        let manager = WebSocketStreamManager::from_ref(state);
        let twilio_client = manager.twilio_client.clone();
        let number = twilio_client.number().unwrap().to_string();

        Ok(OutboundCall {
            inner_extractor,
            twilio_client,
            number,
        })
    }
}

impl<S, E> FromRequestParts<S> for OutboundCall<E>
where
    WebSocketStreamManager: FromRef<S>,
    S: Send + Sync,
    E: FromRequestParts<S>,
{
    type Rejection = E::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        info!("Outbound call request");
        let inner_extractor = E::from_request_parts(parts, state).await?;
        let manager = WebSocketStreamManager::from_ref(state);
        let twilio_client = manager.twilio_client.clone();
        let number = twilio_client.number().unwrap().to_string();

        Ok(OutboundCall {
            inner_extractor,
            twilio_client,
            number,
        })
    }
}

impl<E> OutboundCall<E> {
    pub async fn ring(self, to: impl Into<String>, twiml_url: impl Into<String>) -> Result<CallResponse, Error> {
        let body = CreateCallBody::new(to, self.number, TwimlSrc::url(twiml_url));
        let create_call = CreateCall::new(self.twilio_client.account_sid(), body);
        let call_response = self.twilio_client.hit(create_call).await?;
        Ok(call_response)
    }

    /// Use a different Twilio phone number to make the call
    pub async fn ring_from(
        self,
        to: &str,
        from: &str,
        twiml_url: &str,
    ) -> Result<CallResponse, Error> {
        let create_call_body = CreateCallBody::new(to, from, TwimlSrc::url(twiml_url));
        let create_call = CreateCall::new(self.twilio_client.account_sid(), create_call_body);
        let resp = self.twilio_client.hit(create_call).await?;
        Ok(resp)
    }

    //pub async fn ring_if<P>(self, to: &str, twiml_url: &str, predicate: P) -> Option<Result<CallResponse, Error>>
    //where
    //    P: FnOnce(E) -> bool,
    //{
    //    if predicate(self.inner_extractor) {
    //        let create_call_body = CreateCallBody::new(to, self.number, TwimlSrc::url(twiml_url));
    //        let create_call = CreateCall::new(self.twilio_client.account_sid(), create_call_body);
    //        // TODO: handle error
    //        Some(Ok(self.twilio_client.hit(create_call).await.unwrap()))
    //    } else {
    //        None
    //    }
    //}
}

//pub fn set_status_callback(
//    mut self,
//    status_callback: impl Into<String>,
//) -> Result<Self, Error> {
//    self.stream_noun_builder = Some(
//        self.stream_noun_builder
//            .unwrap()
//            .status_callback(status_callback.into())?,
//    );
//    Ok(self)
//}
//
//pub fn set_status_callback_method(mut self, method: impl Into<String>) -> Self {
//    self.stream_noun_builder = Some(
//        self.stream_noun_builder
//            .unwrap()
//            .status_callback_method(method.into()),
//    );
//    self
//}
//
//pub fn set_custom_parameter(
//    mut self,
//    key: impl Into<String>,
//    value: impl Into<String>,
//) -> Self {
//    self.stream_noun_builder = Some(
//        self.stream_noun_builder
//            .unwrap()
//            .parameter(key.into(), value.into()),
//    );
//    self
//}

type ServerMessageCallback = Box<dyn FnMut(ServerMessage) + Send + 'static>;

type TwilioMessageCallback = Box<dyn FnMut(TwilioMessage) + Send + 'static>;

pub struct TelephonyAgent {
    pub agent_ws: Arc<Mutex<AgentWebSocket>>,
    pub twilio_ws: WebSocketUpgrade,
    pub server_message_cb: Option<ServerMessageCallback>,
    pub twilio_message_cb: Option<TwilioMessageCallback>,
    pub convo_init_rx: Option<Receiver<ConversationInitiationClientData>>,
}

impl<S> FromRequestParts<S> for TelephonyAgent
where
    WebSocketStreamManager: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = WebSocketUpgradeRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ws = WebSocketUpgrade::from_request_parts(parts, state).await?;

        let mut manager = WebSocketStreamManager::from_ref(state);
        let agent_ws = Arc::clone(&manager.agent_ws);
        let convo_init_rx = manager.convo_init_rx.lock().await.take();

        Ok(TelephonyAgent {
            twilio_ws: ws,
            agent_ws,
            server_message_cb: None,
            twilio_message_cb: None,
            convo_init_rx,
        })
    }
}

impl TelephonyAgent {
    pub async fn handle_phone_call(mut self) -> Response {
        let ws = self.twilio_ws;
        let agent_ws = self.agent_ws;
        let server_message_cb = self.server_message_cb.take();
        let twilio_message_cb = self.twilio_message_cb.take();
        let convo_init_rx = self.convo_init_rx;

        ws.on_upgrade(move |socket| async move {
            match Self::handle_websockets(
                socket,
                agent_ws,
                server_message_cb,
                twilio_message_cb,
                convo_init_rx,
            )
            .await
            {
                Ok(_) => info!("Established phone call connection"),
                Err(e) => error!("Error: {:?}", e),
            }
        })
    }

    pub async fn handle_websockets(
        mut twilio_socket: WebSocket,
        agent_ws: Arc<Mutex<AgentWebSocket>>,
        mut e_cb: Option<ServerMessageCallback>,
        mut t_cb: Option<TwilioMessageCallback>,
        convo_init_rx: Option<Receiver<ConversationInitiationClientData>>,
    ) -> Result<(), Error>
where {
        let msg = twilio_socket
            .next()
            .await
            .ok_or(Error::FailedToReceiveConnectedMessage)??;

        let msg: ConnectedMessage = serde_json::from_str(msg.to_text()?)?;
        info!("Connected message: {:?}", msg);

        let msg = twilio_socket
            .next()
            .await
            .ok_or(Error::FailedToReceiveStartMessage)??;

        let msg: StartMessage = serde_json::from_str(msg.to_text()?)?;
        let stream_sid = msg.stream_sid;
        info!("Start message metadata: {:?}", msg.start);

        let (twilio_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let twilio_rx = UnboundedReceiverStream::new(rx);

        let (mut twilio_sink, mut twilio_stream) = twilio_socket.split();

        let agent_ws_for_stop = Arc::clone(&agent_ws);

        //// TODO: JoinHandle ?
        tokio::spawn(async move {
            while let Some(msg) = twilio_stream.next().await {
                let msg = msg?;
                let twilio_msg = TwilioMessage::try_from(msg.to_text()?)?;

                if let Some(cb) = t_cb.as_mut() {
                    cb(twilio_msg.clone());
                }

                match twilio_msg {
                    TwilioMessage::Media(media_msg) => {
                        let payload = media_msg.media.payload;
                        if twilio_tx.send(payload).is_err() {
                            error!("failed to send Twilio payload to agent websocket");
                        }
                    }
                    TwilioMessage::Stop(_) => {
                        return match agent_ws_for_stop.lock().await.end_session().await {
                            Ok(_) => {
                                info!("twilio message: stop, agent websocket session ended");
                                Ok(())
                            }
                            Err(e) => {
                                error!("failed to end agent websocket session: {}", e);
                                Err(e.into())
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok::<(), Error>(())
        });

        if let Some(mut rx) = convo_init_rx {
            let convo_init_data = rx
                .recv()
                .await
                .expect("failed to receive conversation initiation data");
            agent_ws
                .lock()
                .await
                .with_conversation_initiation_client_data(convo_init_data);
        }

        let agent_ws_for_convo = Arc::clone(&agent_ws);
        tokio::spawn(async move {
            let mut convai_stream = agent_ws_for_convo
                .lock()
                .await
                .start_session(twilio_rx)
                .await?;

            while let Some(msg_result) = convai_stream.next().await {
                let server_msg = msg_result?;

                if let Some(cb) = e_cb.as_mut() {
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
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostCallPayload {
    pub r#type: String,
    pub data: GetConversationDetailsResponse,
    pub event_timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct PostCall {
    pub inner_extractor: Json<PostCallPayload>
}

impl<S> FromRequest<S> for PostCall
where
    S: Send + Sync,
    WebSocketStreamManager: FromRef<S>,
{
    type Rejection = JsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        info!("Post call request");
        let extractor = Json::<PostCallPayload>::from_request(req, state).await?;
        Ok(PostCall {
            inner_extractor: extractor,
        })
    }
}
