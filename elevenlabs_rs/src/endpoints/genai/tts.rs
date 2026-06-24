//! The text-to-speech endpoints
use super::*;
use crate::endpoints::ElevenLabsEndpoint;
use crate::shared::{query_params::OutputFormat, DictionaryLocator, VoiceSettings};
use async_stream::try_stream;
use base64::{engine::general_purpose, Engine};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

/// Convert text to speech using Elevenlabs' library of over 3,000 voices across 32 languages.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::genai::tts::{TextToSpeech, TextToSpeechBody};
///
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let body = TextToSpeechBody::new("Look on my Works, ye Mighty, and despair!")
///        .with_model_id(Model::ElevenFlashV2);
///
///     let endpoint = TextToSpeech::new(LegacyVoice::Clyde, body);
///
///     let speech = c.hit(endpoint).await?;
///     play(speech)?;
///
///     Ok(())
/// }
/// ```
/// See [Text To Speech API reference](https://elevenlabs.io/docs/api-reference/text-to-speech/convert)
#[derive(Clone, Debug)]
pub struct TextToSpeech {
    voice_id: String,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeech {
    pub fn new(voice_id: impl Into<String>, body: TextToSpeechBody) -> Self {
        TextToSpeech {
            voice_id: voice_id.into(),
            body,
            query: None,
        }
    }

    pub fn with_query(mut self, query: TextToSpeechQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl crate::endpoints::sealed::Sealed for TextToSpeech {}

impl ElevenLabsEndpoint for TextToSpeech {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// Text to Speech Body
#[derive(Clone, Debug, Serialize, Default)]
pub struct TextToSpeechBody {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice_settings: Option<VoiceSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pronunciation_dictionary_locators: Option<Vec<DictionaryLocator>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_request_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_request_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    apply_text_normalization: Option<Normalization>,
    #[serde(skip_serializing_if = "Option::is_none")]
    apply_language_text_normalization: Option<bool>,
}

impl TryFrom<&TextToSpeechBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(value: &TextToSpeechBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(value)?))
    }
}

impl TextToSpeechBody {
    pub fn new(text: impl Into<String>) -> Self {
        TextToSpeechBody {
            text: text.into(),
            ..Default::default()
        }
    }

    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn with_language_code(mut self, language_code: impl Into<String>) -> Self {
        self.language_code = Some(language_code.into());
        self
    }
    pub fn with_dictionary_locators(mut self, locators: DictionaryLocators) -> Self {
        let values = locators.0.into_iter().flatten().collect();
        self.pronunciation_dictionary_locators = Some(values);
        self
    }
    pub fn with_voice_settings(mut self, voice_settings: VoiceSettings) -> Self {
        self.voice_settings = Some(voice_settings);
        self
    }
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
    pub fn with_previous_text(mut self, previous_text: &str) -> Self {
        self.previous_text = Some(previous_text.to_string());
        self
    }
    pub fn with_next_text(mut self, next_text: &str) -> Self {
        self.next_text = Some(next_text.to_string());
        self
    }
    pub fn with_previous_request_ids(mut self, ids: Vec<String>) -> Self {
        self.previous_request_ids = Some(ids);
        self
    }
    pub fn with_next_request_ids(mut self, ids: Vec<String>) -> Self {
        self.next_request_ids = Some(ids);
        self
    }

    pub fn with_text_normalization(mut self, normalization: Normalization) -> Self {
        self.apply_text_normalization = Some(normalization);
        self
    }
    pub fn with_language_text_normalization(mut self, enable: bool) -> Self {
        self.apply_language_text_normalization = Some(enable);
        self
    }
}

///  Dictionary Locators
/// # Example
/// ```ignore
///  use elevenlabs_rs::endpoints::admin::pronunciation::GetDictionaries;
///
/// // Get all dictionaries
/// let dictionaries = client.hit(GetDictionaries::default()).await?;
/// // Takes up to 3 dictionaries and creates locators from them
/// let locators = DictionaryLocators::from(dictionaries);
///
/// // Or push up to 3 locators into the locators array
/// let mut locators = DictionaryLocators::default();
/// locators.push(DictionaryLocator::new("id", "version_id"));
/// locators.push(DictionaryLocator::new("id", "version_id"));
///
/// let body = TextToSpeechBody::new("txt")
///     .with_model_id(Model::ElevenMultilingualV2)
///     .with_dictionary_locators(locators);
/// ```
#[derive(Clone, Debug, Default, Serialize)]
pub struct DictionaryLocators([Option<DictionaryLocator>; 3]);

impl DictionaryLocators {
    /// Add a new locator if there's space, returns false if full
    pub fn push(&mut self, locator: DictionaryLocator) -> bool {
        for slot in &mut self.0 {
            if slot.is_none() {
                *slot = Some(locator);
                return true;
            }
        }
        false
    }
}

#[cfg(feature = "admin")]
impl From<crate::endpoints::admin::pronunciation::GetDictionariesResponse> for DictionaryLocators {
    fn from(response: crate::endpoints::admin::pronunciation::GetDictionariesResponse) -> Self {
        let mut locators = Self::default();
        response.into_iter().take(3).for_each(|dict| {
            locators.push(DictionaryLocator::new(&dict.id, &dict.latest_version_id));
        });
        locators
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Normalization {
    Auto,
    On,
    Off,
}

impl Normalization {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

impl std::fmt::Display for Normalization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextToSpeechQuery {
    params: QueryValues,
}

impl TextToSpeechQuery {
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.params
            .push(("output_format", output_format.to_string()));
        self
    }
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.params
            .push(("enable_logging", enable_logging.to_string()));
        self
    }
}
/// Text to Speech Stream
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::stream_audio;
/// use elevenlabs_rs::endpoints::genai::tts::{TextToSpeechStream, TextToSpeechBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let txt = "The art of progress is to preserve order amid change \
///        and to preserve change amid order.";
///
///     let body = TextToSpeechBody::new(txt).with_model_id(Model::ElevenFlashV2);
///
///     let endpoint = TextToSpeechStream::new(DefaultVoice::Alice, body);
///
///     let mut stream = c.hit(endpoint).await?;
///     stream_audio(&mut stream).await?;
///
///     Ok(())
/// }
/// ```
/// See [Text To Speech Stream API reference](https://elevenlabs.io/docs/api-reference/text-to-speech/convert-as-stream)
#[derive(Clone, Debug)]
pub struct TextToSpeechStream {
    voice_id: String,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeechStream {
    pub fn new(voice_id: impl Into<String>, body: TextToSpeechBody) -> Self {
        TextToSpeechStream {
            voice_id: voice_id.into(),
            body,
            query: None,
        }
    }
    pub fn with_query(mut self, query: TextToSpeechQuery) -> Self {
        self.query = Some(query);
        self
    }
}

type TextToSpeechStreamResponse = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
impl crate::endpoints::sealed::Sealed for TextToSpeechStream {}

impl ElevenLabsEndpoint for TextToSpeechStream {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id/stream";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToSpeechStreamResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream.map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
}

/// Generate speech from text with precise character-level timing information for audio-text synchronization.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result, Model, LegacyVoice};
/// use elevenlabs_rs::endpoints::genai::tts::{TextToSpeechWithTimestamps, TextToSpeechBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let voice_id = LegacyVoice::Rachel;
///     let model_id = Model::ElevenMultilingualV2;
///
///     let txt = "To see a world in a grain of sand, and a heaven in a wild flower, \
///         hold infinity in the palm of your hand, and eternity in an hour.";
///
///     let body = TextToSpeechBody::new(txt).with_model_id(model_id);
///
///     let endpoint = TextToSpeechWithTimestamps::new(voice_id, body);
///     let resp = c.hit(endpoint).await?;
///
///     let alignment = resp.alignment.unwrap();
///
///     for (char, (start_time, end_time)) in alignment.iter() {
///         println!("{} = {} - {}", char, start_time, end_time);
///     }
///
///     Ok(())
/// }
/// ```
/// See [Text To Speech with Timing API reference](https://elevenlabs.io/docs/api-reference/text-to-speech/convert-with-timestamps)
#[derive(Clone, Debug)]
pub struct TextToSpeechWithTimestamps {
    voice_id: String,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeechWithTimestamps {
    pub fn new(voice_id: impl Into<String>, body: TextToSpeechBody) -> Self {
        TextToSpeechWithTimestamps {
            voice_id: voice_id.into(),
            body,
            query: None,
        }
    }

    pub fn with_query(mut self, query: TextToSpeechQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl crate::endpoints::sealed::Sealed for TextToSpeechWithTimestamps {}

impl ElevenLabsEndpoint for TextToSpeechWithTimestamps {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id/with-timestamps";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToSpeechWithTimestampsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }
    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextToSpeechWithTimestampsResponse {
    pub alignment: Option<Alignment>,
    pub audio_base64: String,
    pub normalized_alignment: Option<Alignment>,
}

impl TextToSpeechWithTimestampsResponse {
    pub fn audio(&self) -> Result<Bytes> {
        let decoded_audio_b64 = general_purpose::STANDARD.decode(&self.audio_base64)?;
        Ok(Bytes::from(decoded_audio_b64))
    }
}

pub struct Timestamps<'a> {
    alignment: &'a Alignment,
    index: usize,
}

impl<'a> Timestamps<'a> {
    pub fn new(alignment: &'a Alignment) -> Self {
        Timestamps {
            alignment,
            index: 0,
        }
    }
}

impl<'a> Iterator for Timestamps<'a> {
    type Item = (&'a String, (f64, f64));

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.alignment.characters.len() {
            let item = (
                self.alignment.characters.get(self.index).unwrap(),
                (
                    self.alignment.character_start_times_seconds[self.index],
                    self.alignment.character_end_times_seconds[self.index],
                ),
            );

            self.index += 1;

            Some(item)
        } else {
            None
        }
    }
}

impl Alignment {
    pub fn iter(&self) -> Timestamps<'_> {
        Timestamps::new(self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Alignment {
    pub character_end_times_seconds: Vec<f64>,
    pub character_start_times_seconds: Vec<f64>,
    pub characters: Vec<String>,
}

/// Stream speech from text with precise character-level timing information for audio-text synchronization.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::genai::tts::{TextToSpeechStreamWithTimestamps, TextToSpeechBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let voice_id = LegacyVoice::Rachel;
///     let model_id = Model::ElevenFlashV2;
///     let txt = "Without Haste! Without Rest!,\
///         Bind the motto to thy breast! \
///         Bear it with thee as a spell; \
///         Storm or sunshine, guard it well!";
///
///     let body = TextToSpeechBody::new(txt).with_model_id(model_id);
///     let endpoint = TextToSpeechStreamWithTimestamps::new(voice_id, body);
///     let resp = c.hit(endpoint).await?;
///     pin_mut!(resp);
///
///     while let Some(result) = resp.next().await {
///         let tts_timestamp_resp = result?;
///         if let Some(alignment) = tts_timestamp_resp.alignment {
///            for (char, (start_time, end_time)) in alignment.iter() {
///               println!("{} = {} - {}", char, start_time, end_time);
///           }
///       }
///    }
///
///     Ok(())
/// }
/// ```
/// See [Text To Speech Stream with Timestamps API reference](https://elevenlabs.io/docs/api-reference/text-to-speech/stream-with-timestamps)
#[derive(Clone, Debug)]
pub struct TextToSpeechStreamWithTimestamps {
    voice_id: String,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeechStreamWithTimestamps {
    pub fn new(voice_id: impl Into<String>, body: TextToSpeechBody) -> Self {
        TextToSpeechStreamWithTimestamps {
            voice_id: voice_id.into(),
            body,
            query: None,
        }
    }
    pub fn with_query(mut self, query: TextToSpeechQuery) -> Self {
        self.query = Some(query);
        self
    }
}

type TextToSpeechStreamWithTimestampsResponse =
    Pin<Box<dyn Stream<Item = Result<TextToSpeechWithTimestampsResponse>> + Send>>;

impl crate::endpoints::sealed::Sealed for TextToSpeechStreamWithTimestamps {}

impl ElevenLabsEndpoint for TextToSpeechStreamWithTimestamps {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id/stream/with-timestamps";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToSpeechStreamWithTimestampsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.and_param(PathParam::VoiceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream_chunks_to_json(stream);
        Ok(Box::pin(stream))
    }
}
// Helper
//
// HTTP chunks don't align with message boundaries, so we buffer raw bytes and let
// serde pull off each complete JSON value, tracking how many bytes it consumed.
// This handles partial messages, several messages in one chunk, and UTF-8
// characters split across chunks — independent of the delimiter the server uses.
fn stream_chunks_to_json(
    stream: impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static,
) -> impl Stream<Item = Result<TextToSpeechWithTimestampsResponse>> + Send {
    try_stream! {
        let mut buffer: Vec<u8> = Vec::new();

        for await chunk in stream {
            buffer.extend_from_slice(&chunk?);

            loop {
                let mut iter = serde_json::Deserializer::from_slice(&buffer)
                    .into_iter::<TextToSpeechWithTimestampsResponse>();
                match iter.next() {
                    Some(Ok(value)) => {
                        let consumed = iter.byte_offset();
                        buffer.drain(..consumed);
                        yield value;
                    }
                    // Incomplete value: wait for more bytes.
                    Some(Err(e)) if e.is_eof() => break,
                    Some(Err(e)) => Err(e)?,
                    // Only trailing whitespace left.
                    None => break,
                }
            }
        }
    }
}

#[cfg(feature = "ws")]
pub mod ws {
    //! Websocket Text to Speech endpoints

    use super::*;
    use crate::OutputFormat;
    use serde_json::{Map, Value};
    use std::pin::Pin;
    use tokio_tungstenite::tungstenite::Message;

    const WS_BASE_URL: &str = "wss://api.elevenlabs.io";
    const WS_PATH: &str = "/v1/text-to-speech/:voice_id/stream-input";
    const MULTI_CONTEXT_WS_PATH: &str = "/v1/text-to-speech/:voice_id/multi-stream-input";

    /// This API provides real-time text-to-speech conversion using WebSockets.
    /// This allows you to send a text message and receive audio data back in real-time.
    ///
    ///  # Example
    ///
    /// ```no_run
    /// use async_stream::stream;
    /// use elevenlabs_rs::endpoints::genai::tts::ws::*;
    /// use elevenlabs_rs::utils::{stream_audio, text_chunker};
    /// use elevenlabs_rs::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///
    ///     let text_stream = stream! {
    ///         yield "Mad Hatter: 'Am I going mad?'".to_string();
    ///         yield "Alice: 'Yes, you're entirely bonkers.'".into();
    ///         yield "But I'll tell you a secret. All the best people are.'".into();
    ///     };
    ///
    ///     let text_stream = text_chunker(text_stream);
    ///
    ///
    ///     let body = WebSocketTTSBody::new(BOSMessage::default(), text_stream)
    ///         .with_flush();
    ///
    ///     let endpoint = WebSocketTTS::new(DefaultVoice::Alice, body);
    ///
    ///     let client = ElevenLabsClient::from_env()?;
    ///     let mut session = client.connect_text_to_speech(endpoint).await?;
    ///
    ///     stream_audio(session.by_ref().map(|r| r?.audio_as_bytes())).await?;
    ///     session.close().await?;
    ///     let _report = session.join().await;
    ///
    ///     Ok(())
    /// }
    /// ```
    /// See [Text To Speech Stream API reference](https://elevenlabs.io/docs/api-reference/text-to-speech/websockets)
    pub struct WebSocketTTS<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub(crate) voice_id: String,
        pub(crate) body: WebSocketTTSBody<S>,
        pub(crate) query: Option<TTSWebSocketQuery>,
        #[cfg(test)]
        pub(crate) base_url: String,
    }

    impl<S> WebSocketTTS<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub fn new(voice_id: impl Into<String>, body: WebSocketTTSBody<S>) -> Self {
            WebSocketTTS {
                voice_id: voice_id.into(),
                body,
                query: None,
                #[cfg(test)]
                base_url: WS_BASE_URL.to_owned(),
            }
        }
        pub fn with_query(mut self, query: TTSWebSocketQuery) -> Self {
            self.query = Some(query);
            self
        }
        #[cfg(test)]
        pub(crate) fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
            self.base_url = base_url.into();
            self
        }
        pub(crate) fn url(&self) -> Result<String> {
            let path_params = [(":voice_id", self.voice_id.as_str())];
            let query_params = self.query.as_ref().into_iter().flat_map(|query| {
                query
                    .params
                    .iter()
                    .map(|(name, value)| (*name, value.as_str()))
            });

            #[cfg(test)]
            let base_url = self.base_url.as_str();
            #[cfg(not(test))]
            let base_url = WS_BASE_URL;

            crate::ws::websocket_url(base_url, WS_PATH, path_params, query_params)
        }

        pub(crate) fn auth(&self) -> crate::ws::WebSocketAuth {
            crate::ws::WebSocketAuth::None
        }

        pub(crate) fn should_inject_bos_api_key(&self) -> bool {
            let query_has_auth = self
                .query
                .as_ref()
                .is_some_and(TTSWebSocketQuery::uses_auth);
            !query_has_auth
                && self.body.bos_message.authorization.is_none()
                && self.body.bos_message.xi_api_key.is_none()
        }
    }

    impl<S> crate::ws::sealed::Sealed for WebSocketTTS<S> where S: Stream<Item = String> + Send + 'static
    {}

    impl<S> crate::ws::WebSocketEndpoint for WebSocketTTS<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        type Codec = crate::ws::JsonTextCodec<WebSocketTTSInput, WebSocketTTSResponse>;
        type InputStream = Pin<Box<dyn Stream<Item = Result<WebSocketTTSInput>> + Send>>;

        fn url(&self) -> Result<String> {
            WebSocketTTS::url(self)
        }

        fn auth(&self) -> crate::ws::WebSocketAuth {
            WebSocketTTS::auth(self)
        }

        fn endpoint_name(&self) -> &'static str {
            "text_to_speech.websocket"
        }

        fn input_stream(mut self, api_key: &str) -> Result<Self::InputStream> {
            if self.should_inject_bos_api_key() {
                self.body.bos_message.xi_api_key = Some(api_key.to_owned());
            }

            let bos_message = self.body.bos_message;
            let text_stream = self.body.text_stream;
            let flush = self.body.flush;

            Ok(Box::pin(async_stream::try_stream! {
                yield WebSocketTTSInput::Bos(bos_message);

                futures_util::pin_mut!(text_stream);
                while let Some(chunk) = text_stream.next().await {
                    yield WebSocketTTSInput::Text(WebSocketTextMessage::new(chunk));
                }

                if flush {
                    yield WebSocketTTSInput::Text(WebSocketTextMessage::flush());
                }

                yield WebSocketTTSInput::Text(WebSocketTextMessage::end_of_sequence());
            }))
        }
    }

    pub struct WebSocketTTSBody<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub bos_message: BOSMessage,
        pub text_stream: S,
        pub flush: bool,
    }

    impl<S> WebSocketTTSBody<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub fn new(bos_message: BOSMessage, text_stream: S) -> Self {
            WebSocketTTSBody {
                bos_message,
                text_stream,
                flush: false,
            }
        }

        pub fn with_flush(mut self) -> Self {
            self.flush = true;
            self
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct TTSWebSocketQuery {
        params: QueryValues,
    }

    impl TTSWebSocketQuery {
        pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
            self.params.push(("model_id", model_id.into()));
            self
        }

        pub fn with_authorization(mut self, authorization: impl Into<String>) -> Self {
            self.params.push(("authorization", authorization.into()));
            self
        }

        pub fn with_single_use_token(mut self, single_use_token: impl Into<String>) -> Self {
            self.params
                .push(("single_use_token", single_use_token.into()));
            self
        }

        pub fn with_language_code(mut self, language_code: impl Into<String>) -> Self {
            self.params.push(("language_code", language_code.into()));
            self
        }

        pub fn with_logging(mut self, enable_logging: bool) -> Self {
            self.params
                .push(("enable_logging", enable_logging.to_string()));
            self
        }

        pub fn with_ssml_parsing(mut self, enable_ssml_parsing: bool) -> Self {
            self.params
                .push(("enable_ssml_parsing", enable_ssml_parsing.to_string()));
            self
        }

        pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
            self.params
                .push(("output_format", output_format.to_string()));
            self
        }

        pub fn with_inactivity_timeout(mut self, timeout: f32) -> Self {
            self.params
                .push(("inactivity_timeout", timeout.to_string()));
            self
        }

        pub fn with_auto_mode(mut self, auto_mode: bool) -> Self {
            self.params.push(("auto_mode", auto_mode.to_string()));
            self
        }

        pub fn with_sync_alignment(mut self, sync_alignment: bool) -> Self {
            self.params
                .push(("sync_alignment", sync_alignment.to_string()));
            self
        }

        pub fn with_text_normalization(mut self, normalization: Normalization) -> Self {
            self.params.push((
                "apply_text_normalization",
                normalization.as_str().to_owned(),
            ));
            self
        }

        pub fn with_seed(mut self, seed: u32) -> Self {
            self.params.push(("seed", seed.to_string()));
            self
        }

        pub(crate) fn uses_auth(&self) -> bool {
            self.params
                .iter()
                .any(|(name, _)| matches!(*name, "authorization" | "single_use_token"))
        }
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct BOSMessage {
        pub text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub xi_api_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub voice_settings: Option<VoiceSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub authorization: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub generation_config: Option<GenerationConfig>,
    }
    impl BOSMessage {
        pub fn with_api_key(mut self, api_key: &str) -> Self {
            self.xi_api_key = Some(api_key.to_string());
            self
        }
        pub fn with_voice_settings(mut self, voice_settings: VoiceSettings) -> Self {
            self.voice_settings = Some(voice_settings);
            self
        }
        pub fn with_authorization(mut self, authorisation: &str) -> Self {
            self.authorization = Some(format!("Bearer {}", authorisation));
            self
        }
        pub fn with_generation_config(mut self, generation_config: [usize; 4]) -> Self {
            self.generation_config = Some(GenerationConfig::new(generation_config));
            self
        }
        pub fn to_message(&self) -> Result<Message> {
            let json = serde_json::to_string(&self)?;
            Ok(Message::Text(json.into()))
        }
    }

    impl Default for BOSMessage {
        fn default() -> Self {
            BOSMessage {
                text: " ".to_string(),
                xi_api_key: None,
                voice_settings: None,
                authorization: None,
                generation_config: None,
            }
        }
    }

    /// Multi-context realtime text-to-speech over a single WebSocket.
    ///
    /// Each outbound message belongs to a `context_id`. A single socket can
    /// keep several contexts alive at once and the server includes the context
    /// id on each audio response.
    pub struct MultiContextWebSocketTTS<S>
    where
        S: Stream<Item = MultiContextTTSInput> + Send + 'static,
    {
        pub(crate) voice_id: String,
        pub(crate) input_stream: S,
        pub(crate) query: Option<TTSWebSocketQuery>,
        #[cfg(test)]
        pub(crate) base_url: String,
    }

    impl<S> MultiContextWebSocketTTS<S>
    where
        S: Stream<Item = MultiContextTTSInput> + Send + 'static,
    {
        pub fn new(voice_id: impl Into<String>, input_stream: S) -> Self {
            Self {
                voice_id: voice_id.into(),
                input_stream,
                query: None,
                #[cfg(test)]
                base_url: WS_BASE_URL.to_owned(),
            }
        }

        pub fn with_query(mut self, query: TTSWebSocketQuery) -> Self {
            self.query = Some(query);
            self
        }

        #[cfg(test)]
        pub(crate) fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
            self.base_url = base_url.into();
            self
        }

        pub(crate) fn url(&self) -> Result<String> {
            let path_params = [(":voice_id", self.voice_id.as_str())];
            let query_params = self.query.as_ref().into_iter().flat_map(|query| {
                query
                    .params
                    .iter()
                    .map(|(name, value)| (*name, value.as_str()))
            });

            #[cfg(test)]
            let base_url = self.base_url.as_str();
            #[cfg(not(test))]
            let base_url = WS_BASE_URL;

            crate::ws::websocket_url(base_url, MULTI_CONTEXT_WS_PATH, path_params, query_params)
        }

        pub(crate) fn auth(&self) -> crate::ws::WebSocketAuth {
            if self
                .query
                .as_ref()
                .is_some_and(TTSWebSocketQuery::uses_auth)
            {
                crate::ws::WebSocketAuth::None
            } else {
                crate::ws::WebSocketAuth::XiApiKeyHeader
            }
        }
    }

    impl<S> crate::ws::sealed::Sealed for MultiContextWebSocketTTS<S> where
        S: Stream<Item = MultiContextTTSInput> + Send + 'static
    {
    }

    impl<S> crate::ws::WebSocketEndpoint for MultiContextWebSocketTTS<S>
    where
        S: Stream<Item = MultiContextTTSInput> + Send + 'static,
    {
        type Codec = crate::ws::JsonTextCodec<MultiContextTTSInput, MultiContextTTSResponse>;
        type InputStream = Pin<Box<dyn Stream<Item = Result<MultiContextTTSInput>> + Send>>;

        fn url(&self) -> Result<String> {
            MultiContextWebSocketTTS::url(self)
        }

        fn auth(&self) -> crate::ws::WebSocketAuth {
            MultiContextWebSocketTTS::auth(self)
        }

        fn endpoint_name(&self) -> &'static str {
            "text_to_speech.multi_context_websocket"
        }

        fn input_stream(self, _api_key: &str) -> Result<Self::InputStream> {
            let input_stream = self.input_stream;

            Ok(Box::pin(async_stream::try_stream! {
                futures_util::pin_mut!(input_stream);
                while let Some(input) = input_stream.next().await {
                    yield input;
                }
            }))
        }
    }

    #[derive(Clone, Debug, Default, Serialize)]
    pub struct MultiContextTTSInput {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub context_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub voice_settings: Option<VoiceSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub generation_config: Option<GenerationConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub flush: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub close_context: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub close_socket: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub keep_context_alive: Option<bool>,
    }

    impl MultiContextTTSInput {
        pub fn text(context_id: impl Into<String>, text: impl Into<String>) -> Self {
            Self {
                text: Some(text.into()),
                context_id: Some(context_id.into()),
                ..Default::default()
            }
        }

        pub fn start_context(context_id: impl Into<String>) -> Self {
            Self::text(context_id, " ")
        }

        pub fn flush(context_id: impl Into<String>) -> Self {
            Self {
                context_id: Some(context_id.into()),
                flush: Some(true),
                ..Default::default()
            }
        }

        pub fn close_context(context_id: impl Into<String>) -> Self {
            Self {
                context_id: Some(context_id.into()),
                close_context: Some(true),
                ..Default::default()
            }
        }

        pub fn keep_context_alive(context_id: impl Into<String>) -> Self {
            Self {
                context_id: Some(context_id.into()),
                keep_context_alive: Some(true),
                ..Default::default()
            }
        }

        pub fn close_socket() -> Self {
            Self {
                close_socket: Some(true),
                ..Default::default()
            }
        }

        pub fn with_voice_settings(mut self, voice_settings: VoiceSettings) -> Self {
            self.voice_settings = Some(voice_settings);
            self
        }

        pub fn with_generation_config(mut self, generation_config: [usize; 4]) -> Self {
            self.generation_config = Some(GenerationConfig::new(generation_config));
            self
        }

        pub fn with_flush(mut self) -> Self {
            self.flush = Some(true);
            self
        }
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct GenerationConfig {
        chunk_length_schedule: [usize; 4],
    }

    impl GenerationConfig {
        pub fn new(chunk_length_schedule: [usize; 4]) -> Self {
            Self {
                chunk_length_schedule,
            }
        }

        pub fn chunk_length_schedule(&self) -> [usize; 4] {
            self.chunk_length_schedule
        }
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(untagged)]
    pub(crate) enum WebSocketTTSInput {
        Bos(BOSMessage),
        Text(WebSocketTextMessage),
    }

    #[derive(Clone, Debug, Serialize)]
    pub(crate) struct WebSocketTextMessage {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        flush: Option<bool>,
    }

    impl WebSocketTextMessage {
        pub(crate) fn new(text: impl Into<String>) -> Self {
            Self {
                text: text.into(),
                flush: None,
            }
        }

        pub(crate) fn flush() -> Self {
            Self {
                text: " ".to_owned(),
                flush: Some(true),
            }
        }

        pub(crate) fn end_of_sequence() -> Self {
            Self {
                text: String::new(),
                flush: None,
            }
        }
    }

    #[derive(Clone, Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketTTSResponse {
        pub audio: Option<String>,
        pub is_final: Option<bool>,
        pub normalized_alignment: Option<WebSocketAlignment>,
        pub alignment: Option<WebSocketAlignment>,
        pub code: Option<WebSocketTTSErrorCode>,
        pub error: Option<String>,
        pub message: Option<String>,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    impl WebSocketTTSResponse {
        pub fn audio_as_bytes(&self) -> Result<Bytes> {
            if self.is_final() {
                return Ok(Bytes::new());
            }
            if let Some(audio_b64) = &self.audio {
                return Ok(Bytes::from(general_purpose::STANDARD.decode(audio_b64)?));
            }
            Ok(Bytes::new())
        }

        pub fn is_final(&self) -> bool {
            self.is_final.unwrap_or_default()
        }

        pub fn audio_base64(&self) -> Option<&str> {
            self.audio.as_deref()
        }

        pub fn has_audio(&self) -> bool {
            self.audio_base64().is_some_and(|audio| !audio.is_empty()) && !self.is_final()
        }

        pub fn is_error(&self) -> bool {
            self.error.is_some() || self.message.is_some() || self.code.is_some()
        }

        pub fn error_name(&self) -> Option<&str> {
            self.error.as_deref()
        }

        pub fn error_message(&self) -> Option<&str> {
            self.message.as_deref()
        }
    }

    #[derive(Clone, Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MultiContextTTSResponse {
        pub audio: Option<String>,
        pub is_final: Option<bool>,
        pub context_id: Option<String>,
        pub normalized_alignment: Option<WebSocketAlignment>,
        pub alignment: Option<WebSocketAlignment>,
        pub code: Option<WebSocketTTSErrorCode>,
        pub error: Option<String>,
        pub message: Option<String>,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    impl MultiContextTTSResponse {
        pub fn audio_as_bytes(&self) -> Result<Bytes> {
            if self.is_final() {
                return Ok(Bytes::new());
            }
            if let Some(audio_b64) = &self.audio {
                return Ok(Bytes::from(general_purpose::STANDARD.decode(audio_b64)?));
            }
            Ok(Bytes::new())
        }

        pub fn is_final(&self) -> bool {
            self.is_final.unwrap_or_default()
        }

        pub fn context_id(&self) -> Option<&str> {
            self.context_id.as_deref()
        }

        pub fn audio_base64(&self) -> Option<&str> {
            self.audio.as_deref()
        }

        pub fn has_audio(&self) -> bool {
            self.audio_base64().is_some_and(|audio| !audio.is_empty()) && !self.is_final()
        }

        pub fn is_error(&self) -> bool {
            self.error.is_some() || self.message.is_some() || self.code.is_some()
        }

        pub fn error_name(&self) -> Option<&str> {
            self.error.as_deref()
        }

        pub fn error_message(&self) -> Option<&str> {
            self.message.as_deref()
        }
    }

    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    #[serde(untagged)]
    pub enum WebSocketTTSErrorCode {
        Number(u16),
        Text(String),
    }

    impl std::fmt::Display for WebSocketTTSErrorCode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Number(code) => write!(f, "{code}"),
                Self::Text(code) => f.write_str(code),
            }
        }
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketAlignment {
        pub char_start_times_ms: Vec<f32>,
        pub char_durations_ms: Vec<f32>,
        pub chars: Vec<String>,
        #[serde(flatten)]
        pub extra: Map<String, Value>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::{json, Value};

        #[test]
        fn text_chunk_message_serializes_with_json_escaping() {
            let text = "quotes \" backslash \\ newline \n snowman \u{2603}".to_string();
            let message = WebSocketTextMessage::new(text.clone());
            let encoded = serde_json::to_string(&message).unwrap();

            let value: Value = serde_json::from_str(&encoded).unwrap();
            assert_eq!(value, json!({ "text": text }));
        }

        #[test]
        fn websocket_control_text_messages_use_same_serializer() {
            let flush = serde_json::to_string(&WebSocketTextMessage::flush()).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&flush).unwrap(),
                json!({ "text": " ", "flush": true })
            );

            let eos = serde_json::to_string(&WebSocketTextMessage::end_of_sequence()).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&eos).unwrap(),
                json!({ "text": "" })
            );
        }

        #[test]
        fn websocket_url_encodes_path_and_query_values() {
            let endpoint = WebSocketTTS::new(
                "voice id/with slash",
                WebSocketTTSBody::new(BOSMessage::default(), futures_util::stream::empty()),
            )
            .with_query(
                TTSWebSocketQuery::default()
                    .with_model_id("model with spaces")
                    .with_language_code("en-US")
                    .with_single_use_token("token/with?chars&symbols"),
            );

            let url = endpoint.url().unwrap();
            let parsed = Url::parse(&url).unwrap();

            assert_eq!(
                parsed.path(),
                "/v1/text-to-speech/voice%20id%2Fwith%20slash/stream-input"
            );
            let query_pairs: Vec<_> = parsed.query_pairs().collect();
            assert!(query_pairs.contains(&("model_id".into(), "model with spaces".into())));
            assert!(query_pairs.contains(&("language_code".into(), "en-US".into())));
            assert!(query_pairs
                .contains(&("single_use_token".into(), "token/with?chars&symbols".into())));
        }

        #[test]
        fn websocket_bos_api_key_injection_respects_explicit_auth() {
            let endpoint = WebSocketTTS::new(
                "voice-id",
                WebSocketTTSBody::new(BOSMessage::default(), futures_util::stream::empty()),
            );
            assert!(endpoint.should_inject_bos_api_key());

            let endpoint = endpoint
                .with_query(TTSWebSocketQuery::default().with_single_use_token("single-use-token"));
            assert!(!endpoint.should_inject_bos_api_key());

            let endpoint = WebSocketTTS::new(
                "voice-id",
                WebSocketTTSBody::new(
                    BOSMessage::default().with_authorization("bearer-token"),
                    futures_util::stream::empty(),
                ),
            );
            assert!(!endpoint.should_inject_bos_api_key());
        }

        #[test]
        fn multi_context_tts_inputs_serialize_to_api_shapes() {
            let start = MultiContextTTSInput::start_context("conv_1")
                .with_voice_settings(
                    VoiceSettings::default()
                        .with_stability(0.5)
                        .with_similarity_boost(0.75),
                )
                .with_generation_config([120, 160, 250, 290]);

            assert_eq!(
                serde_json::to_value(start).unwrap(),
                json!({
                    "text": " ",
                    "context_id": "conv_1",
                    "voice_settings": {
                        "similarity_boost": 0.75,
                        "stability": 0.5
                    },
                    "generation_config": {
                        "chunk_length_schedule": [120, 160, 250, 290]
                    }
                })
            );

            assert_eq!(
                serde_json::to_value(MultiContextTTSInput::text("conv_1", "Hello from one. "))
                    .unwrap(),
                json!({ "text": "Hello from one. ", "context_id": "conv_1" })
            );
            assert_eq!(
                serde_json::to_value(MultiContextTTSInput::flush("conv_1")).unwrap(),
                json!({ "context_id": "conv_1", "flush": true })
            );
            assert_eq!(
                serde_json::to_value(MultiContextTTSInput::close_context("conv_1")).unwrap(),
                json!({ "context_id": "conv_1", "close_context": true })
            );
            assert_eq!(
                serde_json::to_value(MultiContextTTSInput::close_socket()).unwrap(),
                json!({ "close_socket": true })
            );
        }

        #[test]
        fn multi_context_websocket_url_and_auth_follow_api_shape() {
            let endpoint = MultiContextWebSocketTTS::new(
                "voice id/with slash",
                futures_util::stream::empty::<MultiContextTTSInput>(),
            )
            .with_base_url("wss://example.test")
            .with_query(
                TTSWebSocketQuery::default()
                    .with_model_id("model with spaces")
                    .with_single_use_token("token/with?chars&symbols")
                    .with_sync_alignment(true)
                    .with_text_normalization(Normalization::On)
                    .with_seed(42),
            );

            let url = endpoint.url().unwrap();
            let parsed = Url::parse(&url).unwrap();

            assert_eq!(
                parsed.path(),
                "/v1/text-to-speech/voice%20id%2Fwith%20slash/multi-stream-input"
            );
            let query_pairs: Vec<_> = parsed.query_pairs().collect();
            assert!(query_pairs.contains(&("model_id".into(), "model with spaces".into())));
            assert!(query_pairs
                .contains(&("single_use_token".into(), "token/with?chars&symbols".into())));
            assert!(query_pairs.contains(&("sync_alignment".into(), "true".into())));
            assert!(query_pairs.contains(&("apply_text_normalization".into(), "on".into())));
            assert!(query_pairs.contains(&("seed".into(), "42".into())));
            assert!(matches!(endpoint.auth(), crate::ws::WebSocketAuth::None));

            let endpoint = MultiContextWebSocketTTS::new(
                "voice-id",
                futures_util::stream::empty::<MultiContextTTSInput>(),
            );
            assert!(matches!(
                endpoint.auth(),
                crate::ws::WebSocketAuth::XiApiKeyHeader
            ));
        }

        #[test]
        fn multi_context_tts_response_exposes_context_audio_and_errors() {
            let response: MultiContextTTSResponse = serde_json::from_value(json!({
                "audio": "aGVsbG8=",
                "isFinal": false,
                "contextId": "conv_1",
                "futureField": "preserved"
            }))
            .unwrap();

            assert_eq!(response.context_id(), Some("conv_1"));
            assert!(response.has_audio());
            assert_eq!(response.audio_as_bytes().unwrap().as_ref(), b"hello");
            assert_eq!(response.extra.get("futureField"), Some(&json!("preserved")));

            let response: MultiContextTTSResponse = serde_json::from_value(json!({
                "contextId": "conv_1",
                "code": 1008,
                "error": "invalid_api_key",
                "message": "Invalid API key"
            }))
            .unwrap();

            assert!(response.is_error());
            assert_eq!(response.context_id(), Some("conv_1"));
            assert_eq!(response.code, Some(WebSocketTTSErrorCode::Number(1008)));
            assert_eq!(response.error_name(), Some("invalid_api_key"));
            assert_eq!(response.error_message(), Some("Invalid API key"));
        }

        #[test]
        fn websocket_tts_audio_decodes_when_is_final_is_false() {
            let response: WebSocketTTSResponse = serde_json::from_value(json!({
                "audio": "aGVsbG8=",
                "isFinal": false,
                "futureField": "preserved"
            }))
            .unwrap();

            assert!(!response.is_final());
            assert!(response.has_audio());
            assert_eq!(response.audio_as_bytes().unwrap().as_ref(), b"hello");
            assert_eq!(response.extra.get("futureField"), Some(&json!("preserved")));
        }

        #[test]
        fn websocket_tts_final_response_has_no_audio_bytes() {
            let response: WebSocketTTSResponse = serde_json::from_value(json!({
                "audio": "aGVsbG8=",
                "isFinal": true
            }))
            .unwrap();

            assert!(response.is_final());
            assert_eq!(response.audio_as_bytes().unwrap(), Bytes::new());
        }

        #[test]
        fn websocket_tts_error_payloads_are_exposed() {
            let response: WebSocketTTSResponse = serde_json::from_value(json!({
                "code": 1008,
                "error": "invalid_api_key",
                "message": "Invalid API key"
            }))
            .unwrap();

            assert!(response.is_error());
            assert_eq!(response.code, Some(WebSocketTTSErrorCode::Number(1008)));
            assert_eq!(response.error_name(), Some("invalid_api_key"));
            assert_eq!(response.error_message(), Some("Invalid API key"));
            assert!(response.extra.is_empty());

            let response: WebSocketTTSResponse = serde_json::from_value(json!({
                "code": "future_code",
                "message": "Future error shape"
            }))
            .unwrap();

            assert!(response.is_error());
            assert_eq!(
                response.code,
                Some(WebSocketTTSErrorCode::Text("future_code".to_string()))
            );
            assert_eq!(response.error_message(), Some("Future error shape"));
        }
    }
}
