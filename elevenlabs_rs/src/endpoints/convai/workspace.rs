//! Convai workspace endpoints

use super::{agents::RequestHeaders, *};
use crate::endpoints::convai::agents::PhoneNumber;
use crate::shared::AccessLevel;
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

impl crate::endpoints::sealed::Sealed for GetSettings {}

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

impl crate::endpoints::sealed::Sealed for UpdateSettings {}

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
    type Error = crate::error::Error;

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

impl crate::endpoints::sealed::Sealed for GetSecrets {}

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

impl crate::endpoints::sealed::Sealed for CreateSecret {}

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

/// Delete a workspace secret if it’s not in use
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::workspace::DeleteSecret;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let _resp = client.hit(DeleteSecret::new("secret_id")).await?;
///    Ok(())
/// }
/// ```
/// See [Delete Secret API reference](https://elevenlabs.io/docs/api-reference/workspace/delete-secret)
#[derive(Debug)]
pub struct DeleteSecret {
    pub secret_id: String,
}
impl DeleteSecret {
    pub fn new(secret_id: impl Into<String>) -> Self {
        Self {
            secret_id: secret_id.into(),
        }
    }
}

type DeleteSecretResponse = ();

impl crate::endpoints::sealed::Sealed for DeleteSecret {}

impl ElevenLabsEndpoint for DeleteSecret {
    const PATH: &'static str = "v1/convai/secrets/:secret_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = DeleteSecretResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.secret_id.and_param(PathParam::SecretID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
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
    pub phone_numbers: Vec<PhoneNumber>,
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
    type Error = crate::error::Error;

    fn try_into(self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(self)?))
    }
}

/// The kind of resource that can depend on a secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecretDependencyResourceType {
    Tools,
    Agents,
    PhoneNumbers,
}

impl SecretDependencyResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tools => "tools",
            Self::Agents => "agents",
            Self::PhoneNumbers => "phone_numbers",
        }
    }
}

/// Get the resources of a given type that depend on a secret.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::workspace::{
///     GetSecretDependencies, SecretDependencyResourceType,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint =
///         GetSecretDependencies::new("secret_id", SecretDependencyResourceType::Tools);
///     let resp = client.hit(endpoint).await?;
///     println!("{} dependencies", resp.dependencies.len());
///     Ok(())
/// }
/// ```
/// See [Get Secret Dependencies API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/workspace/get-secret-dependencies)
#[derive(Clone, Debug)]
pub struct GetSecretDependencies {
    secret_id: String,
    resource_type: SecretDependencyResourceType,
    query: Option<SecretDependenciesQuery>,
}

impl GetSecretDependencies {
    pub fn new(secret_id: impl Into<String>, resource_type: SecretDependencyResourceType) -> Self {
        Self {
            secret_id: secret_id.into(),
            resource_type,
            query: None,
        }
    }

    pub fn with_query(mut self, query: SecretDependenciesQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetSecretDependencies`].
#[derive(Clone, Debug, Default)]
pub struct SecretDependenciesQuery {
    params: QueryValues,
}

impl SecretDependenciesQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetSecretDependencies {}

impl ElevenLabsEndpoint for GetSecretDependencies {
    const PATH: &'static str = "/v1/convai/secrets/:secret_id/dependencies/:resource_type";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetSecretDependenciesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.secret_id.and_param(PathParam::SecretID),
            (PathParam::ResourceType.into(), self.resource_type.as_str()),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of resources that depend on a secret. The dependency shape varies by
/// resource type, so each is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct GetSecretDependenciesResponse {
    pub dependencies: Vec<Value>,
    pub next_cursor: Option<String>,
}

/// Get the ConvAI dashboard settings for the workspace.
///
/// See [Get Dashboard Settings API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/workspace/get-dashboard-settings)
#[derive(Clone, Debug, Default)]
pub struct GetDashboardSettings;

impl crate::endpoints::sealed::Sealed for GetDashboardSettings {}

impl ElevenLabsEndpoint for GetDashboardSettings {
    const PATH: &'static str = "/v1/convai/settings/dashboard";

    const METHOD: Method = Method::GET;

    type ResponseBody = DashboardSettings;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Update the ConvAI dashboard settings for the workspace.
///
/// See [Update Dashboard Settings API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/workspace/update-dashboard-settings)
#[derive(Clone, Debug)]
pub struct UpdateDashboardSettings {
    body: DashboardSettings,
}

impl UpdateDashboardSettings {
    pub fn new(body: DashboardSettings) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateDashboardSettings {}

impl ElevenLabsEndpoint for UpdateDashboardSettings {
    const PATH: &'static str = "/v1/convai/settings/dashboard";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = DashboardSettings;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The ConvAI dashboard settings.
///
/// Each chart is a polymorphic configuration object, preserved as raw JSON.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DashboardSettings {
    #[serde(default)]
    pub charts: Vec<Value>,
}

impl DashboardSettings {
    pub fn new(charts: impl IntoIterator<Item = Value>) -> Self {
        Self {
            charts: charts.into_iter().collect(),
        }
    }
}
