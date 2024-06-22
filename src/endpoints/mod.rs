pub use crate::client::{Result, BASE_URL};
pub use bytes::Bytes;
pub use reqwest::{multipart::{Form,Part}, Method, Response, Url, };
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;

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
pub mod voice_generation;
pub mod voice_library;
pub mod shared;

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


