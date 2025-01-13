//! Phone numbers endpoints.

use super::*;

/// Import Phone Number from Twilio configuration
///
///
/// # Example
///
/// ```
/// use elevenlabs_rs::endpoints::convai::phone_numbers::{CreatePhoneNumber, CreatePhoneNumberBody};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let body = CreatePhoneNumberBody::from_twilio_env("number", "label")?;
///    let endpoint = CreatePhoneNumber::new(body);
///    let resp = client.hit(endpoint).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See [Create Phone Number API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/phone-numbers/create-phone-number)
pub struct CreatePhoneNumber {
    pub body: CreatePhoneNumberBody,
}

impl CreatePhoneNumber {
    pub fn new(body: CreatePhoneNumberBody) -> Self {
        Self { body }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreatePhoneNumberBody {
    pub phone_number: String,
    pub provider: PhoneNumberProvider,
    pub label: String,
    pub sid: String,
    pub token: String,
}

impl CreatePhoneNumberBody {
    pub fn from_twilio_env(
        phone_number: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            phone_number: phone_number.into(),
            provider: PhoneNumberProvider::Twilio,
            label: label.into(),
            sid: std::env::var("TWILIO_ACCOUNT_SID").map_err(|_| "TWILIO_ACCOUNT_SID not set")?,
            token: std::env::var("TWILIO_AUTH_TOKEN").map_err(|_| "TWILIO_AUTH_TOKEN not set")?,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PhoneNumberProvider {
    Twilio,
}

impl TryFrom<&CreatePhoneNumberBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &CreatePhoneNumberBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

impl ElevenLabsEndpoint for CreatePhoneNumber {
    const PATH: &'static str = "/v1/convai/phone-numbers/create";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreatePhoneNumberResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreatePhoneNumberResponse {
    pub phone_number_id: String,
}
