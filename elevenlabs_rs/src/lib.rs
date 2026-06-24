//! An unofficial lib crate for ElevenLabs.
//!
//! [ElevenLabs' website](https://elevenlabs.io/)
//!
//! [ElevenLabs' API reference](https://elevenlabs.io/docs/api-reference/introduction)
//!
//! # Example
//!
//! ## Text to Speech
//!
//! ```no_run
//! use elevenlabs_rs::{ElevenLabsClient, Result, DefaultVoice, Model};
//! use elevenlabs_rs::endpoints::genai::tts::{TextToSpeech, TextToSpeechBody};
//! use elevenlabs_rs::utils::play;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = ElevenLabsClient::from_env()?;
//!
//!     let txt = "Hello! 你好! Hola! नमस्ते! Bonjour! \
//!         こんにちは! مرحبا! 안녕하세요! Ciao! Cześć! Привіт! வணக்கம்!";
//!
//!     let body = TextToSpeechBody::new(txt)
//!        .with_model_id(Model::ElevenMultilingualV2);
//!
//!     let endpoint = TextToSpeech::new(DefaultVoice::Brian, body);
//!
//!     let speech = client.hit(endpoint).await?;
//!
//!     play(speech)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # WebSocket lifecycle
//!
//! WebSocket entry points such as
//! [`ElevenLabsClient::connect_text_to_speech`],
//! [`ElevenLabsClient::connect_multi_context_text_to_speech`], and
//! [`ElevenLabsClient::connect_realtime_speech_to_text`] return a
//! [`WebSocketSession`] when the `ws` and `genai` features are enabled. The
//! session implements `Stream` for inbound server messages and owns background
//! reader/writer tasks.
//!
//! Prefer calling [`WebSocketSession::close`] when the session has completed so
//! the client sends a close frame before shutting down background work. Dropping
//! the session is still safe and aborts background tasks as a fallback. For
//! diagnostics after a stream ends, or after `close`/`abort`, call
//! [`WebSocketSession::join`] to inspect reader and writer task completion.
//! Use [`WebSocketOptions`] with the `connect_*_with_options` methods to tune
//! inbound buffering and the graceful-close timeout.
pub use crate::client::{ApiResponse, ElevenLabsClient, RawRequestBuilder, Result};
pub use crate::shared::query_params::*;
pub use crate::shared::{DefaultVoice, FilePart, Language, LegacyVoice, Model, VoiceSettings};
#[cfg(all(feature = "ws", feature = "genai"))]
pub use crate::ws::{
    WebSocketOptions, WebSocketSession, WebSocketSessionReport, WebSocketTaskStatus,
};
pub use bytes::Bytes;
pub use futures_util::{pin_mut, StreamExt};
pub use reqwest::{multipart, Method};

mod client;
pub mod endpoints;
pub mod error;
mod shared;
pub mod utils;
#[cfg(all(feature = "ws", feature = "genai"))]
mod ws;
