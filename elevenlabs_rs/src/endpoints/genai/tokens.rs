//! Single-use token endpoints.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SingleUseTokenType(String);

impl SingleUseTokenType {
    pub fn realtime_scribe() -> Self {
        Self("realtime_scribe".to_string())
    }

    pub fn tts_websocket() -> Self {
        Self("tts_websocket".to_string())
    }

    pub fn custom(token_type: impl Into<String>) -> Self {
        Self(token_type.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SingleUseTokenType {
    fn from(token_type: &str) -> Self {
        Self::custom(token_type)
    }
}

impl From<String> for SingleUseTokenType {
    fn from(token_type: String) -> Self {
        Self::custom(token_type)
    }
}

impl AsRef<str> for SingleUseTokenType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for SingleUseTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct CreateSingleUseToken {
    token_type: SingleUseTokenType,
}

impl CreateSingleUseToken {
    pub fn new(token_type: impl Into<SingleUseTokenType>) -> Self {
        Self {
            token_type: token_type.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for CreateSingleUseToken {}

impl ElevenLabsEndpoint for CreateSingleUseToken {
    const PATH: &'static str = "/v1/single-use-token/:token_type";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateSingleUseTokenResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.token_type.0.and_param(PathParam::TokenType)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Bytes(Bytes::new()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateSingleUseTokenResponse {
    pub token: String,
}
