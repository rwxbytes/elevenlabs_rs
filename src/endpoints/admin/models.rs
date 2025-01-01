//! The models endpoint
#![allow(dead_code)]
use crate::client::{Result, BASE_URL};
use crate::endpoints::{Endpoint, Url};
use reqwest::Response;
use serde::Deserialize;

const MODELS_PATH: &str = "v1/models";

#[derive(Clone, Debug)]
pub struct GetModels;

impl Endpoint for GetModels {
    type ResponseBody = ModelResponse;

    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Result<Url> {
        let mut url = BASE_URL.parse::<reqwest::Url>().unwrap();
        url.set_path(MODELS_PATH);
        Ok(url)
    }
}

type ModelResponse = Vec<Model>;

#[derive(Clone, Debug, Deserialize)]
pub struct Model {
    model_id: String,
    name: String,
    can_be_finetuned: bool,
    can_do_text_to_speech: bool,
    can_do_voice_conversion: bool,
    can_use_style: bool,
    can_use_speaker_boost: bool,
    serves_pro_voices: bool,
    token_cost_factor: f32,
    description: String,
    requires_alpha_access: bool,
    max_characters_request_free_user: f32,
    max_characters_request_subscribed_user: f32,
    maximum_text_length_per_request: f32,
    languages: Vec<Language>,
}

#[derive(Clone, Debug, Deserialize)]
struct Language {
    language_id: String,
    name: String,
}
