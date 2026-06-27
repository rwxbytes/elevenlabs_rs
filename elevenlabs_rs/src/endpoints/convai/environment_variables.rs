//! The Conversational AI environment-variables endpoints.
//!
//! Environment variables hold per-environment values (plain strings, secrets,
//! or auth-connection references) that agents and tools can reference. List,
//! create, fetch, and update them.
//!
//! See the [Environment Variables API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/environment-variables).

use super::*;
use std::collections::HashMap;

/// The type of an environment variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentVariableType {
    String,
    Secret,
    AuthConnection,
}

impl EnvironmentVariableType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Secret => "secret",
            Self::AuthConnection => "auth_connection",
        }
    }
}

/// An environment variable.
///
/// `values` maps an environment name to its value; the value shape depends on
/// the variable `type`, so each value is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct EnvironmentVariable {
    pub id: String,
    pub workspace_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub variable_type: EnvironmentVariableType,
    pub values: HashMap<String, Value>,
    pub created_by_user_id: Option<String>,
    pub created_at_unix_secs: i64,
    pub updated_at_unix_secs: i64,
}

// =============================================================================
// GET /v1/convai/environment-variables — List Environment Variables
// =============================================================================

/// Lists the workspace's environment variables.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::environment_variables::{
///     ListEnvironmentVariables, EnvironmentVariablesQuery,
/// };
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint = ListEnvironmentVariables::default()
///         .with_query(EnvironmentVariablesQuery::default().with_page_size(50));
///     let resp = client.hit(endpoint).await?;
///     for env_var in &resp.environment_variables {
///         println!("{}: {}", env_var.id, env_var.label);
///     }
///     Ok(())
/// }
/// ```
/// See [List Environment Variables API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/environment-variables/list)
#[derive(Clone, Debug, Default)]
pub struct ListEnvironmentVariables {
    query: Option<EnvironmentVariablesQuery>,
}

impl ListEnvironmentVariables {
    pub fn with_query(mut self, query: EnvironmentVariablesQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`ListEnvironmentVariables`].
#[derive(Clone, Debug, Default)]
pub struct EnvironmentVariablesQuery {
    params: QueryValues,
}

impl EnvironmentVariablesQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.params.push(("label", label.into()));
        self
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.params.push(("environment", environment.into()));
        self
    }

    pub fn with_type(mut self, variable_type: EnvironmentVariableType) -> Self {
        self.params
            .push(("type", variable_type.as_str().to_owned()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListEnvironmentVariables {}

impl ElevenLabsEndpoint for ListEnvironmentVariables {
    const PATH: &'static str = "/v1/convai/environment-variables";

    const METHOD: Method = Method::GET;

    type ResponseBody = EnvironmentVariablesList;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of environment variables.
#[derive(Clone, Debug, Deserialize)]
pub struct EnvironmentVariablesList {
    pub environment_variables: Vec<EnvironmentVariable>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl IntoIterator for EnvironmentVariablesList {
    type Item = EnvironmentVariable;
    type IntoIter = std::vec::IntoIter<EnvironmentVariable>;

    fn into_iter(self) -> Self::IntoIter {
        self.environment_variables.into_iter()
    }
}

// =============================================================================
// POST /v1/convai/environment-variables — Create Environment Variable
// =============================================================================

/// Creates an environment variable.
///
/// # Example
/// ```no_run
/// use std::collections::HashMap;
/// use elevenlabs_rs::endpoints::convai::environment_variables::{
///     CreateEnvironmentVariable, CreateEnvironmentVariableRequest,
/// };
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let values = HashMap::from([("production".to_string(), "https://api.example.com".to_string())]);
///     let request = CreateEnvironmentVariableRequest::string("API_BASE_URL", values);
///     let resp = client.hit(CreateEnvironmentVariable::new(request)).await?;
///     println!("{}", resp.id);
///     Ok(())
/// }
/// ```
/// See [Create Environment Variable API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/environment-variables/create)
#[derive(Clone, Debug)]
pub struct CreateEnvironmentVariable {
    body: CreateEnvironmentVariableRequest,
}

impl CreateEnvironmentVariable {
    pub fn new(request: CreateEnvironmentVariableRequest) -> Self {
        Self { body: request }
    }
}

/// A typed request to create an environment variable. The `type` discriminator
/// is added automatically during serialization.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateEnvironmentVariableRequest {
    String {
        label: String,
        values: HashMap<String, String>,
    },
    Secret {
        label: String,
        values: HashMap<String, Value>,
    },
    AuthConnection {
        label: String,
        values: HashMap<String, Value>,
    },
}

impl CreateEnvironmentVariableRequest {
    /// A plain-string environment variable, mapping environment name to value.
    pub fn string(label: impl Into<String>, values: HashMap<String, String>) -> Self {
        Self::String {
            label: label.into(),
            values,
        }
    }

    /// A secret environment variable, mapping environment name to secret config.
    pub fn secret(label: impl Into<String>, values: HashMap<String, Value>) -> Self {
        Self::Secret {
            label: label.into(),
            values,
        }
    }

    /// An auth-connection environment variable, mapping environment name to
    /// auth-connection config.
    pub fn auth_connection(label: impl Into<String>, values: HashMap<String, Value>) -> Self {
        Self::AuthConnection {
            label: label.into(),
            values,
        }
    }
}

impl crate::endpoints::sealed::Sealed for CreateEnvironmentVariable {}

impl ElevenLabsEndpoint for CreateEnvironmentVariable {
    const PATH: &'static str = "/v1/convai/environment-variables";

    const METHOD: Method = Method::POST;

    type ResponseBody = EnvironmentVariable;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// GET /v1/convai/environment-variables/{env_var_id} — Get Environment Variable
// =============================================================================

/// Retrieves an environment variable by ID.
///
/// See [Get Environment Variable API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/environment-variables/get)
#[derive(Clone, Debug)]
pub struct GetEnvironmentVariable {
    env_var_id: String,
}

impl GetEnvironmentVariable {
    pub fn new(env_var_id: impl Into<String>) -> Self {
        Self {
            env_var_id: env_var_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetEnvironmentVariable {}

impl ElevenLabsEndpoint for GetEnvironmentVariable {
    const PATH: &'static str = "/v1/convai/environment-variables/:env_var_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = EnvironmentVariable;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.env_var_id.and_param(PathParam::EnvVarID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// PATCH /v1/convai/environment-variables/{env_var_id} — Update
// =============================================================================

/// Updates the values of an environment variable.
///
/// See [Update Environment Variable API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/environment-variables/update)
#[derive(Clone, Debug)]
pub struct UpdateEnvironmentVariable {
    env_var_id: String,
    body: UpdateEnvironmentVariableBody,
}

impl UpdateEnvironmentVariable {
    pub fn new(env_var_id: impl Into<String>, body: UpdateEnvironmentVariableBody) -> Self {
        Self {
            env_var_id: env_var_id.into(),
            body,
        }
    }
}

/// Update-environment-variable body.
///
/// `values` maps environment name to the new value. A value is a plain string,
/// a secret/auth-connection config object, or `null` to clear it.
#[derive(Clone, Debug, Serialize)]
pub struct UpdateEnvironmentVariableBody {
    values: HashMap<String, Value>,
}

impl UpdateEnvironmentVariableBody {
    pub fn new(values: HashMap<String, Value>) -> Self {
        Self { values }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateEnvironmentVariable {}

impl ElevenLabsEndpoint for UpdateEnvironmentVariable {
    const PATH: &'static str = "/v1/convai/environment-variables/:env_var_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = EnvironmentVariable;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.env_var_id.and_param(PathParam::EnvVarID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}
