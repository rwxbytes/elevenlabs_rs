use crate::{
    api::{Client, ClientBuilder},
    error::Error,
    prelude::*,
};
use comparable::*;
use http_body_util::Empty;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

const GET: &str = "GET";
const PATH: &str = "/voices";

#[derive(Debug, Serialize, Deserialize, Clone, Comparable)]
pub struct VoiceSettings {
    pub similarity_boost: f64,
    pub stability: f64,
    //pub style: f64,
    //pub use_speaker_boost: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        VoiceSettings {
            similarity_boost: 0.75,
            stability: 0.5,
            //style: 0.0,
            //use_speaker_boost: true,
        }
    }
}
