use crate::endpoints::tts::ws::{EOSMessage, Flush, TextChunk, WebSocketTTS, WebSocketTTSResponse};
use crate::endpoints::Endpoint;
use crate::error::Error::ClientSendRequestError;
use crate::error::{ElevenLabsClientError, ElevenLabsServerError, WebSocketError};
use async_stream::stream;
use futures_util::{pin_mut, SinkExt, Stream, StreamExt};
use reqwest;
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;
use reqwest::Response;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message,
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
    pub fn default() -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            api_key: std::env::var("ELEVEN_API_KEY")?,
        })
    }
    pub fn new(api_key: String) -> Self {
        Self {
            inner: reqwest::Client::new(),
            api_key,
        }
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
    pub async fn hit_ws<S>(
        &self,
        mut endpoint: WebSocketTTS<S>,
    ) -> Result<impl Stream<Item = Result<WebSocketTTSResponse>>>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        let (ws_stream, _) = connect_async(endpoint.url()).await?;
        let (mut ws_writer, mut ws_reader) = ws_stream.split();
        let (tx, rx) = futures_channel::mpsc::unbounded::<Result<WebSocketTTSResponse>>();

        tokio::spawn(async move {
            while let Some(msg_result) = ws_reader.next().await {
                let msg = msg_result?;
                match msg {
                    Message::Text(text) => {
                        let response: WebSocketTTSResponse = serde_json::from_str(&text)?;
                        tx.unbounded_send(Ok(response))?;
                    }
                    Message::Close(msg) => {
                        if let Some(close_frame) = msg {
                            if close_frame.code == CloseCode::Normal {
                                continue;
                            } else {
                                tx.unbounded_send(Err(Box::new(
                                    WebSocketError::NonNormalCloseCode(
                                        close_frame.reason.to_string(),
                                    ),
                                )))?;
                            }
                        } else {
                            tx.unbounded_send(Err(Box::new(
                                WebSocketError::ClosedWithoutCloseFrame,
                            )))?;
                        }
                    }
                    _ => tx.unbounded_send(Err(Box::new(WebSocketError::UnexpectedMessageType)))?,
                }
            }
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });

        let api_key = self.api_key.clone();
        tokio::spawn(async move {
            // TODO: is this necessary? We always have a api key
            let mut bos_message = endpoint.bos_message().clone();
            if !bos_message.is_api_key() {
                bos_message = bos_message.with_api_key(&api_key);
            }
            let bos_message = serde_json::to_string(&bos_message)?;
            ws_writer.send(Message::text(bos_message)).await?;

            let generation_triggers = endpoint.try_trigger_generation().unwrap_or_default();
            //let flush = endpoint.flush();
            let flush_streams = endpoint.streams_after_flush();
            let text_stream = endpoint.text_stream();
            let stream = text_stream.enumerate();
            pin_mut!(stream);

            // TODO: impl try_trigger_always
            while let Some((i, chunk)) = stream.next().await {
                let trigger_index = i + 1;
                let trigger = generation_triggers.contains(&trigger_index);

                ws_writer
                    .send(Message::text(TextChunk::new(chunk, trigger).json()?))
                    .await?;
            }
            match flush_streams {
                Some(streams) => {
                    ws_writer.send(Message::text(Flush::new().json()?)).await?;
                    //tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    // TODO: impl generation_triggers for flush streams too?
                    for s in streams {
                        pin_mut!(s);
                        while let Some(item) = s.next().await {
                            ws_writer
                                .send(Message::text(TextChunk::new(item, true).json()?))
                                .await?;
                        }
                    }
                    ws_writer
                        .send(Message::text(EOSMessage::default().json()?))
                        .await?;
                }
                None => {
                    ws_writer
                        .send(Message::text(EOSMessage::default().json()?))
                        .await?;
                }
            };
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });
        Ok(rx)
    }
}

async fn handle_http_error(resp: Response) -> Result<Response> {
    if resp.status().is_server_error() {
        //dbg!(resp.status());
        let server_error = resp.json::<ElevenLabsServerError>().await?;
        return Err(Box::new(server_error));
    }
    if resp.status().is_client_error() {
        dbg!(resp.status());
        //return Err(Box::new(ClientSendRequestError(resp.json().await?)));
        let client_error = resp.json::<ElevenLabsClientError>().await?;
        return Err(Box::new(client_error));
    }
    // TODO: improve this error handling
    if !resp.status().is_success() {
        //dbg!(resp.status());
        return Err(Box::new(ClientSendRequestError(resp.json().await?)));
    }
    Ok(resp)
}

impl From<(reqwest::Client, String)> for ElevenLabsClient {
    fn from((client, api_key): (reqwest::Client, String)) -> Self {
        Self {
            inner: client,
            api_key,
        }
    }
}
