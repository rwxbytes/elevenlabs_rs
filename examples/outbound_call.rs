use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use elevenlabs_rs::conversational_ai::client::ElevenLabsAgentClient;
use elevenlabs_rs::conversational_ai::error::ConvAIError;
use elevenlabs_rs::conversational_ai::server_messages::ServerMessage;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::env::var;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, span, Level};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("environment variable not set: {0}")]
    EnvVarError(String),

    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("websocket error: {0}")]
    WebSocketError(#[from] axum::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("stream SID not found")]
    StreamSidNotFound,

    #[error("twilio message parse error: {0}")]
    TwilioMessageParseError(String),

    #[error("tokio join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("conversational_ai error: {0}")]
    ConversationalError(#[from] ConvAIError),

    #[error("send error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<String>),
}

type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
struct Config {
    twilio_auth_token: String,
    twilio_account_sid: String,
    to: String,
    from: String,
    ngrok_url: String,
}

impl Config {
    fn from_env() -> Result<Config> {
        Ok(Config {
            twilio_auth_token: var("TWILIO_AUTH_TOKEN")
                .map_err(|_| AppError::EnvVarError("TWILIO_AUTH_TOKEN not set".to_string()))?,
            twilio_account_sid: var("TWILIO_ACCOUNT_SID")
                .map_err(|_| AppError::EnvVarError("TWILIO_ACCOUNT_SID not set".to_string()))?,
            to: var("TWILIO_TO")
                .map_err(|_| AppError::EnvVarError("TWILIO_TO not set".to_string()))?,
            from: var("TWILIO_FROM")
                .map_err(|_| AppError::EnvVarError("TWILIO_FROM not set".to_string()))?,
            ngrok_url: var("NGROK_URL")
                // TODO: add your ngrok domain
                .unwrap_or_else(|_| "https://yourdomain.ngrok-free.app".to_string()),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let config = Config::from_env()?;

    let t = tokio::spawn(run_server(config.ngrok_url.clone()));

    make_twilio_call(&config).await?;

    let _ = t.await?;

    Ok(())
}

async fn run_server(ngrok_url: String) -> Result<()> {
    let app = Router::new()
        .route("/outbound-call", post(move || twiml(ngrok_url)))
        .route("/call/connection", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn twiml(ngrok_url: String) -> String {
    let url = Url::parse(&ngrok_url).expect("Invalid ngrok URL");
    let domain = url.domain().unwrap();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Connect>
            <Stream url="wss://{}/call/connection" track="inbound_track" />
        </Connect>
    </Response>"#,
        domain
    )
}

async fn make_twilio_call(config: &Config) -> Result<()> {
    let mut params = std::collections::HashMap::new();
    params.insert("To", config.to.clone());
    params.insert("From", config.from.clone());
    params.insert("Url", format!("{}/outbound-call", config.ngrok_url));

    let resp = Client::new()
        .post(format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Calls.json",
            &config.twilio_account_sid
        ))
        .basic_auth(&config.twilio_account_sid, Some(&config.twilio_auth_token))
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        error!("Twilio call failed: {:?}", resp.status());
        let body = resp.text().await?;
        error!("Twilio response: {:#?}", body);
    }

    Ok(())
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

#[instrument(skip(socket))]
async fn handle_socket(socket: WebSocket) {
    let span = span!(Level::INFO, "handle_socket");
    let _enter = span.enter();

    match process_socket(socket).await {
        Ok(_) => info!("Connection closed"),
        Err(e) => error!("Error: {:?}", e),
    }
}

async fn process_socket(mut socket: WebSocket) -> Result<()> {
    let client = ElevenLabsAgentClient::from_env()?;

    let mut client = Arc::new(Mutex::new(client));
    let client_two = Arc::clone(&client);

    // Skip connected message
    socket.next().await;

    // Get stream sid
    let stream_sid = match socket.next().await {
        Some(Ok(Message::Text(msg))) => serde_json::from_str::<StartMessage>(&msg)?.stream_sid,
        _ => return Err(AppError::StreamSidNotFound),
    };

    let (twilio_payload_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_payload_rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = socket.split();

    let twilio_task: JoinHandle<Result<()>> = tokio::spawn(async move {
        // TODO: learn how to use tracing correctly
        //let span = span!(Level::INFO, "twilio_ws");
        //let _enter = span.enter();

        while let Some(msg_result) = twilio_stream.next().await {
            let msg = msg_result?;
            match msg {
                Message::Close(_) => {
                    //info!("Twilio stream closed");
                    break;
                }
                Message::Text(txt) => {
                    let twilio_msg = TwilioMessage::try_from(txt.as_str())?;
                    match twilio_msg {
                        TwilioMessage::Media(media_msg) => {
                            twilio_payload_tx.send(media_msg.media.payload().to_string())?;
                        }
                        TwilioMessage::Stop(_) => {
                            //info!("Stop message received from Twilio");
                            client_two.lock().await.stop_conversation().await?;
                            //info!("Elevenlabs' client stopped conversation");
                            break;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    });

    let el_task: JoinHandle<Result<()>> = tokio::spawn(async move {
        let span = span!(Level::INFO, "elevenlabs_ws");
        let _enter = span.enter();

        let mut conversation_stream = client
            .lock()
            .await
            .start_conversation(twilio_payload_rx)
            .await?;
        info!("Conversation started");

        while let Some(server_msg_result) = conversation_stream.next().await {
            let server_msg = server_msg_result?;
            match server_msg {
                ServerMessage::Audio(audio) => {
                    let payload = audio.audio_event.audio_base_64;
                    let media_msg = MediaMessage::new(&stream_sid, &payload);
                    let json = serde_json::to_string(&media_msg)?;
                    twilio_sink.send(Message::Text(json)).await?;
                }
                ServerMessage::Interruption(_) => {
                    info!("Interruption event received from Elevenlabs");
                    let clear_msg = ClearMessage::new(&stream_sid);
                    let json = serde_json::to_string(&clear_msg)?;
                    twilio_sink.send(Message::Text(json)).await?;
                    info!("Clear message sent to Twilio");
                }
                _ => {}
            }
        }
        Ok(())
    });

    tokio::select! {
        res = twilio_task => {
            info!("Twilio task done");
            res??;
            Ok(())
        }
        res = el_task => {
            info!("Elevenlabs task done");
            res??;
            Ok(())
        }
    }
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
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self> {
        let twilio_message: TwilioMessage = serde_json::from_str(value)
            .map_err(|e| AppError::TwilioMessageParseError(e.to_string()))?;
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
