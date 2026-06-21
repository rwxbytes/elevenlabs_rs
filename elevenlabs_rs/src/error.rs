use reqwest::{header::HeaderMap, StatusCode};
use serde_json::Value;
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

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocketTransportError(Box::new(value))
    }
}

impl<T> From<futures_channel::mpsc::TrySendError<T>> for Error {
    fn from(value: futures_channel::mpsc::TrySendError<T>) -> Self {
        let reason = if value.is_disconnected() {
            "receiver disconnected"
        } else if value.is_full() {
            "channel full"
        } else {
            "unknown channel send failure"
        };
        Self::ChannelSendError(reason.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("NonNormalCloseCode: {0}")]
    NonNormalCloseCode(String),
    #[error("ClosedWithoutCloseFrame")]
    ClosedWithoutCloseFrame,
    #[error("UnexpectedMessageType")]
    UnexpectedMessageType,
}
