#[cfg(feature = "ws")]
use crate::endpoints::genai::tts::ws::*;
use crate::endpoints::{ElevenLabsEndpoint, RequestBody};
#[cfg(feature = "ws")]
use crate::error::WebSocketError;
use crate::error::{ApiError, Error};
#[cfg(feature = "ws")]
use futures_util::{pin_mut, SinkExt, Stream, StreamExt};
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    StatusCode,
};
#[cfg(feature = "ws")]
use tokio::task::JoinHandle;
#[cfg(feature = "ws")]
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
#[cfg(feature = "ws")]
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub type Result<T> = std::result::Result<T, Error>;

const XI_API_KEY_HEADER: &str = "xi-api-key";
const APPLICATION_JSON: &str = "application/json";

#[derive(Clone, Debug)]
pub struct ApiResponse<T> {
    pub body: T,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub character_cost: Option<u64>,
}

#[derive(Clone)]
pub struct ElevenLabsClient {
    inner: reqwest::Client,
    api_key: String,
}

impl ElevenLabsClient {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            api_key: std::env::var("ELEVENLABS_API_KEY")?,
        })
    }
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: reqwest::Client::new(),
            api_key: api_key.into(),
        }
    }

    pub async fn hit<T: ElevenLabsEndpoint>(&self, endpoint: T) -> Result<T::ResponseBody> {
        Ok(self.hit_with_metadata(endpoint).await?.body)
    }

    pub async fn hit_with_metadata<T: ElevenLabsEndpoint>(
        &self,
        endpoint: T,
    ) -> Result<ApiResponse<T::ResponseBody>> {
        let mut builder = self
            .inner
            .request(T::METHOD, endpoint.url())
            .header(XI_API_KEY_HEADER, &self.api_key);

        builder = match endpoint.request_body().await? {
            RequestBody::Json(json) => builder.header(CONTENT_TYPE, APPLICATION_JSON).json(&json),
            RequestBody::Multipart(form) => builder.multipart(form),
            RequestBody::Empty => builder,
        };

        let resp = builder.send().await?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let request_id = header_string(&headers, &["request-id", "x-request-id"]);
        let trace_id = header_string(&headers, &["trace-id", "x-trace-id"]);
        let character_cost = header_u64(
            &headers,
            &[
                "character-cost",
                "x-character-cost",
                "x-elevenlabs-character-count",
            ],
        );

        if !status.is_success() {
            let body = resp.text().await?;
            let error = serde_json::from_str(&body).ok();
            return Err(Error::ApiError(Box::new(ApiError {
                status,
                body,
                error,
                headers,
                request_id,
                trace_id,
                character_cost,
            })));
        }

        let body = endpoint.response_body(resp).await?;

        Ok(ApiResponse {
            body,
            status,
            headers,
            request_id,
            trace_id,
            character_cost,
        })
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
                        let response: WebSocketTTSResponse = serde_json::from_str(&text)?;
                        tx_to_caller.unbounded_send(Ok(response))?;
                    }
                    Message::Close(msg) => {
                        if let Some(close_frame) = msg {
                            if close_frame.code == CloseCode::Normal {
                                continue;
                            } else {
                                tx_to_caller.unbounded_send(Err(
                                    WebSocketError::NonNormalCloseCode(
                                        close_frame.reason.to_string(),
                                    )
                                    .into(),
                                ))?;
                            }
                        } else {
                            tx_to_caller.unbounded_send(Err(
                                WebSocketError::ClosedWithoutCloseFrame.into(),
                            ))?;
                        }
                    }
                    _ => tx_to_caller
                        .unbounded_send(Err(WebSocketError::UnexpectedMessageType.into()))?,
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

fn header_string(headers: &HeaderMap, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        headers
            .get(*name)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned)
    })
}

fn header_u64(headers: &HeaderMap, names: &[&str]) -> Option<u64> {
    header_string(headers, names).and_then(|value| value.parse().ok())
}

impl From<(reqwest::Client, String)> for ElevenLabsClient {
    fn from((client, api_key): (reqwest::Client, String)) -> Self {
        Self {
            inner: client,
            api_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoints::ElevenLabsEndpoint;
    use serde_json::{json, Value};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::oneshot;

    struct DeleteWithBody {
        base_url: String,
        resource_id: String,
    }

    impl ElevenLabsEndpoint for DeleteWithBody {
        const PATH: &'static str = "/v1/resources/:resource_id";
        const METHOD: reqwest::Method = reqwest::Method::DELETE;
        type ResponseBody = Value;

        fn base_url(&self) -> &str {
            &self.base_url
        }

        fn path_params(&self) -> Vec<(&'static str, &str)> {
            vec![(":resource_id", &self.resource_id)]
        }

        async fn request_body(&self) -> Result<RequestBody> {
            Ok(RequestBody::Json(json!({ "force": true })))
        }

        async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
            Ok(resp.json().await?)
        }
    }

    struct EmptyPost {
        base_url: String,
    }

    impl ElevenLabsEndpoint for EmptyPost {
        const PATH: &'static str = "/v1/empty";
        const METHOD: reqwest::Method = reqwest::Method::POST;
        type ResponseBody = Value;

        fn base_url(&self) -> &str {
            &self.base_url
        }

        async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
            Ok(resp.json().await?)
        }
    }

    struct ErrorEndpoint {
        base_url: String,
    }

    impl ElevenLabsEndpoint for ErrorEndpoint {
        const PATH: &'static str = "/v1/fails";
        const METHOD: reqwest::Method = reqwest::Method::GET;
        type ResponseBody = Value;

        fn base_url(&self) -> &str {
            &self.base_url
        }

        async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
            Ok(resp.json().await?)
        }
    }

    #[tokio::test]
    async fn hit_attaches_json_body_for_delete_requests() {
        let response = http_response(
            "200 OK",
            &[
                ("request-id", "req_success"),
                ("x-trace-id", "trace_success"),
                ("character-cost", "17"),
            ],
            r#"{"deleted":true}"#,
        );
        let (base_url, request) = serve_once(response).await;
        let client = ElevenLabsClient::new("test-key");
        let endpoint = DeleteWithBody {
            base_url,
            resource_id: "voice a/b".to_owned(),
        };

        let response = client.hit_with_metadata(endpoint).await.unwrap();
        let request = request.await.unwrap();

        assert_eq!(response.body, json!({ "deleted": true }));
        assert_eq!(response.request_id.as_deref(), Some("req_success"));
        assert_eq!(response.trace_id.as_deref(), Some("trace_success"));
        assert_eq!(response.character_cost, Some(17));
        assert!(request.starts_with("DELETE /v1/resources/voice%20a%2Fb HTTP/1.1"));
        assert!(request.contains("xi-api-key: test-key"));
        assert!(request.contains("content-type: application/json"));
        assert!(request.ends_with(r#"{"force":true}"#));
    }

    #[tokio::test]
    async fn hit_allows_empty_post_requests() {
        let (base_url, request) = serve_once(http_response("200 OK", &[], r#"{"ok":true}"#)).await;
        let client = ElevenLabsClient::new("test-key");

        let response = client.hit(EmptyPost { base_url }).await.unwrap();
        let request = request.await.unwrap();

        assert_eq!(response, json!({ "ok": true }));
        assert!(request.starts_with("POST /v1/empty HTTP/1.1"));
    }

    #[tokio::test]
    async fn hit_returns_typed_api_errors_with_metadata() {
        let response = http_response(
            "429 Too Many Requests",
            &[
                ("request-id", "req_error"),
                ("x-trace-id", "trace_error"),
                ("character-cost", "3"),
            ],
            r#"{"detail":{"message":"rate limited"}}"#,
        );
        let (base_url, _request) = serve_once(response).await;
        let client = ElevenLabsClient::new("test-key");

        let error = client.hit(ErrorEndpoint { base_url }).await.unwrap_err();

        match error {
            Error::ApiError(api_error) => {
                assert_eq!(api_error.status, StatusCode::TOO_MANY_REQUESTS);
                assert_eq!(api_error.request_id.as_deref(), Some("req_error"));
                assert_eq!(api_error.trace_id.as_deref(), Some("trace_error"));
                assert_eq!(api_error.character_cost, Some(3));
                assert_eq!(api_error.body, r#"{"detail":{"message":"rate limited"}}"#);
                assert_eq!(
                    api_error.error.as_ref().unwrap()["detail"]["message"],
                    "rate limited"
                );
            }
            other => panic!("expected api error, got {other:?}"),
        }
    }

    async fn serve_once(response: String) -> (String, oneshot::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (sender, receiver) = oneshot::channel();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_http_request(&mut stream).await;
            let _ = sender.send(request);
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        (format!("http://{address}"), receiver)
    }

    async fn read_http_request(stream: &mut TcpStream) -> String {
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        let mut header_end = None;
        let mut content_length = 0;

        loop {
            let read = stream.read(&mut buffer).await.unwrap();
            if read == 0 {
                break;
            }

            request.extend_from_slice(&buffer[..read]);

            if header_end.is_none() {
                if let Some(end) = find_header_end(&request) {
                    header_end = Some(end);
                    content_length = parse_content_length(&request[..end]);
                }
            }

            if let Some(end) = header_end {
                if request.len() >= end + 4 + content_length {
                    break;
                }
            }
        }

        String::from_utf8(request).unwrap()
    }

    fn find_header_end(request: &[u8]) -> Option<usize> {
        request.windows(4).position(|window| window == b"\r\n\r\n")
    }

    fn parse_content_length(headers: &[u8]) -> usize {
        String::from_utf8_lossy(headers)
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().ok())
                    .flatten()
            })
            .unwrap_or(0)
    }

    fn http_response(status: &str, headers: &[(&str, &str)], body: &str) -> String {
        let extra_headers = headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}\r\n"))
            .collect::<String>();

        format!(
            "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n{extra_headers}\r\n{body}",
            body.len()
        )
    }
}
