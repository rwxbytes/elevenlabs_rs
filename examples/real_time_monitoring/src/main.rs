// TODO: move to another branch before merging into develop
//mod audio_output;
//
//use axum::extract::{FromRef, State};
//use axum::http::StatusCode;
//use axum::response::IntoResponse;
//use axum::{
//    extract::ws::{Message, WebSocket, WebSocketUpgrade},
//    response::Response,
//    routing::{get, post},
//    Form, Json, Router,
//};
//use base64::prelude::BASE64_STANDARD;
//use base64::Engine;
//use cpal::traits::StreamTrait;
//use cpal::SampleRate;
//use elevenlabs_twilio::{
//    AgentWebSocket, CallResponse, CreateCall, CreateCallBody, Error, OutboundCall, ServerMessage,
//    StatusCallbackEvent, TelephonyAgent, TwilioClient, TwilioMessage, TwilioRequestParams,
//    TwimlSrc, VoiceResponse, TelephonyState,
//};
//use futures_util::{SinkExt, StreamExt};
//use reqwest::{Client, Url};
//use serde::{Deserialize, Serialize};
//use std::env::var;
//use std::future::Future;
//use std::sync::Arc;
//use thiserror::Error;
//use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
//use tokio::sync::Mutex;
//use tokio::task::JoinHandle;
//use tracing::{error, info, instrument, span, Level};
//
//impl FromRef<AppState> for TelephonyState {
//    fn from_ref(app_state: &AppState) -> Self {
//        app_state.manager.clone()
//    }
//}
//
//#[derive(Clone)]
//pub struct AppState {
//    decoded_audio_tx: UnboundedSender<Vec<i16>>,
//    output_sample_rate: SampleRate,
//    manager: TelephonyState,
//}
//
//#[tokio::main]
//async fn main() -> Result<(), Box<dyn std::error::Error>> {
//    tracing_subscriber::fmt().init();
//
//    // Set up speaker and play output audio stream
//    let (decoded_audio_tx, decoded_audio_rx) = tokio::sync::mpsc::unbounded_channel();
//    let mut speaker = audio_output::DefaultSpeakersManager::new(decoded_audio_rx);
//    let (speaker_stream, output_sample_rate) = speaker.build_output_stream().await;
//    speaker_stream
//        .play()
//        .expect("speaker stream failed to play");
//
//    let state = AppState {
//        decoded_audio_tx,
//        output_sample_rate,
//        manager: TelephonyState::from_env().unwrap(),
//    };
//
//    let app = Router::new()
//        .route("/ring", post(ring))
//        .route("/outbound-call", post(twiml))
//        .route("/ws", get(handle_ws))
//        .with_state(state);
//
//    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
//    info!("Listening on {}", listener.local_addr()?);
//    axum::serve(listener, app).await?;
//
//    Ok(())
//}
//
//#[derive(Debug, Deserialize, Serialize)]
//struct Ring {
//    to: String,
//    url: Option<String>,
//}
//
//async fn ring(outbound_call: OutboundCall<Json<Ring>>) -> impl IntoResponse {
//    let to = outbound_call.inner_extractor.to.clone();
//    let twiml_url = "https://7c4e-86-18-8-153.ngrok-free.app/outbound-call";
//    outbound_call.ring(&to, twiml_url).await.unwrap();
//    StatusCode::OK
//}
//
//async fn twiml(_twiml: Form<TwilioRequestParams>) -> impl IntoResponse {
//    let url = "wss://7c4e-86-18-8-153.ngrok-free.app/ws";
//    VoiceResponse::new()
//        .connect(url)
//        .to_http_response()
//        .unwrap()
//}
//
////TODO: sync audio
//async fn handle_ws(mut agent: TelephonyAgent, State(app_state): State<AppState>) -> Response {
//    let (audio_tx, mut audio_rx) = tokio::sync::mpsc::unbounded_channel();
//    let el_audio_tx = audio_tx.clone();
//    let twilio_audio_tx = audio_tx.clone();
//
//    let server_cb = move |msg| match msg {
//        ServerMessage::Audio(audio) => {
//            let audio = audio.audio_event.audio_base_64;
//            if el_audio_tx.send(audio).is_err() {
//                error!("Failed to send server message audio");
//            }
//        }
//        _ => {}
//    };
//
//    let twilio_cb = move |msg| match msg {
//        TwilioMessage::Media(media) => {
//            let audio = media.media.payload;
//            if twilio_audio_tx.send(audio).is_err() {
//                error!("Failed to send twilio message audio");
//            }
//        }
//        _ => {}
//    };
//
//    agent.server_message_cb = Some(Box::new(server_cb));
//    agent.twilio_message_cb = Some(Box::new(twilio_cb));
//
//    let decoded_audio_tx = app_state.decoded_audio_tx.clone();
//
//    // TODO: handle interruptions
//    tokio::spawn(async move {
//        while let Some(audio_b64) = audio_rx.recv().await {
//            match audio_output::decode_base64_pcm(&audio_b64) {
//                Ok(mut decoded_bytes) => {
//                    decoded_bytes = audio_output::upmix_mono_to_stereo(&decoded_bytes);
//
//                    if let Err(e) = decoded_audio_tx.send(decoded_bytes) {
//                        error!("Failed to send audio samples: {}", e);
//                        continue;
//                    }
//                }
//                Err(e) => {
//                    error!("Failed to decode audio: {}", e);
//                    continue;
//                }
//            }
//        }
//    });
//
//    agent.handle_phone_call().await
//}
//
//#[derive(Debug)]
//pub enum AudioDecodeError {
//    Base64DecodeError(String),
//    InvalidDataLength(String),
//}
//
//impl std::error::Error for AudioDecodeError {}
//
//impl std::fmt::Display for AudioDecodeError {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match self {
//            Self::Base64DecodeError(msg) => write!(f, "Base64 decode error: {}", msg),
//            Self::InvalidDataLength(msg) => write!(f, "Invalid data length: {}", msg),
//        }
//    }
//}
