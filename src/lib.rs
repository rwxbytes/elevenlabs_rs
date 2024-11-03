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
//! use elevenlabs_rs::*;
//! use elevenlabs_rs::utils::play;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = ElevenLabsClient::default()?;
//!     let body = TextToSpeechBody::new(
//!         "This is the way the world ends, not with a bang but a whimper",
//!         Model::ElevenMultilingualV2,
//!     );
//!     let endpoint = TextToSpeech::new(PreMadeVoiceID::Clyde, body);
//!     let speech = client.hit(endpoint).await?;
//!     play(speech)?;
//!
//!     Ok(())
//! }
//! ```

pub use crate::client::{ElevenLabsClient, Result};
#[cfg(feature = "dev")]
pub use crate::convai_client::ConvAIClient;
pub use crate::endpoints::audio_isolation::*;
pub use crate::endpoints::audio_native::*;
pub use crate::endpoints::dubbing::*;
pub use crate::endpoints::history::*;
pub use crate::endpoints::models::*;
pub use crate::endpoints::projects::*;
pub use crate::endpoints::pronunciation::*;
pub use crate::endpoints::samples::*;
pub use crate::endpoints::sound_generation::*;
pub use crate::endpoints::sts::*;
pub use crate::endpoints::tts::*;
pub use crate::endpoints::tts::ws::*;
pub use crate::endpoints::user::*;
pub use crate::endpoints::voice::*;
pub use crate::endpoints::voice_design::*;
pub use crate::endpoints::voice_generation::*;
pub use crate::endpoints::voice_library::*;
pub use crate::shared::identifiers::{Model, PreMadeVoiceID};
pub use crate::shared::query_params::*;
pub use bytes::Bytes;
pub use futures_util::{pin_mut, StreamExt};

mod client;
pub mod endpoints;
pub mod error;
mod shared;
pub mod utils;
#[cfg(feature = "dev")]
mod convai_client;
