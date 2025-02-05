use thiserror::Error;
use elevenlabs_convai::error::ConvAIError;
pub use axum::extract::State;
pub use axum::Json;
pub use chrono::Utc;
pub use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("environment variable not set: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("websocket error: {0}")]
    WebSocket(#[from] axum::Error),

    #[error("conversational_ai error: {0}")]
    Conversational(#[from] ConvAIError),

    #[error("send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<String>),

    #[error("surreal database error: {0}")]
    SurrealDb(#[from] surrealdb::Error),
}
