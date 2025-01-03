//#![deny(missing_docs)]
use super::{ElevenLabsClient, Message, Result};
use crate::conversational_ai::client_messages::{
    ConversationInitiationClientData, Pong, UserAudioChunk,
};
use crate::conversational_ai::error::ElevenLabsConversationalError;
use crate::conversational_ai::server_messages::ServerMessage;
use crate::endpoints::convai::conversations::GetSignedUrl;
use futures_util::{pin_mut, SinkExt, Stream, StreamExt};
use reqwest;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

const WS_BASE_URL: &str = "wss://api.elevenlabs.io";
const WS_CONVAI_PATH: &str = "/v1/conversational_ai/conversation";

type ConversationStream = UnboundedReceiverStream<Result<ServerMessage>>;

/// Represents a client for interacting with the ElevenLabs Conversational AI.
#[derive(Clone)]
pub struct ElevenLabsConversationalClient {
    api_key: Option<String>,
    agent_id: String,
    conversation_id: Option<String>,
    cancellation_sender: Option<UnboundedSender<()>>,
    conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}


impl ElevenLabsConversationalClient {
    /// Creates a new `ConvAIClient` from environment variables.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            api_key: Some(std::env::var("ELEVENLABS_API_KEY")?),
            agent_id: std::env::var("ELEVENLABS_AGENT_ID")?,
            conversation_id: None,
            cancellation_sender: None,
            conversation_initiation_client_data: None,
        })
    }

    /// Creates a new `ElevenLabsConversationalClient` with the given API key and agent ID.
    pub fn new<T: Into<String>>(api_key: T, agent_id: T) -> Self {
        Self {
            api_key: Some(api_key.into()),
            agent_id: agent_id.into(),
            conversation_id: None,
            cancellation_sender: None,
            conversation_initiation_client_data: None,
        }
    }
    /// Sets initial data to be sent to the server when starting a conversation.
    pub fn with_conversation_initiation_client_data(
        mut self,
        data: ConversationInitiationClientData,
    ) -> Self {
        self.conversation_initiation_client_data = Some(data);
        self
    }

    pub async fn start_conversation<S>(&mut self, stream: S) -> Result<ConversationStream>
    where
        S: Stream<Item = String> + Send + Sync + 'static,
    {
        let url = self.get_signed_url().await?;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(ElevenLabsConversationalError::WebSocketError)?;
        let (mut ws_writer, mut ws_reader) = ws_stream.split();

        // Send the conversation initiation message to the server if it exists
        if let Some(data) = &self.conversation_initiation_client_data {
            ws_writer
                .send(Message::try_from(data.clone())?)
                .await
                .map_err(ElevenLabsConversationalError::WebSocketError)?;
        }

        let (tx_to_caller, rx_for_caller) = unbounded_channel::<Result<ServerMessage>>();
        let (tx_to_writer_task, mut rx_in_writer_task) = unbounded_channel::<Message>();

        // Cancellation channel
        let (tx_to_cancellation, mut rx_for_cancellation) = unbounded_channel::<()>();
        self.cancellation_sender = Some(tx_to_cancellation);

        // Writer task
        let writer_tx = tx_to_writer_task.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = Self::websocket_writer(rx_in_writer_task, &mut ws_writer) => {},
                _ = rx_for_cancellation.recv() => {
                    let _ = ws_writer.close().await;
                }
            }
        });

        // Reader task
        tokio::spawn(Self::websocket_reader(
            ws_reader,
            tx_to_caller,
            tx_to_writer_task.clone(),
        ));

        // Audio chunk sender task
        tokio::spawn(Self::audio_chunk_sender(stream, tx_to_writer_task));

        Ok(UnboundedReceiverStream::new(rx_for_caller))
    }

    pub async fn stop_conversation(&self) -> Result<()> {
        if let Some(tx) = &self.cancellation_sender {
            tx.send(())
                .map_err(|_| ElevenLabsConversationalError::CancellationError)?;
        }
        Ok(())
    }

    async fn get_signed_url(&self) -> Result<String> {
        if let Some(key) = &self.api_key {
            let signed_url = ElevenLabsClient::new(key)
                .hit(GetSignedUrl::new(&self.agent_id))
                .await
                .expect("Failed to get signed URL");
            Ok(signed_url.as_str().into())
        } else {
            Ok(format!(
                "{}/{}?agent_id={}",
                WS_BASE_URL, WS_CONVAI_PATH, &self.agent_id
            ))
        }
    }

    /// Handles writing messages to the WebSocket.
    async fn websocket_writer(
        mut rx: tokio::sync::mpsc::UnboundedReceiver<Message>,
        ws_writer: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
    ) -> Result<()> {
        while let Some(message) = rx.recv().await {
            ws_writer
                .send(message)
                .await
                .map_err(ElevenLabsConversationalError::WebSocketError)?;
        }
        Ok(())
    }

    /// Handles reading messages from the WebSocket.
    async fn websocket_reader(
        mut ws_reader: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        tx_to_caller: UnboundedSender<Result<ServerMessage>>,
        tx_to_writer: UnboundedSender<Message>,
    ) -> Result<()> {
        while let Some(message) = ws_reader.next().await {
            let message = message.map_err(ElevenLabsConversationalError::WebSocketError)?;
            Self::process_websocket_message(message, &tx_to_caller, &tx_to_writer)?;
        }
        Ok(())
    }

    /// Processes a single WebSocket message.
    fn process_websocket_message(
        message: Message,
        tx_to_caller: &UnboundedSender<Result<ServerMessage>>,
        tx_to_writer: &UnboundedSender<Message>,
    ) -> Result<()> {
        match message {
            Message::Text(text) => {
                let response =
                    ServerMessage::try_from(text.as_str())?;
                if response.is_ping() {
                    if let Some(ping) = response.as_ping() {
                        tx_to_writer
                            .send(Message::try_from(Pong::new(ping.id()))?)
                            .map_err(|_| ElevenLabsConversationalError::SendError)?;
                    }
                }
                tx_to_caller
                    .send(Ok(response))
                    .map_err(|_| ElevenLabsConversationalError::SendError)?;
            }
            Message::Close(frame) => {
                if let Some(close_frame) = frame {
                    if close_frame.code != CloseCode::Normal {
                        tx_to_caller
                            .send(Err(ElevenLabsConversationalError::NonNormalCloseCode(
                                close_frame.reason.into_owned(),
                            )))
                            .map_err(|_| ElevenLabsConversationalError::SendError)?;
                    }
                } else {
                    tx_to_caller
                        .send(Err(ElevenLabsConversationalError::ClosedWithoutCloseFrame))
                        .map_err(|_| ElevenLabsConversationalError::SendError)?;
                }
            }
            Message::Ping(ping) => {
                tx_to_writer
                    .send(Message::Pong(ping))
                    .map_err(|_| ElevenLabsConversationalError::SendError)?;
            }
            _ => {
                tx_to_caller
                    .send(Err(ElevenLabsConversationalError::UnexpectedMessageType))
                    .map_err(|_| ElevenLabsConversationalError::SendError)?;
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
                .map_err(|_| ElevenLabsConversationalError::SendError)?;
        }
        Ok(())
    }
}
