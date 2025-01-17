//! An unofficial lib crate for ElevenLabs.
//!
//! ElevenLabs' web app: <https://elevenlabs.io/>
//!
//! ElevenLabs' API documentation: <https://docs.elevenlabs.io/api-reference/quick-start/introduction>.
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
pub use crate::client::{ElevenLabsClient, Result};
pub use crate::shared::{DefaultVoice, Model, LegacyVoice, Language, VoiceSettings};
pub use crate::shared::query_params::*;
pub use bytes::Bytes;
pub use futures_util::{pin_mut, StreamExt};

mod client;
pub mod endpoints;
pub mod error;
mod shared;
pub mod utils;