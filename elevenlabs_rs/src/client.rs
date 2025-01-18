#![allow(unused_imports)]
use crate::endpoints::{ElevenLabsEndpoint, RequestBody};
use crate::error::Error::HttpError;
use crate::error::WebSocketError;
#[cfg(feature = "ws")]
use crate::endpoints::genai::tts::ws::*;
use futures_util::{pin_mut, SinkExt, Stream, StreamExt};
use reqwest::{header::CONTENT_TYPE, Method};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const XI_API_KEY_HEADER: &str = "xi-api-key";
const APPLICATION_JSON: &str = "application/json";
//const MULTIPART_FORM_DATA: &str = "multipart/form-data"; // Client errs with this content type

#[derive(Clone)]
pub struct ElevenLabsClient {
    inner: reqwest::Client,
    api_key: String,
}

impl ElevenLabsClient {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            api_key: std::env::var("ELEVENLABS_API_KEY")
                .map_err(|_| "ELEVENLABS_API_KEY not set")?,
        })
    }
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: reqwest::Client::new(),
            api_key: api_key.into(),
        }
    }

    pub async fn hit<T: ElevenLabsEndpoint>(&self, endpoint: T) -> Result<T::ResponseBody> {
        let mut builder = self
            .inner
            .request(T::METHOD, endpoint.url())
            .header(XI_API_KEY_HEADER, &self.api_key);

        if matches!(T::METHOD, Method::POST | Method::PATCH) {
            let request_body = endpoint.request_body().await?;
            builder = match request_body {
                RequestBody::Json(json) => {
                    builder.header(CONTENT_TYPE, APPLICATION_JSON).json(&json)
                }
                RequestBody::Multipart(form) => builder.multipart(form),
                RequestBody::Empty => return Err("request must have a body".into()),
            };
        }

        let resp = builder.send().await?;

        if !resp.status().is_success() {
            return Err(Box::new(HttpError(resp.json().await?)));
        }

        endpoint.response_body(resp).await
    }

    #[cfg(feature = "ws")]
    const FLUSH_JSON: &'static str = r#"{"text":" ","flush":true}"#;
    #[cfg(feature = "ws")]
    const EOS_JSON: &'static str = r#"{"text":""}"#;

    #[cfg(feature = "ws")]
    pub async fn hit_ws<S>(
        &self,
        mut endpoint: WebSocketTTS<S>,
    ) -> Result<impl Stream<Item = Result<WebSocketTTSResponse>>>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        let (ws_stream, _) = connect_async(endpoint.url()).await?;
        let (mut writer, mut reader) = ws_stream.split();
        let (tx_to_caller, rx_for_caller) =
            futures_channel::mpsc::unbounded::<Result<WebSocketTTSResponse>>();

        // Perhaps remove api key setter from bos_message
        // as it is already set in the client ?
        if endpoint.body.bos_message.authorization.is_none() {
            endpoint.body.bos_message.xi_api_key = Some(self.api_key.clone());
        }

        let _reader_t: JoinHandle<Result<()>> = tokio::spawn(async move {
            while let Some(msg_result) = reader.next().await {
                let msg = msg_result?;
                match msg {
                    Message::Text(text) => {
                        dbg!(&text);
                        let response: WebSocketTTSResponse = serde_json::from_str(&text)?;
                        tx_to_caller.unbounded_send(Ok(response))?;
                    }
                    Message::Close(msg) => {
                        if let Some(close_frame) = msg {
                            if close_frame.code == CloseCode::Normal {
                                continue;
                            } else {
                                tx_to_caller.unbounded_send(Err(Box::new(
                                    WebSocketError::NonNormalCloseCode(
                                        close_frame.reason.to_string(),
                                    ),
                                )))?;
                            }
                        } else {
                            tx_to_caller.unbounded_send(Err(Box::new(
                                WebSocketError::ClosedWithoutCloseFrame,
                            )))?;
                        }
                    }
                    _ => tx_to_caller
                        .unbounded_send(Err(Box::new(WebSocketError::UnexpectedMessageType)))?,
                }
            }
            Ok(())
        });

        let _thread: JoinHandle<Result<()>> = tokio::spawn(async move {
            let bos_message = endpoint.body.bos_message;
            writer.send(bos_message.to_message()?).await?;

            let text_stream = endpoint.body.text_stream;
            pin_mut!(text_stream);

            while let Some(chunk) = text_stream.next().await {
                writer.send(chunk.to_message()?).await?;
            }

            if endpoint.body.flush {
                writer.send(Message::from(Self::FLUSH_JSON)).await?;
            }


            writer.send(Message::from(Self::EOS_JSON)).await?;

            Ok(())
        });
        Ok(rx_for_caller)
    }
}

impl From<(reqwest::Client, String)> for ElevenLabsClient {
    fn from((client, api_key): (reqwest::Client, String)) -> Self {
        Self {
            inner: client,
            api_key,
        }
    }
}
