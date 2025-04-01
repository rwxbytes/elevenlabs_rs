//! The text to sound effects endpoint.

use crate::OutputFormat;
use super::*;

/// Turn text into sound effects for your videos,
/// voice-overs or video games using the most advanced sound effects model in the world.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::utils::{save, play};
/// use elevenlabs_rs::endpoints::genai::sound_effects::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
/// let c = ElevenLabsClient::from_env()?;
///    let text = "Retro video game power-up";
///    let body = CreateSoundEffectBody::new(text);
///    let endpoint = CreateSoundEffect::new(body);
///    let resp = c.hit(endpoint).await?;
///    save("sound_effects.mp3", resp.clone())?;
///    play(resp)?;
///    Ok(())
/// }
/// ```
/// See [Text To Sound Effects API reference](https://elevenlabs.io/docs/api-reference/text-to-sound-effects/convert)
#[derive(Clone, Debug)]
pub struct CreateSoundEffect {
    body: CreateSoundEffectBody,
    query: Option<CreateSoundEffectQuery>,
}

impl CreateSoundEffect {
    pub fn new(body: CreateSoundEffectBody) -> Self {
        Self { body, query: None }
    }
    pub fn with_query(mut self, query: CreateSoundEffectQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// The query parameters for the text to sound effects endpoint.
#[derive(Debug, Clone, Default)]
pub struct CreateSoundEffectQuery {
    params: QueryValues,
}

impl CreateSoundEffectQuery {

    /// Output format of the generated audio.
    /// Formatted as codec_sample_rate_bitrate.
    /// So an mp3 with 22.05kHz sample rate at 32kbs is represented as mp3_22050_32.
    /// MP3 with 192kbps bitrate requires you to be subscribed to Creator tier or above.
    /// PCM with 44.1kHz sample rate requires you to be subscribed to Pro tier or above.
    ///
    /// Note that the μ-law format (sometimes written mu-law, often approximated as u-law)
    /// is commonly used for Twilio audio inputs.
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.params.push(("output_format", output_format.to_string()));
        self
    }
}

impl ElevenLabsEndpoint for CreateSoundEffect {
    const PATH: &'static str = "/v1/sound-generation";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }
    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSoundEffectBody {
    /// The text that will get converted into a sound effect.
    text: String,
    /// The duration of the sound which will be generated in seconds.
    /// Must be at least 0.5 and at most 22.
    /// If set to None we will guess the optimal duration using the prompt.
    ///
    /// Defaults to None.
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<f32>,
    /// The duration of the sound which will be generated in seconds.
    /// Must be at least 0.5 and at most 22.
    /// If set to None we will guess the optimal duration using the prompt.
    ///
    /// Defaults to None.
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_influence: Option<f32>,
}
impl CreateSoundEffectBody {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            duration_seconds: None,
            prompt_influence: Some(0.3)
        }
    }

    pub fn with_duration_seconds(mut self, duration_seconds: f32) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }

    pub fn with_prompt_influence(mut self, prompt_influence: f32) -> Self {
        self.prompt_influence = Some(prompt_influence);
        self
    }
}
