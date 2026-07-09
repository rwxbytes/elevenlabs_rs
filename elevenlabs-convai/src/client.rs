use crate::error::ConvAIError;
use crate::messages::client_messages::{
    ClientToolResult, ContextualUpdate, ConversationInitiationClientData, Pong, UserActivity,
    UserAudioChunk, UserMessage,
};
use crate::messages::server_messages::ServerMessage;
use crate::Result;
use elevenlabs_rs::endpoints::convai::conversations::GetSignedUrl;
use elevenlabs_rs::ElevenLabsClient;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{pin_mut, SinkExt, Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, warn};

const DEFAULT_INBOUND_BUFFER: usize = 64;
const DEFAULT_OUTBOUND_BUFFER: usize = 64;

type WebSocketWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WebSocketReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Options controlling the local websocket task queues.
#[derive(Clone, Debug)]
pub struct AgentWebSocketOptions {
    pub inbound_buffer: usize,
    pub outbound_buffer: usize,
}

impl Default for AgentWebSocketOptions {
    fn default() -> Self {
        Self {
            inbound_buffer: DEFAULT_INBOUND_BUFFER,
            outbound_buffer: DEFAULT_OUTBOUND_BUFFER,
        }
    }
}

/// Completion state for one background websocket task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentWebSocketTaskStatus {
    Completed,
    Failed(String),
    Aborted,
    JoinError(String),
    NotStarted,
}

/// Summary returned by [`AgentWebSocketSession::join`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentWebSocketSessionReport {
    pub reader: AgentWebSocketTaskStatus,
    pub writer: AgentWebSocketTaskStatus,
    pub audio: AgentWebSocketTaskStatus,
}

/// A running Conversational AI websocket session.
///
/// The session implements [`Stream`] for inbound server messages and owns the
/// reader, writer, and audio-forwarding tasks. Prefer calling [`close`](Self::close)
/// and then [`join`](Self::join) when ending a session. Dropping the session
/// aborts any still-running tasks as a fallback.
#[derive(Debug)]
pub struct AgentWebSocketSession {
    inbound: ReceiverStream<Result<ServerMessage>>,
    writer_tx: Sender<Message>,
    reader_task: Option<JoinHandle<Result<()>>>,
    writer_task: Option<JoinHandle<Result<()>>>,
    audio_task: Option<JoinHandle<Result<()>>>,
    closed: bool,
}

impl AgentWebSocketSession {
    fn new(
        inbound_rx: Receiver<Result<ServerMessage>>,
        writer_tx: Sender<Message>,
        reader_task: JoinHandle<Result<()>>,
        writer_task: JoinHandle<Result<()>>,
        audio_task: JoinHandle<Result<()>>,
    ) -> Self {
        Self {
            inbound: ReceiverStream::new(inbound_rx),
            writer_tx,
            reader_task: Some(reader_task),
            writer_task: Some(writer_task),
            audio_task: Some(audio_task),
            closed: false,
        }
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub async fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }

        let close_frame = CloseFrame {
            code: CloseCode::Normal,
            reason: "user stopped conversation".into(),
        };
        self.send_frame(Message::Close(Some(close_frame))).await?;
        self.closed = true;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.closed = true;
        abort_task(&mut self.reader_task);
        abort_task(&mut self.writer_task);
        abort_task(&mut self.audio_task);
    }

    pub async fn join(&mut self) -> AgentWebSocketSessionReport {
        self.closed = true;
        AgentWebSocketSessionReport {
            reader: join_task("reader", self.reader_task.take()).await,
            writer: join_task("writer", self.writer_task.take()).await,
            audio: join_task("audio", self.audio_task.take()).await,
        }
    }

    pub async fn send_tool_result(&self, result: ClientToolResult) -> Result<()> {
        self.send_frame(Message::try_from(result)?).await
    }

    pub async fn send_context_update(&self, context: impl Into<String>) -> Result<()> {
        self.send_frame(Message::try_from(ContextualUpdate::new(context))?)
            .await
    }

    pub async fn send_user_message(&self, text: impl Into<String>) -> Result<()> {
        self.send_frame(Message::try_from(UserMessage::new(text))?)
            .await
    }

    pub async fn send_user_activity(&self) -> Result<()> {
        self.send_frame(Message::try_from(UserActivity::new())?)
            .await
    }

    async fn send_frame(&self, message: Message) -> Result<()> {
        if self.closed {
            return Err(ConvAIError::SessionClosed);
        }

        self.writer_tx
            .send(message)
            .await
            .map_err(|_| ConvAIError::SendQueueClosed)
    }
}

impl Stream for AgentWebSocketSession {
    type Item = Result<ServerMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = Pin::new(&mut self.inbound).poll_next(cx);
        if matches!(poll, Poll::Ready(None)) {
            self.closed = true;
        }
        poll
    }
}

impl Drop for AgentWebSocketSession {
    fn drop(&mut self) {
        self.abort();
    }
}

/// Represents a client for interacting with the ElevenLabs Conversational AI.
#[derive(Debug)]
pub struct AgentWebSocket {
    api_key: String,
    agent_id: String,
    writer_task_tx: Option<Sender<Message>>,
    conversation_initiation_client_data: Option<ConversationInitiationClientData>,
    options: AgentWebSocketOptions,
    #[cfg(test)]
    signed_url_override: Option<String>,
}

impl AgentWebSocket {
    /// Creates a new `AgentWebSocket` from environment variables.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            api_key: std::env::var("ELEVENLABS_API_KEY")?,
            agent_id: std::env::var("ELEVENLABS_AGENT_ID")?,
            conversation_initiation_client_data: None,
            writer_task_tx: None,
            options: AgentWebSocketOptions::default(),
            #[cfg(test)]
            signed_url_override: None,
        })
    }

    /// Creates a new `AgentWebSocket` with the given API key and agent ID.
    pub fn new<T: Into<String>>(api_key: T, agent_id: T) -> Self {
        Self {
            api_key: api_key.into(),
            agent_id: agent_id.into(),
            conversation_initiation_client_data: None,
            writer_task_tx: None,
            options: AgentWebSocketOptions::default(),
            #[cfg(test)]
            signed_url_override: None,
        }
    }

    pub fn with_options(mut self, options: AgentWebSocketOptions) -> Self {
        self.options = options;
        self
    }

    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    pub fn options(&self) -> &AgentWebSocketOptions {
        &self.options
    }

    pub fn conversation_initiation_client_data(&self) -> Option<&ConversationInitiationClientData> {
        self.conversation_initiation_client_data.as_ref()
    }

    #[cfg(test)]
    fn with_signed_url_override(mut self, signed_url: impl Into<String>) -> Self {
        self.signed_url_override = Some(signed_url.into());
        self
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

    pub async fn start_conversation<S>(&mut self, stream: S) -> Result<AgentWebSocketSession>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        let url = self.get_url().await?;

        let (socket, _) = connect_async(url).await.map_err(ConvAIError::from)?;
        let (mut ws_writer, ws_reader) = socket.split();

        if let Some(data) = &self.conversation_initiation_client_data {
            ws_writer
                .send(Message::try_from(data.clone())?)
                .await
                .map_err(ConvAIError::from)?;
        }

        let (caller_tx, caller_rx) = channel::<Result<ServerMessage>>(self.options.inbound_buffer);
        let (writer_task_tx, writer_task_rx) = channel::<Message>(self.options.outbound_buffer);
        self.writer_task_tx = Some(writer_task_tx.clone());

        let writer_task = tokio::spawn(Self::websocket_writer(
            writer_task_rx,
            ws_writer,
            caller_tx.clone(),
        ));
        let reader_task = tokio::spawn(Self::websocket_reader(
            ws_reader,
            caller_tx.clone(),
            writer_task_tx.clone(),
        ));
        let audio_task = tokio::spawn(Self::audio_chunk_sender(
            stream,
            writer_task_tx.clone(),
            caller_tx,
        ));

        Ok(AgentWebSocketSession::new(
            caller_rx,
            writer_task_tx,
            reader_task,
            writer_task,
            audio_task,
        ))
    }

    pub async fn stop_conversation(&mut self) -> Result<()> {
        let close_frame = CloseFrame {
            code: CloseCode::Normal,
            reason: "user stopped conversation".into(),
        };

        self.send_frame(Message::Close(Some(close_frame))).await
    }

    async fn get_url(&self) -> Result<String> {
        #[cfg(test)]
        if let Some(signed_url) = &self.signed_url_override {
            return Ok(signed_url.clone());
        }

        let signed_url = ElevenLabsClient::new(&self.api_key)
            .hit(GetSignedUrl::new(&self.agent_id))
            .await?;
        Ok(signed_url.signed_url)
    }

    async fn send_frame(&self, message: Message) -> Result<()> {
        let tx = self
            .writer_task_tx
            .as_ref()
            .ok_or(ConvAIError::SessionClosed)?;
        tx.send(message)
            .await
            .map_err(|_| ConvAIError::SendQueueClosed)
    }

    async fn websocket_writer(
        mut rx: Receiver<Message>,
        mut ws_writer: WebSocketWriter,
        tx_to_caller: Sender<Result<ServerMessage>>,
    ) -> Result<()> {
        while let Some(message) = rx.recv().await {
            let is_close = matches!(message, Message::Close(_));
            if let Err(error) = ws_writer.send(message).await {
                let _ = tx_to_caller
                    .send(Err(ConvAIError::from(error)))
                    .await
                    .map_err(|_| ConvAIError::SendQueueClosed);
                return Ok(());
            }

            if is_close {
                return Ok(());
            }
        }
        Ok(())
    }

    async fn websocket_reader(
        mut ws_reader: WebSocketReader,
        tx_to_caller: Sender<Result<ServerMessage>>,
        tx_to_writer: Sender<Message>,
    ) -> Result<()> {
        while let Some(message) = ws_reader.next().await {
            let message = match message {
                Ok(message) => message,
                Err(error) => {
                    forward_error(&tx_to_caller, ConvAIError::from(error)).await?;
                    return Ok(());
                }
            };

            match Self::process_websocket_message(message, &tx_to_caller, &tx_to_writer).await {
                Ok(true) => {}
                Ok(false) => return Ok(()),
                Err(error) => {
                    let _ = forward_error(&tx_to_caller, error).await;
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn process_websocket_message(
        message: Message,
        tx_to_caller: &Sender<Result<ServerMessage>>,
        tx_to_writer: &Sender<Message>,
    ) -> Result<bool> {
        match message {
            Message::Text(text) => {
                let server_msg = ServerMessage::try_from(text.as_str())?;
                if let ServerMessage::Ping(ping) = &server_msg {
                    tx_to_writer
                        .send(Message::try_from(Pong::new(ping.ping_event.event_id))?)
                        .await
                        .map_err(|_| ConvAIError::SendQueueClosed)?;
                }
                tx_to_caller
                    .send(Ok(server_msg))
                    .await
                    .map_err(|_| ConvAIError::SendQueueClosed)?;
                Ok(true)
            }
            Message::Close(frame) => {
                if let Some(close_frame) = frame {
                    if close_frame.code != CloseCode::Normal {
                        warn!(
                            "WebSocket closed: code={:?}, reason={}",
                            close_frame.code, close_frame.reason
                        );
                        forward_error(
                            tx_to_caller,
                            ConvAIError::NonNormalClose {
                                code: format!("{:?}", close_frame.code),
                                reason: close_frame.reason.to_string(),
                            },
                        )
                        .await?;
                    }
                } else {
                    warn!("WebSocket closed without a close frame");
                    forward_error(tx_to_caller, ConvAIError::ClosedWithoutCloseFrame).await?;
                }
                Ok(false)
            }
            Message::Ping(ping) => {
                tx_to_writer
                    .send(Message::Pong(ping))
                    .await
                    .map_err(|_| ConvAIError::SendQueueClosed)?;
                Ok(true)
            }
            Message::Pong(_) => Ok(true),
            unexpected => {
                warn!("Unexpected websocket message: {:?}", unexpected);
                forward_error(
                    tx_to_caller,
                    ConvAIError::UnexpectedFrame {
                        expected: "text, ping, pong, or close",
                        received: frame_kind(&unexpected),
                    },
                )
                .await?;
                Ok(false)
            }
        }
    }

    async fn audio_chunk_sender<S>(
        stream: S,
        tx_to_writer: Sender<Message>,
        tx_to_caller: Sender<Result<ServerMessage>>,
    ) -> Result<()>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pin_mut!(stream);
        while let Some(audio_chunk) = stream.next().await {
            let chunk = UserAudioChunk::new(audio_chunk);
            let message = match Message::try_from(chunk) {
                Ok(message) => message,
                Err(error) => {
                    let _ = forward_error(&tx_to_caller, error).await;
                    return Ok(());
                }
            };

            if tx_to_writer
                .send(message)
                .await
                .map_err(|_| ConvAIError::SendQueueClosed)
                .is_err()
            {
                debug!("audio sender stopped because writer queue is closed");
                return Ok(());
            }
        }
        Ok(())
    }

    /// Send a `ClientToolResult` message to the server.
    pub async fn send_tool_result(&self, result: ClientToolResult) -> Result<()> {
        self.send_frame(Message::try_from(result)?).await
    }

    /// Send a `ContextualUpdate` message to the server.
    pub async fn send_context_update(&self, context: impl Into<String>) -> Result<()> {
        self.send_frame(Message::try_from(ContextualUpdate::new(context))?)
            .await
    }

    /// Send a text user message into the active conversation.
    pub async fn send_user_message(&self, text: impl Into<String>) -> Result<()> {
        self.send_frame(Message::try_from(UserMessage::new(text))?)
            .await
    }

    /// Notify the active conversation of user activity.
    pub async fn send_user_activity(&self) -> Result<()> {
        self.send_frame(Message::try_from(UserActivity::new())?)
            .await
    }
}

async fn forward_error(
    tx_to_caller: &Sender<Result<ServerMessage>>,
    error: ConvAIError,
) -> Result<()> {
    tx_to_caller
        .send(Err(error))
        .await
        .map_err(|_| ConvAIError::SendQueueClosed)
}

fn abort_task(task: &mut Option<JoinHandle<Result<()>>>) {
    if let Some(task) = task {
        task.abort();
    }
}

async fn join_task(
    task_name: &'static str,
    task: Option<JoinHandle<Result<()>>>,
) -> AgentWebSocketTaskStatus {
    let Some(task) = task else {
        return AgentWebSocketTaskStatus::NotStarted;
    };

    match task.await {
        Ok(Ok(())) => AgentWebSocketTaskStatus::Completed,
        Ok(Err(error)) => AgentWebSocketTaskStatus::Failed(error.to_string()),
        Err(error) if error.is_cancelled() => AgentWebSocketTaskStatus::Aborted,
        Err(error) => AgentWebSocketTaskStatus::JoinError(format!("{task_name}: {error}")),
    }
}

fn frame_kind(message: &Message) -> &'static str {
    match message {
        Message::Text(_) => "text",
        Message::Binary(_) => "binary",
        Message::Ping(_) => "ping",
        Message::Pong(_) => "pong",
        Message::Close(_) => "close",
        Message::Frame(_) => "frame",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;
    use serde_json::{json, Value};
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

    #[test]
    fn server_messages_are_tagged_and_unknown_safe() {
        let ping: ServerMessage = serde_json::from_value(json!({
            "type": "ping",
            "ping_event": { "event_id": 42, "ping_ms": 10 }
        }))
        .unwrap();
        assert!(ping.is_ping());
        assert_eq!(ping.message_type(), "ping");

        let unknown: ServerMessage = serde_json::from_value(json!({
            "type": "new_event",
            "payload": { "value": true }
        }))
        .unwrap();
        assert_eq!(unknown.message_type(), "new_event");
        assert!(unknown.as_unknown().is_some());
    }

    #[tokio::test]
    async fn session_sends_audio_and_replies_to_protocol_ping() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("ws://{}", listener.local_addr().unwrap());

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = accept_async(stream).await.unwrap();

            let audio = ws.next().await.unwrap().unwrap();
            let Message::Text(audio) = audio else {
                panic!("expected audio text frame");
            };
            assert_eq!(
                serde_json::from_str::<Value>(&audio).unwrap(),
                json!({ "user_audio_chunk": "base64-audio" })
            );

            ws.send(Message::Text(
                json!({
                    "type": "ping",
                    "ping_event": { "event_id": 7, "ping_ms": 1 }
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();

            let pong = ws.next().await.unwrap().unwrap();
            let Message::Text(pong) = pong else {
                panic!("expected pong text frame");
            };
            assert_eq!(
                serde_json::from_str::<Value>(&pong).unwrap(),
                json!({ "type": "pong", "event_id": 7 })
            );

            let close = ws.next().await.unwrap().unwrap();
            assert!(matches!(close, Message::Close(_)));
        });

        let mut client =
            AgentWebSocket::new("api-key", "agent/id").with_signed_url_override(base_url);
        let audio = stream::iter(["base64-audio".to_owned()]);
        let mut session = client.start_conversation(audio).await.unwrap();

        let message = session.next().await.unwrap().unwrap();
        assert!(message.is_ping());
        session.close().await.unwrap();

        let report = session.join().await;
        assert_eq!(report.writer, AgentWebSocketTaskStatus::Completed);
        assert_eq!(report.audio, AgentWebSocketTaskStatus::Completed);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn session_can_send_tool_result_and_context_update() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("ws://{}", listener.local_addr().unwrap());

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = accept_async(stream).await.unwrap();

            let tool = ws.next().await.unwrap().unwrap();
            let Message::Text(tool) = tool else {
                panic!("expected tool result frame");
            };
            assert_eq!(
                serde_json::from_str::<Value>(&tool).unwrap(),
                json!({
                    "type": "client_tool_result",
                    "tool_call_id": "tool-1",
                    "result": "done"
                })
            );

            let context = ws.next().await.unwrap().unwrap();
            let Message::Text(context) = context else {
                panic!("expected context update frame");
            };
            assert_eq!(
                serde_json::from_str::<Value>(&context).unwrap(),
                json!({
                    "type": "contextual_update",
                    "text": "user changed topic"
                })
            );

            let close = ws.next().await.unwrap().unwrap();
            assert!(matches!(close, Message::Close(_)));
        });

        let mut client =
            AgentWebSocket::new("api-key", "agent/id").with_signed_url_override(base_url);
        let mut session = client
            .start_conversation(stream::iter(Vec::<String>::new()))
            .await
            .unwrap();

        session
            .send_tool_result(ClientToolResult::new("tool-1").with_result("done".to_owned()))
            .await
            .unwrap();
        session
            .send_context_update("user changed topic")
            .await
            .unwrap();
        session.close().await.unwrap();

        let report = session.join().await;
        assert_eq!(report.writer, AgentWebSocketTaskStatus::Completed);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn non_normal_close_is_forwarded_to_stream() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("ws://{}", listener.local_addr().unwrap());

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = accept_async(stream).await.unwrap();
            ws.send(Message::Close(Some(CloseFrame {
                code: CloseCode::Policy,
                reason: "bad request".into(),
            })))
            .await
            .unwrap();
        });

        let mut client =
            AgentWebSocket::new("api-key", "agent/id").with_signed_url_override(base_url);
        let mut session = client
            .start_conversation(stream::iter(Vec::<String>::new()))
            .await
            .unwrap();

        let error = session.next().await.unwrap().unwrap_err();
        assert!(matches!(
            error,
            ConvAIError::NonNormalClose { reason, .. } if reason == "bad request"
        ));
        session.abort();
        let report = session.join().await;
        assert_eq!(report.reader, AgentWebSocketTaskStatus::Completed);
        server.await.unwrap();
    }
}
