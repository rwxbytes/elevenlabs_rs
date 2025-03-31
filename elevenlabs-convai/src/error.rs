use thiserror::Error;
use tokio_tungstenite::tungstenite;

#[derive(Debug, Error)]
pub enum ConvAIError {
    #[error("json deserialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),

    #[error("boxed error: {0}")]
    Boxed(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("websocket error: {0}")]
    WebSocketError(#[source] tungstenite::Error),

    #[error("websocket connection closed with a non-normal close code: {0}")]
    NonNormalCloseCode(String),

    #[error("websocket connection closed without close frame")]
    ClosedWithoutCloseFrame,

    #[error("unexpected WebSocket message type")]
    UnexpectedMessageType,

    #[error("failed to send message through channel")]
    SendError,

    #[error("failed to cancel the operation")]
    CancellationError,

    #[error("failed to get signed url")]
    SignedUrlError,
}