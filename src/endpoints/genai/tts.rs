//! The text-to-speech endpoints
use super::*;
use crate::endpoints::admin::pronunciation::GetDictionariesResponse;
use crate::endpoints::ElevenLabsEndpoint;
use crate::shared::VoiceSettings;
use async_stream::try_stream;
use base64::{engine::general_purpose, Engine as _};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

/// Convert text to speech using Elevenlabs' library of over 3,000 voices across 32 languages.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let body = TextToSpeechBody::new("Look on my Works, ye Mighty, and despair!")
///        .with_model_id(Model::ElevenTurboV2);
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
    voice_id: VoiceID,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeech {
    pub fn new(voice_id: impl Into<VoiceID>, body: TextToSpeechBody) -> Self {
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

impl ElevenLabsEndpoint for TextToSpeech {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.as_path_param()]
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
    pronunciation_dictionary_locators: Option<DictionaryLocators>,
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
}

impl TryFrom<&TextToSpeechBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

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
        self.pronunciation_dictionary_locators = Some(locators);
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
/// let mut locators = DictionaryLocators::new();
/// locators.push(DictionaryLocator::default("id", "version_id"));
/// locators.push(DictionaryLocator::default("id", "version_id"));
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

impl From<GetDictionariesResponse> for DictionaryLocators {
    fn from(response: GetDictionariesResponse) -> Self {
        let mut locators = Self::default();
        response.into_iter().take(3).for_each(|dict| {
            locators.push(DictionaryLocator::new(&dict.id, &dict.latest_version_id));
        });
        locators
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DictionaryLocator {
    pronunciation_dictionary_id: String,
    version_id: String,
}

impl DictionaryLocator {
    pub fn new(dictionary_id: &str, version_id: &str) -> Self {
        DictionaryLocator {
            pronunciation_dictionary_id: dictionary_id.to_string(),
            version_id: version_id.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Normalization {
    Auto,
    On,
    Off,
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
/// use elevenlabs_rs::utils::stream_audio;
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let txt = "The art of progress is to preserve order amid change \
///        and to preserve change amid order.";
///
///     let body = TextToSpeechBody::new(txt).with_model_id(Model::ElevenTurboV2);
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
    voice_id: VoiceID,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeechStream {
    pub fn new(voice_id: impl Into<VoiceID>, body: TextToSpeechBody) -> Self {
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
impl ElevenLabsEndpoint for TextToSpeechStream {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id/stream";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToSpeechStreamResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.as_path_param()]
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
/// use elevenlabs_rs::*;
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
    voice_id: VoiceID,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeechWithTimestamps {
    pub fn new(voice_id: impl Into<VoiceID>, body: TextToSpeechBody) -> Self {
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

impl ElevenLabsEndpoint for TextToSpeechWithTimestamps {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id/with-timestamps";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToSpeechWithTimestampsResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.as_path_param()]
    }
    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
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
    type Item = (&'a String, (f32, f32));

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
    pub fn iter(&self) -> Timestamps {
        Timestamps::new(self)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Alignment {
    pub character_end_times_seconds: Vec<f32>,
    pub character_start_times_seconds: Vec<f32>,
    pub characters: Vec<String>,
}

/// Stream speech from text with precise character-level timing information for audio-text synchronization.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let voice_id = LegacyVoice::Rachel;
///     let model_id = Model::ElevenTurboV2;
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
    voice_id: VoiceID,
    body: TextToSpeechBody,
    query: Option<TextToSpeechQuery>,
}

impl TextToSpeechStreamWithTimestamps {
    pub fn new(voice_id: impl Into<VoiceID>, body: TextToSpeechBody) -> Self {
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
    Pin<Box<dyn Stream<Item = Result<TextToSpeechWithTimestampsResponse>>>>;

impl ElevenLabsEndpoint for TextToSpeechStreamWithTimestamps {
    const PATH: &'static str = "/v1/text-to-speech/:voice_id/stream/with-timestamps";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToSpeechStreamWithTimestampsResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.voice_id.as_path_param()]
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
fn stream_chunks_to_json(
    stream: impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static,
) -> impl Stream<Item = Result<TextToSpeechWithTimestampsResponse>> {
    try_stream! {
        let mut buffer = String::new();

        for await chunk in stream {
            let chunk = chunk?;
            buffer.push_str(std::str::from_utf8(&chunk)?);

            if chunk.ends_with(b"\n\n") {
                let response: TextToSpeechWithTimestampsResponse =
                    serde_json::from_str(&buffer)?;
                yield response;
                buffer.clear();
            }
        }
    }
}

#[cfg(feature = "ws_tts")]
pub mod ws {
    //! Websocket Text to Speech endpoints

    use super::*;
    use tokio_tungstenite::tungstenite::Message;

    const WS_BASE_URL: &str = "wss://api.elevenlabs.io";
    const WS_PATH: &str = "/v1/text-to-speech/:voice_id/stream-input";

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
    ///     let stream = client.hit_ws(endpoint).await?;
    ///
    ///     stream_audio(stream.map(|r| r?.audio_as_bytes())).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    /// See [Text To Speech Stream API reference](https://elevenlabs.io/docs/api-reference/text-to-speech/websockets)
    pub struct WebSocketTTS<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub(crate) voice_id: VoiceID,
        pub(crate) body: WebSocketTTSBody<S>,
        pub(crate) query: Option<TTSWebSocketQuery>,
    }

    impl<S> WebSocketTTS<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub fn new(voice_id: impl Into<VoiceID>, body: WebSocketTTSBody<S>) -> Self {
            WebSocketTTS {
                voice_id: voice_id.into(),
                body,
                query: None,
            }
        }
        pub fn with_query(mut self, query: TTSWebSocketQuery) -> Self {
            self.query = Some(query);
            self
        }
        pub(crate) fn url(&self) -> String {
            let mut base_url = WS_BASE_URL.parse::<Url>().unwrap();

            let mut path = WS_PATH.to_string();

            path = path.replace(":voice_id", &self.voice_id._inner);

            base_url.set_path(&path);

            if let Some(query_values) = &self.query {
                let query_string = query_values
                    .params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<String>>()
                    .join("&");

                base_url.set_query(Some(&query_string));
            }
            base_url.to_string()
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
        pub fn with_model_id(mut self, model_id: impl Into<ModelID>) -> Self {
            self.params.push(("model_id", model_id.into()._inner));
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
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct BOSMessage {
        pub text: String,
        pub xi_api_key: Option<String>,
        pub voice_settings: Option<VoiceSettings>,
        pub authorization: Option<String>,
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
            self.generation_config = Some(GenerationConfig {
                chunk_length_schedule: generation_config,
            });
            self
        }
        pub fn to_message(&self) -> Result<Message> {
            let json = serde_json::to_string(&self)?;
            Ok(Message::Text(json))
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

    #[derive(Clone, Debug, Serialize)]
    struct GenerationConfig {
        chunk_length_schedule: [usize; 4],
    }

    pub(crate) trait TextChunkMessage {
        fn to_message(&self) -> Result<Message>;
    }

    impl TextChunkMessage for String {
        fn to_message(&self) -> Result<Message> {
            let json = format!("{{\"text\":\"{}\"}}", self);
            Ok(Message::Text(json))
        }
    }

    #[derive(Clone, Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketTTSResponse {
        pub audio: Option<String>,
        pub is_final: Option<bool>,
        pub normalized_alignment: Option<WebSocketAlignment>,
        pub alignment: Option<WebSocketAlignment>,
    }

    impl WebSocketTTSResponse {
        pub fn audio_as_bytes(&self) -> Result<Bytes> {
            if self.is_final.is_some() {
                return Ok(Bytes::new());
            }
            if let Some(audio_b64) = &self.audio {
                return Ok(Bytes::from(general_purpose::STANDARD.decode(audio_b64)?));
            }
            Ok(Bytes::new())
        }
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketAlignment {
        pub char_start_times_ms: Vec<f32>,
        pub char_durations_ms: Vec<f32>,
        pub chars: Vec<String>,
    }
}
