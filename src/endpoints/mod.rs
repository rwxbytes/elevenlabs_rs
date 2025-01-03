pub(crate) use crate::client::Result;
pub(crate) use crate::shared::identifiers::*;
pub(crate) use crate::shared::query_params::*;
pub(crate) use crate::shared::response_bodies::*;
pub(crate) use base64::prelude::{Engine, BASE64_STANDARD};
pub(crate) use bytes::Bytes;
pub(crate) use reqwest::{
    multipart::{Form, Part},
    Method, Response, Url,
};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_json::Value;

#[cfg(feature = "admin")]
pub mod admin;
#[cfg(feature = "convai")]
pub mod convai;
#[cfg(feature = "genai")]
pub mod genai;

type QueryValues = Vec<(&'static str, String)>;

#[derive(Debug)]
pub enum RequestBody {
    Json(Value),
    Multipart(Form),
    Empty,
}

#[allow(async_fn_in_trait)]
pub trait ElevenLabsEndpoint {

    const BASE_URL: &'static str = "https://api.elevenlabs.io";

    const PATH: &'static str;

    const METHOD: Method;

    type ResponseBody;

    fn query_params(&self) -> Option<QueryValues> {
        None
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Empty)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody>;

    fn url(&self) -> Url {
        let mut url = Self::BASE_URL.parse::<Url>().unwrap();

        let mut path = Self::PATH.to_string();

        for (placeholder, id) in self.path_params() {
            path = path.replace(placeholder, id);
        }

        url.set_path(&path);

        if let Some(query_params) = self.query_params() {
            let query_string = query_params
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");

            url.set_query(Some(&query_string))
        }

        url
    }
}
