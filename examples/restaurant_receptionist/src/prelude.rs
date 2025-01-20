use thiserror::Error;
use elevenlabs_convai::error::ConvAIError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("environment variable not set: {0}")]
    EnvVarError(String),

    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("websocket error: {0}")]
    WebSocketError(#[from] axum::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("stream SID not found")]
    StreamSidNotFound,

    #[error("twilio message parse error: {0}")]
    TwilioMessageParseError(String),

    #[error("tokio join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("conversational_ai error: {0}")]
    ConversationalError(#[from] ConvAIError),

    #[error("send error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<String>),
}

type Result<T> = std::result::Result<T, AppError>;