#![allow(dead_code)]
//! The text-to-speech endpoints
use super::*;
use crate::endpoints::voice::VoiceSettings;
use async_stream::try_stream;
use base64::{engine::general_purpose, Engine as _};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

const TTS_PATH: &str = "/v1/text-to-speech";
const WITH_TIMESTAMP_PATH: &str = "/with-timestamps";
const LATENCY_QUERY: &str = "optimize_streaming_latency";
const OUTPUT_FORMAT_QUERY: &str = "output_format";
const ENABLE_LOGGING_QUERY: &str = "enable_logging";
const ENABLE_SSML_PARSING_QUERY: &str = "enable_ssml_parsing";

///// Text to Speech endpoint
/////
///// # Example
///// ```no_run
///// use elevenlabs_rs::*;
///// use::elevenlabs_rs::utils::play;
/////
///// #[tokio::main]
///// async fn main() -> Result<()> {
/////     let c = ElevenLabsClient::default()?;
/////     let body = TextToSpeechBody::new(
/////         "This is the way the world ends, not with a bang but a whimper",
/////         Model::ElevenMultilingualV2,
/////     );
/////     let endpoint = TextToSpeech::new(LegacyVoice::Clyde, body);
/////     let speech = c.hit(endpoint).await?;
/////     play(speech)?;
/////
/////     Ok(())
///// }
///// ```
//#[derive(Clone, Debug)]
//pub struct TextToSpeech {
//    voice_id: VoiceID,
//    text_to_speech_body: TextToSpeechBody,
//    speech_query: Option<SpeechQuery>,
//}
//
//impl Endpoint for TextToSpeech {
//    type ResponseBody = Bytes;
//
//    const METHOD: Method = Method::POST;
//    async fn request_body(&self) -> Result<RequestBody> {
//        Ok(RequestBody::Json(serde_json::to_value(
//            &self.text_to_speech_body,
//        )?))
//    }
//
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.bytes().await?)
//    }
//    fn url(&self) -> Result<Url> {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!("{}/{}", TTS_PATH, self.voice_id.0));
//        url.set_query(self.any_query().as_deref());
//        Ok(url)
//    }
//}
//
//impl TextToSpeech {
//    pub fn new<T: Into<String>>(voice_id: T, text_to_speech_body: TextToSpeechBody) -> Self {
//        TextToSpeech {
//            voice_id: VoiceID::from(voice_id.into()),
//            text_to_speech_body,
//            speech_query: None,
//        }
//    }
//    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
//        self.speech_query = Some(speech_query);
//        self
//    }
//
//    fn any_query(&self) -> Option<String> {
//        if let Some(query) = &self.speech_query {
//            Some(query.to_string())
//        } else {
//            None
//        }
//    }
//}
//
///// Text to Speech Body for all TTS endpoints
//#[derive(Clone, Debug, Serialize, Default)]
//pub struct TextToSpeechBody {
//    text: String,
//    model_id: ModelID,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    voice_settings: Option<VoiceSettings>,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    pronunciation_dictionary_locators: Option<Vec<PronunciationDictionaryLocator>>,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    seed: Option<u64>,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    previous_text: Option<String>,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    next_text: Option<String>,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    previous_text_ids: Option<Vec<String>>,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    next_text_ids: Option<Vec<String>>,
//}
//
//impl TextToSpeechBody {
//    pub fn new<T: Into<String>>(text: &str, model_id: T) -> Self {
//        TextToSpeechBody {
//            text: text.to_string(),
//            model_id: ModelID::from(model_id.into()),
//            ..Default::default()
//        }
//    }
//    pub fn with_pronunciation_dict(mut self, pronunciation_id: &str, version_id: &str) -> Self {
//        if let Some(dictionary) = &mut self.pronunciation_dictionary_locators {
//            dictionary.push(PronunciationDictionaryLocator {
//                pronunciation_dictionary_id: pronunciation_id.to_string(),
//                version_id: version_id.to_string(),
//            });
//        } else {
//            self.pronunciation_dictionary_locators = Some(vec![PronunciationDictionaryLocator {
//                pronunciation_dictionary_id: pronunciation_id.to_string(),
//                version_id: version_id.to_string(),
//            }]);
//        }
//        self
//    }
//    pub fn with_voice_settings(mut self, voice_settings: VoiceSettings) -> Self {
//        self.voice_settings = Some(voice_settings);
//        self
//    }
//    pub fn with_seed(mut self, seed: u64) -> Self {
//        self.seed = Some(seed);
//        self
//    }
//    pub fn with_previous_text(mut self, previous_text: &str) -> Self {
//        self.previous_text = Some(previous_text.to_string());
//        self
//    }
//    pub fn with_next_text(mut self, next_text: &str) -> Self {
//        self.next_text = Some(next_text.to_string());
//        self
//    }
//    pub fn with_previous_text_ids(mut self, previous_text_ids: Vec<String>) -> Self {
//        self.previous_text_ids = Some(previous_text_ids);
//        self
//    }
//    pub fn with_next_text_ids(mut self, next_text_ids: Vec<String>) -> Self {
//        self.next_text_ids = Some(next_text_ids);
//        self
//    }
//}
//#[derive(Clone, Debug, Serialize)]
//struct PronunciationDictionaryLocator {
//    pronunciation_dictionary_id: String,
//    version_id: String,
//}
//
#[derive(Clone, Debug, Default)]
pub struct SpeechQuery {
    latency: Option<String>,
    output_format: Option<String>,
    enable_logging: Option<String>,
    enable_ssml_parsing: Option<String>,
}
//
//impl SpeechQuery {
//    pub fn with_latency(mut self, latency: Latency) -> Self {
//        self.latency = Some(format!("{}={}", LATENCY_QUERY, latency as u8));
//        self
//    }
//    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
//        self.output_format = Some(format!(
//            "{}={}",
//            OUTPUT_FORMAT_QUERY,
//            output_format.to_query()
//        ));
//        self
//    }
//    pub fn with_logging(mut self, enable_logging: bool) -> Self {
//        self.enable_logging = Some(format!("{}={}", ENABLE_LOGGING_QUERY, enable_logging));
//        self
//    }
//    pub fn with_ssml_parsing(mut self, enable_ssml_parsing: bool) -> Self {
//        self.enable_ssml_parsing = Some(format!(
//            "{}={}",
//            ENABLE_SSML_PARSING_QUERY, enable_ssml_parsing
//        ));
//        self
//    }
//
//    pub fn to_string(&self) -> String {
//        let mut query = String::new();
//
//        if let Some(latency) = &self.latency {
//            query.push_str(latency);
//        }
//        if let Some(output_format) = &self.output_format {
//            if !query.is_empty() {
//                query.push('&');
//            }
//            query.push_str(output_format);
//        }
//        if let Some(enable_logging) = &self.enable_logging {
//            if !query.is_empty() {
//                query.push('&');
//            }
//            query.push_str(enable_logging);
//        }
//        if let Some(enable_ssml_parsing) = &self.enable_ssml_parsing {
//            if !query.is_empty() {
//                query.push('&');
//            }
//            query.push_str(enable_ssml_parsing);
//        }
//        query
//    }
//}
//
///// Text to Speech Stream
///// # Example
/////
///// ```no_run
///// use elevenlabs_rs::utils::stream_audio;
///// use elevenlabs_rs::*;
/////
///// #[tokio::main]
///// async fn main() -> Result<()> {
/////     let c = ElevenLabsClient::default()?;
/////     let body = TextToSpeechBody::new(
/////         "The art of progress is to preserve order amid change and to preserve change amid order.",
/////             Model::ElevenMultilingualV2
/////         );
/////         let endpoint = TextToSpeechStream::new(LegacyVoice::Alice, body);
/////         let mut stream = c.hit(endpoint).await?;
/////         stream_audio(&mut stream).await?;
/////     Ok(())
///// }
///// ```
//#[derive(Clone, Debug)]
//pub struct TextToSpeechStream {
//    voice_id: VoiceID,
//    text_to_speech_body: TextToSpeechBody,
//    speech_query: Option<SpeechQuery>,
//}
//
//impl TextToSpeechStream {
//    pub fn new<T: Into<String>>(voice_id: T, text_to_speech_body: TextToSpeechBody) -> Self {
//        TextToSpeechStream {
//            voice_id: VoiceID::from(voice_id.into()),
//            text_to_speech_body,
//            speech_query: None,
//        }
//    }
//    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
//        self.speech_query = Some(speech_query);
//        self
//    }
//    fn any_query(&self) -> Option<String> {
//        if let Some(query) = &self.speech_query {
//            Some(query.to_string())
//        } else {
//            None
//        }
//    }
//}
//
//type TextToSpeechStreamResponse = Pin<Box<dyn Stream<Item = Result<Bytes>>>>;
//impl Endpoint for TextToSpeechStream {
//    type ResponseBody = TextToSpeechStreamResponse;
//
//    const METHOD: Method = Method::POST;
//    async fn request_body(&self) -> Result<RequestBody> {
//        Ok(RequestBody::Json(serde_json::to_value(
//            &self.text_to_speech_body,
//        )?))
//    }
//
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        let stream = resp.bytes_stream();
//        let stream = stream.map(|r| r.map_err(Into::into));
//        Ok(Box::pin(stream))
//    }
//    fn url(&self) -> Result<Url> {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!("{}/{}{}", TTS_PATH, self.voice_id.0, STREAM_PATH));
//        Ok(url)
//    }
//}
///// Text to Speech with Timestamps endpoint
/////
///// # Example
///// ```no_run
///// use elevenlabs_rs::*;
///// use elevenlabs_rs::endpoints::tts::*;
///// use elevenlabs_rs::utils::play;
/////
///// #[tokio::main]
///// async fn main() -> Result<()> {
/////     let c = ElevenLabsClient::default()?;
/////     let voice_id = LegacyVoice::Rachel;
/////     let model_id = Model::ElevenMultilingualV2;
/////
/////     let txt = "To see a world in a grain of sand, and a heaven in a wild flower, \
/////     hold infinity in the palm of your hand, and eternity in an hour.";
/////     let body = TextToSpeechBody::new(txt, model_id);
/////     let endpoint = TextToSpeechWithTimestamps::new(voice_id, body);
/////     let resp = c.hit(endpoint).await?;
/////     let mut timestamps = resp.iter().unwrap();
/////     for (char, (start_time, end_time)) in timestamps {
/////         println!("{} = {} - {}", char, start_time, end_time);
/////     }
/////     let audio = resp.audio()?;
/////     play(audio)?;
/////     Ok(())
///// }
///// ```
//#[derive(Clone, Debug)]
//pub struct TextToSpeechWithTimestamps {
//    voice_id: VoiceID,
//    text_to_speech_body: TextToSpeechBody,
//    speech_query: Option<SpeechQuery>,
//}
//
//impl TextToSpeechWithTimestamps {
//    pub fn new<T: Into<String>>(voice_id: T, text_to_speech_body: TextToSpeechBody) -> Self {
//        TextToSpeechWithTimestamps {
//            voice_id: VoiceID::from(voice_id.into()),
//            text_to_speech_body,
//            speech_query: None,
//        }
//    }
//    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
//        self.speech_query = Some(speech_query);
//        self
//    }
//    fn any_query(&self) -> Option<String> {
//        if let Some(query) = &self.speech_query {
//            Some(query.to_string())
//        } else {
//            None
//        }
//    }
//}
//
//impl Endpoint for TextToSpeechWithTimestamps {
//    type ResponseBody = TextToSpeechWithTimestampsResponse;
//
//    const METHOD: Method = Method::POST;
//    async fn request_body(&self) -> Result<RequestBody> {
//        Ok(RequestBody::Json(serde_json::to_value(
//            &self.text_to_speech_body,
//        )?))
//    }
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//    fn url(&self) -> Result<Url> {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!(
//            "{}/{}{}",
//            TTS_PATH, self.voice_id.0, WITH_TIMESTAMP_PATH
//        ));
//        url.set_query(self.any_query().as_deref());
//        Ok(url)
//    }
//}
//
///// The response from the TextToSpeechWithTimestamps endpoint
/////
//
//#[derive(Clone, Debug, Deserialize)]
//pub struct TextToSpeechWithTimestampsResponse {
//    alignment: Option<Alignment>,
//    audio_base64: String,
//    normalized_alignment: Option<Alignment>,
//}
//
//impl TextToSpeechWithTimestampsResponse {
//    pub fn alignment(&self) -> Option<&Alignment> {
//        self.alignment.as_ref()
//    }
//    pub fn audio_base64(&self) -> &str {
//        &self.audio_base64
//    }
//    pub fn audio(&self) -> Result<Bytes> {
//        let decoded_audio_b64 = general_purpose::STANDARD.decode(&self.audio_base64())?;
//        Ok(Bytes::from(decoded_audio_b64))
//    }
//    pub fn normalized_alignment(&self) -> Option<&Alignment> {
//        self.normalized_alignment.as_ref()
//    }
//    pub fn iter(&self) -> Option<impl Iterator<Item = (&String, (&f32, &f32))>> {
//        if let Some(alignment) = &self.alignment {
//            Some(
//                alignment.characters().iter().zip(
//                    alignment
//                        .character_start_times_seconds()
//                        .iter()
//                        .zip(alignment.character_end_times_seconds().iter()),
//                ),
//            )
//        } else {
//            None
//        }
//        //self.alignment().characters().iter().zip(
//        //self.alignment
//        //.character_start_times_seconds
//        //.iter()
//        //.zip(self.alignment.character_end_times_seconds.iter()),
//        //)
//    }
//}
//
//#[derive(Clone, Debug, Deserialize)]
//pub struct Alignment {
//    character_end_times_seconds: Vec<f32>,
//    character_start_times_seconds: Vec<f32>,
//    characters: Vec<String>,
//}
//
//impl Alignment {
//    pub fn character_end_times_seconds(&self) -> &[f32] {
//        &self.character_end_times_seconds
//    }
//    pub fn character_start_times_seconds(&self) -> &[f32] {
//        &self.character_start_times_seconds
//    }
//    pub fn characters(&self) -> &[String] {
//        &self.characters
//    }
//}
//
///// Text to Speech Stream with Timestamps endpoint
/////
///// # Example
///// ```no_run
///// use elevenlabs_rs::*;
///// use elevenlabs_rs::utils::play;
/////
///// #[tokio::main]
///// async fn main() -> Result<()> {
/////     let c = ElevenLabsClient::default()?;
/////     let voice_id = LegacyVoice::Rachel;
/////     let model_id = Model::ElevenTurboV2;
/////     let txt = "Without Haste! Without Rest!, /
/////     Bind the motto to thy breast! /
/////     Bear it with thee as a spell; /
/////     Storm or sunshine, guard it well!";
/////
/////     let body = TextToSpeechBody::new(txt, model_id);
/////     let endpoint = TextToSpeechStreamWithTimestamps::new(voice_id, body);
/////     let resp = c.hit(endpoint).await?;
/////     pin_mut!(resp);
/////
/////     let mut audio_bytes = Vec::new();
/////     while let Some(result) = resp.next().await {
/////         let tts_timestamp_resp = result?;
/////         if tts_timestamp_resp.audio().is_ok() {
/////             audio_bytes.extend(tts_timestamp_resp.audio().unwrap());
/////         }
/////         if tts_timestamp_resp.iter().is_some() {
/////             for (char, (start_time, end_time)) in tts_timestamp_resp.iter().unwrap() {
/////                 println!("{} = {} - {}", char, start_time, end_time);
/////             }
/////         }
/////     }
/////     play(Bytes::from(audio_bytes))?;
/////     Ok(())
///// }
///// ```
//
//#[derive(Clone, Debug)]
//pub struct TextToSpeechStreamWithTimestamps {
//    voice_id: VoiceID,
//    text_to_speech_body: TextToSpeechBody,
//    speech_query: Option<SpeechQuery>,
//}
//
//impl TextToSpeechStreamWithTimestamps {
//    pub fn new<T: Into<String>>(voice_id: T, text_to_speech_body: TextToSpeechBody) -> Self {
//        TextToSpeechStreamWithTimestamps {
//            voice_id: VoiceID::from(voice_id.into()),
//            text_to_speech_body,
//            speech_query: None,
//        }
//    }
//    pub fn with_query(mut self, speech_query: SpeechQuery) -> Self {
//        self.speech_query = Some(speech_query);
//        self
//    }
//    fn any_query(&self) -> Option<String> {
//        if let Some(query) = &self.speech_query {
//            Some(query.to_string())
//        } else {
//            None
//        }
//    }
//}
//
//type TextToSpeechStreamWithTimestampsResponse =
//    Pin<Box<dyn Stream<Item = Result<TextToSpeechWithTimestampsResponse>>>>;
//
//impl Endpoint for TextToSpeechStreamWithTimestamps {
//    type ResponseBody = TextToSpeechStreamWithTimestampsResponse;
//
//    const METHOD: Method = Method::POST;
//    async fn request_body(&self) -> Result<RequestBody> {
//        Ok(RequestBody::Json(serde_json::to_value(
//            &self.text_to_speech_body,
//        )?))
//    }
//
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        let stream = resp.bytes_stream();
//        let end_of_chunk = b"\n\n";
//        let mut buf = String::new();
//        Ok(Box::pin(try_stream! {
//            for await bytes_result in stream {
//                let bytes = bytes_result?;
//                if bytes.ends_with(end_of_chunk) {
//                    buf.push_str(std::str::from_utf8(&bytes)?);
//                    let json: TextToSpeechWithTimestampsResponse = serde_json::from_str(&buf)?;
//                    yield json;
//                    buf.clear();
//                } else {
//                    buf.push_str(std::str::from_utf8(&bytes)?);
//                }
//            }
//        }))
//    }
//    fn url(&self) -> Result<Url> {
//        let mut url = BASE_URL.parse::<Url>().unwrap();
//        url.set_path(&format!(
//            "{}/{}{}{}",
//            TTS_PATH, self.voice_id.0, STREAM_PATH, WITH_TIMESTAMP_PATH
//        ));
//        url.set_query(self.any_query().as_deref());
//        Ok(url)
//    }
//}

pub mod ws {
    #![allow(dead_code)]
    //! Websocket Text to Speech endpoints
    use super::*;


    const WS_BASE_URL: &str = "wss://api.elevenlabs.io";
    const WS_STREAM_PATH: &str = "/stream-input";
    const MODEL_ID_QUERY: &str = "model_id";

    pub type StreamAfterFlush = Pin<Box<dyn Stream<Item = String> + Send + 'static>>;

    /// Websocket Text to Speech endpoint
    ///
    /// # Example
    ///
    /// ```no_run
    /// use async_stream::stream;
    /// use elevenlabs_rs::*;
    /// use elevenlabs_rs::utils::{stream_audio, text_chunker};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let text_stream = stream! {
    ///         yield "Then you should say what you mean the March Hare went on.".to_string();
    ///         yield "I do Alice hastily replied; at least - at least I mean what I say \
    ///          - that's the same thing you know.".into();
    ///         yield "Not the same thing a bit! said the Hatter.".into();
    ///         yield "You might just as well say that I see what I eat is the same thing as I eat what I see!".into();
    ///     };
    ///
    ///     let text_stream = text_chunker(text_stream);
    ///
    ///     let body = WebSocketTTSBody::new(BOSMessage::default(), text_stream);
    ///     let endpoint = WebSocketTTS::new(LegacyVoice::Alice, Model::ElevenTurboV2, body);
    ///     let client = ElevenLabsClient::from_env()?;
    ///     let stream = client.hit_ws(endpoint).await?;
    ///
    ///     stream_audio(stream.map(|r| r?.audio_as_bytes())).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## With Flush
    /// ```no_run
    /// use async_stream::stream;
    /// use elevenlabs_rs::*;
    /// use elevenlabs_rs::utils::stream_audio;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let text = "It is a profoundly erroneous truism \
    ///         that we should cultivate the habit of thinking of what we are doing."
    ///         .split_ascii_whitespace()
    ///         .map(|w| w.to_string())
    ///         .collect::<Vec<String>>();
    ///     let text_stream = stream! {
    ///         for word in text {
    ///             yield word;
    ///         }
    ///     };
    ///
    ///     let text_stream_2 = stream! {
    ///         yield "The".to_string();
    ///         yield "precise".into();
    ///         yield "opposite".into();
    ///         yield "is".into();
    ///         yield "the".into();
    ///         yield "case".into();
    ///     };
    ///
    ///     let text = "Civilization advances by extending the number of important operations \
    ///     which we can perform without thinking about them."
    ///         .split_ascii_whitespace()
    ///         .map(|w| w.to_string())
    ///         .collect::<Vec<String>>();
    ///     let text_stream_3 = stream! {
    ///         for word in text {
    ///             yield word;
    ///         }
    ///     };
    ///
    ///     let body = WebSocketTTSBody::new(BOSMessage::default(), text_stream)
    ///         .with_streams_after_flush(vec![
    ///             Box::pin(text_stream_2) as StreamAfterFlush,
    ///             Box::pin(text_stream_3) as StreamAfterFlush,
    ///         ]);
    ///     let endpoint = WebSocketTTS::new(LegacyVoice::Liam, Model::ElevenTurboV2, body);
    ///
    ///     let client = ElevenLabsClient::from_env()?;
    ///     let stream = client.hit_ws(endpoint).await?;
    ///
    ///     stream_audio(stream.map(|r| r?.audio_as_bytes())).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub struct WebSocketTTS<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        path_params: WebSocketTTSPathParams,
        text_to_speech_body: WebSocketTTSBody<S>,
        speech_query: Option<SpeechQuery>,
    }

    impl<S> WebSocketTTS<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub fn new<V, M>(voice_id: V, model_id: M, text_to_speech_body: WebSocketTTSBody<S>) -> Self
        where
            V: Into<String>,
            M: Into<String>,
        {
            let path_params = WebSocketTTSPathParams {
                voice_id: VoiceID::from(voice_id.into()),
                model_id: ModelID::from(model_id.into()),
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
        pub fn url(&self) -> String {
            let mut url = WS_BASE_URL.parse::<Url>().unwrap();
            url.set_path(&format!(
                "{}/{}{}",
                TTS_PATH, self.path_params.voice_id.0, WS_STREAM_PATH
            ));
            let mut query = String::new();
            if let Some(q) = &self.speech_query {
                query.push_str(&q.to_string());
                query.push('&');
                query.push_str(&format!(
                    "{}={}",
                    MODEL_ID_QUERY, self.path_params.model_id.0
                ));
                url.set_query(Some(&query));
            } else {
                query.push_str(&format!(
                    "?{}={}",
                    MODEL_ID_QUERY, self.path_params.model_id.0
                ));
                url.set_query(Some(&query));
            }
            url.to_string()
        }
        pub fn bos_message(&self) -> &BOSMessage {
            self.text_to_speech_body.bos_message()
        }
        pub fn text_stream(self) -> S {
            self.text_to_speech_body.text_stream
        }
        pub fn try_trigger_generation(&self) -> Option<Vec<usize>> {
            self.text_to_speech_body.try_trigger_generation.clone()
        }
        pub fn flush(&self) -> bool {
            self.text_to_speech_body.flush
        }
        pub fn streams_after_flush(&mut self) -> Option<Vec<StreamAfterFlush>> {
            self.text_to_speech_body.streams_after_flush.take()
        }
    }

    #[derive(Clone, Debug)]
    struct WebSocketTTSPathParams {
        voice_id: VoiceID,
        model_id: ModelID,
    }
    #[derive(Serialize)]
    pub struct WebSocketTTSBody<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        bos_message: BOSMessage,
        #[serde(skip_serializing)]
        text_stream: S,
        try_trigger_generation: Option<Vec<usize>>,
        flush: bool,
        #[serde(skip_serializing)]
        streams_after_flush: Option<Vec<StreamAfterFlush>>,
        //#[serde(skip_serializing)]
        //is_try_trigger_always: bool,
    }

    impl<S> WebSocketTTSBody<S>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        pub fn new(bos_message: BOSMessage, text_stream: S) -> Self {
            WebSocketTTSBody {
                bos_message,
                text_stream,
                try_trigger_generation: None,
                flush: false,
                streams_after_flush: None,
                //is_try_trigger_always: false,
            }
        }
        pub fn with_try_trigger_generation(mut self, try_trigger_generation: Vec<usize>) -> Self {
            self.try_trigger_generation = Some(try_trigger_generation);
            self
        }
        //pub fn with_try_trigger_always(mut self) -> Self {
        //    self.is_try_trigger_always = true;
        //    self
        //}
        pub fn with_flush(mut self) -> Self {
            self.flush = true;
            self
        }
        pub fn with_streams_after_flush(mut self, flush_stream: Vec<StreamAfterFlush>) -> Self {
            self.streams_after_flush = Some(flush_stream);
            self
        }
        pub fn bos_message(&self) -> &BOSMessage {
            &self.bos_message
        }
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct BOSMessage {
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

    #[derive(Clone, Debug, Serialize)]
    pub struct TextChunk {
        text: String,
        try_trigger_generation: bool,
    }

    impl TextChunk {
        pub fn new(text: String, try_trigger_generation: bool) -> Self {
            TextChunk {
                text,
                try_trigger_generation,
            }
        }
        pub fn json(self) -> Result<String> {
            serde_json::to_string(&self).map_err(Into::into)
        }
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct Flush {
        text: String,
        flush: bool,
    }
    impl Flush {
        pub fn new() -> Self {
            Flush {
                text: " ".to_string(),
                flush: true,
            }
        }
        pub fn json(self) -> Result<String> {
            serde_json::to_string(&self).map_err(Into::into)
        }
    }

    #[derive(Clone, Debug, Default, Serialize)]
    pub struct EOSMessage {
        text: String,
    }
    impl EOSMessage {
        pub fn json(self) -> Result<String> {
            serde_json::to_string(&self).map_err(Into::into)
        }
    }

    #[derive(Clone, Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketTTSResponse {
        audio: Option<String>,
        is_final: Option<bool>,
        normalized_alignment: Option<WebSocketAlignment>,
        alignment: Option<WebSocketAlignment>,
    }

    impl WebSocketTTSResponse {
        pub fn audio_b64(&self) -> Option<&str> {
            self.audio.as_deref()
        }
        pub fn audio_as_bytes(&self) -> Result<Bytes> {
            if self.is_final().is_some() {
                return Ok(Bytes::new());
            }
            if let Some(audio_b64) = self.audio_b64() {
                return Ok(Bytes::from(general_purpose::STANDARD.decode(audio_b64)?));
            }

            // 'Self' is returned after a `Flush` message with all fields set to `None`
            Ok(Bytes::new())
        }
        pub fn is_final(&self) -> Option<bool> {
            self.is_final
        }
        pub fn normalized_alignment(&self) -> Option<&WebSocketAlignment> {
            self.normalized_alignment.as_ref()
        }
        pub fn alignment(&self) -> Option<&WebSocketAlignment> {
            self.alignment.as_ref()
        }
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WebSocketAlignment {
        char_start_times_ms: Vec<f32>,
        char_durations_ms: Vec<f32>,
        chars: Vec<String>,
    }
}
