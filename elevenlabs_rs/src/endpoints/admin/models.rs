//! The models endpoint

use super::*;

/// Gets a list of available models.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::models::GetModels;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let client = ElevenLabsClient::from_env()?;
///   let models = client.hit(GetModels).await?;
///   println!("{:#?}", models);
///   Ok(())
/// }
/// ```
/// See the [Get Models API reference](https://elevenlabs.io/docs/api-reference/models/get-all)
#[derive(Clone, Debug)]
pub struct GetModels;

impl ElevenLabsEndpoint for GetModels {
    const PATH: &'static str = "v1/models";
    const METHOD: Method = Method::GET;
    type ResponseBody = GetModelsResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

type GetModelsResponse = Vec<Model>;

#[derive(Clone, Debug, Deserialize)]
pub struct Model {
    pub model_id: String,
    pub name: String,
    pub can_be_finetuned: bool,
    pub can_do_text_to_speech: bool,
    pub can_do_voice_conversion: bool,
    pub can_use_style: bool,
    pub can_use_speaker_boost: bool,
    pub serves_pro_voices: bool,
    pub token_cost_factor: f32,
    pub description: String,
    pub requires_alpha_access: bool,
    pub max_characters_request_free_user: f32,
    pub max_characters_request_subscribed_user: f32,
    pub maximum_text_length_per_request: f32,
    pub languages: Vec<Language>,
    pub model_rates: ModelRates,
    pub concurrency_group: ConcurrencyGroup,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Language {
    pub language_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelRates {
    pub character_cost_multiplier: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConcurrencyGroup {
    Standard,
    Turbo,
}
