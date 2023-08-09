//! ElevenLabs RS
//!
//! An unofficial ElevenLabs API client.
//!
//! ElevenLabs' web app: <https://elevenlabs.io/>
//!
//! ElevenLabs' API documentation: <https://docs.elevenlabs.io/api-reference/quick-start/introduction>.
//!
//! # Example
//!
//! ```no_run
//! use elevenlabs_rs::{Speech, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let speech = Speech::new(
//!         "This is the way the world ends, not with a bang but a whimper",
//!         "Clyde",
//!         "eleven_monolingual_v1",
//!         0,
//!     ).await?;
//!
//!     speech.play()?;
//!
//!     Ok(())   
//! }
//! ```

pub use crate::api::history::*;
pub use crate::api::tts::Speech;
pub use crate::api::user::get_user_subscription;
pub use crate::api::voice::{get_voice, get_voices, Voice, VoiceCloneBuilder};
pub use crate::prelude::Result;

pub mod api;
pub mod error;
pub mod prelude;
pub mod support;
pub mod utils;
