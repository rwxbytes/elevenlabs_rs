#![allow(dead_code)]
use crate::prelude::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Caller {
    pub caller: String,
    caller_city: Option<String>,
    caller_country: Option<String>,
    caller_state: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TwilioMessage {
    Start(StartMessage),
    Media(MediaMessage),
    Stop(StopMessage),
}

impl TryFrom<&str> for TwilioMessage {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let twilio_message: TwilioMessage = serde_json::from_str(value)?;
        Ok(twilio_message)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartMessage {
    pub event: String,
    pub sequence_number: String,
    pub start: StartMetadata,
    pub stream_sid: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopMessage {
    pub event: String,
    pub stream_sid: String,
    pub sequence_number: String,
    pub stop: Stop,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub account_sid: String,
    pub call_sid: String,
}

/// [Sending Clear Messages](https://www.twilio.com/docs/voice/media-streams/websocket-messages#send-a-clear-message)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearMessage {
    pub event: String,
    pub stream_sid: String,
}

impl ClearMessage {
    pub fn new(sid: &str) -> Self {
        ClearMessage {
            event: "clear".to_string(),
            stream_sid: sid.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartMetadata {
    pub stream_sid: String,
    pub account_sid: String,
    pub call_sid: String,
    pub tracks: Vec<Track>,
    pub custom_parameters: serde_json::Value,
    pub media_format: MediaFormat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaFormat {
    pub encoding: String,
    pub sample_rate: u32,
    pub channels: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Track {
    #[serde(rename = "inbound")]
    Inbound,
    #[serde(rename = "outbound")]
    Outbound,
}

/// [Sending Media Messages](https://www.twilio.com/docs/voice/media-streams/websocket-messages#send-a-media-message)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaMessage {
    pub event: String,
    pub stream_sid: String,
    pub media: Media,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub payload: String,
}

impl MediaMessage {
    pub fn new(stream_sid: &str, payload: &str) -> Self {
        MediaMessage {
            event: "media".to_string(),
            stream_sid: stream_sid.to_string(),
            media: Media {
                payload: payload.to_string(),
            },
        }
    }
}