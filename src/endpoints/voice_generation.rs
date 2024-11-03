#![allow(dead_code)]
//! The voice generation endpoints
use super::*;
use crate::client::{Result, BASE_URL};
use crate::endpoints::Endpoint;
use crate::error::Error;
use reqwest::Response;
use serde::{Deserialize, Serialize};

const VOICE_GENERATION_PATH: &str = "/v1/voice-generation/generate-voice";
const PARAMETERS_PATH: &str = "/parameters";
const GENERATED_VOICE_ID_HEADER: &str = "generated_voice_id";
const ACCENT_STRENGTH_MIN: f32 = 0.3;
const ACCENT_STRENGTH_MAX: f32 = 2.0;
const TEXT_LENGTH_MIN: u64 = 100;
const TEXT_LENGTH_MAX: u64 = 1000;

/// Generate a random voice
///
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///
///     let body = GenerateVoiceBody::new(
///         GenderType::Female,
///         Accent::African,
///         Age::Old,
///         2.0,
///         "Hello, I am a random voice",
///     );
///
///     let resp = c.hit(GenerateARandomVoice::new(body)).await?;
///     let id = resp.generated_voice_id();
///     println!("Generated voice id: {}", id);
///
///     let sample = resp.sample().clone();
///
///     save("sample_01.mp3", sample)?;
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
#[deprecated]
pub struct GenerateARandomVoice(GenerateVoiceBody);

impl GenerateARandomVoice {
    pub fn new(body: GenerateVoiceBody) -> Self {
        Self(body)
    }
}

impl Endpoint for GenerateARandomVoice {
    type ResponseBody = GenerateARandomVoiceResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.0)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let generated_voice_id = resp
            .headers()
            .get(GENERATED_VOICE_ID_HEADER)
            .ok_or(Box::new(Error::GeneratedVoiceIDHeaderNotFound))?
            .to_str()?
            .to_string();
        Ok(GenerateARandomVoiceResponse {
            generated_voice_id,
            sample: resp.bytes().await?,
        })
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(VOICE_GENERATION_PATH);
        url
    }
}

#[deprecated]
pub struct GenerateARandomVoiceResponse {
    generated_voice_id: String,
    sample: Bytes,
}

impl GenerateARandomVoiceResponse {
    pub fn generated_voice_id(&self) -> &str {
        &self.generated_voice_id
    }
    pub fn sample(&self) -> &Bytes {
        &self.sample
    }
}

/// Accent strength, must be between 0.3 and 2.0
///
/// Text length has to be between 100 and 1000
///
/// See [ElevenLabs API documentation](https://elevenlabs.io/docs/api-reference/generate-voice) for more information
#[derive(Clone, Debug, Serialize)]
#[deprecated]
pub struct GenerateVoiceBody {
    pub gender: GenderType,
    pub accent: Accent,
    pub age: Age,
    pub accent_strength: f32,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GenderType {
    Female,
    Male,
}

impl GenderType {
    pub fn as_str(&self) -> &str {
        match self {
            GenderType::Female => "female",
            GenderType::Male => "male",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    African,
    American,
    Australian,
    British,
    Indian,
}

impl Accent {
    pub fn as_str(&self) -> &str {
        match self {
            Accent::African => "african",
            Accent::American => "american",
            Accent::Australian => "australian",
            Accent::British => "british",
            Accent::Indian => "indian",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Age {
    Young,
    MiddleAged,
    Old,
}

impl Age {
    pub fn as_str(&self) -> &str {
        match self {
            Age::Young => "young",
            Age::MiddleAged => "middle_aged",
            Age::Old => "old",
        }
    }
}

impl GenerateVoiceBody {
    pub fn new(gender: GenderType, accent: Accent, age: Age, accent_strength: f32, text: &str) -> Self {
        Self {
            gender,
            accent,
            age,
            accent_strength,
            text: text.to_string(),
        }
    }
}

/// Get possible parameters for voice generation endpoint [GenerateARandomVoice]
#[derive(Clone, Debug)]
#[deprecated]
pub struct GetGenerationParams;

impl Endpoint for GetGenerationParams {
    type ResponseBody = VoiceGenerationParamsResponse;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}{}", VOICE_GENERATION_PATH, PARAMETERS_PATH));
        url
    }
}

#[derive(Clone, Debug, Deserialize)]
#[deprecated]
pub struct VoiceGenerationParamsResponse {
    genders: Vec<VoiceGenerationParams>,
    accents: Vec<VoiceGenerationParams>,
    ages: Vec<VoiceGenerationParams>,
    minimum_characters: u32,
    maximum_characters: u32,
    minimum_accent_strength: f32,
    maximum_accent_strength: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
#[deprecated]
pub struct VoiceGenerationParams {
    name: String,
    code: String,
}
