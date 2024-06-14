use super::*;
use crate::client::BASE_URL;
use crate::endpoints::voice::{VoiceID, VoiceSettings};
use async_stream::try_stream;
use base64::{
    engine::{self, general_purpose},
    Engine as _,
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde::Serialize;
use std::pin::Pin;

const TTS_PATH: &str = "/v1/text-to-speech";
pub const STREAM_PATH: &str = "/stream";
const WITH_TIMESTAMP_PATH: &str = "/with-timestamps";
const LATENCY_QUERY: &str = "optimize_streaming_latency";
const OUTPUT_FORMAT_QUERY: &str = "output_format";
const ENABLE_LOGGING_QUERY: &str = "enable_logging";
const ENABLE_SSML_PARSING_QUERY: &str = "enable_ssml_parsing";

// TODO: Timestamps Endpoints
// TODO: Websocket Endpoint

#[derive(Clone, Debug)]
pub struct TextToSpeech {
    voice_id: VoiceID,
    text_to_speech_body: TextToSpeechBody,
    speech_query: Option<SpeechQuery>,
}

impl Endpoint for TextToSpeech {
    type ResponseBody = Bytes;

    fn method(&self) -> Method {
        Method::POST
    }
    fn json_request_body(&self) -> Option<Result<Value>> {
        Some(serde_json::to_value(&self.text_to_speech_body).map_err(Into::into))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", TTS_PATH, self.voice_id.0));
        url.set_query(self.any_query().as_deref());
        // url.set_query(self.speech_query(self.speech_query));
        url
    }
}

impl TextToSpeech {
    pub fn new(voice_id: &str, text_to_speech_body: TextToSpeechBody) -> Self {
        let voice_id = VoiceID::from(voice_id);
        TextToSpeech {
            voice_id,
            text_to_speech_body,
            speech_query: None,
        }
    }
    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
        self.speech_query = Some(speech_query);
        self
    }

    fn any_query(&self) -> Option<String> {
        if let Some(query) = &self.speech_query {
            Some(query.to_string())
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct TextToSpeechBody {
    text: String,
    model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice_settings: Option<VoiceSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pronunciation_dictionary_locators: Option<Vec<PronunciationDictionaryLocator>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_text_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_text_ids: Option<Vec<String>>,
}

impl TextToSpeechBody {
    pub fn new(text: &str, model_id: &str) -> Self {
        TextToSpeechBody {
            text: text.to_string(),
            model_id: model_id.to_string(),
            ..Default::default()
        }
    }
    pub fn with_pronunciation_dict(mut self, pronunciation_id: &str, version_id: &str) -> Self {
        if let Some(dictionary) = &mut self.pronunciation_dictionary_locators {
            dictionary.push(PronunciationDictionaryLocator {
                pronunciation_dictionary_id: pronunciation_id.to_string(),
                version_id: version_id.to_string(),
            });
        } else {
            self.pronunciation_dictionary_locators = Some(vec![PronunciationDictionaryLocator {
                pronunciation_dictionary_id: pronunciation_id.to_string(),
                version_id: version_id.to_string(),
            }]);
        }
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
    pub fn with_previous_text_ids(mut self, previous_text_ids: Vec<String>) -> Self {
        self.previous_text_ids = Some(previous_text_ids);
        self
    }
    pub fn with_next_text_ids(mut self, next_text_ids: Vec<String>) -> Self {
        self.next_text_ids = Some(next_text_ids);
        self
    }
}
#[derive(Clone, Debug, Serialize)]
struct PronunciationDictionaryLocator {
    pronunciation_dictionary_id: String,
    version_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct SpeechQuery {
    latency: Option<String>,
    output_format: Option<String>,
    enable_logging: Option<String>,
    enable_ssml_parsing: Option<String>,
}

impl SpeechQuery {
    pub fn new() -> Self {
        SpeechQuery::default()
    }
    pub fn with_latency(mut self, latency: Latency) -> Self {
        self.latency = Some(format!("{}={}", LATENCY_QUERY, latency as u8));
        self
    }
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.output_format = Some(format!(
            "{}={}",
            OUTPUT_FORMAT_QUERY,
            output_format.to_query()
        ));
        self
    }
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.enable_logging = Some(format!("{}={}", ENABLE_LOGGING_QUERY, enable_logging));
        self
    }
    pub fn with_ssml_parsing(mut self, enable_ssml_parsing: bool) -> Self {
        self.enable_ssml_parsing = Some(format!(
            "{}={}",
            ENABLE_SSML_PARSING_QUERY,
            enable_ssml_parsing
        ));
        self
    }

    pub fn to_string(&self) -> String {
        let mut query = String::new();

        if let Some(latency) = &self.latency {
            query.push_str(latency);
        }
        if let Some(output_format) = &self.output_format {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(output_format);
        }
        if let Some(enable_logging) = &self.enable_logging {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(enable_logging);
        }
        if let Some(enable_ssml_parsing) = &self.enable_ssml_parsing {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(enable_ssml_parsing);
        }
        query
    }
}

#[derive(Clone, Debug)]
pub enum Latency {
    /// Default latency
    None = 0,
    ///  normal latency optimizations (about 50% of possible latency improvement of option 3)
    Normal = 1,
    /// strong latency optimizations (about 75% of possible latency improvement of option 3)
    Strong = 2,
    /// max latency optimizations
    Max = 3,
    /// max latency optimizations, but also with text normalizer turned off for even more latency
    /// savings (the best latency, but can mispronounce e.g. numbers and dates)
    MaxBest = 4,
}

/// See Elevenlabs documentation on [supported output formats](https://help.elevenlabs.io/hc/en-us/articles/15754340124305-What-audio-formats-do-you-support).
#[derive(Clone, Debug)]
pub enum OutputFormat {
    Mp3_22050Hz32kbps,
    Mp3_44100Hz32kbps,
    Mp3_44100Hz64kbps,
    Mp3_44100Hz96kbps,
    Mp3_44100Hz192kbps,
    Pcm16000Hz,
    Pcm22050Hz,
    Pcm24000Hz,
    Pcm44100Hz,
    MuLaw8000Hz,
}
impl OutputFormat {
    fn to_query(&self) -> &str {
        match self {
            OutputFormat::Pcm16000Hz => "pcm_16000",
            OutputFormat::Pcm22050Hz => "pcm_22050",
            OutputFormat::Pcm24000Hz => "pcm_24000",
            OutputFormat::Pcm44100Hz => "pcm_44100",
            OutputFormat::Mp3_22050Hz32kbps => "mp3_22050_32",
            OutputFormat::Mp3_44100Hz32kbps => "mp3_44100_32",
            OutputFormat::Mp3_44100Hz64kbps => "mp3_44100_64",
            OutputFormat::Mp3_44100Hz96kbps => "mp3_44100_96",
            OutputFormat::Mp3_44100Hz192kbps => "mp3_44100_192",
            OutputFormat::MuLaw8000Hz => "ulaw_8000",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextToSpeechStream {
    voice_id: VoiceID,
    text_to_speech_body: TextToSpeechBody,
    speech_query: Option<SpeechQuery>,
}

impl TextToSpeechStream {
    pub fn new(voice_id: &str, text_to_speech_body: TextToSpeechBody) -> Self {
        let voice_id = VoiceID::from(voice_id);
        TextToSpeechStream {
            voice_id,
            text_to_speech_body,
            speech_query: None,
        }
    }
    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
        self.speech_query = Some(speech_query);
        self
    }
    fn any_query(&self) -> Option<String> {
        if let Some(query) = &self.speech_query {
            Some(query.to_string())
        } else {
            None
        }
    }
}

impl Endpoint for TextToSpeechStream {
    type ResponseBody = Pin<Box<dyn Stream<Item = Result<Bytes>>>>;

    fn method(&self) -> Method {
        Method::POST
    }
    fn json_request_body(&self) -> Option<Result<Value>> {
        Some(serde_json::to_value(&self.text_to_speech_body).map_err(Into::into))
    }

    async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream = stream.map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}{}", TTS_PATH, self.voice_id.0, STREAM_PATH));
        url
    }
}
/// Text to Speech with Timestamps
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::client::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::tts::*;
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let voice_id = "21m00Tcm4TlvDq8ikWAM";
///     let model_id = "eleven_multilingual_v2";
///     let txt = "To see a world in a grain of sand, and a heaven in a wild flower, \
///     hold infinity in the palm of your hand, and eternity in an hour.";
///     let body = TextToSpeechBody::new(txt, model_id);
///     let endpoint = TextToSpeechWithTimestamps::new(voice_id, body);
///     let resp = c.hit(endpoint).await?;
///     let mut timestamps = resp.iter();
///     for (char, (start_time, end_time)) in timestamps {
///         println!("{} = {} - {}", char, start_time, end_time);
///     }
///     let audio = resp.audio()?;
///     play(audio)?;
///     Ok(())
/// }
/// ```

#[derive(Clone, Debug)]
pub struct TextToSpeechWithTimestamps {
    voice_id: VoiceID,
    text_to_speech_body: TextToSpeechBody,
    speech_query: Option<SpeechQuery>,
}

impl TextToSpeechWithTimestamps {
    pub fn new(voice_id: &str, text_to_speech_body: TextToSpeechBody) -> Self {
        let voice_id = VoiceID::from(voice_id);
        TextToSpeechWithTimestamps {
            voice_id,
            text_to_speech_body,
            speech_query: None,
        }
    }
    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
        self.speech_query = Some(speech_query);
        self
    }
    fn any_query(&self) -> Option<String> {
        if let Some(query) = &self.speech_query {
            Some(query.to_string())
        } else {
            None
        }
    }
}

impl Endpoint for TextToSpeechWithTimestamps {
    type ResponseBody = TextToSpeechWithTimestampsResponse;

    fn method(&self) -> Method {
        Method::POST
    }
    fn json_request_body(&self) -> Option<Result<Value>> {
        Some(serde_json::to_value(&self.text_to_speech_body).map_err(Into::into))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            TTS_PATH, self.voice_id.0, WITH_TIMESTAMP_PATH
        ));
        url.set_query(self.any_query().as_deref());
        url
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TextToSpeechWithTimestampsResponse {
    alignment: Alignment,
    audio_base64: String,
    normalized_alignment: Alignment,
}

impl TextToSpeechWithTimestampsResponse {
    pub fn alignment(&self) -> &Alignment {
        &self.alignment
    }
    pub fn audio_base64(&self) -> &str {
        &self.audio_base64
    }
    pub fn audio(&self) -> Result<Bytes> {
        let decoded_audio_b64 = general_purpose::STANDARD.decode(&self.audio_base64())?;
        Ok(Bytes::from(decoded_audio_b64))
    }
    pub fn normalized_alignment(&self) -> &Alignment {
        &self.normalized_alignment
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, (&f32, &f32))> {
        self.alignment().characters().iter().zip(
            self.alignment
                .character_start_times_seconds
                .iter()
                .zip(self.alignment.character_end_times_seconds.iter()),
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Alignment {
    character_end_times_seconds: Vec<f32>,
    character_start_times_seconds: Vec<f32>,
    characters: Vec<String>,
}

impl Alignment {
    pub fn character_end_times_seconds(&self) -> &[f32] {
        &self.character_end_times_seconds
    }
    pub fn character_start_times_seconds(&self) -> &[f32] {
        &self.character_start_times_seconds
    }
    pub fn characters(&self) -> &[String] {
        &self.characters
    }
}

#[derive(Clone, Debug)]
pub struct TextToSpeechStreamWithTimestamps {
    voice_id: VoiceID,
    text_to_speech_body: TextToSpeechBody,
    speech_query: Option<SpeechQuery>,
}

impl TextToSpeechStreamWithTimestamps {
    pub fn new(voice_id: &str, text_to_speech_body: TextToSpeechBody) -> Self {
        let voice_id = VoiceID::from(voice_id);
        TextToSpeechStreamWithTimestamps {
            voice_id,
            text_to_speech_body,
            speech_query: None,
        }
    }
    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
        self.speech_query = Some(speech_query);
        self
    }
    fn any_query(&self) -> Option<String> {
        if let Some(query) = &self.speech_query {
            Some(query.to_string())
        } else {
            None
        }
    }
}

impl Endpoint for TextToSpeechStreamWithTimestamps {
    type ResponseBody = Pin<Box<dyn Stream<Item = Result<TextToSpeechWithTimestampsResponse>>>>;
    //type ResponseBody = AsyncStream<Result<Result<Pin<Box<Result<TextToSpeechWithTimestampsResponse>>>>>, U>;

    //type ResponseBody = impl Stream<Item = Result<TextToSpeechWithTimestampsResponse>>;
    //type ResponseBody = impl Stream<Item = Result<Value>>;

    fn method(&self) -> Method {
        Method::POST
    }
    fn json_request_body(&self) -> Option<Result<Value>> {
        Some(serde_json::to_value(&self.text_to_speech_body).map_err(Into::into))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream();
        let stream =
            stream.map(|r| r.map_err(Into::<Box<dyn std::error::Error + Send + Sync>>::into));

        Ok(Box::pin(try_stream! {
            for await value in stream {
                let bytes = value?;
                let chunk = std::str::from_utf8(&bytes)?;
                let json: TextToSpeechWithTimestampsResponse = serde_json::from_str(&chunk)?;
                yield json;
            }
        }))
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}{}",
            TTS_PATH, self.voice_id.0, STREAM_PATH, WITH_TIMESTAMP_PATH
        ));
        url.set_query(self.any_query().as_deref());
        url
    }
}

pub mod ws {
    use super::*;
    //use crate::client::Result;
    //use crate::endpoints::Deserialize;
    //use async_stream::try_stream;
    //use base64::engine::general_purpose;
    //use base64::Engine;
    //use bytes::Bytes;
    //use futures_util::{Stream, StreamExt};
    use crate::utils::stream_audio;
    use futures_util::stream::SplitStream;
    use serde::de::Unexpected::Str;
    use tokio::net::TcpStream;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

    const WS_BASE_URL: &str = "wss://api.elevenlabs.io";
    const WS_STREAM_PATH: &str = "/stream-input";
    const MODEL_ID_QUERY: &str = "model_id";
    

    #[derive(Clone, Debug)]
    pub struct WebSocketTTS<T>
        where T: IntoIterator<Item = String> + Send + Sync + 'static {
        path_params: WebSocketTTSPathParams,
        text_to_speech_body: WebSocketTTSBody<T>,
        speech_query: Option<SpeechQuery>,
    }

    impl<T> WebSocketTTS<T>
    where T: IntoIterator<Item = String> + Send + Sync + 'static {
        pub fn new(voice_id: &str, model_id: &str, text_to_speech_body: WebSocketTTSBody<T>) -> Self {
            let path_params = WebSocketTTSPathParams {
                voice_id: VoiceID::from(voice_id),
                model_id: model_id.to_string(),
            };
            WebSocketTTS {
                path_params,
                text_to_speech_body,
                speech_query: None,
            }
        }
        pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
            self.speech_query = Some(speech_query);
            self
        }
        pub fn url(&self) -> Url {
            let mut url = WS_BASE_URL.parse::<Url>().unwrap();
            url.set_path(&format!(
                "{}/{}{}",
                TTS_PATH, self.path_params.voice_id.0, WS_STREAM_PATH
            ));
            let mut query = String::new();
            if let Some(q) = &self.speech_query {
                query.push_str(&q.to_string());
                query.push('&');
                query.push_str(&format!("{}={}", MODEL_ID_QUERY, self.path_params.model_id));
            } else {
                query.push_str(&format!("?{}={}", MODEL_ID_QUERY, self.path_params.model_id));
            }
            url
        }
        //pub fn body(&self) -> &WebSocketTTSBody<'a, T> {
        //    &self.text_to_speech_body
        //}
        pub fn body(self) -> WebSocketTTSBody<T> {
            self.text_to_speech_body
        }
        pub fn initial_message(&self) -> &InitialMessage {
            self.text_to_speech_body.initial_message()
        }
        pub fn text(self) -> T {
            self.text_to_speech_body.text
        }
    }


     #[derive(Clone, Debug)]
    struct WebSocketTTSPathParams {
        voice_id: VoiceID,
        model_id: String,
    }
    #[derive(Clone, Debug, Serialize)]
    pub struct WebSocketTTSBody<T>
    where T: IntoIterator<Item = String> + Send + Sync + 'static {
        initial_message: InitialMessage,
        text: T,
        try_trigger_generation: Option<Vec<usize>>,
        flush: Option<bool>,
    }

    impl<T> WebSocketTTSBody<T>
    where T: IntoIterator<Item = String> + Send + Sync + 'static {
        pub fn new(initial_message: InitialMessage, text: T) -> Self {
            WebSocketTTSBody {
                initial_message,
                text,
                try_trigger_generation: None,
                flush: None,
            }
        }
        pub fn with_try_trigger_generation(mut self, try_trigger_generation: Vec<usize>) -> Self {
            self.try_trigger_generation = Some(try_trigger_generation);
            self
        }
        pub fn with_flush(mut self, flush: bool) -> Self {
            self.flush = Some(flush);
            self
        }
        pub fn initial_message(&self) -> &InitialMessage {
            &self.initial_message
        }
    }


    #[derive(Clone, Debug, Serialize)]
    pub struct InitialMessage {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        xi_api_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        voice_settings: Option<VoiceSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        authorization: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        generation_config: Option<GenerationConfig>,
    }
    impl InitialMessage {
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
                chunk_length_schedule: generation_config
            });
            self
        }
    }

    impl Default for InitialMessage {
        fn default() -> Self {
            InitialMessage {
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
        // TODO: find out if the docs just used four elements as an example, so this should be a Vec.
        chunk_length_schedule: [usize;4]
    }







    #[derive(Clone, Debug, Default,  Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketTTSResponse {
        audio: Option<String>,
        is_final: Option<bool>,
        normalized_alignment: Option<WebSocketAlignment>,
        alignment: Option<WebSocketAlignment>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketAlignment {
        char_start_times_ms: Vec<f32>,
        char_durations_ms: Vec<f32>,
        chars: Vec<String>,
    }

    // TODO: Make generic
    pub async fn stream_ws_audio(
        mut ws_reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> Result<()> {
        let audio_stream = try_stream! {
            let mut buf = String::new();
            for await msg in ws_reader {
                match msg? {
                    Message::Text(text) => {
                        buf.push_str(&text);
                        match serde_json::from_str::<WebSocketTTSResponse>(&buf) {
                            Ok(json) => {
                                if json.audio.is_some() {
                                    let decoded_audio_b64 = general_purpose::STANDARD.decode(&json.audio.unwrap())?;
                                    buf.clear();
                                    yield Bytes::from(decoded_audio_b64);
                                }
                            }
                            Err(e) => {
                                if e.is_eof() {
                                    continue
                                } else {
                                    // Other errors, clear the buffer
                                     eprintln!("Failed to parse JSON: {}", e);
                                     buf.clear();
                                }
                            }
                        }
                    }
                    Message::Close(close_msg) => {
                        if close_msg.is_some() {
                            println!("{:?}", close_msg.unwrap())
                        }
                    }
                    _ => panic!("unexpected websocket response from Elevenlabs")
                }
            }
        };
        stream_audio(audio_stream).await.expect("playing audio");
        Ok(())
    }
}
