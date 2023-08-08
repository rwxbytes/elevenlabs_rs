use serde_json::Value;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid API response: {0}")]
    InvalidApiResponse(String),
    #[error("Client build error: {0}")]
    ClientBuildError(String),
    #[error("ClientSendRequestError: {0}")]
    ClientSendRequestError(Value),
    #[error("InvalidTimestamp: {0}")]
    InvalidTimestamp(String),
    #[error("VoiceNotFound")]
    VoiceNotFound,
    #[error("SpeechGenerationError: {0}")]
    SpeechGenerationError(String),
    #[error("VoiceCloneBuilderError: {0}")]
    VoiceCloneBuilderError(String),
}
