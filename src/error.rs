use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid API response: {0}")]
    InvalidApiResponse(String),
    #[error("Client build error: {0}")]
    ClientBuildError(String),
    #[error("ClientSendRequestError: {0}")]
    ClientSendRequestError(Value),
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
    #[error("SpeechGenerationError: {0}")]
    SpeechGenerationError(String),
    #[error("GeneratedVoiceIDHeaderNotFound")]
    GeneratedVoiceIDHeaderNotFound,
    //#[error("ElevenLabsClientError: {detail:?}")]
    //ElevenLabsClientError{
    //    detail: Vec<DetailObject>,
    //},
}

#[derive(Error, Debug, Deserialize)]
#[error("ElevenLabsServerError: {detail:?}")]
pub struct ElevenLabsServerError {
    detail: Detail,
}

#[derive(Debug, Deserialize, Error)]
#[error("ClientErrorDetail: {message:?}, {status:?}")]
struct Detail {
    message: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct DetailObject {
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
