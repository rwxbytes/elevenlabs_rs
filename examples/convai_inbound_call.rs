use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use elevenlabs_rs::conversational_ai::server_messages::ServerMessage;
use futures_util::{SinkExt, StreamExt};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use elevenlabs_rs::conversational_ai::client::ElevenLabsConversationalClient;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/call/incoming", post(twiml))
        .route("/call/connection", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give us a call");
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut ws_stream: WebSocket) {
    let mut client = ElevenLabsConversationalClient::from_env().expect("Failed to create ConvAIClient");

    // Skip connected message
    ws_stream.next().await;

    // Get stream sid
    let stream_sid = if let Some(msg_result) = ws_stream.next().await {
        let msg = msg_result.unwrap();
        let msg_json = msg.to_text().unwrap();
        let start_msg = serde_json::from_str::<StartMessage>(msg_json).unwrap();
        start_msg.stream_sid
    } else {
        panic!("no stream sid")
    };

    let (twilio_payload_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_encoded_audio_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(msg_result) = twilio_stream.next().await {
            let msg = msg_result.unwrap();
            match msg {
                Message::Close(_) => {
                    break;
                }
                Message::Text(txt) => {
                    let twilio_msg = TwilioMessage::try_from(txt.as_str()).unwrap();
                    match twilio_msg {
                        TwilioMessage::Media(media_msg) => {
                            let payload = media_msg.media.payload().to_string();
                            twilio_payload_tx.send(payload).unwrap()
                        }
                        TwilioMessage::Mark(mark_msg) => {
                            println!("Mark: {:?}", mark_msg)
                        }
                        TwilioMessage::Stop(stop_msg) => {
                            println!("Stop: {:?}", stop_msg);
                            break;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        let mut convai_stream = client
            .start_conversation(twilio_encoded_audio_stream)
            .await
            .unwrap();

        while let Some(resp_result) = convai_stream.next().await {
            let convai_msg = resp_result.unwrap(); // TODO: errs after max duration
            match convai_msg {
                ServerMessage::Audio(audio) => {
                    let payload = audio.event().base_64();
                    let media_msg = MediaMessage::new(&stream_sid, payload);
                    let json = serde_json::to_string(&media_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();


                    let mark_msg = MarkMessage {
                        event: "mark".to_string(),
                        stream_sid: stream_sid.to_string(),
                        sequence_number: None,
                        mark: Mark {
                            name: "elabs_audio".to_string(),
                        },
                    };
                    let json = serde_json::to_string(&mark_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();
                }
                ServerMessage::Interruption(_) => {
                    let clear_msg = ClearMessage::new(&stream_sid);
                    let json = serde_json::to_string(&clear_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();
                    println!("Sent clear message")
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
            <Stream url="wss://.ngrok-free.app/call/connection" track="inbound_track" />
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
    event: String,
    protocol: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartMessage {
    event: String,
    sequence_number: String,
    start: StartMetadata,
    stream_sid: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopMessage {
    event: String,
    stream_sid: String,
    sequence_number: String,
    stop: Stop,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    account_sid: String,
    call_sid: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearMessage {
    event: String,
    stream_sid: String,
}

impl ClearMessage {
    fn new(sid: &str) -> Self {
        ClearMessage {
            event: "clear".to_string(),
            stream_sid: sid.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarkMessage {
    event: String,
    stream_sid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sequence_number: Option<String>,
    mark: Mark,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mark {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartMetadata {
    stream_sid: String,
    account_sid: String,
    call_sid: String,
    tracks: Vec<Track>,
    custom_parameters: serde_json::Value,
    media_format: MediaFormat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaFormat {
    encoding: String,
    sample_rate: u32,
    channels: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Track {
    #[serde(rename = "inbound")]
    Inbound,
    #[serde(rename = "outbound")]
    Outbound,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaMessage {
    event: String,
    stream_sid: String,
    media: Media,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    payload: String,
}

impl Media {
    fn payload(&self) -> &str {
        self.payload.as_str()
    }
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
    event: String,
    stream_sid: String,
    sequence_number: u32,
    dtmf: Dtmf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dtmf {
    digit: String,
    track: Track,
}
