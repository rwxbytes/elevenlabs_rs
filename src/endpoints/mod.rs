pub use crate::client::{Result, BASE_URL};
pub(crate) use crate::shared::identifiers::*;
pub(crate) use crate::shared::path_segments::*;
pub use crate::shared::query_params::*;
pub use crate::shared::response_bodies::*;
pub use bytes::Bytes;
pub use reqwest::{
    multipart::{Form, Part},
    Method, Response, Url,
};
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
pub mod audio_isolation;

#[allow(async_fn_in_trait)]
pub trait Endpoint {
    type ResponseBody;

    fn method(&self) -> Method;
    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Empty)
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody>;
    fn url(&self) -> Url;
}

pub enum RequestBody {
    Json(Value),
    Multipart(Form),
    Empty,
}
