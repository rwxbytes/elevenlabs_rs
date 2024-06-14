use crate::client::{Result, BASE_URL};
use crate::endpoints::Endpoint;
use crate::error::Error;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use validator::Validate;

const VOICE_GENERATION_PATH: &str = "/v1/voice-generation/generate-voice";
const PARAMETERS_PATH: &str = "/parameters";
const GENERATED_VOICE_ID_HEADER: &str = "generated_voice_id";
const ACCENT_STRENGTH_MIN: f32 = 0.3;
const ACCENT_STRENGTH_MAX: f32 = 2.0;
const TEXT_LENGTH_MIN: u64 = 100;
const TEXT_LENGTH_MAX: u64 = 1000;

/// Generate a random voice
///
/// This endpoint generates a random voice based on the provided parameters.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::client::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::voice_generation::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///
///     let body = GenerateVoiceBody::new(
///         Gender::Female,
///         Accent::African,
///         Age::Old,
///         2.0,
///         &std::fs::read_to_string("poem.txt")?,
///     );
///
///     let resp = c.hit(GenerateARandomVoice(body)).await?;
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
pub struct GenerateARandomVoice(pub GenerateVoiceBody);

impl Endpoint for GenerateARandomVoice {
    type ResponseBody = GenerateARandomVoiceResponse;

    fn method(&self) -> reqwest::Method {
        reqwest::Method::POST
    }
    fn json_request_body(&self) -> Option<Result<serde_json::Value>> {
        match self.0.validate() {
            Ok(_) => Some(serde_json::to_value(&self.0).map_err(Into::into)),
            Err(e) => Some(Err(Box::new(e))),
        }
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
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(VOICE_GENERATION_PATH);
        url
    }
}

pub struct GenerateARandomVoiceResponse {
    generated_voice_id: String,
    sample: bytes::Bytes,
}

impl GenerateARandomVoiceResponse {
    pub fn generated_voice_id(&self) -> &str {
        &self.generated_voice_id
    }
    pub fn sample(&self) -> &bytes::Bytes {
        &self.sample
    }
}

/// Accent strength, must be between 0.3 and 2.0
///
/// Text length has to be between 100 and 1000
///
/// See [ElevenLabs API documentation](https://elevenlabs.io/docs/api-reference/generate-voice) for more information
#[derive(Clone, Debug, Serialize, Validate)]
pub struct GenerateVoiceBody {
    pub gender: Gender,
    pub accent: Accent,
    pub age: Age,
    #[validate(range(min = "ACCENT_STRENGTH_MIN", max = "ACCENT_STRENGTH_MAX"))]
    pub accent_strength: f32,
    #[validate(length(min = "TEXT_LENGTH_MIN", max = "TEXT_LENGTH_MAX"))]
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
}

impl Gender {
    pub fn as_str(&self) -> &str {
        match self {
            Gender::Female => "female",
            Gender::Male => "male",
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
    pub fn new(gender: Gender, accent: Accent, age: Age, accent_strength: f32, text: &str) -> Self {
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
pub struct GetGenerationParams;

impl Endpoint for GetGenerationParams {
    type ResponseBody = VoiceGenerationParamsResponse;

    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> reqwest::Url {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(&format!("{}{}", VOICE_GENERATION_PATH, PARAMETERS_PATH));
        url
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceGenerationParamsResponse {
    genders: Vec<VoiceGenerationParams>,
    accents: Vec<VoiceGenerationParams>,
    ages: Vec<VoiceGenerationParams>,
    minimum_characters: u32,
    maximum_characters: u32,
    minimum_accent_strength: f32,
    maximum_accent_strength: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceGenerationParams {
    name: String,
    code: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_a_random_voice_errs_when_body_accent_strength_has_invalid_values() {
        let text = std::iter::repeat("a").take(101).collect::<String>();
        let accents_s = vec![0.0, 0.1, 0.2, 2.1, 2.2, 3.0];
        let body = GenerateVoiceBody::new(
            Gender::Female,
            Accent::African,
            Age::Young,
            accents_s[0],
            &text,
        );
        let mut endpoint = GenerateARandomVoice(body);
        for accent_s in accents_s {
            endpoint.0.accent_strength = accent_s;
            assert!(endpoint.json_request_body().unwrap().is_err());
        }
    }
}
