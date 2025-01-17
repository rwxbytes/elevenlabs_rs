//! The text to sound effects endpoint.
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
///    let body = TextToSoundEffectsBody::new(text);
///    let endpoint = TextToSoundEffects::new(body);
///    let resp = c.hit(endpoint).await?;
///    save("sound_effects.mp3", resp.clone())?;
///    play(resp)?;
///    Ok(())
/// }
/// ```
/// See [Text To Sound Effects API reference](https://elevenlabs.io/docs/api-reference/text-to-sound-effects/convert)
#[derive(Clone, Debug)]
pub struct TextToSoundEffects {
    body: TextToSoundEffectsBody,
}

impl TextToSoundEffects {
    pub fn new(body: TextToSoundEffectsBody) -> Self {
        Self { body }
    }
}

impl ElevenLabsEndpoint for TextToSoundEffects {
    const PATH: &'static str = "/v1/sound-generation";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// The request body for the text to sound effects endpoint.
///
/// - `text`: The text to convert to sound effects.
///
///
/// - `duration_seconds`: The duration of the sound which will be generated in seconds.
/// Must be at least 0.5 and at most 22.
/// If set to None we will guess the optimal duration using the prompt. Defaults to None.
///
///
/// - `prompt_influence`: A higher prompt influence makes your generation follow
/// the prompt more closely while also making generations less variable.
/// Must be a value between 0 and 1. Defaults to 0.3.
#[derive(Debug, Clone, Serialize)]
pub struct TextToSoundEffectsBody {
    text: String,
    duration_seconds: Option<f32>,
    prompt_influence: Option<f32>,
}
impl TextToSoundEffectsBody {
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
