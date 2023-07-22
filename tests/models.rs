use std::fs;

use comparable::Comparable;
use elevenlabs::api::models::{parse_models_resp, Model};
use elevenlabs::error::Error;

#[test]
fn parse_models_resp_is_parsing_all_model_json_objs() {
    let data = fs::read_to_string("tests/testdata/models_res.json").expect("reading models.json");
    let models: Vec<Model> = parse_models_resp(&data).expect("parsing models.json");
    let want = Model {
        model_id: "eleven_monolingual_v1".to_string(),
        name: "Eleven English v1".to_string(),
        description: "Use our standard English language model to generate speech in a variety of voices, styles and moods.".to_string(),
        token_cost_factor: 1.0,
    };

    let got = &models[0];

    let identity = want.comparison(&got);

    if !identity.is_unchanged() {
        panic!("{:?}", identity)
    }

    let want = Model {
        model_id: "eleven_multilingual_v1".to_string(),
        name: "Eleven Multilingual v1".to_string(),
        description: "Generate lifelike speech in multiple languages and create content that resonates with a broader audience. ".to_string(),
        token_cost_factor: 1.0,
    };

    let got = &models[1];

    let identity = want.comparison(&got);

    if !identity.is_unchanged() {
        panic!("{:?}", identity)
    }
}

#[test]
fn parse_models_resp_is_returning_invalid_api_response_error_when_given_invalid_json() {
    let data =
        fs::read_to_string("tests/testdata/models_invalid.json").expect("reading models.json");
    let models: Result<Vec<Model>, _> = parse_models_resp(&data);

    if !models.is_err() {
        panic!("parse_models_resp should return an error when given invalid json")
    }
}
