use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HttpError: {0}")]
    HttpError(Value),
    #[error("FileExtensionNotFound")]
    FileExtensionNotFound,
    #[error("FileExtensionNotValidUTF8")]
    FileExtensionNotValidUTF8,
    #[error("FileExtensionNotSupported")]
    FileExtensionNotSupported,
    #[error("PathNotValidUTF8")]
    PathNotValidUTF8,
    #[error("VoiceNotFound")]
    VoiceNotFound,
    #[error("GeneratedVoiceIDHeaderNotFound")]
    GeneratedVoiceIDHeaderNotFound,
}

#[derive(Error, Debug, Deserialize)]
#[error("ElevenLabsServerError: {detail:?}")]
pub struct ElevenLabsServerError {
    detail: Detail,
}

#[derive(Debug, Deserialize, Error)]
#[error("ClientErrorDetail: {message:?}, {status:?}")]
pub struct Detail {
    message: String,
    status: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DetailObject {
    loc: Vec<String>,
    msg: String,
    r#type: String,
}

#[derive(Debug, Deserialize, Error)]
#[serde(untagged)]
pub enum ElevenLabsClientError {
    #[error("ElevenLabsClientError: {detail:?}")]
    BadRequest { detail: Detail },
    #[error("ElevenLabsClientError: {detail:?}")]
    NotFound { detail: Detail },
    #[error("ElevenLabsClientError: {detail:?}")]
    UnprocessableEntity { detail: Vec<DetailObject> },
    #[error("ElevenLabsClientError: {detail:?}")]
    Code4xx { detail: String },
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
