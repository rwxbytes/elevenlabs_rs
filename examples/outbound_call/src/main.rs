use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::env::var;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, span, Level};
use elevenlabs_twilio::{CreateCall, CreateCallBody, ElevenLabsOutboundCallAgent, ElevenLabsTelephonyAgent};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("environment variable not set: {0}")]
    EnvVarError(String),

    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

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

    //#[error("conversational_ai error: {0}")]
    //ConversationalError(#[from] ConvAIError),

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

    //let body = CreateCallBody::new("to", "from", "url");
    //let endpoint = CreateCall::new(body);

    let agent = ElevenLabsOutboundCallAgent::from_env()?;
    //let e =  agent.twilio_client.hit(agent.create_call_endpoint).await?;
    let _ = agent.ring("to").await?;

    //let config = Config::from_env()?;

    let t = tokio::spawn(run_server(config.ngrok_url.clone()));

    let app = Router::new()
        .route("/outbound-call", post(move || twiml(ngrok_url)))
        .route("/ws", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    make_twilio_call(&config).await?;

    let _ = t.await?;

    Ok(())
}

async fn run_server(ngrok_url: String) -> Result<()> {
    let app = Router::new()
        .route("/outbound-call", post(move || twiml(ngrok_url)))
        .route("/ws", get(handler));

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
            <Stream url="wss://{}/ws" track="inbound_track" />
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
    let agent = ElevenLabsTelephonyAgent::from_env()?;
    let  _ = agent.handle_call(socket).await?;

    //tokio::select! {
    //    res = twilio_task => {
    //        info!("Twilio task done");
    //        res??;
    //        Ok(())
    //    }
    //    res = el_task => {
    //        info!("Elevenlabs task done");
    //        res??;
    //        Ok(())
    //    }
    //}
    Ok(())
}

