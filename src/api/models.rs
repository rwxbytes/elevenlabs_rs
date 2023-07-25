use crate::{
    api::{Client, ClientBuilder},
    error::Error,
    prelude::*,
};
use comparable::*;
use serde::{Deserialize, Serialize};

const GET: &str = "GET";
const PATH: &str = "/models";

#[derive(Debug, Serialize, Deserialize, Clone, Comparable)]
pub struct Model {
    pub model_id: String,
    pub name: String,
    pub description: String,
    pub token_cost_factor: f64,
}

pub fn build_models_client() -> Result<Client> {
    let mut cb = ClientBuilder::new()?;
    let c = cb
        .path(PATH)?
        .method(GET)?
        .header("ACCEPT", "application/json")?
        .build()?;
    Ok(c)
}

pub fn get_models() -> Result<Vec<Model>> {
    let c = build_models_client()?;
    // let req = c.build_request()?;
    todo!()
}

pub fn parse_models_resp(data: &str) -> Result<Vec<Model>> {
    let models_resp: serde_json::Value = serde_json::from_str(data)?;
    if models_resp.as_array().is_none() {
        return Err(Box::new(Error::InvalidApiResponse(
            "models api response is not an array".to_string(),
        )));
    } else if models_resp.as_array().unwrap().len() < 1 {
        return Err(Box::new(Error::InvalidApiResponse(
            "models response is an empty array, want at least one model".to_string(),
        )));
    }
    //todo!("Fix token_cost_factor unwrap");
    let mut models: Vec<Model> = Vec::new();
    for model in models_resp.as_array().unwrap() {
        models.push(Model {
            model_id: model["model_id"].as_str().unwrap().to_string(),
            name: model["name"].as_str().unwrap().to_string(),
            description: model["description"].as_str().unwrap().to_string(),
            token_cost_factor: model["token_cost_factor"].as_f64().unwrap(),
        });
    }
    Ok(models)
}
