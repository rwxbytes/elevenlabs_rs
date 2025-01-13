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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

/// Retrieve all Phone Numbers
///
///
/// # Example
///
/// ```
/// use elevenlabs_rs::endpoints::convai::phone_numbers::ListPhoneNumbers;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let resp = client.hit(ListPhoneNumbers).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See [List Phone Numbers API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/phone-numbers/get-phone-numbers)
#[derive(Clone, Debug)]
pub struct ListPhoneNumbers;

impl ElevenLabsEndpoint for ListPhoneNumbers {
    const PATH: &'static str = "/v1/convai/phone-numbers";

    const METHOD: Method = Method::GET;

    type ResponseBody = Vec<PhoneNumberResponse>;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PhoneNumberResponse {
    pub phone_number_id: String,
    pub phone_number: String,
    pub provider: PhoneNumberProvider,
    pub label: String,
    pub assigned_agent: Option<AssignedAgent>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AssignedAgent {
    pub agent_id: String,
    pub agent_name: String,
}

/// Retrieve Phone Number details by ID
///
///
/// # Example
///
/// ```
/// use elevenlabs_rs::endpoints::convai::phone_numbers::GetPhoneNumber;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let client = ElevenLabsClient::from_env()?;
///   let endpoint = GetPhoneNumber::new("phone_number_id");
///   let resp = client.hit(endpoint).await?;
///   println!("{:?}", resp);
///   Ok(())
/// }
/// ```
/// See [Get Phone Number API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/phone-numbers/get-phone-number)
#[derive(Clone, Debug)]
pub struct GetPhoneNumber {
    pub phone_number_id: String,
}

impl GetPhoneNumber {
    pub fn new(phone_number_id: impl Into<String>) -> Self {
        Self {
            phone_number_id: phone_number_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetPhoneNumber {
    const PATH: &'static str = "/v1/convai/phone-numbers/:phone_number_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = PhoneNumberResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.phone_number_id.and_param(PathParam::PhoneNumberID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Update Phone Number details by ID
///
///
/// # Example
///
/// ```
/// use elevenlabs_rs::endpoints::convai::phone_numbers::{UpdatePhoneNumber, UpdatePhoneNumberBody};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let client = ElevenLabsClient::from_env()?;
///   let body = UpdatePhoneNumberBody::new("agent_id");
///   let endpoint = UpdatePhoneNumber::new("phone_number_id", body);
///   let resp = client.hit(endpoint).await?;
///   println!("{:?}", resp);
///   Ok(())
/// }
/// ```
/// See [Update Phone Number API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/phone-numbers/update-phone-number)
#[derive(Clone, Debug)]
pub struct UpdatePhoneNumber {
    pub phone_number_id: String,
    pub body: UpdatePhoneNumberBody,
}

impl UpdatePhoneNumber {
    pub fn new(phone_number_id: impl Into<String>, body: UpdatePhoneNumberBody) -> Self {
        Self {
            phone_number_id: phone_number_id.into(),
            body,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdatePhoneNumberBody {
    pub agent_id: String,
}

impl UpdatePhoneNumberBody {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
        }
    }
}

impl TryFrom<&UpdatePhoneNumberBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &UpdatePhoneNumberBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

impl ElevenLabsEndpoint for UpdatePhoneNumber {
    const PATH: &'static str = "/v1/convai/phone-numbers/:phone_number_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = PhoneNumberResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.phone_number_id.and_param(PathParam::PhoneNumberID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Delete Phone Number by ID
///
///
/// # Example
///
/// ```
/// use elevenlabs_rs::endpoints::convai::phone_numbers::DeletePhoneNumber;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let client = ElevenLabsClient::from_env()?;
///   let endpoint = DeletePhoneNumber::new("phone_number_id");
///   let resp = client.hit(endpoint).await?;
///   println!("{:?}", resp);
///   Ok(())
/// }
/// ```
/// See [Delete Phone Number API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/phone-numbers/delete-phone-number)
#[derive(Clone, Debug)]
pub struct DeletePhoneNumber {
    pub phone_number_id: String,
}

impl DeletePhoneNumber {
    pub fn new(phone_number_id: impl Into<String>) -> Self {
        Self {
            phone_number_id: phone_number_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for DeletePhoneNumber {
    const PATH: &'static str = "/v1/convai/phone-numbers/:phone_number_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.phone_number_id.and_param(PathParam::PhoneNumberID)]
    }

    async fn response_body(self, _: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}
