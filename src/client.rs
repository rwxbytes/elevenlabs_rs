use crate::endpoints::tts::ws::{WebSocketTTS, WebSocketTTSResponse};
use crate::endpoints::Endpoint;
use crate::error::Error::ClientSendRequestError;
use crate::error::{ElevenLabsClientError, ElevenLabsServerError};
use async_stream::{stream, try_stream};
use futures_util::{pin_mut, SinkExt, Stream, StreamExt};
use reqwest;
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;
use reqwest::Response;
use reqwest::StatusCode;
use serde_json::json;
use std::sync::mpsc::channel;
use std::thread;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::CloseFrame, tungstenite::protocol::Message,
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const BASE_URL: &str = "https://api.elevenlabs.io";
const XI_API_KEY_HEADER: &str = "xi-api-key";
const APPLICATION_JSON: &str = "application/json";
const MULTIPART_FORM_DATA: &str = "multipart/form-data";

pub struct ElevenLabsClient {
    inner: reqwest::Client,
    api_key: String,
}

impl ElevenLabsClient {
    // TODO: impl Default for ElevenLabsClient
    pub fn default() -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            api_key: std::env::var("ELEVEN_API_KEY")?,
        })
    }
    pub async fn hit<T: Endpoint>(&self, endpoint: T) -> Result<T::ResponseBody> {
        let init = self
            .inner
            .request(endpoint.method(), endpoint.url())
            .header(XI_API_KEY_HEADER, &self.api_key);

        let method = endpoint.method();
        let resp: Response;

        match method {
            Method::GET | Method::DELETE => {
                resp = init.send().await?;
            }
            Method::POST => {
                if endpoint.json_request_body().is_some() {
                    resp = init
                        .header(CONTENT_TYPE, APPLICATION_JSON)
                        // TODO: This should be a custom error
                        .json(&endpoint.json_request_body().unwrap()?)
                        .send()
                        .await?;
                } else if endpoint.multipart_request_body().is_some() {
                    resp = init
                        //.header(CONTENT_TYPE, MULTIPART_FORM_DATA)
                        // TODO: This should be a custom error
                        .multipart(endpoint.multipart_request_body().unwrap()?)
                        .send()
                        .await?;
                } else {
                    panic!("a post request must have a json or multipart body for ElevenLabs API");
                }
            }
            _ => {
                panic!("Unsupported method for ElevenLabs API");
            }
        }
        endpoint.response_body(handle_http_error(resp).await?).await
    }
    pub async fn hit_ws<T>(
        &self,
        endpoint: WebSocketTTS<T>,
    ) -> Result<impl Stream<Item = Result<WebSocketTTSResponse>>>
    where
        T: IntoIterator<Item = String> + Send + Sync + 'static,
    {
        let url = endpoint.url();
        // TODO: handle default and new api key and authorization
        let initial_msg = serde_json::to_string(endpoint.initial_message())?;

        // TODO: make fn to handle response if there is an error
        let (ws_stream, _) = connect_async(url.to_string()).await?;
        let (mut ws_writer, mut ws_reader) = ws_stream.split();

        ws_writer
            .send(Message::text(initial_msg))
            .await?;
        let text = endpoint.text();
        let text_stream = text_chunker(text);
        pin_mut!(text_stream);

        while let Some(w) = text_stream.next().await {
            let json = serde_json::json!({
                "text": w,
                "try_trigger_generation": true,
                "flush": false,
            });

            let json_str = serde_json::to_string(&json)?;

            ws_writer
                .send(Message::text(json_str))
                .await?;
        }
        ws_writer
            .send(Message::text(serde_json::to_string(&json!({"text": ""}))?, ))
            .await?;

        let stream = try_stream! {
            for await msg in ws_reader {
                match msg? {
                    Message::Text(text) => {
                        let response: WebSocketTTSResponse = serde_json::from_str(&text)?;
                        yield response
                    }
                    // TODO: impl this properly
                    Message::Close(frame) => {
                        if let Some(reason) = frame {
                            yield WebSocketTTSResponse::default()
                        } else {
                            panic!("WebSocket closed");
                        }
                    }
                    _ => panic!("Unexpected websocket message type from ElevenLabs")
                }
            }
        };
        Ok(stream)
    }
}


// TODO: Simplify this function
async fn handle_http_error(resp: Response) -> Result<Response> {
    if resp.status().is_server_error() {
        let server_error = resp.json::<ElevenLabsServerError>().await?;
        return Err(Box::new(server_error));
    }
    if resp.status().is_client_error() {
        let client_error = resp.json::<ElevenLabsClientError>().await?;
        return Err(Box::new(client_error));
    }
    // TODO: improve this error handling
    if !resp.status().is_success() {
        return Err(Box::new(ClientSendRequestError(resp.json().await?)));
    }
    Ok(resp)
}
fn text_chunker<T>(chunks: T) -> impl Stream<Item = String>
where
    T: IntoIterator<Item = String> + Send + Sync + 'static,
{
    let splitters = [
        '.', ',', '?', '!', ';', ':', '—', '-', '(', ')', '[', ']', '{', '}', ' ',
    ];
    let mut buf = String::new();
    let (tx, rx) = channel::<String>();

    // TODO: maybe use tokio::task instead of std::thread
    tokio::spawn(async move  {
        for text in chunks.into_iter() {
            if buf.ends_with(splitters) {
                tx.send(format!("{} ", buf.clone())).unwrap();
                buf = text
            } else if text.starts_with(splitters) {
                tx.send(format!("{} ", text.char_indices().next().unwrap().1))
                    .unwrap();
                buf = text[1..].to_string();
            } else {
                buf.push_str(&text)
            }
        }
        if !buf.is_empty() {
            tx.send(buf).unwrap()
        }
    });

    stream! {
        while let Ok(buf) = rx.recv() {
            yield buf
        }
    }
}
