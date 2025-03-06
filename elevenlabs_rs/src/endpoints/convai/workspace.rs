//! Convai workspace endpoints

use super::{agents::{AccessLevel ,RequestHeaders}, *};
use std::collections::HashMap;

/// Retrieve Convai settings for the workspace
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::workspace::GetSettings;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let resp = client.hit(GetSettings).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See [Get Setting API reference](https://elevenlabs.io/docs/api-reference/workspace/get-settings)
#[derive(Clone, Debug, Serialize)]
pub struct GetSettings;

impl ElevenLabsEndpoint for GetSettings {
    const PATH: &'static str = "v1/convai/settings";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetSettingsResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetSettingsResponse {
    pub conversation_initiation_client_data_webhook:
        Option<ConversationInitiationClientDataWebhook>,
    pub webhooks: Option<Webhooks>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationInitiationClientDataWebhook {
    pub url: String,
    pub request_headers: Option<HashMap<String, RequestHeaders>>,
}

impl ConversationInitiationClientDataWebhook {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            request_headers: Some(HashMap::new()),
        }
    }

    pub fn with_request_headers(
        mut self,
        request_headers: HashMap<String, RequestHeaders>,
    ) -> Self {
        self.request_headers = Some(request_headers);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Webhooks {
    pub post_call_webhook_id: Option<String>,
}

impl Webhooks {
    pub fn new(post_call_webhook_id: impl Into<String>) -> Self {
        Self {
            post_call_webhook_id: Some(post_call_webhook_id.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UsedTools {
    pub r#type: String,
    pub access_level: Option<AccessLevel>,
    pub created_at_unix_secs: Option<u64>,
    pub id: Option<String>,
    pub name: Option<String>,
}

/// Update Convai settings for the workspace
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::workspace::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let init_webhook = ConversationInitiationClientDataWebhook::new("https://example.com/webhook");
///     let body = UpdateSettingsBody::default()
///        .with_initiation_webhook(init_webhook);
///     let endpoint = UpdateSettings::new(body);
///     let resp = client.hit(endpoint).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See [Update Settings API reference](https://elevenlabs.io/docs/api-reference/workspace/update-settings)
#[derive(Clone)]
pub struct UpdateSettings {
    pub body: UpdateSettingsBody,
}

impl UpdateSettings {
    pub fn new(body: UpdateSettingsBody) -> Self {
        Self { body }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateSettingsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_initiation_client_data_webhook:
        Option<ConversationInitiationClientDataWebhook>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Webhooks>,
}

impl UpdateSettingsBody {
    //pub fn new(secrets: Vec<Secret>) -> Self {
    //    Self {
    //        conversation_initiation_client_data_webhook: None,
    //        webhooks: None,
    //    }
    //}

    pub fn with_initiation_webhook(
        mut self,
        webhook: ConversationInitiationClientDataWebhook,
    ) -> Self {
        self.conversation_initiation_client_data_webhook = Some(webhook);
        self
    }

    pub fn with_webhooks(mut self, webhooks: Webhooks) -> Self {
        self.webhooks = Some(webhooks);
        self
    }
}

type UpdateSettingsResponse = GetSettingsResponse;

impl ElevenLabsEndpoint for UpdateSettings {
    const PATH: &'static str = "v1/convai/settings";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = UpdateSettingsResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

impl TryInto<RequestBody> for &UpdateSettingsBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_into(self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(self)?))
    }
}

/// Get all secrets for the workspace
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::workspace::GetSecrets;
///
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let resp = client.hit(GetSecrets).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See [Get Secrets API reference](https://elevenlabs.io/docs/api-reference/workspace/get-secrets)
#[derive(Clone, Debug, Serialize)]
pub struct GetSecrets;

impl ElevenLabsEndpoint for GetSecrets {
    const PATH: &'static str = "v1/convai/secrets";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetSecretsResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetSecretsResponse {
    pub secrets: Vec<Secret>,
}

/// Create a new secret for the workspace
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::workspace::CreateSecret;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint = CreateSecret::new("name", "value");
///     let resp = client.hit(endpoint).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See [Create Secret API reference](https://elevenlabs.io/docs/api-reference/workspace/create-secret)
#[derive(Clone, Debug)]
pub struct CreateSecret {
    pub body: CreateSecretBody,
}

impl CreateSecret {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        let secret = Secret::new(name, value);
        Self {
            body: CreateSecretBody { secret },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateSecretBody {
    #[serde(flatten)]
    pub secret: Secret,
}

type CreateSecretResponse = Secret;

impl ElevenLabsEndpoint for CreateSecret {
    const PATH: &'static str = "v1/convai/secrets";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateSecretResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum Secret {
    New {
        name: String,
        value: String,
        #[serde(default = "SecretType::new")]
        r#type: SecretType,
        used_by: Option<UsedBy>,
    },
    Stored {
        name: String,
        secret_id: String,
        #[serde(default = "SecretType::stored")]
        r#type: SecretType,
        used_by: Option<UsedBy>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UsedBy {
    pub tools: Vec<UsedTools>,
    pub agent_tools: Vec<AgentTool>,
    pub others: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentTool {
    pub agent_id: String,
    pub agent_name: String,
    pub r#type: String,
    pub access_level: AccessLevel,
    pub created_at_unix_secs: u64,
    pub used_by: Vec<String>,
}

impl Secret {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Secret::New {
            name: name.into(),
            value: value.into(),
            r#type: SecretType::New,
            used_by: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SecretType {
    New,
    Stored,
}

impl SecretType {
    fn new() -> Self {
        SecretType::New
    }

    fn stored() -> Self {
        SecretType::Stored
    }
}

impl TryInto<RequestBody> for &CreateSecretBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_into(self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(self)?))
    }
}
