use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use elevenlabs_convai::error::ConvAIError;
use elevenlabs_convai::messages::server_messages::Audio;
pub use elevenlabs_convai::{client::AgentWebSocket, messages::server_messages::ServerMessage};
use futures_util::{SinkExt, StreamExt};
pub use rusty_twilio::endpoints::accounts::*;
pub use rusty_twilio::endpoints::applications::*;
pub use rusty_twilio::endpoints::voice::{call::*, stream::*};
pub use rusty_twilio::error::TwilioError;
pub use rusty_twilio::twiml::voice::VoiceResponse;
pub use rusty_twilio::TwilioClient;
use std::ops::ControlFlow;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, info};

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

#[async_trait]
pub trait TelephonyAgent: Send + Sync {
    fn agent_ws(&self) -> Arc<Mutex<AgentWebSocket>>;

    fn server_message_callback(&self) -> Option<Box<dyn FnMut(ServerMessage) + Send>> {
        None
    }

    fn twilio_message_callback() -> Option<Box<dyn FnMut(TwilioMessage) + Send>> {
        None
    }

    async fn extract_stream_sid(&self, socket: &mut WebSocket) -> Result<String, Error> {
        let msg = socket
            .next()
            .await
            .ok_or(Error::FailedToReceiveStartMessage)??;
        let msg: StartMessage = serde_json::from_str(msg.to_text()?)?;
        Ok(msg.stream_sid)
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

    // rename to handle_call or relay?
    async fn talk(&self, mut socket: WebSocket) -> Result<(), Error> {
        if let Some(Ok(msg)) = socket.next().await {
            let msg: ConnectedMessage = serde_json::from_str(msg.to_text()?)?;
            info!("Connected message: {:?}", msg);
        }

        let stream_sid = self.extract_stream_sid(&mut socket).await?;

        let (twilio_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let twilio_rx = UnboundedReceiverStream::new(rx);

        let (mut twilio_sink, mut twilio_stream) = socket.split();

        // Spawn task for incoming Twilio messages.
        let agent_ws = Arc::clone(&self.agent_ws());
        tokio::spawn(async move {
            while let Some(Ok(msg)) = twilio_stream.next().await {
                if Self::handle_twilio_message(msg, &agent_ws, &twilio_tx)
                    .await
                    .is_break()
                {
                    break;
                }
            }
        });

        let mut cb = self.server_message_callback();

        let agent_ws_for_convo = Arc::clone(&self.agent_ws());
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

// or AudioToTwilio ?
//pub struct LabAudio<'a> {
//    audio: Audio,
//    stream_sid: &'a str,
//}
//
//impl<'a> LabAudio<'a> {
//    pub fn new(audio: Audio, stream_sid: &'a str) -> Self {
//        LabAudio { audio, stream_sid }
//    }
//}
//
//impl<'a> TryFrom<LabAudio<'a>> for Message {
//    type Error = Error;
//    fn try_from(audio: LabAudio) -> Result<Self, Error> {
//        let payload = audio.audio.audio_event.audio_base_64;
//        let media_msg = MediaMessage::new(audio.stream_sid, &payload);
//        let json = serde_json::to_string(&media_msg)?;
//        Ok(Message::Text(json.into()))
//    }
//}

#[derive(Clone, Debug)]
pub struct OutboundAgent {
    pub agent_ws: Arc<Mutex<AgentWebSocket>>,
    pub twilio_client: TwilioClient,
    pub twiml_src: Option<TwimlSrc>,
    pub create_call_body: Option<CreateCallBody>,
    pub twiml_for_connection: Option<String>,
}

impl TelephonyAgent for OutboundAgent {
    fn agent_ws(&self) -> Arc<Mutex<AgentWebSocket>> {
        Arc::clone(&self.agent_ws)
    }

    fn server_message_callback(&self) -> Option<Box<dyn FnMut(ServerMessage) + Send>> {
        None
    }

    fn twilio_message_callback() -> Option<Box<dyn FnMut(TwilioMessage) + Send>> {
        None
    }
}

impl OutboundAgent {
    pub fn new(agent_ws: AgentWebSocket, twilio_client: TwilioClient) -> Self {
        OutboundAgent {
            agent_ws: Arc::new(Mutex::new(agent_ws)),
            twilio_client,
            create_call_body: None,
            twiml_src: None,
            twiml_for_connection: None,
        }
    }

    pub async fn ring(
        &self,
        create_call_body: impl Into<CreateCallBody>,
    ) -> Result<CallResponse, Error> {
        let body = create_call_body.into();
        let endpoint = CreateCall::new(self.twilio_client.account_sid(), body);
        Ok(self.twilio_client.hit(endpoint).await?)
    }

    pub async fn ring_by_endpoint(&self, endpoint: CreateCall) -> Result<CallResponse, Error> {
        Ok(self.twilio_client.hit(endpoint).await?)
    }

    // TODO: name it something else
    pub fn set_twiml_src(mut self, url: impl Into<String>) -> Self {
        self.twiml_src = Some(TwimlSrc::Url(url.into()));
        self
    }

    pub fn set_twiml_for_connection(mut self, url: impl Into<String>) -> Result<Self, Error> {
        let twiml = VoiceResponse::new().connect(url.into()).to_string()?;
        self.twiml_for_connection = Some(twiml);
        Ok(self)
    }

    pub fn get_twiml_for_connection(&self) -> Option<String> {
        self.twiml_for_connection.clone()
    }
}

#[derive(Clone, Debug)]
pub struct InboundAgent {
    pub elevenlabs_client: Arc<Mutex<AgentWebSocket>>,
    pub twilio_client: TwilioClient,
    pub msg_tx: Option<tokio::sync::mpsc::UnboundedSender<ServerMessage>>,
}

impl TelephonyAgent for InboundAgent {
    fn agent_ws(&self) -> Arc<Mutex<AgentWebSocket>> {
        Arc::clone(&self.elevenlabs_client)
    }

    fn server_message_callback(&self) -> Option<Box<dyn FnMut(ServerMessage) + Send>> {
        //if let Some(tx) = &self.msg_tx {
        //    let tx = tx.clone();
        //    Some(Box::new(move |msg| {
        //        if tx.send(msg).is_err() {
        //            error!("failed to send server message to websocket");
        //        }
        //    }))
        //} else {
        //    None
        //}
        None
    }

    fn twilio_message_callback() -> Option<Box<dyn FnMut(TwilioMessage) + Send>> {
        None
    }
}

impl InboundAgent {
    pub fn from_env() -> Result<Self, &'static str> {
        let eleven_client = AgentWebSocket::from_env().expect("Failed to create eleven client");
        let twilio_client = TwilioClient::from_env().expect("Failed to create twilio client");
        Ok(InboundAgent {
            elevenlabs_client: Arc::new(Mutex::new(eleven_client)),
            twilio_client,
            msg_tx: None,
        })
    }
}
