use crate::client::Result;
use crate::error::Error;
use crate::error::{WebSocketDirection, WebSocketError, WebSocketErrorContext};
use futures_util::{SinkExt, Stream, StreamExt};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::Message;

const DEFAULT_INBOUND_BUFFER: usize = 64;
const DEFAULT_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
const XI_API_KEY_HEADER: &str = "xi-api-key";

/// Runtime settings for ElevenLabs WebSocket sessions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WebSocketOptions {
    /// Number of inbound server messages buffered while the caller is not
    /// polling the returned [`WebSocketSession`].
    pub inbound_buffer: usize,
    /// How long [`WebSocketSession::close`] waits for the writer task to send a
    /// close frame before aborting the session.
    pub close_timeout: Duration,
}

impl WebSocketOptions {
    pub const fn new() -> Self {
        Self {
            inbound_buffer: DEFAULT_INBOUND_BUFFER,
            close_timeout: DEFAULT_CLOSE_TIMEOUT,
        }
    }

    pub const fn with_inbound_buffer(mut self, inbound_buffer: usize) -> Self {
        self.inbound_buffer = inbound_buffer;
        self
    }

    pub const fn with_close_timeout(mut self, close_timeout: Duration) -> Self {
        self.close_timeout = close_timeout;
        self
    }

    fn normalized(self) -> Self {
        Self {
            inbound_buffer: self.inbound_buffer.max(1),
            close_timeout: self.close_timeout,
        }
    }
}

impl Default for WebSocketOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Completion status for one background WebSocket task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebSocketTaskStatus {
    Completed,
    Failed { error: String },
    Aborted,
    Panicked { error: String },
    AlreadyJoined,
}

/// Completion report for the reader and writer tasks owned by a session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebSocketSessionReport {
    pub reader: WebSocketTaskStatus,
    pub writer: WebSocketTaskStatus,
}

pub(crate) enum WebSocketAuth {
    XiApiKeyHeader,
    None,
}

pub(crate) mod sealed {
    pub trait Sealed {}
}

pub(crate) trait WebSocketCodec {
    type Input: Send + 'static;
    type Output: Send + 'static;

    fn encode(input: Self::Input, context: WebSocketErrorContext) -> Result<Message>;
    fn decode(message: Message, context: WebSocketErrorContext) -> Result<Option<Self::Output>>;
}

pub(crate) struct JsonTextCodec<I, O>(PhantomData<(I, O)>);

impl<I, O> WebSocketCodec for JsonTextCodec<I, O>
where
    I: Serialize + Send + 'static,
    O: DeserializeOwned + Send + 'static,
{
    type Input = I;
    type Output = O;

    fn encode(input: Self::Input, context: WebSocketErrorContext) -> Result<Message> {
        Ok(Message::Text(
            serde_json::to_string(&input)
                .map_err(|source| WebSocketError::Encode { context, source })?
                .into(),
        ))
    }

    fn decode(message: Message, context: WebSocketErrorContext) -> Result<Option<Self::Output>> {
        match message {
            Message::Text(text) => {
                let response =
                    serde_json::from_str::<O>(&text).map_err(|source| WebSocketError::Decode {
                        context,
                        source,
                        payload_preview: payload_preview(&text),
                    })?;
                Ok(Some(response))
            }
            Message::Ping(_) | Message::Pong(_) => Ok(None),
            _ => Err(WebSocketError::UnexpectedFrame {
                context,
                expected: "text or close",
                received: frame_kind(&message),
            }
            .into()),
        }
    }
}

pub(crate) trait WebSocketEndpoint: sealed::Sealed + Sized + Send + 'static {
    type Codec: WebSocketCodec;
    type InputStream: Stream<Item = Result<<Self::Codec as WebSocketCodec>::Input>> + Send + 'static;

    fn url(&self) -> Result<String>;
    fn auth(&self) -> WebSocketAuth;
    fn close_after_inputs(&self) -> bool {
        false
    }
    fn endpoint_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn input_stream(self, api_key: &str) -> Result<Self::InputStream>;
}

pub(crate) async fn connect_endpoint<E>(
    endpoint: E,
    api_key: &str,
) -> Result<WebSocketSession<<E::Codec as WebSocketCodec>::Output>>
where
    E: WebSocketEndpoint,
{
    connect_endpoint_with_options(endpoint, api_key, WebSocketOptions::default()).await
}

pub(crate) async fn connect_endpoint_with_options<E>(
    endpoint: E,
    api_key: &str,
    options: WebSocketOptions,
) -> Result<WebSocketSession<<E::Codec as WebSocketCodec>::Output>>
where
    E: WebSocketEndpoint,
{
    let request = websocket_request(endpoint.url()?, endpoint.auth(), api_key)?;
    let close_after_inputs = endpoint.close_after_inputs();
    let endpoint_name = endpoint.endpoint_name();
    let input_stream = endpoint.input_stream(api_key)?;
    let (ws_stream, _) = connect_async(request).await?;

    Ok(spawn_session::<E::Codec, _, _, _>(
        ws_stream,
        move || async move { Ok(input_stream) },
        close_after_inputs,
        endpoint_name,
        options,
    ))
}

pub(crate) fn websocket_url<'a>(
    base_url: &str,
    path: &str,
    path_params: impl IntoIterator<Item = (&'a str, &'a str)>,
    query_params: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<String> {
    let mut url = Url::parse(base_url)
        .map_err(|e| Error::InvalidInput(format!("invalid websocket base URL: {e}")))?;
    let path = path.trim_start_matches('/');
    let path_params: Vec<_> = path_params.into_iter().collect();

    let mut segments: Vec<&str> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            path_params
                .iter()
                .find_map(|(placeholder, value)| (*placeholder == segment).then_some(*value))
                .unwrap_or(segment)
        })
        .collect();

    if path.ends_with('/') {
        segments.push("");
    }

    {
        let mut url_segments = url.path_segments_mut().map_err(|_| {
            Error::InvalidInput(format!(
                "websocket base URL cannot contain a relative path: {base_url}"
            ))
        })?;
        url_segments.clear();
        url_segments.extend(segments);
    }

    {
        let mut query = url.query_pairs_mut();
        for (name, value) in query_params {
            query.append_pair(name, value);
        }
    }

    Ok(url.to_string())
}

pub(crate) fn websocket_request(
    url: String,
    auth: WebSocketAuth,
    api_key: &str,
) -> Result<Request> {
    let mut request = url.into_client_request()?;

    if matches!(auth, WebSocketAuth::XiApiKeyHeader) {
        let api_key = HeaderValue::from_str(api_key)
            .map_err(|e| Error::InvalidInput(format!("invalid API key header value: {e}")))?;
        request.headers_mut().insert(XI_API_KEY_HEADER, api_key);
    }

    Ok(request)
}

enum WriterCommand {
    Close {
        response: oneshot::Sender<Result<()>>,
    },
}

/// A live ElevenLabs WebSocket session.
///
/// The session implements [`Stream`] for inbound server messages and owns the
/// background reader/writer tasks for the connection. Dropping the session
/// aborts those tasks. Call [`WebSocketSession::close`] when you want to send a
/// close frame first, or [`WebSocketSession::abort`] when you need immediate
/// cancellation.
pub struct WebSocketSession<T> {
    inner: mpsc::Receiver<Result<T>>,
    reader_task: Option<JoinHandle<WebSocketTaskStatus>>,
    writer_task: Option<JoinHandle<WebSocketTaskStatus>>,
    writer_commands: Option<mpsc::Sender<WriterCommand>>,
    endpoint: &'static str,
    options: WebSocketOptions,
    closed: bool,
}

impl<T> WebSocketSession<T> {
    fn new(
        inner: mpsc::Receiver<Result<T>>,
        reader_task: JoinHandle<WebSocketTaskStatus>,
        writer_task: JoinHandle<WebSocketTaskStatus>,
        writer_commands: mpsc::Sender<WriterCommand>,
        endpoint: &'static str,
        options: WebSocketOptions,
    ) -> Self {
        Self {
            inner,
            reader_task: Some(reader_task),
            writer_task: Some(writer_task),
            writer_commands: Some(writer_commands),
            endpoint,
            options,
            closed: false,
        }
    }

    /// Request a graceful WebSocket close frame, then stop the background tasks.
    ///
    /// This method is idempotent after the session has closed or been aborted.
    pub async fn close(&mut self) -> Result<()> {
        if self.is_closed() {
            self.closed = true;
            return Ok(());
        }

        if self
            .writer_task
            .as_ref()
            .is_none_or(JoinHandle::is_finished)
        {
            self.abort_reader();
            return Ok(());
        }

        let Some(writer_commands) = self.writer_commands.as_ref() else {
            return Err(WebSocketError::SendQueueClosed {
                endpoint: self.endpoint,
            }
            .into());
        };
        let (response_tx, response_rx) = oneshot::channel();

        writer_commands
            .send(WriterCommand::Close {
                response: response_tx,
            })
            .await
            .map_err(|_| WebSocketError::SendQueueClosed {
                endpoint: self.endpoint,
            })?;

        let result = match tokio::time::timeout(self.options.close_timeout, response_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => {
                return Err(WebSocketError::WriterFinished {
                    endpoint: self.endpoint,
                }
                .into());
            }
            Err(_) => {
                self.abort();
                return Err(WebSocketError::CloseTimeout {
                    endpoint: self.endpoint,
                    timeout: self.options.close_timeout,
                }
                .into());
            }
        };

        if result.is_ok() {
            self.abort_reader();
            self.writer_commands = None;
            self.closed = true;
        } else {
            self.abort();
        }

        result
    }

    /// Immediately abort the background reader and writer tasks.
    pub fn abort(&mut self) {
        if let Some(reader_task) = self.reader_task.as_ref() {
            reader_task.abort();
        }
        if let Some(writer_task) = self.writer_task.as_ref() {
            writer_task.abort();
        }
        self.writer_commands = None;
        self.closed = true;
    }

    /// Whether both background tasks have finished, or the session was aborted.
    pub fn is_closed(&self) -> bool {
        self.closed
            || (self
                .reader_task
                .as_ref()
                .is_none_or(JoinHandle::is_finished)
                && self
                    .writer_task
                    .as_ref()
                    .is_none_or(JoinHandle::is_finished))
    }

    /// Wait for the background reader and writer tasks to finish and return
    /// their completion status.
    ///
    /// Call this after the stream has ended, after [`WebSocketSession::close`],
    /// or after [`WebSocketSession::abort`]. Calling it while a session is still
    /// active can wait indefinitely.
    pub async fn join(&mut self) -> WebSocketSessionReport {
        let reader = join_task(self.reader_task.take()).await;
        let writer = join_task(self.writer_task.take()).await;
        self.writer_commands = None;
        self.closed = true;

        WebSocketSessionReport { reader, writer }
    }

    /// Return the configured runtime options for this session.
    pub fn options(&self) -> WebSocketOptions {
        self.options
    }

    /// Return the endpoint label used in WebSocket diagnostics.
    pub fn endpoint(&self) -> &'static str {
        self.endpoint
    }

    fn abort_reader(&mut self) {
        if let Some(reader_task) = self.reader_task.as_ref() {
            reader_task.abort();
        }
    }
}

impl<T> Stream for WebSocketSession<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = self.inner.poll_recv(cx);
        if matches!(poll, Poll::Ready(None)) {
            self.closed = true;
        }
        poll
    }
}

impl<T> Drop for WebSocketSession<T> {
    fn drop(&mut self) {
        self.abort();
    }
}

async fn join_task(task: Option<JoinHandle<WebSocketTaskStatus>>) -> WebSocketTaskStatus {
    let Some(task) = task else {
        return WebSocketTaskStatus::AlreadyJoined;
    };

    match task.await {
        Ok(status) => status,
        Err(error) if error.is_cancelled() => WebSocketTaskStatus::Aborted,
        Err(error) if error.is_panic() => WebSocketTaskStatus::Panicked {
            error: error.to_string(),
        },
        Err(error) => WebSocketTaskStatus::Failed {
            error: error.to_string(),
        },
    }
}

pub(crate) fn spawn_session<C, S, F, Fut>(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    write_messages: F,
    close_after_messages: bool,
    endpoint: &'static str,
    options: WebSocketOptions,
) -> WebSocketSession<C::Output>
where
    C: WebSocketCodec,
    S: Stream<Item = Result<C::Input>> + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<S>> + Send + 'static,
{
    let options = options.normalized();
    let (mut writer, mut reader) = ws_stream.split();
    let (tx_to_caller, rx_for_caller) = mpsc::channel(options.inbound_buffer);
    let (writer_commands_tx, mut writer_commands_rx) = mpsc::channel(1);

    let reader_tx = tx_to_caller.clone();
    let inbound_context = WebSocketErrorContext::new(endpoint, WebSocketDirection::Inbound);
    let outbound_context = WebSocketErrorContext::new(endpoint, WebSocketDirection::Outbound);
    let reader_task: JoinHandle<WebSocketTaskStatus> = tokio::spawn(async move {
        let result: Result<()> = async {
            while let Some(msg_result) = reader.next().await {
                let msg = msg_result?;
                match msg {
                    Message::Close(msg) => {
                        return handle_close_frame(msg, &reader_tx, inbound_context).await;
                    }
                    msg => {
                        if let Some(response) = C::decode(msg, inbound_context)? {
                            if reader_tx.send(Ok(response)).await.is_err() {
                                return Ok(());
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => WebSocketTaskStatus::Completed,
            Err(error) => {
                let error_message = error.to_string();
                let _ = reader_tx.send(Err(error)).await;
                WebSocketTaskStatus::Failed {
                    error: error_message,
                }
            }
        }
    });

    let writer_tx = tx_to_caller;
    let writer_task: JoinHandle<WebSocketTaskStatus> = tokio::spawn(async move {
        let result: Result<()> = async {
            let messages = write_messages().await?;
            futures_util::pin_mut!(messages);
            let mut messages_done = false;

            loop {
                if messages_done {
                    if close_after_messages {
                        writer.close().await?;
                        return Ok(());
                    }

                    match writer_commands_rx.recv().await {
                        Some(command) => {
                            handle_writer_command(command, &mut writer).await;
                            return Ok(());
                        }
                        None => return Ok(()),
                    }
                }

                tokio::select! {
                    command = writer_commands_rx.recv() => {
                        match command {
                            Some(command) => {
                                handle_writer_command(command, &mut writer).await;
                                return Ok(());
                            }
                            None => return Ok(()),
                        }
                    }
                    message = messages.next() => {
                        match message {
                            Some(message) => writer.send(C::encode(message?, outbound_context)?).await?,
                            None => messages_done = true,
                        }
                    }
                }
            }
        }
        .await;

        match result {
            Ok(()) => WebSocketTaskStatus::Completed,
            Err(error) => {
                let error_message = error.to_string();
                let _ = writer_tx.send(Err(error)).await;
                WebSocketTaskStatus::Failed {
                    error: error_message,
                }
            }
        }
    });

    WebSocketSession::new(
        rx_for_caller,
        reader_task,
        writer_task,
        writer_commands_tx,
        endpoint,
        options,
    )
}

async fn handle_writer_command<W>(command: WriterCommand, writer: &mut W)
where
    W: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    match command {
        WriterCommand::Close { response } => {
            let result = writer.close().await.map_err(Error::from);
            let _ = response.send(result);
        }
    }
}

async fn handle_close_frame<T>(
    msg: Option<tokio_tungstenite::tungstenite::protocol::CloseFrame>,
    tx: &mpsc::Sender<Result<T>>,
    context: WebSocketErrorContext,
) -> Result<()> {
    if let Some(close_frame) = msg {
        if close_frame.code == CloseCode::Normal {
            return Ok(());
        }

        let _ = tx
            .send(Err(WebSocketError::NonNormalClose {
                context,
                code: close_frame.code.to_string(),
                reason: close_frame.reason.to_string(),
            }
            .into()))
            .await;
        return Ok(());
    }

    let _ = tx
        .send(Err(
            WebSocketError::ClosedWithoutCloseFrame { context }.into()
        ))
        .await;
    Ok(())
}

fn frame_kind(message: &Message) -> &'static str {
    match message {
        Message::Text(_) => "text",
        Message::Binary(_) => "binary",
        Message::Ping(_) => "ping",
        Message::Pong(_) => "pong",
        Message::Close(_) => "close",
        Message::Frame(_) => "raw frame",
    }
}

fn payload_preview(payload: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 256;

    let mut preview: String = payload.chars().take(MAX_PREVIEW_CHARS).collect();
    if payload.chars().count() > MAX_PREVIEW_CHARS {
        preview.push_str("...");
    }
    preview
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_url_encodes_path_segments_and_query_values() {
        let url = websocket_url(
            "wss://api.elevenlabs.io",
            "/v1/text-to-speech/:voice_id/stream-input",
            [(":voice_id", "voice id/with slash")],
            [
                ("model_id", "model with spaces"),
                ("single_use_token", "token/with?chars&symbols"),
            ],
        )
        .unwrap();

        let parsed = Url::parse(&url).unwrap();
        assert_eq!(
            parsed.path(),
            "/v1/text-to-speech/voice%20id%2Fwith%20slash/stream-input"
        );
        let query_pairs: Vec<_> = parsed.query_pairs().collect();
        assert!(query_pairs.contains(&("model_id".into(), "model with spaces".into())));
        assert!(
            query_pairs.contains(&("single_use_token".into(), "token/with?chars&symbols".into()))
        );
    }

    #[test]
    fn websocket_request_applies_header_auth_only_when_requested() {
        let request = websocket_request(
            "wss://api.elevenlabs.io/v1/speech-to-text/realtime".to_string(),
            WebSocketAuth::XiApiKeyHeader,
            "test-key",
        )
        .unwrap();
        assert_eq!(
            request.headers().get(XI_API_KEY_HEADER).unwrap(),
            "test-key"
        );

        let request = websocket_request(
            "wss://api.elevenlabs.io/v1/speech-to-text/realtime?token=one-use".to_string(),
            WebSocketAuth::None,
            "test-key",
        )
        .unwrap();
        assert!(request.headers().get(XI_API_KEY_HEADER).is_none());
    }

    #[test]
    fn payload_preview_is_bounded_without_splitting_unicode_scalars() {
        let payload = format!("{}{}", "a".repeat(256), "\u{2603}".repeat(20));
        let preview = payload_preview(&payload);

        assert!(preview.ends_with("..."));
        assert_eq!(preview.trim_end_matches("...").chars().count(), 256);
    }

    #[tokio::test]
    async fn session_close_sends_writer_command_and_marks_closed() {
        let (_inbound_tx, inbound_rx) = mpsc::channel::<Result<()>>(1);
        let (command_tx, mut command_rx) = mpsc::channel(1);

        let reader_task = tokio::spawn(async { WebSocketTaskStatus::Completed });
        let writer_task = tokio::spawn(async move {
            let Some(WriterCommand::Close { response }) = command_rx.recv().await else {
                panic!("expected close command");
            };
            let _ = response.send(Ok(()));
            WebSocketTaskStatus::Completed
        });

        let mut session: WebSocketSession<()> = WebSocketSession::new(
            inbound_rx,
            reader_task,
            writer_task,
            command_tx,
            "test.websocket",
            WebSocketOptions::default(),
        );

        assert!(!session.is_closed());
        session.close().await.unwrap();
        assert!(session.is_closed());
        assert_eq!(session.endpoint(), "test.websocket");
    }

    #[tokio::test]
    async fn session_abort_marks_closed_and_aborts_tasks() {
        let (_inbound_tx, inbound_rx) = mpsc::channel::<Result<()>>(1);
        let (command_tx, _command_rx) = mpsc::channel(1);

        let reader_task =
            tokio::spawn(async { std::future::pending::<WebSocketTaskStatus>().await });
        let writer_task =
            tokio::spawn(async { std::future::pending::<WebSocketTaskStatus>().await });

        let mut session: WebSocketSession<()> = WebSocketSession::new(
            inbound_rx,
            reader_task,
            writer_task,
            command_tx,
            "test.websocket",
            WebSocketOptions::default(),
        );

        assert!(!session.is_closed());
        session.abort();
        assert!(session.is_closed());

        let report = session.join().await;
        assert_eq!(report.reader, WebSocketTaskStatus::Aborted);
        assert_eq!(report.writer, WebSocketTaskStatus::Aborted);
    }

    #[tokio::test]
    async fn session_close_times_out_when_writer_does_not_respond() {
        let (_inbound_tx, inbound_rx) = mpsc::channel::<Result<()>>(1);
        let (command_tx, _command_rx) = mpsc::channel(1);

        let reader_task =
            tokio::spawn(async { std::future::pending::<WebSocketTaskStatus>().await });
        let writer_task =
            tokio::spawn(async { std::future::pending::<WebSocketTaskStatus>().await });
        let options = WebSocketOptions::default().with_close_timeout(Duration::from_millis(1));
        let mut session: WebSocketSession<()> = WebSocketSession::new(
            inbound_rx,
            reader_task,
            writer_task,
            command_tx,
            "test.websocket",
            options,
        );

        let error = session.close().await.unwrap_err();
        assert!(matches!(
            error,
            Error::WebSocketError(WebSocketError::CloseTimeout {
                endpoint: "test.websocket",
                timeout,
            }) if timeout == Duration::from_millis(1)
        ));
        assert!(session.is_closed());
    }
}
