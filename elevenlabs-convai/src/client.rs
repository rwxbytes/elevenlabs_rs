use crate::error::ConvAIError;
use crate::messages::client_messages::{
    ClientToolResult, ContextualUpdate, ConversationInitiationClientData, Pong, UserAudioChunk,
};
use crate::messages::server_messages::ServerMessage;
use crate::Result;
use elevenlabs_rs::endpoints::convai::conversations::GetSignedUrl;
use elevenlabs_rs::ElevenLabsClient;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{pin_mut, SinkExt, Stream, StreamExt};
use std::borrow::Cow;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::warn;

const WS_BASE_URL: &str = "wss://api.elevenlabs.io";
const WS_CONVAI_PATH: &str = "/v1/conversational_ai/conversation";
const AGENT_ID_QUERY: &str = "agent_id";

type ConversationStream = UnboundedReceiverStream<Result<ServerMessage>>;
type WebSocketWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WebSocketReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Represents a client for interacting with the ElevenLabs Conversational AI.
#[derive(Debug)]
pub struct AgentWebSocket {
    pub api_key: Option<String>,
    pub agent_id: String,
    pub writer_task_tx: Option<UnboundedSender<Message>>,
    pub conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}

impl AgentWebSocket {
    /// Creates a new `AgentWebSocket` from environment variables.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            api_key: Some(std::env::var("ELEVENLABS_API_KEY")?),
            agent_id: std::env::var("ELEVENLABS_AGENT_ID")?,
            conversation_initiation_client_data: None,
            writer_task_tx: None,
        })
    }

    /// Creates a new `AgentWebSocket` with the given API key and agent ID.
    pub fn new<T: Into<String>>(api_key: T, agent_id: T) -> Self {
        Self {
            api_key: Some(api_key.into()),
            agent_id: agent_id.into(),
            conversation_initiation_client_data: None,
            writer_task_tx: None,
        }
    }

    pub fn with_agent_id<T: Into<String>>(&mut self, agent_id: T) {
        self.agent_id = agent_id.into();
    }

    /// Sets initial data to be sent to the server when starting a conversation.
    pub fn with_conversation_initiation_client_data(
        &mut self,
        data: ConversationInitiationClientData,
    ) {
        self.conversation_initiation_client_data = Some(data);
    }

    /// Get mutable reference to the `conversation_initiation_client_data` field.
    pub fn init_data_mut(&mut self) -> Option<&mut ConversationInitiationClientData> {
        self.conversation_initiation_client_data.as_mut()
    }

    pub async fn start_conversation<S>(&mut self, stream: S) -> Result<ConversationStream>
    where
        S: Stream<Item = String> + Send + Sync + 'static,
    {
        let url = self.get_url().await?;

        let (socket, _) = connect_async(url)
            .await
            .map_err(ConvAIError::WebSocketError)?;
        let (mut ws_writer, ws_reader) = socket.split();

        // Send the conversation initiation message to the server if it exists
        if let Some(data) = &self.conversation_initiation_client_data {
            ws_writer
                .send(Message::try_from(data.clone())?)
                .await
                .map_err(ConvAIError::WebSocketError)?;
        }

        let (caller_tx, caller_rx) = unbounded_channel::<Result<ServerMessage>>();
        let (writer_task_tx, writer_task_rx) = unbounded_channel::<Message>();
        self.writer_task_tx = Some(writer_task_tx.clone());

        // Writer task
        tokio::spawn(Self::websocket_writer(writer_task_rx, ws_writer));

        // Reader task
        tokio::spawn(Self::websocket_reader(
            ws_reader,
            caller_tx.clone(),
            writer_task_tx.clone(),
        ));

        // Audio chunk sender task
        tokio::spawn(Self::audio_chunk_sender(stream, writer_task_tx));

        Ok(UnboundedReceiverStream::new(caller_rx))
    }

    pub async fn stop_conversation(&mut self) -> Result<()> {
        let close_frame = CloseFrame {
            code: CloseCode::Normal,
            reason: Cow::from("user stopped conversation"),
        };

        let close_message = Message::Close(Some(close_frame));

        self.writer_task_tx
            .as_ref()
            .unwrap()
            .send(close_message)
            .map_err(|_| ConvAIError::SendError)?;

        Ok(())
    }

    async fn get_url(&self) -> Result<String> {
        if let Some(key) = &self.api_key {
            let signed_url = ElevenLabsClient::new(key)
                .hit(GetSignedUrl::new(&self.agent_id))
                .await?;
            Ok(signed_url.signed_url)
        } else {
            Ok(format!(
                "{}/{}?{}={}",
                WS_BASE_URL, WS_CONVAI_PATH, AGENT_ID_QUERY, &self.agent_id
            ))
        }
    }

    /// Handles writing messages to the WebSocket.
    async fn websocket_writer(
        mut rx: UnboundedReceiver<Message>,
        mut ws_writer: WebSocketWriter,
    ) -> Result<()> {
        while let Some(message) = rx.recv().await {
            ws_writer
                .send(message)
                .await
                .map_err(ConvAIError::WebSocketError)?;
        }
        Ok(())
    }

    /// Handles reading messages from the WebSocket.
    async fn websocket_reader(
        mut ws_reader: WebSocketReader,
        tx_to_caller: UnboundedSender<Result<ServerMessage>>,
        tx_to_writer: UnboundedSender<Message>,
    ) -> Result<()> {
        while let Some(message) = ws_reader.next().await {
            let message = message.map_err(ConvAIError::WebSocketError)?;
            Self::process_websocket_message(message, &tx_to_caller, &tx_to_writer)?;
        }
        Ok(())
    }

    // Processes a single WebSocket message.
    fn process_websocket_message(
        message: Message,
        tx_to_caller: &UnboundedSender<Result<ServerMessage>>,
        tx_to_writer: &UnboundedSender<Message>,
    ) -> Result<()> {
        match message {
            Message::Text(text) => {
                let server_msg = ServerMessage::try_from(text.as_str())?;
                if server_msg.is_ping() {
                    let ping = server_msg.as_ping().unwrap();
                    tx_to_writer
                        .send(Message::try_from(Pong::new(ping.ping_event.event_id))?)
                        .map_err(|_| ConvAIError::SendError)?;
                }
                tx_to_caller
                    .send(Ok(server_msg))
                    .map_err(|_| ConvAIError::SendError)?;
            }
            Message::Close(frame) => {
                if let Some(close_frame) = frame {
                    if close_frame.code != CloseCode::Normal {
                        warn!(
                            "WebSocket closed: code={:?}, reason={}",
                            close_frame.code, close_frame.reason
                        );
                        tx_to_caller
                            .send(Err(ConvAIError::NonNormalCloseCode(
                                close_frame.reason.into_owned(),
                            )))
                            .map_err(|_| ConvAIError::SendError)?;
                    }
                } else {
                    warn!("WebSocket closed without a close frame");
                    tx_to_caller
                        .send(Err(ConvAIError::ClosedWithoutCloseFrame))
                        .map_err(|_| ConvAIError::SendError)?;
                }
            }
            Message::Ping(ping) => {
                tx_to_writer
                    .send(Message::Pong(ping))
                    .map_err(|_| ConvAIError::SendError)?;
            }
            unexpected => {
                warn!("Unexpected websocket message: {:?}", unexpected);
                tx_to_caller
                    .send(Err(ConvAIError::UnexpectedMessageType))
                    .map_err(|_| ConvAIError::SendError)?;
            }
        }
        Ok(())
    }

    /// Handles sending audio chunks to the WebSocket writer task.
    async fn audio_chunk_sender<S>(stream: S, tx_to_writer: UnboundedSender<Message>) -> Result<()>
    where
        S: Stream<Item = String> + Send + Sync + 'static,
    {
        pin_mut!(stream);
        while let Some(audio_chunk) = stream.next().await {
            let chunk = UserAudioChunk::new(audio_chunk);
            tx_to_writer
                .send(Message::try_from(chunk)?)
                .map_err(|_| ConvAIError::SendError)?;
        }
        Ok(())
    }

    /// Send a `ClientToolResult` message to the server.
    pub async fn send_tool_result(&self, result: ClientToolResult) -> Result<()> {
        if let Some(tx_to_writer) = &self.writer_task_tx {
            tx_to_writer
                .send(Message::try_from(result)?)
                .map_err(|_| ConvAIError::SendError)?;
        }
        Ok(())
    }

    /// Send a `ContextualUpdate` message to the server.
    pub async fn send_context_update(&self, context: impl Into<String>) -> Result<()> {
        if let Some(tx_to_writer) = &self.writer_task_tx {
            tx_to_writer
                .send(Message::try_from(ContextualUpdate::new(context))?)
                .map_err(|_| ConvAIError::SendError)?;
        }
        Ok(())
    }
}
