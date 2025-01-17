use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("http error: {0}")]
    HttpError(Value),
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


#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("NonNormalCloseCode: {0}")]
    NonNormalCloseCode(String),
    #[error("ClosedWithoutCloseFrame")]
    ClosedWithoutCloseFrame,
    #[error("UnexpectedMessageType")]
    UnexpectedMessageType,
}

#[derive(Debug, Error)]
pub enum ConvAIError {
    #[error("JSON deserialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Missing or invalid 'type' field in the response")]
    InvalidTypeField,
    #[error("Unknown response type: {0}")]
    UnknownResponseType(String),
    #[error("WebSocket message error: {0}")]
    WebSocketError(String),
}
