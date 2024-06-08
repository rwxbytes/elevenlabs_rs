use super::*;
use crate::client::BASE_URL;
use crate::endpoints::voice::{VoiceID, VoiceSettings};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde::Serialize;
use std::pin::Pin;

const TTS_PATH: &str = "/v1/text-to-speech";
const STREAM_PATH: &str = "/stream";
const LATENCY_QUERY: &str = "optimize_streaming_latency";
const OUTPUT_FORMAT_QUERY: &str = "output_format";
const ENABLE_LOGGING_QUERY: &str = "enable_logging";

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
    // TODO: Implement query parameters
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", TTS_PATH, self.voice_id.0));
        url.set_query(self.any_query().as_deref());
        //if let Some(query) = &self.speech_query {
        //    let query = query.to_string();
        //    url.set_query(Some(&query));
        //}
        url
    }
}

// TODO: any_query() method? `url.set_query(self.any_query())`
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
    pub fn with_voice_settings(
        mut self,
        similarity_boost: f32,
        stability: f32,
        style: Option<f32>,
        use_speaker_boost: Option<bool>,
    ) -> Self {
        self.voice_settings = Some(VoiceSettings {
            similarity_boost,
            stability,
            style,
            use_speaker_boost,
        });
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
        self.output_format = Some(format!("{}={}", OUTPUT_FORMAT_QUERY, output_format.to_query()));
        self
    }
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.enable_logging = Some(format!("{}={}", ENABLE_LOGGING_QUERY, enable_logging));
        self
    }

    fn to_string(&self) -> String {
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
    pub fn new_stream(voice_id: &str, text_to_speech_body: TextToSpeechBody) -> Self {
        let voice_id = VoiceID::from(voice_id);
        TextToSpeechStream {
            voice_id,
            text_to_speech_body,
            speech_query: None,
        }
    }
    pub fn with_stream_query(mut self, speech_query: SpeechQuery) -> Self {
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

impl TextToSpeechStream {
    pub fn new(voice_id: VoiceID, text_to_speech_body: TextToSpeechBody) -> Self {
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
}
