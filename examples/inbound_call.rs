use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use elevenlabs_rs::conversational_ai::client::ElevenLabsAgentClient;
use elevenlabs_rs::conversational_ai::server_messages::ServerMessage;
use futures_util::{SinkExt, StreamExt};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/inbound-call", post(twiml))
        .route("/connection", get(ws_handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give us a call");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let client = ElevenLabsAgentClient::from_env().unwrap();

    let client = Arc::new(Mutex::new(client));
    let client_2 = Arc::clone(&client);

    // Skip connected message
    socket.next().await;

    // Get stream_sid
    let stream_sid = match socket.next().await {
        Some(Ok(Message::Text(msg))) => {
            let start_msg = serde_json::from_str::<StartMessage>(&msg).unwrap();
            start_msg.stream_sid
        }
        _ => panic!("Expected start message with stream sid"),
    };

    let (twilio_payload_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_payload_rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = socket.split();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = twilio_stream.next().await {
            let txt = msg.to_text().expect("Expected text message");
            let twilio_msg = TwilioMessage::try_from(txt).unwrap();
            match twilio_msg {
                TwilioMessage::Media(media_msg) => {
                    let payload = media_msg.media.payload;
                    twilio_payload_tx.send(payload).unwrap()
                }
                TwilioMessage::Stop(_) => {
                    client_2.lock().await.stop_conversation().await.unwrap();
                    println!("Caller hung up");
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        let mut convai_stream = client
            .lock()
            .await
            .start_conversation(twilio_payload_rx)
            .await
            .unwrap();

        while let Some(msg_result) = convai_stream.next().await {
            let server_msg = msg_result.unwrap();
            match server_msg {
                ServerMessage::Audio(audio) => {
                    let payload = audio.audio_event.audio_base_64;
                    let media_msg = MediaMessage::new(&stream_sid, &payload);
                    let json = serde_json::to_string(&media_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();
                }
                ServerMessage::Interruption(_) => {
                    let clear_msg = ClearMessage::new(&stream_sid);
                    let json = serde_json::to_string(&clear_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();
                }
                _ => {}
            }
        }
    });
}

// TODO: add your ngrok domain
async fn twiml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Connect>
            <Stream url="wss://your_domain.ngrok-free.app/connection" track="inbound_track" />
        </Connect>
    </Response>
    "#
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TwilioMessage {
    Connected(ConnectedMessage),
    Start(StartMessage),
    Media(MediaMessage),
    Mark(MarkMessage),
    Stop(StopMessage),
    Dtmf(DtmfMessage),
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
pub struct ConnectedMessage {
    pub event: String,
    pub protocol: String,
    pub version: String,
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
    fn new(sid: &str) -> Self {
        ClearMessage {
            event: "clear".to_string(),
            stream_sid: sid.to_string(),
        }
    }
}

/// [Sending Mark Messages](https://www.twilio.com/docs/voice/media-streams/websocket-messages#send-a-mark-message)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarkMessage {
    pub event: String,
    pub stream_sid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<String>,
    pub mark: Mark,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mark {
    pub name: String,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DtmfMessage {
    pub event: String,
    pub stream_sid: String,
    pub sequence_number: u32,
    pub dtmf: Dtmf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dtmf {
    pub digit: String,
    pub track: Track,
}
