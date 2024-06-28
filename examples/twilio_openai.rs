use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::{Form, State},
    response::Response,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use elevenlabs_rs::endpoints::tts::{OutputFormat, SpeechQuery, TextToSpeech, TextToSpeechBody};
use elevenlabs_rs::*;

#[tokio::main]
async fn main() {
    let gather_state = GatherState::default();

    let app = Router::new()
        .route("/call/incoming", post(twiml))
        .route("/gather", post(gather))
        .route("/audio", get(audio_handler))
        .with_state(gather_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give us a call");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone, Default)]
struct GatherState {
    data: Arc<Mutex<Option<String>>>,
}

async fn get_chat(prompt: &str, model: &str) -> String {
    let openai_client = Client::new();
    let chat = openai_client.chat();
    let chat_req = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: String::from(prompt).into(),
                name: None,
            },
        )])
        .build()
        .unwrap();

    let chat_resp = chat.create(chat_req).await.unwrap();
    let choices = chat_resp.choices;
    let mut chat_completion = String::new();
    for choice in choices {
        let message = choice.message;
        if let Some(content) = message.content {
            chat_completion.push_str(&content);
        }
    }
    chat_completion
}

async fn audio_handler(State(gather_state): State<GatherState>) -> impl IntoResponse {
    let mut speech_results = gather_state.data.lock().await;
    let text = speech_results.take().unwrap();

    let chat_completion = get_chat(&text, "gpt-4o").await;
    println!("Chat completion: {chat_completion}");

    let elevenlabs_client = ElevenLabsClient::default().unwrap();
    let model_id = Model::ElevenTurboV2;
    let tts_body = TextToSpeechBody::new(&chat_completion, model_id);
    let voice_id = PreMadeVoiceID::Adam;
    let tts = TextToSpeech::new(voice_id, tts_body)
        .with_query(SpeechQuery::default().with_output_format(DownloadOutputFormat::MuLaw8000Hz));
    let bytes = elevenlabs_client.hit(tts).await.unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "audio/basic")
        .body(Body::from(bytes))
        .unwrap()
}

async fn gather(
    State(gather_state): State<GatherState>,
    Form(gather): Form<Gather>,
) -> impl IntoResponse {
    println!("Gather results: {gather:?}");

    let mut data = gather_state.data.lock().await;
    *data = Some(gather.speech_result);

    // TODO: Add your ngrok domain
    let xml_data = r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Play>https://yourdomain.ngrok-free.app/audio</Play>
        <Redirect method="POST">https://yourdomain.ngrok-free.app/call/incoming</Redirect>
    </Response>
    "#;
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/xml")],
        xml_data,
    )
}

#[derive(Debug, Deserialize)]
struct Gather {
    #[serde(rename = "SpeechResult")]
    speech_result: String,
}

// TODO: Add your ngrok domain
async fn twiml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Gather input="speech" speechTimeout="1" action="https://yourdomain.ngrok-free.app/gather" />
    </Response>
    "#
}
