pub use crate::client::{Result, BASE_URL};
pub(crate) use crate::shared::identifiers::*;
pub(crate) use crate::shared::path_segments::*;
pub use crate::shared::query_params::*;
pub use crate::shared::response_bodies::*;
pub use base64::prelude::{Engine, BASE64_STANDARD};
pub use bytes::Bytes;
pub use reqwest::{
    multipart::{Form, Part},
    Method, Response, Url,
};
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub use validator::Validate;

pub mod audio_native;
pub mod dubbing;
pub mod history;
pub mod models;
pub mod projects;
pub mod pronunciation;
pub mod samples;
pub mod sound_generation;
pub mod sts;
pub mod tts;
pub mod user;
pub mod voice;
#[deprecated(since = "0.3.2 ", note = "Use `voice_design` instead")]
pub mod voice_generation;
pub mod voice_library;
pub mod audio_isolation;
pub mod voice_design;
pub mod convai;

#[allow(async_fn_in_trait)]
pub trait Endpoint {
    const PATH: &'static str;
    const METHOD: Method;
    type ResponseBody;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Empty)
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody>;
    fn url(&self) -> Url;
}

#[derive(Debug)]
pub enum RequestBody {
    Json(Value),
    Multipart(Form),
    Empty,
}

trait PathAndQueryParams {
    /// generate key:value pair for path replacements (e.g {user_id}) to `user_id_123`.
    fn get_path_params(&self) -> Vec<(&'static str, String)>;

    ///generate vector with queries params, in `?cursor=...&page_size` fashion
    fn get_query_params(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

fn build_url<T: PathAndQueryParams>(path: &str, params: T) -> Url {
    let mut url = BASE_URL.parse::<Url>().unwrap();

    let mut built_path = path.to_string();
    for (k,v) in params.get_path_params(){
        built_path = built_path.replace(k, &v);
    }

    url.set_path(&built_path);

    let query_string = params.get_query_params()
        .into_iter()
        .map(|(k,v)| format!("{}={}", k,v) ).collect::<Vec<_>>()
        .join("&");

    if !query_string.is_empty() {
        url.set_query(Some(query_string.as_str()))
    }

    url
}
