use async_stream::stream;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use elevenlabs_rs::endpoints::tts::ws::{BOSMessage, WebSocketTTS, WebSocketTTSBody};
use elevenlabs_rs::endpoints::tts::{OutputFormat, SpeechQuery};
use elevenlabs_rs::*;
use elevenlabs_rs::endpoints::shared::identifiers::Model;
use futures_util::{pin_mut, StreamExt};
use serde::{Deserialize, Serialize};

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
    // Skip connected message
    ws_stream.next().await;

    let stream_sid = if let Some(msg_result) = ws_stream.next().await {
        let msg = msg_result.unwrap();
        let msg_json = msg.to_text().unwrap();
        let start_msg = serde_json::from_str::<StartMessage>(msg_json).unwrap();
        start_msg.stream_sid
    } else {
        panic!("no stream sid")
    };

    let voice_id = PreMadeVoiceID::Adam;
    let model_id = Model::ElevenTurboV2;

    let text_stream = stream! {
        let text: Vec<String> = "This is a test, you can now hang up. Thank you."
        .split_ascii_whitespace()
        .map(|w| w.to_string())
        .collect();
        for word in text {
            yield word;
        }
    };

    let body = WebSocketTTSBody::new(BOSMessage::default(), text_stream);
    let speech_query = SpeechQuery::default().with_output_format(OutputFormat::MuLaw8000Hz);
    let endpoint = WebSocketTTS::new(voice_id, model_id, body).with_query(speech_query);
    let c = ElevenLabsClient::default().unwrap();

    let mut stream = c.hit_ws(endpoint).await.unwrap();
    pin_mut!(stream);
    while let Some(r) = stream.next().await {
        let resp = r.expect("resp");
        if let Some(audio) = resp.audio_b64() {
            let media = MediaMessage::new(&stream_sid, audio);
            ws_stream
                .send(Message::Text(serde_json::to_string(&media).unwrap()))
                .await
                .unwrap()
        }
    }

    let mut media_msg_counter = 0;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg.expect("msg");
        match msg {
            Message::Text(text) => {
                let value = serde_json::from_str::<serde_json::Value>(&text).unwrap();
                if value["event"] == "media" {
                    media_msg_counter += 1;
                    println!("Media messages received: {:?}", media_msg_counter);
                    continue;
                } else if value["event"] == "stop" {
                    println!("Caller has ended the call");
                    return;
                } else {
                    return;
                }
            }
            _ => {
                return;
            }
        }
    }
}

// TODO: add your ngrok domain
async fn twiml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Connect>
            <Stream url="wss://yourdomain.ngrok.app/call/connection" />
        </Connect>
    </Response>
    "#
}

#[derive(Serialize, Deserialize, Debug)]
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
pub struct Media {
    payload: String,
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
