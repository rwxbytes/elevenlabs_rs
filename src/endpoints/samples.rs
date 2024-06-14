use crate::client::{ElevenLabsClient, Result, BASE_URL};
use crate::endpoints::voice::{GetVoice, VoiceID, VOICES_PATH};
use crate::endpoints::Endpoint;
use crate::utils::play;
use bytes::Bytes;
use reqwest::Response;

const SAMPLES_PATH: &str = "/samples";
const AUDIO_PATH: &str = "/audio";

/// Removes a sample by its ID.
/// ``` no_run
/// use elevenlabs_rs::client::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::samples::*;
/// use elevenlabs_rs::endpoints::voice::{GetVoice, VoiceID};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let voice_id = VoiceID::from("some voice id");
///     let voice = c.hit(GetVoice(voice_id.clone())).await?;
///     let sample_id = voice
///         .get_samples()
///         .unwrap()
///         .get(0)
///         .unwrap()
///         .get_sample_id()
///         .clone();
///     let params = SamplePathParams {
///         voice_id,
///         sample_id,
///     };
///     let status = c.hit(DeleteSample(params)).await?;
///     println!("{:#?}", status);
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct DeleteSample(pub SamplePathParams);

impl Endpoint for DeleteSample {
    type ResponseBody = super::Status;

    fn method(&self) -> reqwest::Method {
        reqwest::Method::DELETE
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}/{}",
            VOICES_PATH, self.0.voice_id.0, SAMPLES_PATH, self.0.sample_id
        ));
        url
    }
}

#[derive(Clone, Debug)]
pub struct SamplePathParams {
    pub voice_id: VoiceID,
    pub sample_id: String,
}

/// Returns the audio corresponding to a sample attached to a voice.
/// ``` no_run
/// use elevenlabs_rs::client::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::samples::*;
/// use elevenlabs_rs::endpoints::voice::{GetVoice, VoiceID};
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let voice_id = VoiceID::from("some voice id");
///     let voice = c.hit(GetVoice(voice_id.clone())).await?;
///     let sample_id = voice
///         .get_samples()
///         .unwrap() // TODO: unwrap is not advised in documentation, do ok_or
///         .get(0)
///         .unwrap()
///         .get_sample_id()
///         .clone();
///     let params = SamplePathParams {
///         voice_id,
///         sample_id,
///     };
///
///     let sample_audio = c.hit(GetAudioFromSample(params)).await?;
///     play(sample_audio)?;
///     Ok(())
/// }
/// ```

#[derive(Clone, Debug)]
pub struct GetAudioFromSample(pub SamplePathParams);
impl Endpoint for GetAudioFromSample {
    type ResponseBody = Bytes;

    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}/{}{}",
            VOICES_PATH, self.0.voice_id.0, SAMPLES_PATH, self.0.sample_id, AUDIO_PATH
        ));
        url
    }
}
