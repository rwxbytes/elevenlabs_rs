use thiserror::Error;
use tokio::task::JoinError;
use tokio_tungstenite::tungstenite;

#[derive(Debug, Error)]
pub enum ConvAIError {
    #[error("json deserialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("boxed error: {0}")]
    Boxed(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("elevenlabs_rs error: {0}")]
    ElevenLabs(#[source] Box<elevenlabs_rs::error::Error>),

    #[error("websocket error: {0}")]
    WebSocketError(#[source] Box<tungstenite::Error>),

    #[error("websocket connection closed with non-normal code {code}: {reason}")]
    NonNormalClose { code: String, reason: String },

    #[error("websocket connection closed with a non-normal close code: {0}")]
    #[deprecated(note = "use NonNormalClose { code, reason }")]
    NonNormalCloseCode(String),

    #[error("websocket connection closed without close frame")]
    ClosedWithoutCloseFrame,

    #[error("unexpected WebSocket frame: expected {expected}, received {received}")]
    UnexpectedFrame {
        expected: &'static str,
        received: &'static str,
    },

    #[error("unexpected WebSocket message type")]
    #[deprecated(note = "use UnexpectedFrame")]
    UnexpectedMessageType,

    #[error("failed to send message through channel")]
    SendError,

    #[error("websocket send queue is closed")]
    SendQueueClosed,

    #[error("{task} task failed: {source}")]
    TaskFailed {
        task: &'static str,
        #[source]
        source: Box<ConvAIError>,
    },

    #[error("{task} task join failed: {source}")]
    TaskJoin {
        task: &'static str,
        #[source]
        source: JoinError,
    },

    #[error("websocket session is already closed")]
    SessionClosed,

    #[error("failed to cancel the operation")]
    CancellationError,

    #[error("failed to get signed url")]
    SignedUrlError,
}

impl From<tungstenite::Error> for ConvAIError {
    fn from(error: tungstenite::Error) -> Self {
        Self::WebSocketError(Box::new(error))
    }
}

impl From<elevenlabs_rs::error::Error> for ConvAIError {
    fn from(error: elevenlabs_rs::error::Error) -> Self {
        Self::ElevenLabs(Box::new(error))
    }
}
