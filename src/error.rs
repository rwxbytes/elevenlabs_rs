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
    detail: ServerErrorDetail,
}

#[derive(Error, Debug, Deserialize)]
#[error("ServerErrorDetail: {message:?}, {status:?}")]
struct ServerErrorDetail {
    message: String,
    status: String,
}

#[derive(Error, Debug, Deserialize)]
#[error("ClientErrorDetail: {detail:?}")]
pub struct ElevenLabs400 {
    detail: ClientErrorDetail,
}
#[derive(Debug, Deserialize, Error)]
#[error("ClientErrorDetail: {message:?}, {status:?}")]
struct ClientErrorDetail {
    message: String,
    status: String,
}

#[derive(Error, Debug, Deserialize)]
#[error("ElevenLabsClientError: {detail:?}")]
pub struct ElevenLabsClientError {
    detail: Vec<DetailObject>,
}
#[derive(Debug, Deserialize)]
struct DetailObject {
    loc: Vec<String>,
    msg: String,
    r#type: String,
}

#[derive(Debug, Error)]
pub enum ElevenLabsError {
    #[error("ElevenLabsClientError: {0}")]
    BadRequest(ElevenLabs400),
    #[error("ElevenLabsClientError: {detail:?}")]
    UnprocessableEntity { detail: Vec<DetailObject> },
    #[error("ElevenLabsClientError: {0}")]
    Code4xx(Code4xx),
}

#[derive(Debug, Deserialize, Error)]
#[error("Error4xx: {detail:?}")]
pub struct Code4xx {
    detail: String,
}
