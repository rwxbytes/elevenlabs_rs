use crate::{
    api::{Client, ClientBuilder},
    error::Error,
    prelude::*,
};
use comparable::*;
use http_body_util::Empty;
use hyper::body::Bytes;
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

pub async fn get_models() -> Result<Vec<Model>> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(PATH)?
        .method(GET)?
        .header("ACCEPT", "application/json")?
        .build()?;
    let data = c.send_request(Empty::<Bytes>::new()).await?;
    let models = serde_json::from_slice::<Vec<Model>>(&data)?;
    Ok(models)
}
