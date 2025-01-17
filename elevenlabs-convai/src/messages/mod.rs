/// A module that provides websocket messages that can be sent to the server.
pub mod client_messages;
/// A module that provides websocket messages that are sent to the client from the server.
pub mod server_messages;

pub(crate) use crate::error::ConvAIError;
pub(crate) use tokio_tungstenite::tungstenite::protocol::Message;
pub(crate) use serde::{Deserialize, Serialize};