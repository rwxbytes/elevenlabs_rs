//! Phone numbers endpoints.

use super::*;

/// Import Phone Number from Twilio configuration
///
///
/// # Example
///
/// ```no_run
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
#[serde(untagged)]
pub enum CreatePhoneNumberBody {
    Twilio {
        phone_number: String,
        label: String,
        sid: String,
        token: String,
        provider: PhoneNumberProvider,
    },
    SipTrunk {
        phone_number: String,
        label: String,
        termination_uri: String,
        provider: PhoneNumberProvider,
        credentials: Option<SipTrunkCredentials>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct SipTrunkCredentials {
    pub username: String,
    pub password: String,
}

impl CreatePhoneNumberBody {
    pub fn new_twilio(
        phone_number: impl Into<String>,
        label: impl Into<String>,
        sid: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self::Twilio {
            phone_number: phone_number.into(),
            label: label.into(),
            sid: sid.into(),
            token: token.into(),
            provider: PhoneNumberProvider::Twilio,
        }
    }
    pub fn from_twilio_env(
        phone_number: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self::Twilio {
            phone_number: phone_number.into(),
            provider: PhoneNumberProvider::Twilio,
            label: label.into(),
            sid: std::env::var("TWILIO_ACCOUNT_SID").map_err(|_| "TWILIO_ACCOUNT_SID not set")?,
            token: std::env::var("TWILIO_AUTH_TOKEN").map_err(|_| "TWILIO_AUTH_TOKEN not set")?,
        })
    }

    pub fn new_sip_trunk(
        phone_number: impl Into<String>,
        label: impl Into<String>,
        termination_uri: impl Into<String>,
        credentials: Option<SipTrunkCredentials>,
    ) -> Self {
        Self::SipTrunk {
            phone_number: phone_number.into(),
            label: label.into(),
            termination_uri: termination_uri.into(),
            provider: PhoneNumberProvider::SipTrunk,
            credentials,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhoneNumberProvider {
    Twilio,
    SipTrunk,
}

impl TryFrom<&CreatePhoneNumberBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &CreatePhoneNumberBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

impl crate::endpoints::sealed::Sealed for CreatePhoneNumber {}

impl ElevenLabsEndpoint for CreatePhoneNumber {
    const PATH: &'static str = "/v1/convai/phone-numbers";

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
/// ```no_run
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

impl crate::endpoints::sealed::Sealed for ListPhoneNumbers {}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssignedAgent {
    pub agent_id: String,
    pub agent_name: String,
}

/// Retrieve Phone Number details by ID
///
///
/// # Example
///
/// ```no_run
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

impl crate::endpoints::sealed::Sealed for GetPhoneNumber {}

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
/// ```no_run
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
    type Error = crate::error::Error;

    fn try_from(body: &UpdatePhoneNumberBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

impl crate::endpoints::sealed::Sealed for UpdatePhoneNumber {}

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
/// ```no_run
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

impl crate::endpoints::sealed::Sealed for DeletePhoneNumber {}

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

/// Get the SIP log messages for a phone number.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::phone_numbers::{GetSipMessages, SipMessagesQuery};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint =
///         GetSipMessages::new("phone_number_id").with_query(SipMessagesQuery::default().with_page_size(50));
///     let resp = client.hit(endpoint).await?;
///     for message in &resp.sip_messages {
///         println!("{}: {}", message.call_id, message.raw_message);
///     }
///     Ok(())
/// }
/// ```
/// See [Get SIP Messages API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/phone-numbers/get-sip-messages)
#[derive(Clone, Debug)]
pub struct GetSipMessages {
    phone_number_id: String,
    query: Option<SipMessagesQuery>,
}

impl GetSipMessages {
    pub fn new(phone_number_id: impl Into<String>) -> Self {
        Self {
            phone_number_id: phone_number_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: SipMessagesQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetSipMessages`].
#[derive(Clone, Debug, Default)]
pub struct SipMessagesQuery {
    params: QueryValues,
}

impl SipMessagesQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetSipMessages {}

impl ElevenLabsEndpoint for GetSipMessages {
    const PATH: &'static str = "/v1/convai/phone-numbers/:phone_number_id/sip-messages";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetSipMessagesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.phone_number_id.and_param(PathParam::PhoneNumberID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of SIP log messages.
#[derive(Clone, Debug, Deserialize)]
pub struct GetSipMessagesResponse {
    pub sip_messages: Vec<SipLogMessage>,
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

/// A single SIP log message.
#[derive(Clone, Debug, Deserialize)]
pub struct SipLogMessage {
    pub call_id: String,
    pub phone_numbers: Vec<String>,
    pub local_address: String,
    pub remote_address: String,
    pub transport: String,
    pub raw_message: String,
    pub error_message: String,
    pub direction: SipLogMessageDirection,
    pub created_at_unix_micro: i64,
}

/// The direction of a SIP log message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SipLogMessageDirection {
    In,
    Out,
}
