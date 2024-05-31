use crate::client::Result;
use serde::{Deserialize, Serialize};

pub mod history;
pub mod models;
pub mod samples;
pub mod tts;
pub mod user;
mod voice;

#[allow(async_fn_in_trait)]
pub trait Endpoint {
    type ResponseBody;

    fn method(&self) -> reqwest::Method;
    fn json_request_body(&self) -> Option<Result<serde_json::Value>> {
        None
    }
    fn multipart_request_body(&self) -> Option<Result<reqwest::multipart::Form>> {
        None
    }
    async fn response_body(self, resp: reqwest::Response) -> Result<Self::ResponseBody>;
    fn url(&self) -> reqwest::Url;
}

#[derive(Clone, Debug, Deserialize)]
pub struct Status {
    status: String,
}
