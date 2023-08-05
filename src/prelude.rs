pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const DELETE: &str = "DELETE";
pub const GET: &str = "GET";
pub const POST: &str = "POST";

pub const ACCEPT: &str = "ACCEPT";
pub const APPLICATION_JSON: &str = "application/json";
pub const AUDIO_ALL: &str = "audio/*";
pub const AUDIO_MPEG: &str = "audio/mpeg";
