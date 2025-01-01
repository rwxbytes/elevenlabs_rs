//! The samples endpoints.
#[allow(dead_code)]
use super::*;

const SAMPLES_PATH: &str = "/samples";
const AUDIO_PATH: &str = "/audio";

/// Removes a sample by its ID.
/// ``` no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::samples::*;
/// use elevenlabs_rs::endpoints::voice::GetVoice;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let voice_id = "some voice id".to_string();
///     let voice = c.hit(GetVoice::new(voice_id.clone())).await?;
///     let sample_id = voice
///         .get_samples()
///         .unwrap()
///         .get(0)
///         .unwrap()
///         .get_sample_id()
///         .clone();
///     let status = c.hit(DeleteSample::new(voice_id, &sample_id)).await?;
///     println!("{:#?}", status);
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct DeleteSample(SamplePathParams);

impl DeleteSample {
    pub fn new<T: Into<String>>(voice_id: T, sample_id: &str) -> Self {
        Self {
            0: SamplePathParams::new(voice_id, sample_id),
        }
    }
}

impl Endpoint for DeleteSample {
    type ResponseBody = StatusResponseBody;

    const METHOD: Method = Method::DELETE;
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Result<Url> {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}/{}",
            VOICES_PATH, self.0.voice_id.0, SAMPLES_PATH, self.0.sample_id
        ));
        Ok(url)
    }
}

#[derive(Clone, Debug)]
pub struct SamplePathParams {
    voice_id: VoiceID,
    sample_id: String,
}

impl SamplePathParams {
    pub fn new<T: Into<String>>(voice_id: T, sample_id: &str) -> Self {
        Self {
            voice_id: VoiceID::from(voice_id.into()),
            sample_id: sample_id.to_string(),
        }
    }
}

/// Returns the audio corresponding to a sample attached to a voice.
/// ``` no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let voice_id = "some voice id".to_string();
///     let voice = c.hit(GetVoice::new(voice_id.clone())).await?;
///     let sample_id = voice
///         .get_samples()
///         .unwrap()
///         .get(0)
///         .unwrap()
///         .get_sample_id()
///         .clone();
///     let sample_audio = c.hit(GetAudioFromSample::new(voice_id, &sample_id)).await?;
///     play(sample_audio)?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetAudioFromSample(SamplePathParams);

impl GetAudioFromSample {
    pub fn new<T: Into<String>>(voice_id: T, sample_id: &str) -> Self {
        Self {
            0: SamplePathParams::new(voice_id, sample_id),
        }
    }
}
impl Endpoint for GetAudioFromSample {
    type ResponseBody = Bytes;

    const METHOD: Method = Method::GET;
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> Result<Url> {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}/{}{}",
            VOICES_PATH, self.0.voice_id.0, SAMPLES_PATH, self.0.sample_id, AUDIO_PATH
        ));
        Ok(url)
    }
}
