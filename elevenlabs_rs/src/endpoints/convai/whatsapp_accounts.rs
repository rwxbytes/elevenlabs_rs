//! The Conversational AI WhatsApp-accounts endpoints.
//!
//! WhatsApp accounts connect a WhatsApp Business phone number to the workspace
//! so that agents can place calls and exchange messages. List, fetch, update,
//! and delete them.
//!
//! See the [WhatsApp Accounts API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/whatsapp).

use super::*;

/// A connected WhatsApp account.
#[derive(Clone, Debug, Deserialize)]
pub struct WhatsAppAccount {
    pub phone_number_id: String,
    pub phone_number: String,
    pub phone_number_name: String,
    pub business_account_id: String,
    pub business_account_name: String,
    pub assigned_agent_id: Option<String>,
    pub assigned_agent_name: Option<String>,
    #[serde(default)]
    pub enable_messaging: bool,
    #[serde(default)]
    pub enable_audio_message_response: bool,
    #[serde(default)]
    pub is_token_expired: bool,
}

// =============================================================================
// GET /v1/convai/whatsapp-accounts — List WhatsApp Accounts
// =============================================================================

/// Lists the workspace's WhatsApp accounts.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::whatsapp_accounts::ListWhatsAppAccounts;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let resp = client.hit(ListWhatsAppAccounts::default()).await?;
///     for account in &resp.items {
///         println!("{}: {}", account.phone_number_id, account.phone_number);
///     }
///     Ok(())
/// }
/// ```
/// See [List WhatsApp Accounts API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/whatsapp/list-accounts)
#[derive(Clone, Debug, Default)]
pub struct ListWhatsAppAccounts {
    agent_id: Option<String>,
}

impl ListWhatsAppAccounts {
    /// Filter to the accounts assigned to a specific agent.
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListWhatsAppAccounts {}

impl ElevenLabsEndpoint for ListWhatsAppAccounts {
    const PATH: &'static str = "/v1/convai/whatsapp-accounts";

    const METHOD: Method = Method::GET;

    type ResponseBody = ListWhatsAppAccountsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.agent_id
            .as_ref()
            .map(|agent_id| vec![("agent_id", agent_id.clone())])
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`ListWhatsAppAccounts`].
#[derive(Clone, Debug, Deserialize)]
pub struct ListWhatsAppAccountsResponse {
    pub items: Vec<WhatsAppAccount>,
}

impl IntoIterator for ListWhatsAppAccountsResponse {
    type Item = WhatsAppAccount;
    type IntoIter = std::vec::IntoIter<WhatsAppAccount>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

// =============================================================================
// GET /v1/convai/whatsapp-accounts/{phone_number_id} — Get WhatsApp Account
// =============================================================================

/// Retrieves a WhatsApp account by its phone-number ID.
///
/// See [Get WhatsApp Account API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/whatsapp/get-account)
#[derive(Clone, Debug)]
pub struct GetWhatsAppAccount {
    phone_number_id: String,
}

impl GetWhatsAppAccount {
    pub fn new(phone_number_id: impl Into<String>) -> Self {
        Self {
            phone_number_id: phone_number_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetWhatsAppAccount {}

impl ElevenLabsEndpoint for GetWhatsAppAccount {
    const PATH: &'static str = "/v1/convai/whatsapp-accounts/:phone_number_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = WhatsAppAccount;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.phone_number_id.and_param(PathParam::PhoneNumberID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// PATCH /v1/convai/whatsapp-accounts/{phone_number_id} — Update WhatsApp Account
// =============================================================================

/// Updates a WhatsApp account, e.g. its assigned agent or messaging settings.
///
/// See [Update WhatsApp Account API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/whatsapp/update-account)
#[derive(Clone, Debug)]
pub struct UpdateWhatsAppAccount {
    phone_number_id: String,
    body: UpdateWhatsAppAccountBody,
}

impl UpdateWhatsAppAccount {
    pub fn new(phone_number_id: impl Into<String>, body: UpdateWhatsAppAccountBody) -> Self {
        Self {
            phone_number_id: phone_number_id.into(),
            body,
        }
    }
}

/// Update-WhatsApp-account body. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateWhatsAppAccountBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    assigned_agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_messaging: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_audio_message_response: Option<bool>,
}

impl UpdateWhatsAppAccountBody {
    pub fn with_assigned_agent_id(mut self, assigned_agent_id: impl Into<String>) -> Self {
        self.assigned_agent_id = Some(assigned_agent_id.into());
        self
    }

    pub fn with_enable_messaging(mut self, enable_messaging: bool) -> Self {
        self.enable_messaging = Some(enable_messaging);
        self
    }

    pub fn with_enable_audio_message_response(mut self, enable: bool) -> Self {
        self.enable_audio_message_response = Some(enable);
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateWhatsAppAccount {}

impl ElevenLabsEndpoint for UpdateWhatsAppAccount {
    const PATH: &'static str = "/v1/convai/whatsapp-accounts/:phone_number_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.phone_number_id.and_param(PathParam::PhoneNumberID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

// =============================================================================
// DELETE /v1/convai/whatsapp-accounts/{phone_number_id} — Delete WhatsApp Account
// =============================================================================

/// Deletes a WhatsApp account.
///
/// See [Delete WhatsApp Account API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/whatsapp/delete-account)
#[derive(Clone, Debug)]
pub struct DeleteWhatsAppAccount {
    phone_number_id: String,
}

impl DeleteWhatsAppAccount {
    pub fn new(phone_number_id: impl Into<String>) -> Self {
        Self {
            phone_number_id: phone_number_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteWhatsAppAccount {}

impl ElevenLabsEndpoint for DeleteWhatsAppAccount {
    const PATH: &'static str = "/v1/convai/whatsapp-accounts/:phone_number_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.phone_number_id.and_param(PathParam::PhoneNumberID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}
