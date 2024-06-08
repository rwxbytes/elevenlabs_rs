pub use crate::client::{Result, BASE_URL};
pub use bytes::Bytes;
pub use reqwest::{multipart::Form, Method, Response, Url};
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;

pub mod dubbing;
pub mod history;
pub mod models;
pub mod samples;
pub mod tts;
pub mod user;
pub mod voice;
pub mod voice_generation;
pub mod voice_library;
pub mod pronunciation;
pub mod sts;

#[allow(async_fn_in_trait)]
pub trait Endpoint {
    type ResponseBody;

    fn method(&self) -> Method;
    fn json_request_body(&self) -> Option<Result<Value>> {
        None
    }
    fn multipart_request_body(&self) -> Option<Result<Form>> {
        None
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody>;
    fn url(&self) -> Url;
}

// TODO: move to a more appropriate module, make shared
#[derive(Clone, Debug, Deserialize)]
pub struct Status {
    status: String,
}
