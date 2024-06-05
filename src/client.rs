use crate::endpoints::Endpoint;
use crate::error::Error::ClientSendRequestError;
use crate::error::{
    Code4xx, ElevenLabs400, ElevenLabsClientError, ElevenLabsError, ElevenLabsServerError,
};
use reqwest;
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;
use reqwest::Response;
use reqwest::StatusCode;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const BASE_URL: &str = "https://api.elevenlabs.io";
const XI_API_KEY_HEADER: &str = "xi-api-key";
const APPLICATION_JSON: &str = "application/json";
const MULTIPART_FORM_DATA: &str = "multipart/form-data";

pub struct ElevenLabsClient {
    inner: reqwest::Client,
    api_key: String,
}

impl ElevenLabsClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            api_key: std::env::var("ELEVEN_API_KEY")?,
        })
    }
    pub async fn hit<T: Endpoint>(&self, endpoint: T) -> Result<T::ResponseBody> {
        let init = self
            .inner
            .request(endpoint.method(), endpoint.url())
            .header(XI_API_KEY_HEADER, &self.api_key);

        let method = endpoint.method();
        let resp: Response;

        match method {
            Method::GET | Method::DELETE => {
                resp = init.send().await?;
            }
            Method::POST => {
                if endpoint.json_request_body().is_some() {
                    resp = init
                        .header(CONTENT_TYPE, APPLICATION_JSON)
                        // TODO: This should be a custom error
                        .json(&endpoint.json_request_body().unwrap()?)
                        .send()
                        .await?;
                } else if endpoint.multipart_request_body().is_some() {
                    resp = init
                        //.header(CONTENT_TYPE, MULTIPART_FORM_DATA)
                        // TODO: This should be a custom error
                        .multipart(endpoint.multipart_request_body().unwrap()?)
                        .send()
                        .await?;
                } else {
                    panic!("a post request must have a json or multipart body for ElevenLabs API");
                }
            }
            _ => {
                panic!("Unsupported method for ElevenLabs API");
            }
        }
        endpoint.response_body(handle_http_error(resp).await?).await
    }
}

async fn handle_http_error(resp: Response) -> Result<Response> {
    if resp.status().is_server_error() {
        let server_error = resp.json::<ElevenLabsServerError>().await?;
        return Err(Box::new(server_error));
    }

    if resp.status().is_client_error() {
        return match resp.status() {
            StatusCode::UNPROCESSABLE_ENTITY => {
                let client_error = resp.json::<ElevenLabsClientError>().await?;
                Err(Box::new(client_error))
            }
            StatusCode::BAD_REQUEST => Err(Box::new(ElevenLabsError::BadRequest(
                resp.json::<ElevenLabs400>().await?,
            ))),
            _ => Err(Box::new(ElevenLabsError::Code4xx(
                resp.json::<Code4xx>().await?,
            ))),
        };
    }

    if !resp.status().is_success() {
        return Err(Box::new(ClientSendRequestError(resp.json().await?)));
    }
    Ok(resp)
}
