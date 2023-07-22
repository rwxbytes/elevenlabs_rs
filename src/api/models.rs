use crate::prelude::*;
use comparable::*;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};

#[derive(Debug, Serialize, Deserialize, Clone, Comparable)]
pub struct Model {
    pub model_id: String,
    pub name: String,
    pub description: String,
    pub token_cost_factor: f64,
}

pub fn parse_models_resp(data: &str) -> Result<Vec<Model>> {
    //let models_resp = json!(data);
    let models_resp: serde_json::Value = serde_json::from_str(data)?;
    //todo!("Fix token_cost_factor unwrap");
    let models = vec![
        Model {
            model_id: models_resp[0]["model_id"].as_str().unwrap().to_string(),
            name: models_resp[0]["name"].as_str().unwrap().to_string(),
            description: models_resp[0]["description"].as_str().unwrap().to_string(),
            token_cost_factor: models_resp[0]["token_cost_factor"].as_f64().unwrap(),
        },
        Model {
            model_id: models_resp[1]["model_id"].as_str().unwrap().to_string(),
            name: models_resp[1]["name"].as_str().unwrap().to_string(),
            description: models_resp[1]["description"].as_str().unwrap().to_string(),
            token_cost_factor: models_resp[1]["token_cost_factor"].as_f64().unwrap(),
        },
    ];
    Ok(models)
}
