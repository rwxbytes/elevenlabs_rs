use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use elevenlabs_twilio::ElevenLabsTelephonyAgent;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/inbound-call", post(twiml))
        .route("/ws", get(ws_handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give us a call");
    axum::serve(listener, app).await.expect("Failed to start server");
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let calle = ElevenLabsTelephonyAgent::from_env().expect("Failed to create calle agent");
    println!("Answering call");
    let _ = calle.handle_call(socket).await.expect("Failed to answer call");
}

// TODO: add your ngrok domain
async fn twiml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Connect>
            <Stream url="wss://9f86-86-18-8-153.ngrok-free.app/ws" track="inbound_track" />
        </Connect>
    </Response>
    "#
}
