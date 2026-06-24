#[cfg(all(feature = "ws", feature = "genai"))]
use crate::endpoints::genai::speech_to_text::ws::{
    RealtimeSpeechToText, RealtimeSpeechToTextInput, RealtimeSpeechToTextResponse,
};
#[cfg(all(feature = "ws", feature = "genai"))]
use crate::endpoints::genai::tts::ws::{
    MultiContextTTSInput, MultiContextTTSResponse, MultiContextWebSocketTTS, WebSocketTTS,
    WebSocketTTSResponse,
};
use crate::endpoints::{ElevenLabsEndpoint, RequestBody, DEFAULT_BASE_URL};
use crate::error::{ApiError, Error};
use bytes::Bytes;
#[cfg(all(feature = "ws", feature = "genai"))]
use futures_util::Stream;
use reqwest::{
    header::{HeaderMap, CONTENT_LENGTH, CONTENT_TYPE},
    Method, StatusCode, Url,
};
use serde::{de::DeserializeOwned, Serialize};

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

#[derive(Clone, Debug)]
struct ResponseMetadata {
    status: StatusCode,
    headers: HeaderMap,
    request_id: Option<String>,
    trace_id: Option<String>,
    character_cost: Option<u64>,
}

impl ResponseMetadata {
    fn from_response(resp: &reqwest::Response) -> Self {
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

        Self {
            status,
            headers,
            request_id,
            trace_id,
            character_cost,
        }
    }

    fn into_api_response<T>(self, body: T) -> ApiResponse<T> {
        ApiResponse {
            body,
            status: self.status,
            headers: self.headers,
            request_id: self.request_id,
            trace_id: self.trace_id,
            character_cost: self.character_cost,
        }
    }
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

    /// Build an authenticated request for an ElevenLabs endpoint that is not
    /// modeled by this crate yet.
    pub fn raw(&self, method: Method, path: impl Into<String>) -> RawRequestBuilder<'_> {
        RawRequestBuilder::new(self, method, path)
    }

    pub async fn hit_with_metadata<T: ElevenLabsEndpoint>(
        &self,
        endpoint: T,
    ) -> Result<ApiResponse<T::ResponseBody>> {
        let builder = attach_request_body(
            self.authenticated_request(T::METHOD, endpoint.url()),
            endpoint.request_body().await?,
        );
        let (resp, metadata) = self.send_request(builder).await?;
        let body = endpoint.response_body(resp).await?;

        Ok(metadata.into_api_response(body))
    }

    fn authenticated_request(&self, method: Method, url: Url) -> reqwest::RequestBuilder {
        self.inner
            .request(method, url)
            .header(XI_API_KEY_HEADER, &self.api_key)
    }

    async fn send_request(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<(reqwest::Response, ResponseMetadata)> {
        let resp = builder.send().await?;
        let metadata = ResponseMetadata::from_response(&resp);

        if !metadata.status.is_success() {
            let body = resp.text().await?;
            let error = serde_json::from_str(&body).ok();
            return Err(Error::ApiError(Box::new(ApiError {
                status: metadata.status,
                body,
                error,
                headers: metadata.headers,
                request_id: metadata.request_id,
                trace_id: metadata.trace_id,
                character_cost: metadata.character_cost,
            })));
        }

        Ok((resp, metadata))
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    /// Connect to the realtime text-to-speech WebSocket API.
    pub async fn connect_text_to_speech<S>(
        &self,
        endpoint: WebSocketTTS<S>,
    ) -> Result<crate::ws::WebSocketSession<WebSocketTTSResponse>>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        crate::ws::connect_endpoint(endpoint, &self.api_key).await
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    /// Connect to the realtime text-to-speech WebSocket API with custom
    /// session options.
    pub async fn connect_text_to_speech_with_options<S>(
        &self,
        endpoint: WebSocketTTS<S>,
        options: crate::ws::WebSocketOptions,
    ) -> Result<crate::ws::WebSocketSession<WebSocketTTSResponse>>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        crate::ws::connect_endpoint_with_options(endpoint, &self.api_key, options).await
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    /// Compatibility alias for [`ElevenLabsClient::connect_text_to_speech`].
    #[deprecated(
        since = "0.7.0",
        note = "use connect_text_to_speech; hit_ws will be removed after one release"
    )]
    pub async fn hit_ws<S>(
        &self,
        endpoint: WebSocketTTS<S>,
    ) -> Result<crate::ws::WebSocketSession<WebSocketTTSResponse>>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        self.connect_text_to_speech(endpoint).await
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    /// Connect to the multi-context realtime text-to-speech WebSocket API.
    pub async fn connect_multi_context_text_to_speech<S>(
        &self,
        endpoint: MultiContextWebSocketTTS<S>,
    ) -> Result<crate::ws::WebSocketSession<MultiContextTTSResponse>>
    where
        S: Stream<Item = MultiContextTTSInput> + Send + 'static,
    {
        crate::ws::connect_endpoint(endpoint, &self.api_key).await
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    /// Connect to the multi-context realtime text-to-speech WebSocket API with
    /// custom session options.
    pub async fn connect_multi_context_text_to_speech_with_options<S>(
        &self,
        endpoint: MultiContextWebSocketTTS<S>,
        options: crate::ws::WebSocketOptions,
    ) -> Result<crate::ws::WebSocketSession<MultiContextTTSResponse>>
    where
        S: Stream<Item = MultiContextTTSInput> + Send + 'static,
    {
        crate::ws::connect_endpoint_with_options(endpoint, &self.api_key, options).await
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    /// Connect to the realtime speech-to-text WebSocket API.
    pub async fn connect_realtime_speech_to_text<S>(
        &self,
        endpoint: RealtimeSpeechToText<S>,
    ) -> Result<crate::ws::WebSocketSession<RealtimeSpeechToTextResponse>>
    where
        S: Stream<Item = RealtimeSpeechToTextInput> + Send + 'static,
    {
        crate::ws::connect_endpoint(endpoint, &self.api_key).await
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    /// Connect to the realtime speech-to-text WebSocket API with custom
    /// session options.
    pub async fn connect_realtime_speech_to_text_with_options<S>(
        &self,
        endpoint: RealtimeSpeechToText<S>,
        options: crate::ws::WebSocketOptions,
    ) -> Result<crate::ws::WebSocketSession<RealtimeSpeechToTextResponse>>
    where
        S: Stream<Item = RealtimeSpeechToTextInput> + Send + 'static,
    {
        crate::ws::connect_endpoint_with_options(endpoint, &self.api_key, options).await
    }
}

pub struct RawRequestBuilder<'a> {
    client: &'a ElevenLabsClient,
    method: Method,
    base_url: String,
    path: String,
    query_params: Vec<(String, String)>,
    body: RequestBody,
}

impl<'a> RawRequestBuilder<'a> {
    fn new(
        client: &'a ElevenLabsClient,
        method: Method,
        path: impl Into<String>,
    ) -> RawRequestBuilder<'a> {
        Self {
            client,
            method,
            base_url: DEFAULT_BASE_URL.to_owned(),
            path: path.into(),
            query_params: Vec::new(),
            body: RequestBody::Empty,
        }
    }

    /// Override the default ElevenLabs API base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Add a URL-encoded query parameter.
    pub fn query(mut self, name: impl Into<String>, value: impl ToString) -> Self {
        self.query_params.push((name.into(), value.to_string()));
        self
    }

    /// Attach a JSON request body.
    pub fn json<T>(mut self, body: &T) -> Result<Self>
    where
        T: Serialize + ?Sized,
    {
        self.body = RequestBody::Json(serde_json::to_value(body)?);
        Ok(self)
    }

    /// Attach a multipart request body.
    pub fn multipart(mut self, form: reqwest::multipart::Form) -> Self {
        self.body = RequestBody::Multipart(form);
        self
    }

    /// Send the request and deserialize a successful JSON response body.
    pub async fn send_json<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(self.send_json_with_metadata().await?.body)
    }

    /// Send the request and deserialize JSON plus response metadata.
    pub async fn send_json_with_metadata<T>(self) -> Result<ApiResponse<T>>
    where
        T: DeserializeOwned,
    {
        let (resp, metadata) = self.send().await?;
        let body = resp.json().await?;
        Ok(metadata.into_api_response(body))
    }

    /// Send the request and return a successful response body as bytes.
    pub async fn send_bytes(self) -> Result<Bytes> {
        Ok(self.send_bytes_with_metadata().await?.body)
    }

    /// Send the request and return bytes plus response metadata.
    pub async fn send_bytes_with_metadata(self) -> Result<ApiResponse<Bytes>> {
        let (resp, metadata) = self.send().await?;
        let body = resp.bytes().await?;
        Ok(metadata.into_api_response(body))
    }

    async fn send(self) -> Result<(reqwest::Response, ResponseMetadata)> {
        let client = self.client;
        let builder = self.request_builder()?;
        client.send_request(builder).await
    }

    fn request_builder(self) -> Result<reqwest::RequestBuilder> {
        let url = self.url()?;
        let builder = self.client.authenticated_request(self.method, url);
        Ok(attach_request_body(builder, self.body))
    }

    fn url(&self) -> Result<Url> {
        let mut url = self.base_url.parse::<Url>().map_err(|error| {
            Error::InvalidInput(format!(
                "invalid raw endpoint base URL `{}`: {error}",
                self.base_url
            ))
        })?;
        let path = self.path.trim_start_matches('/');
        let segments = path.split('/').filter(|segment| !segment.is_empty());

        {
            let mut url_segments = url.path_segments_mut().map_err(|_| {
                Error::InvalidInput(format!(
                    "raw endpoint base URL cannot contain a relative path: {}",
                    self.base_url
                ))
            })?;
            url_segments.clear();
            url_segments.extend(segments);

            if path.ends_with('/') {
                url_segments.push("");
            }
        }

        if !self.query_params.is_empty() {
            url.query_pairs_mut().extend_pairs(
                self.query_params
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_str())),
            );
        }

        Ok(url)
    }
}

fn attach_request_body(
    builder: reqwest::RequestBuilder,
    body: RequestBody,
) -> reqwest::RequestBuilder {
    match body {
        RequestBody::Json(json) => builder.header(CONTENT_TYPE, APPLICATION_JSON).json(&json),
        RequestBody::Multipart(form) => builder.multipart(form),
        RequestBody::Bytes(bytes) if bytes.is_empty() => builder.header(CONTENT_LENGTH, "0"),
        RequestBody::Bytes(bytes) => builder.body(bytes),
        RequestBody::Empty => builder,
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

    impl crate::endpoints::sealed::Sealed for DeleteWithBody {}

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

    impl crate::endpoints::sealed::Sealed for EmptyPost {}

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

    struct EmptyBytesPost {
        base_url: String,
    }

    impl crate::endpoints::sealed::Sealed for EmptyBytesPost {}

    impl ElevenLabsEndpoint for EmptyBytesPost {
        const PATH: &'static str = "/v1/empty-bytes";
        const METHOD: reqwest::Method = reqwest::Method::POST;
        type ResponseBody = Value;

        fn base_url(&self) -> &str {
            &self.base_url
        }

        async fn request_body(&self) -> Result<RequestBody> {
            Ok(RequestBody::Bytes(Bytes::new()))
        }

        async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
            Ok(resp.json().await?)
        }
    }

    struct ErrorEndpoint {
        base_url: String,
    }

    impl crate::endpoints::sealed::Sealed for ErrorEndpoint {}

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
    async fn hit_sends_content_length_for_empty_byte_posts() {
        let (base_url, request) = serve_once(http_response("200 OK", &[], r#"{"ok":true}"#)).await;
        let client = ElevenLabsClient::new("test-key");

        let response = client.hit(EmptyBytesPost { base_url }).await.unwrap();
        let request = request.await.unwrap();

        assert_eq!(response, json!({ "ok": true }));
        assert!(request.starts_with("POST /v1/empty-bytes HTTP/1.1"));
        assert!(request.contains("content-length: 0"));
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

    #[tokio::test]
    async fn raw_json_requests_use_auth_query_body_and_metadata() {
        let response = http_response(
            "200 OK",
            &[
                ("request-id", "req_raw"),
                ("x-trace-id", "trace_raw"),
                ("character-cost", "29"),
            ],
            r#"{"ok":true}"#,
        );
        let (base_url, request) = serve_once(response).await;
        let client = ElevenLabsClient::new("test-key");

        let response: ApiResponse<Value> = client
            .raw(reqwest::Method::POST, "/v1/raw resources")
            .with_base_url(base_url)
            .query("voice_id", "voice a/b")
            .json(&json!({ "text": "hello" }))
            .unwrap()
            .send_json_with_metadata()
            .await
            .unwrap();
        let request = request.await.unwrap();

        assert_eq!(response.body, json!({ "ok": true }));
        assert_eq!(response.request_id.as_deref(), Some("req_raw"));
        assert_eq!(response.trace_id.as_deref(), Some("trace_raw"));
        assert_eq!(response.character_cost, Some(29));
        assert!(request.starts_with("POST /v1/raw%20resources?voice_id=voice+a%2Fb HTTP/1.1"));
        assert!(request.contains("xi-api-key: test-key"));
        assert!(request.contains("content-type: application/json"));
        assert!(request.ends_with(r#"{"text":"hello"}"#));
    }

    #[tokio::test]
    async fn raw_requests_can_return_bytes() {
        let (base_url, _request) =
            serve_once(http_response("200 OK", &[], r#"not-json-audio"#)).await;
        let client = ElevenLabsClient::new("test-key");

        let response = client
            .raw(reqwest::Method::GET, "/v1/audio")
            .with_base_url(base_url)
            .send_bytes()
            .await
            .unwrap();

        assert_eq!(response.as_ref(), b"not-json-audio");
    }

    #[tokio::test]
    async fn raw_requests_return_typed_api_errors_with_metadata() {
        let response = http_response(
            "404 Not Found",
            &[
                ("request-id", "req_raw_error"),
                ("x-trace-id", "trace_raw_error"),
                ("character-cost", "5"),
            ],
            r#"{"detail":{"message":"unknown endpoint"}}"#,
        );
        let (base_url, _request) = serve_once(response).await;
        let client = ElevenLabsClient::new("test-key");

        let error = client
            .raw(reqwest::Method::GET, "/v1/future")
            .with_base_url(base_url)
            .send_json::<Value>()
            .await
            .unwrap_err();

        match error {
            Error::ApiError(api_error) => {
                assert_eq!(api_error.status, StatusCode::NOT_FOUND);
                assert_eq!(api_error.request_id.as_deref(), Some("req_raw_error"));
                assert_eq!(api_error.trace_id.as_deref(), Some("trace_raw_error"));
                assert_eq!(api_error.character_cost, Some(5));
                assert_eq!(
                    api_error.error.as_ref().unwrap()["detail"]["message"],
                    "unknown endpoint"
                );
            }
            other => panic!("expected api error, got {other:?}"),
        }
    }

    #[cfg(all(feature = "ws", feature = "genai"))]
    mod websocket_integration {
        use super::*;
        use crate::endpoints::genai::speech_to_text::ws::{
            RealtimeSpeechToText, RealtimeSpeechToTextInput, RealtimeSpeechToTextQuery,
            RealtimeSpeechToTextResponse,
        };
        use crate::endpoints::genai::tts::ws::{BOSMessage, WebSocketTTS, WebSocketTTSBody};
        use crate::error::WebSocketError;
        use futures_util::{SinkExt, StreamExt};
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        use tokio::task::JoinHandle;
        use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
        use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
        use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Message};
        use tokio_tungstenite::{accept_async, accept_hdr_async};

        #[tokio::test]
        async fn tts_websocket_sends_bos_chunks_flush_and_eos_in_order() {
            let (base_url, listener) = local_ws_listener().await;
            let (messages_tx, messages_rx) = oneshot::channel();
            let server = spawn_tts_collector(listener, messages_tx);

            let text_stream =
                futures_util::stream::iter(["hello".to_string(), "world".to_string()]);
            let body = WebSocketTTSBody::new(BOSMessage::default(), text_stream).with_flush();
            let endpoint = WebSocketTTS::new("voice-id", body).with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let mut session = client.connect_text_to_speech(endpoint).await.unwrap();
            let messages = messages_rx.await.unwrap();
            session.close().await.unwrap();
            await_server(server).await;

            assert_eq!(messages[0]["text"], json!(" "));
            assert_eq!(messages[0]["xi_api_key"], json!("test-key"));
            assert_eq!(messages[1], json!({ "text": "hello" }));
            assert_eq!(messages[2], json!({ "text": "world" }));
            assert_eq!(messages[3], json!({ "text": " ", "flush": true }));
            assert_eq!(messages[4], json!({ "text": "" }));
        }

        #[tokio::test]
        async fn tts_websocket_does_not_overwrite_explicit_bearer_auth() {
            let (base_url, listener) = local_ws_listener().await;
            let (messages_tx, messages_rx) = oneshot::channel();
            let server = spawn_tts_collector(listener, messages_tx);

            let body = WebSocketTTSBody::new(
                BOSMessage::default().with_authorization("bearer-token"),
                futures_util::stream::empty(),
            );
            let endpoint = WebSocketTTS::new("voice-id", body).with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let mut session = client.connect_text_to_speech(endpoint).await.unwrap();
            let messages = messages_rx.await.unwrap();
            session.close().await.unwrap();
            await_server(server).await;

            assert_eq!(messages[0]["authorization"], json!("Bearer bearer-token"));
            assert!(messages[0].get("xi_api_key").is_none());
        }

        #[tokio::test]
        async fn stt_websocket_sends_header_auth_without_token_query() {
            let (base_url, listener) = local_ws_listener().await;
            let captured = Arc::new(Mutex::new(None));
            let server = spawn_handshake_capture(listener, Arc::clone(&captured));

            let endpoint = RealtimeSpeechToText::new(
                "scribe_v2_realtime",
                futures_util::stream::empty::<RealtimeSpeechToTextInput>(),
            )
            .with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let _session = client
                .connect_realtime_speech_to_text(endpoint)
                .await
                .unwrap();
            await_server(server).await;
            let captured = captured.lock().unwrap().clone().unwrap();

            assert_eq!(captured.xi_api_key.as_deref(), Some("test-key"));
            assert!(captured.uri.contains("model_id=scribe_v2_realtime"));
        }

        #[tokio::test]
        async fn stt_websocket_uses_token_query_without_api_key_header() {
            let (base_url, listener) = local_ws_listener().await;
            let captured = Arc::new(Mutex::new(None));
            let server = spawn_handshake_capture(listener, Arc::clone(&captured));

            let endpoint = RealtimeSpeechToText::new(
                "scribe_v2_realtime",
                futures_util::stream::empty::<RealtimeSpeechToTextInput>(),
            )
            .with_query(RealtimeSpeechToTextQuery::default().with_token("single-use-token"))
            .with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let _session = client
                .connect_realtime_speech_to_text(endpoint)
                .await
                .unwrap();
            await_server(server).await;
            let captured = captured.lock().unwrap().clone().unwrap();

            assert!(captured.xi_api_key.is_none());
            assert!(captured.uri.contains("token=single-use-token"));
        }

        #[tokio::test]
        async fn stt_websocket_sends_base64_audio_json_frames() {
            let (base_url, listener) = local_ws_listener().await;
            let (message_tx, message_rx) = oneshot::channel();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut ws = accept_async(stream).await.unwrap();
                let message = read_next_json_text(&mut ws).await;
                let _ = message_tx.send(message);
                wait_for_close(&mut ws).await;
            });

            let input_stream =
                futures_util::stream::iter([
                    RealtimeSpeechToTextInput::audio(b"hello").with_commit(true)
                ]);
            let endpoint = RealtimeSpeechToText::new("scribe_v2_realtime", input_stream)
                .with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let _session = client
                .connect_realtime_speech_to_text(endpoint)
                .await
                .unwrap();
            let message = message_rx.await.unwrap();
            await_server(server).await;

            assert_eq!(
                message,
                json!({
                    "message_type": "input_audio_chunk",
                    "audio_base_64": "aGVsbG8=",
                    "commit": true
                })
            );
        }

        #[tokio::test]
        async fn websocket_ping_frames_do_not_terminate_session() {
            let (base_url, listener) = local_ws_listener().await;
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut ws = accept_async(stream).await.unwrap();
                ws.send(Message::Ping(vec![1, 2, 3].into())).await.unwrap();
                ws.send(Message::Text(
                    r#"{"message_type":"partial_transcript","text":"hello"}"#.into(),
                ))
                .await
                .unwrap();
                wait_for_close(&mut ws).await;
            });

            let endpoint = RealtimeSpeechToText::new(
                "scribe_v2_realtime",
                futures_util::stream::pending::<RealtimeSpeechToTextInput>(),
            )
            .with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let mut session = client
                .connect_realtime_speech_to_text(endpoint)
                .await
                .unwrap();
            let response = session.next().await.unwrap().unwrap();
            session.close().await.unwrap();
            await_server(server).await;

            assert!(matches!(
                response,
                RealtimeSpeechToTextResponse::PartialTranscript(transcript)
                    if transcript.text == "hello"
            ));
        }

        #[tokio::test]
        async fn websocket_non_normal_close_reports_code_and_reason() {
            let (base_url, listener) = local_ws_listener().await;
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut ws = accept_async(stream).await.unwrap();
                ws.send(Message::Close(Some(CloseFrame {
                    code: CloseCode::Error,
                    reason: "boom".into(),
                })))
                .await
                .unwrap();
            });

            let endpoint = RealtimeSpeechToText::new(
                "scribe_v2_realtime",
                futures_util::stream::pending::<RealtimeSpeechToTextInput>(),
            )
            .with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let mut session = client
                .connect_realtime_speech_to_text(endpoint)
                .await
                .unwrap();
            let error = session.next().await.unwrap().unwrap_err();
            await_server(server).await;

            assert!(matches!(
                error,
                Error::WebSocketError(WebSocketError::NonNormalClose {
                    context,
                    code,
                    reason
                }) if context.endpoint == "speech_to_text.realtime"
                    && context.direction == crate::error::WebSocketDirection::Inbound
                    && code == "1011"
                    && reason == "boom"
            ));
        }

        #[tokio::test]
        async fn websocket_malformed_json_reports_bounded_decode_preview() {
            let (base_url, listener) = local_ws_listener().await;
            let invalid_payload = format!(
                "{{\"message_type\":\"partial_transcript\",\"text\":\"{}\"",
                "x".repeat(400)
            );
            let server_payload = invalid_payload.clone();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut ws = accept_async(stream).await.unwrap();
                ws.send(Message::Text(server_payload.into())).await.unwrap();
            });

            let endpoint = RealtimeSpeechToText::new(
                "scribe_v2_realtime",
                futures_util::stream::pending::<RealtimeSpeechToTextInput>(),
            )
            .with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let mut session = client
                .connect_realtime_speech_to_text(endpoint)
                .await
                .unwrap();
            let error = session.next().await.unwrap().unwrap_err();
            session.abort();
            await_server(server).await;

            assert!(matches!(
                error,
                Error::WebSocketError(WebSocketError::Decode { context, payload_preview, .. })
                    if context.endpoint == "speech_to_text.realtime"
                        && context.direction == crate::error::WebSocketDirection::Inbound
                        && payload_preview.ends_with("...")
                        && payload_preview.chars().count() == 259
                        && invalid_payload.starts_with(payload_preview.trim_end_matches("..."))
            ));
        }

        #[tokio::test]
        async fn websocket_session_close_sends_close_frame() {
            let (base_url, listener) = local_ws_listener().await;
            let (closed_tx, closed_rx) = oneshot::channel();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut ws = accept_async(stream).await.unwrap();
                wait_for_close(&mut ws).await;
                let _ = closed_tx.send(());
            });

            let endpoint = RealtimeSpeechToText::new(
                "scribe_v2_realtime",
                futures_util::stream::pending::<RealtimeSpeechToTextInput>(),
            )
            .with_base_url(base_url);
            let client = ElevenLabsClient::new("test-key");

            let mut session = client
                .connect_realtime_speech_to_text(endpoint)
                .await
                .unwrap();
            session.close().await.unwrap();
            closed_rx.await.unwrap();
            await_server(server).await;
        }

        #[derive(Clone, Debug)]
        struct CapturedHandshake {
            xi_api_key: Option<String>,
            uri: String,
        }

        async fn local_ws_listener() -> (String, TcpListener) {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            (format!("ws://{address}"), listener)
        }

        fn spawn_tts_collector(
            listener: TcpListener,
            messages_tx: oneshot::Sender<Vec<Value>>,
        ) -> JoinHandle<()> {
            tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut ws = accept_async(stream).await.unwrap();
                let mut messages = Vec::new();

                loop {
                    let message = read_next_json_text(&mut ws).await;
                    let is_eos = message.get("text") == Some(&json!(""));
                    messages.push(message);
                    if is_eos {
                        break;
                    }
                }

                let _ = messages_tx.send(messages);
                wait_for_close(&mut ws).await;
            })
        }

        fn spawn_handshake_capture(
            listener: TcpListener,
            captured: Arc<Mutex<Option<CapturedHandshake>>>,
        ) -> JoinHandle<()> {
            tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let captured_for_callback = Arc::clone(&captured);
                let mut ws =
                    accept_hdr_async(stream, move |request: &Request, response: Response| {
                        let xi_api_key = request
                            .headers()
                            .get("xi-api-key")
                            .and_then(|value| value.to_str().ok())
                            .map(ToOwned::to_owned);
                        let uri = request.uri().to_string();
                        *captured_for_callback.lock().unwrap() =
                            Some(CapturedHandshake { xi_api_key, uri });
                        Ok(response)
                    })
                    .await
                    .unwrap();
                wait_for_close(&mut ws).await;
            })
        }

        async fn read_next_json_text<S>(ws: &mut S) -> Value
        where
            S: futures_util::Stream<
                    Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>,
                > + Unpin,
        {
            loop {
                let message = ws.next().await.unwrap().unwrap();
                if let Message::Text(text) = message {
                    return serde_json::from_str(&text).unwrap();
                }
            }
        }

        async fn wait_for_close<S>(ws: &mut S)
        where
            S: futures_util::Stream<
                    Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>,
                > + Unpin,
        {
            while let Some(message) = ws.next().await {
                if matches!(message.unwrap(), Message::Close(_)) {
                    break;
                }
            }
        }

        async fn await_server(server: JoinHandle<()>) {
            tokio::time::timeout(Duration::from_secs(3), server)
                .await
                .expect("websocket test server timed out")
                .expect("websocket test server panicked");
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
