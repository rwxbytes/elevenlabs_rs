use reqwest::{header::HeaderMap, StatusCode};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub body: String,
    pub error: Option<Value>,
    pub headers: HeaderMap,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub character_cost: Option<u64>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            Some(error) => write!(f, "{}: {}", self.status, error),
            None if self.body.is_empty() => write!(f, "{}", self.status),
            None => write!(f, "{}: {}", self.status, self.body),
        }
    }
}

impl std::error::Error for ApiError {}

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("api error: {0}")]
    ApiError(Box<ApiError>),
    #[error("http error: {0}")]
    HttpError(Value),
    #[error("environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("utf-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[cfg(feature = "playback")]
    #[error("audio stream error: {0}")]
    AudioStreamError(#[from] rodio::StreamError),
    #[cfg(feature = "playback")]
    #[error("audio playback error: {0}")]
    AudioPlaybackError(#[from] rodio::PlayError),
    #[cfg(feature = "playback")]
    #[error("audio decode error: {0}")]
    AudioDecodeError(#[from] rodio::decoder::DecoderError),
    #[error("websocket error: {0}")]
    WebSocketError(#[from] WebSocketError),
    #[cfg(feature = "ws")]
    #[error("websocket transport error: {0}")]
    WebSocketTransportError(#[source] Box<tokio_tungstenite::tungstenite::Error>),
    #[error("channel send error: {0}")]
    ChannelSendError(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("file extension not found")]
    FileExtensionNotFound,
    #[error("file extension not valid utf8")]
    FileExtensionNotValidUTF8,
    #[error("file extension not supported")]
    FileExtensionNotSupported,
    #[error("path not valid utf8")]
    PathNotValidUTF8,
    #[error("voice not found")]
    VoiceNotFound,
    #[error("generated voice id header not found")]
    GeneratedVoiceIDHeaderNotFound,
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::InvalidInput(value.to_owned())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::InvalidInput(value)
    }
}

#[cfg(feature = "ws")]
impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocketTransportError(Box::new(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebSocketDirection {
    Inbound,
    Outbound,
}

impl std::fmt::Display for WebSocketDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inbound => f.write_str("inbound"),
            Self::Outbound => f.write_str("outbound"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WebSocketErrorContext {
    pub endpoint: &'static str,
    pub direction: WebSocketDirection,
}

impl WebSocketErrorContext {
    pub const fn new(endpoint: &'static str, direction: WebSocketDirection) -> Self {
        Self {
            endpoint,
            direction,
        }
    }
}

impl std::fmt::Display for WebSocketErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.endpoint, self.direction)
    }
}

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("websocket closed with non-normal close code {code}: {reason} ({context})")]
    NonNormalClose {
        context: WebSocketErrorContext,
        code: String,
        reason: String,
    },
    #[error("websocket closed without close frame ({context})")]
    ClosedWithoutCloseFrame { context: WebSocketErrorContext },
    #[error("unexpected websocket frame: expected {expected}, received {received} ({context})")]
    UnexpectedFrame {
        context: WebSocketErrorContext,
        expected: &'static str,
        received: &'static str,
    },
    #[error("websocket message encode error: {source} ({context})")]
    Encode {
        context: WebSocketErrorContext,
        source: serde_json::Error,
    },
    #[error(
        "websocket message decode error: {source}; payload preview: {payload_preview} ({context})"
    )]
    Decode {
        context: WebSocketErrorContext,
        source: serde_json::Error,
        payload_preview: String,
    },
    #[error("websocket writer task has already finished for {endpoint}")]
    WriterFinished { endpoint: &'static str },
    #[error("websocket close command channel is closed for {endpoint}")]
    SendQueueClosed { endpoint: &'static str },
    #[error("websocket close timed out after {timeout:?} for {endpoint}")]
    CloseTimeout {
        endpoint: &'static str,
        timeout: Duration,
    },
}
