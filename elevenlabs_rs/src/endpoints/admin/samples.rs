//! The samples endpoints.
use super::*;

/// Removes a sample by its ID.
///
/// # Example
/// ``` no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::samples::DeleteSample;
/// use elevenlabs_rs::endpoints::admin::voice::GetVoice;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let voice_resp = c.hit(GetVoice::new("voice_id")).await?;
///
///     if let Some(samples) = voice_resp.samples {
///         let sample = samples.iter().next().unwrap();
///         let sample_id = sample.sample_id.as_str();
///         let status = c
///             .hit(DeleteSample::new("voice_id", sample_id))
///             .await?;
///         println!("{:#?}", status);
///     }
///     Ok(())
/// }
/// ```
/// See the [Delete Sample API reference](https://elevenlabs.io/docs/api-reference/samples/delete)
#[derive(Clone, Debug)]
pub struct DeleteSample {
    sample_id: SampleID,
    voice_id: VoiceID,
}

impl DeleteSample {
    pub fn new(voice_id: impl Into<VoiceID>, sample_id: impl Into<SampleID>) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for DeleteSample {
    const PATH: &'static str = "/v1/voices/:voice_id/samples/:sample_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.as_path_param(),
            self.sample_id.as_path_param(),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Returns the audio corresponding to a sample attached to a voice.
///
/// # Example
/// ``` no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::samples::GetAudioFromSample;
/// use elevenlabs_rs::endpoints::admin::voice::GetVoice;
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let voice = c.hit(GetVoice::new("voice_id")).await?;
///
///     let voice_id = voice.voice_id.as_str();
///
///     if voice.samples.is_none() {
///        return Err("no samples found".into());
///     }
///
///    let samples = voice.samples.unwrap();
///    let sample = samples.first().unwrap();
///    let sample_id = sample.sample_id.as_str();
///    let resp_bytes = c.hit(GetAudioFromSample::new(voice_id, sample_id)).await?;
///
///    play(resp_bytes)?;
///
///   Ok(())
/// }
/// ```
/// See the [Get Audio from Sample API reference](https://elevenlabs.io/docs/api-reference/samples/get-audio)
#[derive(Clone, Debug)]
pub struct GetAudioFromSample {
    sample_id: SampleID,
    voice_id: VoiceID,
}

impl GetAudioFromSample {
    pub fn new(voice_id: impl Into<VoiceID>, sample_id: impl Into<SampleID>) -> Self {
        Self {
            voice_id: voice_id.into(),
            sample_id: sample_id.into(),
        }
    }
}
impl ElevenLabsEndpoint for GetAudioFromSample {

    const PATH: &'static str = "/v1/voices/:voice_id/samples/:sample_id/audio";

    const METHOD: Method = Method::GET;

    type ResponseBody = Bytes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.voice_id.as_path_param(),
            self.sample_id.as_path_param(),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}
