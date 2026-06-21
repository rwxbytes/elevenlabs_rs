pub(crate) use crate::client::Result;
#[cfg(any(feature = "admin", feature = "genai"))]
pub(crate) use crate::shared::response_bodies::*;
#[cfg(any(feature = "admin", feature = "convai", feature = "genai"))]
pub(crate) use crate::shared::url::*;
#[cfg(any(feature = "admin", feature = "convai", feature = "genai"))]
pub(crate) use bytes::Bytes;
#[cfg(any(feature = "admin", feature = "convai", feature = "genai"))]
pub(crate) use reqwest::multipart::Part;
pub(crate) use reqwest::{multipart::Form, Method, Response, Url};
#[cfg(any(feature = "admin", feature = "convai", feature = "genai"))]
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

    fn base_url(&self) -> &str {
        Self::BASE_URL
    }

    fn url(&self) -> Url {
        let mut url = self
            .base_url()
            .parse::<Url>()
            .expect("endpoint base URL must be valid");

        let path = Self::PATH.trim_start_matches('/');
        let path_params = self.path_params();
        let mut segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| {
                path_params
                    .iter()
                    .find_map(|(placeholder, value)| (*placeholder == segment).then_some(*value))
                    .unwrap_or(segment)
            })
            .collect();

        if path.ends_with('/') {
            segments.push("");
        }

        {
            let mut url_segments = url
                .path_segments_mut()
                .expect("endpoint base URL must support path segments");
            url_segments.clear();
            url_segments.extend(segments);
        }

        if let Some(query_params) = self.query_params() {
            url.query_pairs_mut().extend_pairs(query_params);
        }

        url
    }
}
