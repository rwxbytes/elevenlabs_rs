use crate::conversations::GetConversationDetailsResponse;
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
use chrono::Utc;
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
use secrecy::{ExposeSecret, SecretString};
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
    webhook_secret: Option<SecretString>,
}

impl WebSocketStreamManager {
    pub fn new(
        agent_ws: Arc<Mutex<AgentWebSocket>>,
        twilio_client: TwilioClient,
    ) -> Result<Self, Error> {
        if !twilio_client.number().is_some() {
            return Err(Error::TwilioClient(TwilioError::MissingPhoneNumberEnvVar));
        }

        let convo_init_rx = Arc::new(Mutex::new(None));
        let webhook_secret = std::env::var("WEBHOOK_SECRET").ok().map(SecretString::from);

        Ok(WebSocketStreamManager {
            agent_ws,
            twilio_client,
            convo_init_rx,
            webhook_secret,
        })
    }

    pub fn from_env() -> Result<Self, Error> {
        let agent_ws = Arc::new(Mutex::new(AgentWebSocket::from_env()?));
        let twilio_client = TwilioClient::from_env()?;

        WebSocketStreamManager::new(agent_ws, twilio_client)
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
    pub async fn ring(
        self,
        to: impl Into<String>,
        twiml_url: impl Into<String>,
    ) -> Result<CallResponse, Error> {
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
    pub payload: PostCallPayload,
}

use hex::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

impl<S> FromRequest<S> for PostCall
where
    S: Send + Sync,
    WebSocketStreamManager: FromRef<S>,
{
    type Rejection = PostCallRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        info!("Post call request");
        let manager = WebSocketStreamManager::from_ref(state);

        let secret_str = manager
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

        let computed_digest = format!("v0={}", hex::encode(mac.finalize().into_bytes()));

        if signature != computed_digest {
            return Err(PostCallRejection::Unauthorized(
                "Request unauthorized".to_string(),
            ));
        }

        let payload = serde_json::from_str(&body_str)
            .map_err(|e| PostCallRejection::BadRequest(format!("Invalid JSON payload: {}", e)))?;

        Ok(PostCall { payload })
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use chrono::Utc;
    use hmac::{Hmac, Mac};
    use http_body_util::BodyExt;
    use sha2::Sha256;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct AppState {
        web_socket_stream_manager: WebSocketStreamManager,
    }

    impl AppState {
        fn new(secret: Option<String>) -> Self {
            let agent_ws = AgentWebSocket::new("test", "test");
            let twilio_client = TwilioClient::new("test", "test");
            let web_socket_stream_manager = WebSocketStreamManager {
                agent_ws: Arc::new(Mutex::new(agent_ws)),
                twilio_client,
                convo_init_rx: Arc::new(Mutex::new(None)),
                webhook_secret: secret.map(SecretString::from),
            };
            AppState {
                web_socket_stream_manager,
            }
        }
    }

    impl FromRef<AppState> for WebSocketStreamManager {
        fn from_ref(app: &AppState) -> Self {
            app.web_socket_stream_manager.clone()
        }
    }

    fn app(secret: Option<String>) -> Router {
        let app_state = AppState::new(secret);
        Router::new()
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

    #[tokio::test]
    async fn post_call_extractor_is_rejecting_with_internal_error_when_webhook_secret_is_not_set() {
        let app = app(None);
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
        let app = app(Some("test_secret".to_string()));
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
        let app = app(Some("test_secret".to_string()));
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
        let app = app(Some("test_secret".to_string()));
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
        let app = app(Some("test_secret".to_string()));
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
        let app = app(Some("test_secret".to_string()));
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
        let app = app(Some("test_secret".to_string()));
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
        let mut mac = HmacSha256::new_from_slice(b"test_secret").unwrap();
        let timestamp = Utc::now().timestamp();
        let payload = r#"{"type": "test"}"#;
        let message = format!("{}.{}", timestamp, payload);
        mac.update(message.as_bytes());
        let signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));
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

        let app = app(Some("test_invalid_secret".to_string()));
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
        let signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));
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
        let app = app(Some("test_secret".to_string()));
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
        let signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));
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
        let app = app(Some("test_secret".to_string()));
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
