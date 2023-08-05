use crate::{api::ClientBuilder, prelude::*};
use comparable::*;
use http_body_util::Empty;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

const PATH: &str = "/models";

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn get_models_is_fetching_all_available_models() {
        let mut want = Model::mono();
        let models = get_models().await.unwrap();
        let mut iter = models.iter();
        let mut got = iter.next().unwrap();
        let mut identity = want.comparison(got);
        if !identity.is_unchanged() {
            panic!("identity: {:#?}", identity);
        }
        want = Model::multi();
        got = iter.next().unwrap();
        identity = want.comparison(got);
        if !identity.is_unchanged() {
            panic!("identity: {:#?}", identity);
        }
        // There ought be only two models as of now.
        assert!(iter.next().is_none());
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Comparable)]
pub struct Model {
    pub model_id: String,
    pub name: String,
    pub description: String,
    pub token_cost_factor: f64,
}

impl Model {
    pub fn mono() -> Self {
        Self {
            model_id: "eleven_monolingual_v1".to_string(),
            name: "Eleven English v1".to_string(),
            description: "Use our standard English language model to generate speech in a variety of voices, styles and moods.".to_string(),
            token_cost_factor: 1.0,
        }
    }
    pub fn multi() -> Self {
        Self {
            model_id: "eleven_multilingual_v1".to_string(),
            name: "Eleven Multilingual v1".to_string(),
            description: "Generate lifelike speech in multiple languages and create content that resonates with a broader audience. ".to_string(),
            token_cost_factor: 1.0,
        }
    }
}

/// Get a list of all available models.
pub async fn get_models() -> Result<Vec<Model>> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(PATH)?
        .method(GET)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let resp = c.send_request(Empty::<Bytes>::new()).await?;
    let models = serde_json::from_slice::<Vec<Model>>(&resp)?;
    Ok(models)
}
