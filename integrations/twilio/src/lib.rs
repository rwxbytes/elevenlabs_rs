use crate::conversations::GetConversationDetailsResponse;
use axum::body::Body;
use axum::extract::rejection::FormRejection;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{FromRef, FromRequest, FromRequestParts, Request, WebSocketUpgrade};
use axum::http::request::Parts;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use chrono::Utc;
use dashmap::DashMap;
pub use elevenlabs_convai::convai::*;
pub use elevenlabs_convai::error::ConvAIError;
pub use elevenlabs_convai::messages::client_messages::{
    AgentOverrideData, ClientToolResult, ConversationInitiationClientData, OverrideData,
    PromptOverrideData,
};
pub use elevenlabs_convai::messages::server_messages::{Audio, ConversationInitiationMetadata};
pub use elevenlabs_convai::{client::AgentWebSocket, messages::server_messages::*};
pub use elevenlabs_convai::{DefaultVoice, ElevenLabsClient, LegacyVoice};
use futures_util::{SinkExt, StreamExt};
pub use rusty_twilio::endpoints::voice::{call::*, conference::*, stream::*};
pub use rusty_twilio::error::TwilioError;
pub use rusty_twilio::request_parameters::*;
pub use rusty_twilio::twiml::voice::{Conference, Number, Parameter, Stream, VoiceResponse};
pub use rusty_twilio::validation::SignatureValidationError;
pub use rusty_twilio::{TwilioClient, TwilioClientExt};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, field, info, instrument, warn, Span};

#[derive(Error, Debug)]
pub enum Error {
    #[error("agent websocket error: {0}")]
    AgentWebSocket(#[from] ConvAIError),
    #[error("twilio client error: {0}")]
    TwilioClient(#[from] TwilioError),
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("unexpected websocket message")]
    FailedToReceiveConnectedMessage,
    #[error("unexpected websocket message")]
    FailedToReceiveStartMessage,
    #[error("env var error: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("no agent websocket has been registered with the key {0}")]
    AgentWebSocketNotFound(String),
    #[error("an agent websocket with the key {0} already exists")]
    AgentWebSocketAlreadyExists(String),
    #[error("an agent websocket must be set to handle the phone call")]
    AgentWebSocketNotSet,
}

// TODO: Rename ?
// AgentCallState
// AgentPhoneManager
#[derive(Clone, Debug)]
pub struct TelephonyState {
    // or DashMap?
    pub agent_registry: Arc<Mutex<HashMap<String, Arc<Mutex<AgentWebSocket>>>>>,
    pub convo_init_data_map: Arc<DashMap<String, ConversationInitiationClientData>>,
    pub twilio_client: Arc<TwilioClient>,
    webhook_secret: Option<SecretString>,
}

impl TelephonyState {
    pub fn new(
        key: String,
        agent_ws: Arc<Mutex<AgentWebSocket>>,
        twilio_client: Arc<TwilioClient>,
    ) -> Result<Self, Error> {
        if twilio_client.number().is_none() {
            return Err(Error::TwilioClient(TwilioError::MissingPhoneNumberEnvVar));
        }

        let mut agents = HashMap::new();
        agents.insert(key, agent_ws);
        let agent_registry = Arc::new(Mutex::new(agents));

        let convo_init_data_map = Arc::new(DashMap::new());
        let webhook_secret = std::env::var("WEBHOOK_SECRET").ok().map(SecretString::from);

        Ok(TelephonyState {
            agent_registry,
            convo_init_data_map,
            twilio_client,
            webhook_secret,
        })
    }

    pub fn with_webhook_secret(mut self, secret: &str) -> Self {
        self.webhook_secret = Some(SecretString::from(secret));
        self
    }

    pub async fn register_agent_ws(
        &self,
        key: String,
        agent_ws: AgentWebSocket,
    ) -> Result<(), Error> {
        let mut agent_registry = self.agent_registry.lock().await;
        if agent_registry.contains_key(&key) {
            return Err(Error::AgentWebSocketAlreadyExists(key));
        }
        agent_registry.insert(key, Arc::new(Mutex::new(agent_ws)));
        Ok(())
    }

    pub async fn get_agent_ws(&self, key: &str) -> Option<Arc<Mutex<AgentWebSocket>>> {
        let agents = self.agent_registry.lock().await;
        agents.get(key).cloned()
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

#[derive(Debug, Clone)]
pub struct TwilioParams {
    pub params: TwilioRequestParams,
    pub twilio_client: Arc<TwilioClient>,
}

impl TwilioParams {
    pub fn connect(self, ws_url: impl Into<String>) -> Result<Response, Error> {
        Ok(VoiceResponse::new()
            .connect(Stream::new(ws_url.into()))
            .to_http_response()?
            .map(Body::from))
    }

    pub fn connect_stream(self, stream: Stream) -> Result<Response, Error> {
        Ok(VoiceResponse::new()
            .connect(stream)
            .to_http_response()?
            .map(Body::from))
    }

    pub fn from_map(map: BTreeMap<String, String>) -> Result<TwilioRequestParams, String> {
        let params = serde_qs::from_str::<TwilioRequestParams>(
            &serde_qs::to_string(&map).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(params)
    }

    pub fn params(&self) -> &TwilioRequestParams {
        &self.params
    }

    pub fn into_params(self) -> TwilioRequestParams {
        self.params
    }

    pub async fn create_call(self, body: CreateCallBody<'_>) -> Result<CallResponse, Error> {
        let create_call = CreateCall::new(self.twilio_client.account_sid(), body);
        let resp = self.twilio_client.hit(create_call).await?;
        Ok(resp)
    }
}

impl Deref for TwilioParams {
    type Target = TwilioRequestParams;

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}

impl<S> FromRequest<S> for TwilioParams
where
    TelephonyState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = TwilioValidationRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let t_state = TelephonyState::from_ref(state);
        let twilio_client = &t_state.twilio_client;

        let method = req.method().clone();
        let uri = req.uri().clone();
        let headers = req.headers().clone();

        let form_data: BTreeMap<String, String> = Form::from_request(req, state)
            .await
            .map_err(TwilioValidationRejection::FormRejection)?
            .0;

        let post_params = if method == Method::POST
            && headers.get("Content-Type").is_some_and(|ct| {
                ct.to_str()
                    .unwrap_or("")
                    .starts_with("application/x-www-form-urlencoded")
            }) {
            Some(&form_data)
        } else {
            None
        };

        twilio_client
            .validate_request(&method, &uri, &headers, post_params)
            .map_err(TwilioValidationRejection::ValidationError)?;

        let params = TwilioParams::from_map(form_data)
            .map_err(TwilioValidationRejection::DeserializationError)?;

        Ok(TwilioParams {
            params,
            twilio_client: Arc::clone(twilio_client),
        })
    }
}

#[derive(Debug)]
pub enum TwilioValidationRejection {
    ValidationError(TwilioError),
    DeserializationError(String),
    FormRejection(FormRejection),
    WebSocketUpgradeRejection(WebSocketUpgradeRejection),
}

impl IntoResponse for TwilioValidationRejection {
    fn into_response(self) -> Response {
        match self {
            Self::ValidationError(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            Self::DeserializationError(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            Self::FormRejection(e) => e.into_response(),
            Self::WebSocketUpgradeRejection(e) => e.into_response(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InboundCall {
    pub params: TwilioRequestParams,
    pub convo_init_map: Arc<DashMap<String, ConversationInitiationClientData>>,
}

impl<S> FromRequest<S> for InboundCall
where
    TelephonyState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = TwilioValidationRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        info!("Inbound call request");

        let twilio_params = TwilioParams::from_request(req, state).await?;
        let t_state = TelephonyState::from_ref(state);
        let convo_init_map = Arc::clone(&t_state.convo_init_data_map);

        Ok(InboundCall {
            convo_init_map,
            params: twilio_params.into_params(),
        })
    }
}

impl InboundCall {
    pub fn answer(self, ws_url: impl Into<String>) -> Result<Response, Error> {
        Ok(VoiceResponse::new()
            .connect(Stream::new(ws_url.into()))
            .to_http_response()?
            .map(Body::from))
    }

    pub fn answer_stream(self, stream: Stream) -> Result<Response, Error> {
        Ok(VoiceResponse::new()
            .connect(stream)
            .to_http_response()?
            .map(Body::from))
    }

    pub fn answer_if<P>(self, ws_url: impl Into<String>, predicate: P) -> Result<Response, Error>
    where
        P: FnOnce(&TwilioRequestParams) -> bool,
    {
        if predicate(&self.params) {
            self.answer(ws_url)
        } else {
            Ok(VoiceResponse::new()
                .reject()
                .to_http_response()?
                .map(Body::from))
        }
    }

    pub fn answer_and_config<F, Fut>(
        self,
        ws_url: impl Into<String>,
        f: F,
    ) -> Result<Response, Error>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ConversationInitiationClientData> + Send,
    {
        let call_sid = self.params.call_sid.clone();
        let convo_init_map = Arc::clone(&self.convo_init_map);
        tokio::spawn(async move {
            convo_init_map.insert(call_sid, f().await);
        });

        Ok(VoiceResponse::new()
            .connect(Stream::new(ws_url.into()))
            .to_http_response()?
            .map(Body::from))
    }

    /// Transfer the call to another number immediately
    /// # Example
    ///
    /// ```rust
    /// use axum::http::StatusCode;
    /// use axum::response::IntoResponse;
    /// use elevenlabs_twilio::InboundCall;
    ///
    /// async fn call_transfer_handler(inbound_call: InboundCall) -> impl IntoResponse {
    ///     match inbound_call
    ///         .transfer_now("+4411111111111")
    ///         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    ///     {
    ///         Ok(voice_response) => voice_response.into_response(),
    ///         Err(err) => {
    ///             eprintln!("Error: {:?}", err);
    ///             err.into_response()
    ///         }
    ///     }
    /// }
    /// ```
    pub fn transfer_now(self, to: impl Into<String>) -> Result<Response, Error> {
        Ok(VoiceResponse::new()
            .dial(Number::new(to.into()))
            .to_http_response()?
            .map(Body::from))
    }
}

#[derive(Clone, Debug)]
pub struct OutboundCall<E> {
    pub inner_extractor: E,
    pub twilio_client: Arc<TwilioClient>,
    pub number: String,
    pub convo_init_map: Arc<DashMap<String, ConversationInitiationClientData>>,
}

impl<S, E> FromRequest<S> for OutboundCall<E>
where
    TelephonyState: FromRef<S>,
    S: Send + Sync,
    E: FromRequest<S>,
{
    type Rejection = E::Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        info!("Outbound call request");
        let inner_extractor = E::from_request(req, state).await?;
        let t_state = TelephonyState::from_ref(state);
        let twilio_client = Arc::clone(&t_state.twilio_client);
        let number = twilio_client.number().unwrap().to_string();
        let convo_init_map = Arc::clone(&t_state.convo_init_data_map);

        Ok(OutboundCall {
            inner_extractor,
            twilio_client,
            number,
            convo_init_map,
        })
    }
}

impl<S, E> FromRequestParts<S> for OutboundCall<E>
where
    TelephonyState: FromRef<S>,
    S: Send + Sync,
    E: FromRequestParts<S>,
{
    type Rejection = E::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        info!("Outbound call request");
        let inner_extractor = E::from_request_parts(parts, state).await?;
        let t_state = TelephonyState::from_ref(state);
        let twilio_client = t_state.twilio_client.clone();
        let number = twilio_client.number().unwrap().to_string();
        let convo_init_map = Arc::clone(&t_state.convo_init_data_map);

        Ok(OutboundCall {
            inner_extractor,
            twilio_client,
            number,
            convo_init_map,
        })
    }
}

impl<E> OutboundCall<E> {
    pub fn as_inner(&self) -> &E {
        &self.inner_extractor
    }
    pub async fn ring(self, to: &str, twiml_url: &str) -> Result<CallResponse, Error> {
        let call_response = self
            .twilio_client
            .create_call_with_url(to, &self.number, twiml_url)
            .await?;
        Ok(call_response)
    }

    pub async fn ring_and_config<F>(
        self,
        to: &str,
        twiml_url: &str,
        f: F,
    ) -> Result<CallResponse, Error>
    where
        F: FnOnce() -> ConversationInitiationClientData + Send + 'static,
    {
        let convo_init_map = Arc::clone(&self.convo_init_map);
        match self.ring(to, twiml_url).await {
            Ok(call_response) => {
                let call_sid = call_response.sid.clone();
                convo_init_map.insert(call_sid, f());
                Ok(call_response)
            }
            Err(e) => Err(e),
        }
    }

    /// Use a different Twilio phone number to make the call
    pub async fn ring_from(
        self,
        to: &str,
        from: &str,
        twiml_url: &str,
    ) -> Result<CallResponse, Error> {
        let call_response = self
            .twilio_client
            .create_call_with_url(to, from, twiml_url)
            .await?;
        Ok(call_response)
    }

    pub fn into_inner(self) -> E {
        self.inner_extractor
    }
}

type ServerMessageCallback = Box<dyn FnMut(ServerMessage) + Send + 'static>;

type TwilioMessageCallback = Box<dyn FnMut(TwilioMessage) + Send + 'static>;

#[derive(Clone, Debug, Serialize)]
pub struct PhoneCallTool {
    // or ClientTool ?
    pub client_tool_call: ClientToolCall,
    pub call_sid: String,
    pub conversation_id: Option<String>,
}

impl PhoneCallTool {
    pub fn new(
        client_tool_call: ClientToolCall,
        call_sid: String,
        conversation_id: Option<String>,
    ) -> Self {
        Self {
            client_tool_call,
            call_sid,
            conversation_id,
        }
    }
}

//
//#[derive(Debug)]
//pub enum ToolAction {
//    SendResult(ClientToolResult),
//    StopConversation,
//    Continue
//}
//
//
//pub trait TelephonyTool: Send + Sync {
//    fn name(&self) -> &str;
//
//    async fn execute(
//        &self,
//        tool: &PhoneCallTool,
//        agent_ws: &Arc<Mutex<AgentWebSocket>>,
//        twilio_client: &TwilioClient,
//    ) -> Result<ToolAction, Error>;
//}

pub struct TelephonyAgent {
    pub agent_registry: Arc<Mutex<HashMap<String, Arc<Mutex<AgentWebSocket>>>>>,
    pub agent_ws: Option<Arc<Mutex<AgentWebSocket>>>,
    pub twilio_ws: WebSocketUpgrade,
    pub twilio_client: Arc<TwilioClient>,
    pub server_message_cb: Option<ServerMessageCallback>,
    pub twilio_message_cb: Option<TwilioMessageCallback>,
    pub convo_init_map: Arc<DashMap<String, ConversationInitiationClientData>>,
    pub tools_tx: Option<UnboundedSender<PhoneCallTool>>,
}

impl<S> FromRequestParts<S> for TelephonyAgent
where
    TelephonyState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = TwilioValidationRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let t_state = TelephonyState::from_ref(state);
        let twilio_client = Arc::clone(&t_state.twilio_client);

        // TODO: just make it take &Parts ?
        twilio_client
            .validate_request(&parts.method, &parts.uri, &parts.headers, None)
            .map_err(TwilioValidationRejection::ValidationError)?;

        let ws = WebSocketUpgrade::from_request_parts(parts, state)
            .await
            .map_err(TwilioValidationRejection::WebSocketUpgradeRejection)?;

        let agent_registry = Arc::clone(&t_state.agent_registry);
        let convo_init_map = Arc::clone(&t_state.convo_init_data_map);

        Ok(TelephonyAgent {
            agent_registry,
            convo_init_map,
            agent_ws: None,
            twilio_ws: ws,
            server_message_cb: None,
            twilio_message_cb: None,
            twilio_client,
            tools_tx: None,
        })
    }
}

impl TelephonyAgent {
    pub async fn set_agent_ws(&mut self, key: &str) -> Result<(), Error> {
        let agent_registry = self.agent_registry.lock().await;
        if let Some(agent_ws) = agent_registry.get(key).cloned() {
            self.agent_ws = Some(agent_ws);
            Ok(())
        } else {
            Err(Error::AgentWebSocketNotFound(key.to_string()))
        }
    }

    pub async fn handle_phone_call(mut self) -> Result<Response, Error> {
        let ws = self.twilio_ws;
        let agent_ws = self.agent_ws.ok_or(Error::AgentWebSocketNotSet)?;
        let server_message_cb = self.server_message_cb.take();
        let twilio_message_cb = self.twilio_message_cb.take();
        let convo_init_map = self.convo_init_map;
        let tools_tx = self.tools_tx.take();

        let resp = ws.on_upgrade(move |socket| async move {
            match Self::handle_websockets(
                socket,
                agent_ws,
                server_message_cb,
                twilio_message_cb,
                convo_init_map,
                tools_tx,
            )
            .await
            {
                Ok(_) => info!("Phone call established"),
                Err(e) => error!("Failed to establish phone call: {:?}", e),
            }
        });
        Ok(resp)
    }

    #[instrument(
        name = "handle_websockets",
        skip_all,
        fields(call_sid = field::Empty, stream_sid = field::Empty)
    )]
    pub async fn handle_websockets(
        mut twilio_socket: WebSocket,
        agent_ws: Arc<Mutex<AgentWebSocket>>,
        mut server_cb: Option<ServerMessageCallback>,
        mut twilio_cb: Option<TwilioMessageCallback>,
        convo_init_map: Arc<DashMap<String, ConversationInitiationClientData>>,
        tools_tx: Option<UnboundedSender<PhoneCallTool>>,
    ) -> Result<(), Error> {
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
        let call_sid = msg.start.call_sid.clone();
        info!(call_sid = %call_sid, stream_sid = %stream_sid, "Received Start message");
        // Add CallSid and StreamSid to the current tracing span
        Span::current().record("call_sid", &call_sid);
        Span::current().record("stream_sid", &stream_sid);

        // -- Set possible `ConversationInitiationClientData` --
        let init_data_opt = convo_init_map.remove(&call_sid);
        if let Some((_, data)) = init_data_opt {
            let mut agent_ws_locked = agent_ws.lock().await;
            agent_ws_locked.with_conversation_initiation_client_data(data);
            info!("Applied conversation initiation data to agent websocket");
        } else {
            warn!("No conversation initiation data found for this call.");
        }

        let (twilio_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let twilio_rx_stream = UnboundedReceiverStream::new(rx);

        let (mut twilio_sink, mut twilio_stream) = twilio_socket.split();

        let agent_ws_for_stop = Arc::clone(&agent_ws);

        tokio::spawn(async move {
            while let Some(msg) = twilio_stream.next().await {
                let msg = msg?;
                let twilio_msg = TwilioMessage::try_from(msg.to_text()?)?;

                if let Some(cb) = twilio_cb.as_mut() {
                    cb(twilio_msg.clone());
                }

                match twilio_msg {
                    TwilioMessage::Media(media_msg) => {
                        let payload = media_msg.media.payload;
                        // TODO: system end_call tool gives us error here, handle it
                        if twilio_tx.send(payload).is_err() {
                            error!("Failed to send Twilio payload to agent websocket");
                            break;
                        }
                    }
                    TwilioMessage::Stop(msg) => {
                        return match agent_ws_for_stop.lock().await.stop_conversation().await {
                            Ok(_) => {
                                info!("Twilio Stop Message for CallSid: {}", msg.stop.call_sid);
                                Ok(())
                            }
                            Err(e) => {
                                error!("Failed to end agent websocket session: {}", e);
                                Err(e.into())
                            }
                        };
                    }
                    _ => {}
                }
            }
            Ok::<(), Error>(())
        });

        let agent_ws_for_convo = Arc::clone(&agent_ws);
        tokio::spawn(async move {
            let agent_ws = agent_ws_for_convo;
            let mut conversation_id: Option<String> = None;

            let mut agent_ws_locked = agent_ws.lock().await;
            let convai_stream_result = agent_ws_locked.start_conversation(twilio_rx_stream).await;
            drop(agent_ws_locked);

            let mut convai_stream = match convai_stream_result {
                Ok(stream) => stream,
                Err(e) => {
                    error!(call_sid = %call_sid, "Failed to start agent conversation: {}", e);
                    return;
                }
            };
            info!(call_sid = %call_sid, "Agent conversation stream started.");

            while let Some(msg_result) = convai_stream.next().await {
                match msg_result {
                    Ok(server_msg) => {
                        if let Some(cb) = server_cb.as_mut() {
                            cb(server_msg.clone());
                        }

                        match server_msg {
                            ServerMessage::Audio(audio) => {
                                let twilio_audio_msg = match (audio, &stream_sid).to_twilio() {
                                    Ok(msg) => msg,
                                    Err(e) => {
                                        error!(call_sid = %call_sid, "Failed to create Twilio media message: {}", e);
                                        continue; // Skip sending this message
                                    }
                                };
                                if let Err(e) = twilio_sink.send(twilio_audio_msg).await {
                                    error!(call_sid = %call_sid, "Failed to send audio to Twilio: {}", e);
                                    break;
                                }
                            }
                            ServerMessage::Interruption(_) => {
                                info!(call_sid = %call_sid, "Received agent Interruption");
                                let clear_msg = ClearMessage::new(&stream_sid);
                                match serde_json::to_string(&clear_msg) {
                                    Ok(json) => {
                                        if let Err(e) =
                                            twilio_sink.send(Message::Text(json.into())).await
                                        {
                                            error!(call_sid = %call_sid, "Failed to send Clear message to Twilio: {}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        error!(call_sid = %call_sid, "Failed to serialize Clear message: {}", e)
                                    }
                                }
                            }
                            ServerMessage::ClientToolCall(tool_call) => {
                                match &tools_tx {
                                    Some(tools_tx) => {
                                        info!(call_sid = %call_sid, tool.name = %tool_call.name(), tool.id = %tool_call.id(), "Received agent tool call");
                                        let phone_call_tool = PhoneCallTool::new(
                                            tool_call,
                                            call_sid.clone(),
                                            conversation_id.clone(),
                                        );
                                        if tools_tx.send(phone_call_tool).is_err() {
                                            error!(call_sid = %call_sid, "Failed to send tool call to receiver)");
                                        }
                                    }
                                    None => {
                                        error!(call_sid = %call_sid, tool.name = %tool_call.name(), tool.id = %tool_call.id(), "Received tool call but tools_tx channel is not configured.");
                                        // Send tool error back to agent
                                        let tool_result = ClientToolResult::new(tool_call.id())
                                            .has_error(true)
                                            .with_result(
                                                "Tool processing channel not available".to_string(),
                                            );
                                        let agent_ws_locked = agent_ws.lock().await;
                                        if let Err(e) =
                                            agent_ws_locked.send_tool_result(tool_result).await
                                        {
                                            error!(call_sid=%call_sid, "Failed to send tool error back to agent: {}", e);
                                        }
                                    }
                                }
                            }
                            ServerMessage::ConversationInitiationMetadata(metadata) => {
                                info!("Received conversation initiation metadata");
                                conversation_id = Some(
                                    metadata
                                        .conversation_initiation_metadata_event
                                        .conversation_id,
                                );
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        // Should we return error from core client on different ws close codes?
                        error!(call_sid = %call_sid, "Failed to receive message from a get websocket: {}", e);
                        break;
                    }
                }
            }
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

/// Requires env var `WEBHOOK_SECRET` to be set, returns `PostCallRejection::InternalServerError` if not set
#[derive(Clone, Debug)]
pub struct PostCall {
    pub payload: PostCallPayload,
    pub twilio_client: Arc<TwilioClient>,
    pub convo_init_map: Arc<DashMap<String, ConversationInitiationClientData>>,
    pub agent_registry: Arc<Mutex<HashMap<String, Arc<Mutex<AgentWebSocket>>>>>,
}

impl PostCall {
    pub fn summary(&self) -> Option<&str> {
        if let Some(analysis) = &self.payload.data.analysis {
            Some(&analysis.transcript_summary)
        } else {
            None
        }
    }

    pub async fn create_call(&self, body: CreateCallBody<'_>) -> Result<CallResponse, Error> {
        let create_call = CreateCall::new(self.twilio_client.account_sid(), body);
        Ok(self.twilio_client.hit(create_call).await?)
    }
}

use hex::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

impl<S> FromRequest<S> for PostCall
where
    S: Send + Sync,
    TelephonyState: FromRef<S>,
{
    type Rejection = PostCallRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        info!("Post call request");
        let t_state = TelephonyState::from_ref(state);

        let secret_str = t_state
            .webhook_secret
            .ok_or(PostCallRejection::InternalServerError(
                "Webhook secret not set".to_string(),
            ))?;

        let webhook_secret = secret_str.expose_secret();

        let (parts, body) = req.into_parts();

        if let Some(content_type) = parts.headers.get("Content-Type") {
            if content_type != "application/json" {
                return Err(PostCallRejection::BadRequest(
                    "Invalid content type".to_string(),
                ));
            }
        }

        let signature_header = parts
            .headers
            .get("ElevenLabs-Signature")
            .ok_or(PostCallRejection::BadRequest(
                "Missing ElevenLabs-Signature header".to_string(),
            ))?
            .to_str()
            .map_err(|_| PostCallRejection::BadRequest("Invalid signature header".to_string()))?;

        let headers_parts: Vec<&str> = signature_header.split(',').collect();

        let timestamp_str = headers_parts
            .iter()
            .find(|&&e| e.starts_with("t="))
            .ok_or(PostCallRejection::BadRequest(
                "Missing timestamp in signature".to_string(),
            ))?
            .strip_prefix("t=")
            .unwrap();

        let signature = headers_parts
            .iter()
            .find(|&&e| e.starts_with("v0="))
            .ok_or(PostCallRejection::BadRequest(
                "Missing hash in signature".to_string(),
            ))?
            .to_string();

        let timestamp_sec = timestamp_str
            .parse::<i64>()
            .map_err(|_| PostCallRejection::BadRequest("Invalid timestamp format".to_string()))?;

        let now_sec = Utc::now().timestamp();

        if timestamp_sec < now_sec - 30 * 60 {
            return Err(PostCallRejection::Forbidden("Request expired".to_string()));
        }

        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|_| {
            PostCallRejection::InternalServerError("Failed to read body".to_string())
        })?;

        let body_str = String::from_utf8(body_bytes.to_vec())
            .map_err(|_| PostCallRejection::BadRequest("Body is not valid UTF-8".to_string()))?;

        let message = format!("{}.{}", timestamp_str, body_str);

        let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes()).map_err(|_| {
            PostCallRejection::InternalServerError("HMAC initialization failed".to_string())
        })?;

        mac.update(message.as_bytes());

        let computed_digest = format!("v0={}", encode(mac.finalize().into_bytes()));

        if signature != computed_digest {
            return Err(PostCallRejection::Unauthorized(
                "Request unauthorized".to_string(),
            ));
        }

        let payload = serde_json::from_str(&body_str)
            .map_err(|e| PostCallRejection::BadRequest(format!("Invalid JSON payload: {}", e)))?;

        let twilio_client = Arc::clone(&t_state.twilio_client);
        let convo_init_map = Arc::clone(&t_state.convo_init_data_map);
        let agent_registry = Arc::clone(&t_state.agent_registry);

        Ok(PostCall {
            payload,
            twilio_client,
            convo_init_map,
            agent_registry,
        })
    }
}

#[derive(Error, Debug)]
pub enum PostCallRejection {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for PostCallRejection {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg).into_response(),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg).into_response(),
            Self::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use base64::Engine;
    use chrono::Utc;
    use hmac::Mac;
    use http_body_util::BodyExt;
    use serde_json::json;
    use sha1::Sha1;
    use std::future::IntoFuture;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct TestAppState {
        sub_state: TelephonyState,
    }

    impl TestAppState {
        fn new(webhook_secret: &str, auth_token: &str) -> Self {
            let agent_ws = Arc::new(Mutex::new(AgentWebSocket::new("test", "test")));
            let twilio_client =
                Arc::new(TwilioClient::new("test", auth_token).with_number("1234567890"));

            let test_app = if webhook_secret.is_empty() {
                let mut agents = HashMap::new();
                agents.insert("test".to_string(), agent_ws);
                let agent_registry = Arc::new(Mutex::new(agents));

                let sub_state = TelephonyState {
                    agent_registry,
                    convo_init_data_map: Arc::new(Default::default()),
                    twilio_client,
                    webhook_secret: None,
                };

                TestAppState { sub_state }
            } else {
                let sub_state = TelephonyState::new("test".to_string(), agent_ws, twilio_client)
                    .expect("Failed to create telephony state")
                    .with_webhook_secret(webhook_secret);
                TestAppState { sub_state }
            };

            test_app
        }
    }

    impl FromRef<TestAppState> for TelephonyState {
        fn from_ref(app: &TestAppState) -> Self {
            app.sub_state.clone()
        }
    }

    fn app(webhook_secret: &str, auth_token: &str) -> Router {
        let app_state = TestAppState::new(webhook_secret, auth_token);
        Router::new()
            .route(
                "/connect_twiml",
                post(|p: TwilioParams| async {
                    match p.connect("wss://example.com/ws") {
                        Ok(twiml) => twiml.into_response(),
                        Err(e) => {
                            error!("Failed to connect: {:?}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                        }
                    }
                }),
            )
            .route("/ws", get(telephony_agent_testable_handler))
            .route("/post_call", post(|_: PostCall| async { StatusCode::OK }))
            .with_state(app_state)
    }

    //fn create_test_request(secret: &str, payload: &str) -> Request<Body> {
    //    let timestamp = Utc::now().timestamp();
    //    let message = format!("{}.{}", timestamp, payload);
    //    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    //    mac.update(message.as_bytes());
    //    let signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));
    //    Request::builder()
    //        .method("POST")
    //        .uri("/webhook")
    //        .header(
    //            "ElevenLabs-Signature",
    //            format!("t={},{}", timestamp, signature),
    //        )
    //        .body(Body::from(payload.to_string()))
    //        .unwrap()
    //}

    type HmacSha1 = Hmac<Sha1>;

    fn generate_valid_signature(
        auth_token: &str,
        url: &str,
        params: Option<&BTreeMap<String, String>>,
    ) -> String {
        let mut data = url.to_string();

        if let Some(params) = params {
            for (key, value) in params {
                data.push_str(key);
                data.push_str(value);
            }
        }

        let mut mac =
            HmacSha1::new_from_slice(auth_token.as_bytes()).expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    fn required_twilio_params_for_deserialization() -> BTreeMap<String, String> {
        let mut params = BTreeMap::new();
        params.insert("CallSid".to_string(), "CA123456789".to_string());
        params.insert("AccountSid".to_string(), "AC123456789".to_string());
        params.insert("From".to_string(), "+12345678901".to_string());
        params.insert("To".to_string(), "+12345678902".to_string());
        params.insert("CallStatus".to_string(), "queued".to_string());
        params.insert("ApiVersion".to_string(), "2010-04-01".to_string());
        params.insert("Direction".to_string(), "outbound-api".to_string());
        params
    }

    async fn telephony_agent_testable_handler(agent: TelephonyAgent) -> Response {
        agent.twilio_ws.on_upgrade(send_first_media_payload)
    }

    async fn send_first_media_payload(mut socket: WebSocket) {
        let msg = socket.next().await.unwrap().unwrap();
        let msg: ConnectedMessage = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        let msg = socket.next().await.unwrap().unwrap();
        let msg: StartMessage = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        let stream_sid = msg.stream_sid;
        let media_msg = MediaMessage::new(stream_sid, "test_payload");
        let json = serde_json::to_string(&media_msg).unwrap();
        socket.send(Message::Text(json.into())).await.unwrap();
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_internal_error_when_webhook_secret_is_not_set() {
        let app = app("", "test_auth_token");
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header("ElevenLabs-Signature", "t=12345,v0=hash")
            .body(Body::from(r#"{"type": "test"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Webhook secret not set");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_bad_request_when_content_type_is_not_json() {
        let app = app("test_secret", "test_auth_token");
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "text/plain")
            .header("ElevenLabs-Signature", "t=12345,v0=hash")
            .body(Body::from(r#"{"type": "test"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Invalid content type");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_bad_request_when_signature_header_is_missing() {
        let app = app("test_secret", "test_auth_token");
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"type": "test"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Missing ElevenLabs-Signature header");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_bad_request_when_missing_timestamp() {
        let app = app("test_secret", "test_auth_token");
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header("ElevenLabs-Signature", "v0=hash")
            .body(Body::from(r#"{"type": "test"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Missing timestamp in signature");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_bad_request_when_missing_hash() {
        let app = app("test_secret", "test_auth_token");
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header("ElevenLabs-Signature", "t=12345")
            .body(Body::from(r#"{"type": "test"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Missing hash in signature");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_forbidden_when_timestamp_is_from_31_minutes_ago()
    {
        let app = app("test_secret", "test_auth_token");
        let timestamp = Utc::now().timestamp() - 31 * 60;
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header("ElevenLabs-Signature", format!("t={},v0=hash", timestamp))
            .body(Body::from(r#"{"type": "test"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Request expired");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_bad_request_when_body_is_not_valid_utf8() {
        let app = app("test_secret", "test_auth_token");
        let timestamp = Utc::now().timestamp();
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header("ElevenLabs-Signature", format!("t={},v0=hash", timestamp))
            .body(Body::from(vec![0, 159, 146, 150]))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Body is not valid UTF-8");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_unauthorized_when_signature_is_invalid() {
        let mut mac = HmacSha256::new_from_slice(b"invalid_secret").unwrap();
        let timestamp = Utc::now().timestamp();
        let payload = r#"{"type": "test"}"#;
        let message = format!("{}.{}", timestamp, payload);
        mac.update(message.as_bytes());
        let signature = format!("v0={}", encode(mac.finalize().into_bytes()));
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header(
                "ElevenLabs-Signature",
                format!("t={},{}", timestamp, signature),
            )
            .body(Body::from(payload.to_string()))
            .unwrap();

        let app = app("test_secret", "test_auth_token");
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Request unauthorized");
    }

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_bad_request_when_json_payload_is_invalid() {
        let mut mac = HmacSha256::new_from_slice(b"test_secret").unwrap();
        let timestamp = Utc::now().timestamp();
        let payload = json!({"type": "test"});
        let message = format!("{}.{}", timestamp, payload);
        mac.update(message.as_bytes());
        let signature = format!("v0={}", encode(mac.finalize().into_bytes()));
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header(
                "ElevenLabs-Signature",
                format!("t={},{}", timestamp, signature),
            )
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();
        let app = app("test_secret", "test_auth_token");
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &body[..],
            b"Invalid JSON payload: missing field `data` at line 1 column 15"
        );
    }

    #[tokio::test]
    async fn post_call_extractor_is_returning_self() {
        let mut mac = HmacSha256::new_from_slice(b"test_secret").unwrap();
        let timestamp = Utc::now().timestamp();
        let payload = json!({
            "type": "post_call_transcription",
            "event_timestamp": timestamp,
            "data": {
                "agent_id": "test",
                "conversation_id": "test",
                "status": "done",
                "transcript": [],
                "metadata": {
                    "start_time_unix_secs": timestamp,
                    "call_duration_secs": 11,
                },
                "analysis": null,
                "conversation_initiation_client_data": null,
            }
        });
        let message = format!("{}.{}", timestamp, payload);
        mac.update(message.as_bytes());
        let signature = format!("v0={}", encode(mac.finalize().into_bytes()));
        let request = Request::builder()
            .method("POST")
            .uri("/post_call")
            .header("Content-Type", "application/json")
            .header(
                "ElevenLabs-Signature",
                format!("t={},{}", timestamp, signature),
            )
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();
        let app = app("test_secret", "test_auth_token");
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn twilio_params_extractor_is_returning_self() {
        let app_state = TestAppState::new("test_secret", "test_auth_token");

        let params = required_twilio_params_for_deserialization();

        let form_data = serde_urlencoded::to_string(&params).unwrap();

        let url = "https://example.com/webhook";

        let signature = generate_valid_signature("test_auth_token", url, Some(&params));

        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("Host", "example.com")
            .header("X-Twilio-Signature", signature)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .body(Body::from(form_data))
            .unwrap();

        let result = TwilioParams::from_request(request, &app_state).await;

        assert!(result.is_ok(), "Valid request should extract successfully");
    }

    #[tokio::test]
    async fn twilio_params_extractor_is_rejecting_with_bad_request_when_twilio_signature_is_invalid(
    ) {
        let app_state = TestAppState::new("test_secret", "test_auth_token");
        let params = required_twilio_params_for_deserialization();
        let form_data = serde_urlencoded::to_string(&params).unwrap();
        let url = "https://example.com/webhook";
        let signature = generate_valid_signature("invalid_auth_token", url, Some(&params));

        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("Host", "example.com")
            .header("X-Twilio-Signature", signature)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .body(Body::from(form_data))
            .unwrap();

        let result = TwilioParams::from_request(request, &app_state).await;
        let rej = result.unwrap_err().into_response();
        assert_eq!(rej.status(), StatusCode::BAD_REQUEST,);
        let body = rej.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &body[..],
            b"signature validation error: Invalid Twilio signature",
        );
    }

    #[tokio::test]
    async fn twilio_params_extractor_is_rejecting_with_bad_request_when_signature_is_missing() {
        let app_state = TestAppState::new("test_secret", "test_auth_token");
        let params = required_twilio_params_for_deserialization();
        let form_data = serde_urlencoded::to_string(&params).unwrap();
        let url = "https://example.com/webhook";
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("Host", "example.com")
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .body(Body::from(form_data))
            .unwrap();
        let result = TwilioParams::from_request(request, &app_state).await;
        let rej = result.unwrap_err().into_response();
        assert_eq!(rej.status(), StatusCode::BAD_REQUEST,);
        let body = rej.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &body[..],
            b"signature validation error: Missing X-Twilio-Signature header",
        );
    }

    #[tokio::test]
    async fn twilio_params_extractor_is_rejecting_with_bad_request_when_host_missing() {
        let app_state = TestAppState::new("test_secret", "test_auth_token");
        let params = required_twilio_params_for_deserialization();
        let form_data = serde_urlencoded::to_string(&params).unwrap();
        let url = "https://example.com/webhook";
        let signature = generate_valid_signature("test_auth_token", url, Some(&params));
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("X-Twilio-Signature", signature)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .body(Body::from(form_data))
            .unwrap();
        let result = TwilioParams::from_request(request, &app_state).await;
        let rej = result.unwrap_err().into_response();
        assert_eq!(rej.status(), StatusCode::BAD_REQUEST,);
        let body = rej.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &body[..],
            b"signature validation error: Missing Host header",
        );
    }

    #[tokio::test]
    async fn connect_is_returning_connect_twiml_response_in_handler() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app("test_secret", "test_auth_token"))
                .await
                .unwrap();
        });

        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        let url = format!("https://{}/connect_twiml", addr);
        let params = required_twilio_params_for_deserialization();
        let form_data = serde_urlencoded::to_string(&params).unwrap();
        let signature = generate_valid_signature("test_auth_token", &url, Some(&params));

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("http://{}/connect_twiml", addr))
            .header("Host", addr.to_string())
            .header("X-Twilio-Signature", signature)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .body(Body::from(form_data))
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let twiml_response = String::from_utf8(body.to_vec()).unwrap();

        let want = r#"<?xml version="1.0" encoding="UTF-8"?><Response><Connect><Stream url="wss://example.com/ws" /></Connect></Response>"#;
        assert_eq!(twiml_response, want);
    }

    #[tokio::test]
    async fn telephony_agent_extractor_is_sending_media_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

        let addr = listener.local_addr().unwrap();

        let app = app("test_secret", "test_auth_token");

        let signature =
            generate_valid_signature("test_auth_token", &format!("wss://{addr}/ws"), None);

        tokio::spawn(axum::serve(listener, app).into_future());

        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("ws://{addr}/ws"))
            .header("Host", addr.to_string())
            .header("X-Twilio-Signature", signature)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(())
            .unwrap();

        let (mut socket, _response) = tokio_tungstenite::connect_async(req).await.unwrap();

        // send connected message
        let connected_msg = json!({
            "event": "connected",
            "protocol": "Call",
            "version": "1.0.0",
        });

        let connected_msg = serde_json::to_string(&connected_msg).unwrap();
        socket
            .send(tungstenite::Message::Text(connected_msg.into()))
            .await
            .unwrap();

        // send start message
        let start_msg = json!({
            "event": "start",
            "sequenceNumber": "1",
            "streamSid": "test_stream_sid",
            "start": {
                "streamSid": "test_stream_sid",
                "accountSid": "AC123456789",
                "callSid": "CA123456789",
                "tracks": [
                    "inbound"
                ],
                "customParameters": {
                    "foo": "bar"
                },
                "mediaFormat": {
                    "encoding": "audio/x-mulaw",
                    "sampleRate": 8000,
                    "channels": 1,
                }
            }
        });

        let start_msg = serde_json::to_string(&start_msg).unwrap();
        socket
            .send(tungstenite::Message::Text(start_msg.into()))
            .await
            .unwrap();

        let msg = match socket.next().await.unwrap().unwrap() {
            tungstenite::Message::Text(msg) => msg,
            other => panic!("expected a text message but got {other:?}"),
        };

        let want =
            r#"{"event":"media","streamSid":"test_stream_sid","media":{"payload":"test_payload"}}"#;

        assert_eq!(msg.as_str(), want);
    }
}
