//! The sound generation endpoint.
#[allow(dead_code)]
use super::*;

const SOUND_GENERATION_PATH: &str = "/v1/sound-generation";

/// Sound Generation endpoint
///
/// Use Sound Generation API to generate sound from text.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::{save, play};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let text = "Retro video game power-up";
///    let settings = GenerationSettings::new(true, 0.5, 1.0 );
///    let endpoint = SoundGeneration::new(text, settings);
///    let resp = c.hit(endpoint).await?;
///    save("sound_generation.mp3", resp.clone())?;
///    play(resp)?;
///    Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SoundGeneration(SoundGenerationBody);

#[derive(Debug, Clone, Serialize)]
pub struct SoundGenerationBody {
    text: String,
    generation_settings: GenerationSettings,
}

/// `duration_seconds` expected to be greater or equal to 0.5 and less or equal to 22,
/// `prompt_influence` expected to be greater or equal to 0.0 and less or equal to 1.0
#[derive(Debug, Clone, Serialize)]
pub struct GenerationSettings {
    use_auto_duration: bool,
    duration_seconds: f64,
    prompt_influence: f64,
}

impl SoundGeneration {
    pub fn new(text: &str, generation_settings: GenerationSettings) -> Self {
        SoundGeneration(SoundGenerationBody {
            text: text.to_string(),
            generation_settings,
        })
    }
}

impl GenerationSettings {
    pub fn new(use_auto_duration: bool, duration_seconds: f64, prompt_influence: f64) -> Self {
        GenerationSettings {
            use_auto_duration,
            duration_seconds,
            prompt_influence,
        }
    }
}

impl Endpoint for SoundGeneration {
    type ResponseBody = Bytes;

    const METHOD: Method = Method::POST;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.0)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }

    fn url(&self) -> Result<Url> {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(SOUND_GENERATION_PATH);
        Ok(url)
    }
}
