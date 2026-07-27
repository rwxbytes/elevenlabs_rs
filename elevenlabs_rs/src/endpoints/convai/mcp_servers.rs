//! The Conversational AI MCP-servers endpoints.
//!
//! MCP (Model Context Protocol) servers expose external tools to agents. These
//! endpoints create and manage servers, their approval policy, per-tool
//! approvals, and per-tool config overrides.
//!
//! The server configuration is large and nested, so the `config` payload and a
//! few deeply-nested request fields are modeled as raw JSON.
//!
//! See the [MCP Servers API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp).

use super::*;
use std::collections::HashMap;

// =============================================================================
// Shared types
// =============================================================================

/// The approval policy applied to all of an MCP server's tools.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpApprovalPolicy {
    AutoApproveAll,
    RequireApprovalAll,
    RequireApprovalPerTool,
}

/// The approval policy applied to a single MCP tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpToolApprovalPolicy {
    AutoApproved,
    RequiresApproval,
}

/// Controls when an MCP tool may interrupt the agent's speech.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolInterruptionMode {
    Allow,
    DisableDuringTool,
    DisableDuringToolAndTurn,
}

/// An MCP server.
///
/// The `config` payload is large and nested, so it is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub config: Value,
    pub metadata: McpServerMetadata,
    pub access_info: Option<Value>,
    #[serde(default)]
    pub dependent_agents: Vec<Value>,
}

/// Metadata about an MCP server.
#[derive(Clone, Debug, Deserialize)]
pub struct McpServerMetadata {
    pub created_at: i64,
    pub owner_user_id: Option<String>,
}

// =============================================================================
// POST /v1/convai/mcp-servers — Create MCP Server
// =============================================================================

/// Creates an MCP server.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::mcp_servers::{CreateMcpServer, McpServerConfig};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let config = McpServerConfig::new("My MCP", "https://mcp.example.com/sse")
///         .with_description("Internal tools");
///     let server = client.hit(CreateMcpServer::new(config)).await?;
///     println!("{}", server.id);
///     Ok(())
/// }
/// ```
/// See [Create MCP Server API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/create)
#[derive(Clone, Debug)]
pub struct CreateMcpServer {
    config: McpServerConfig,
}

impl CreateMcpServer {
    pub fn new(config: McpServerConfig) -> Self {
        Self { config }
    }
}

impl crate::endpoints::sealed::Sealed for CreateMcpServer {}

impl ElevenLabsEndpoint for CreateMcpServer {
    const PATH: &'static str = "/v1/convai/mcp-servers";

    const METHOD: Method = Method::POST;

    type ResponseBody = McpServer;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "config": self.config }),
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Configuration for a new MCP server.
///
/// The common fields are typed; the credential locators (`secret_token`,
/// `auth_connection`) are accepted as raw JSON because of their many shapes.
#[derive(Clone, Debug, Serialize)]
pub struct McpServerConfig {
    name: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_policy: Option<McpApprovalPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_timeout_secs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_compression: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_token: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_connection: Option<Value>,
}

impl McpServerConfig {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            description: None,
            transport: None,
            approval_policy: None,
            request_headers: None,
            response_timeout_secs: None,
            disable_compression: None,
            secret_token: None,
            auth_connection: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The transport to use, e.g. `"SSE"` or `"STREAMABLE_HTTP"`.
    pub fn with_transport(mut self, transport: impl Into<String>) -> Self {
        self.transport = Some(transport.into());
        self
    }

    pub fn with_approval_policy(mut self, approval_policy: McpApprovalPolicy) -> Self {
        self.approval_policy = Some(approval_policy);
        self
    }

    pub fn with_request_headers(mut self, request_headers: HashMap<String, String>) -> Self {
        self.request_headers = Some(request_headers);
        self
    }

    pub fn with_response_timeout_secs(mut self, response_timeout_secs: u32) -> Self {
        self.response_timeout_secs = Some(response_timeout_secs);
        self
    }

    pub fn with_disable_compression(mut self, disable_compression: bool) -> Self {
        self.disable_compression = Some(disable_compression);
        self
    }

    /// The secret-token credential locator, supplied as raw JSON.
    pub fn with_secret_token(mut self, secret_token: Value) -> Self {
        self.secret_token = Some(secret_token);
        self
    }

    /// The auth-connection credential locator, supplied as raw JSON.
    pub fn with_auth_connection(mut self, auth_connection: Value) -> Self {
        self.auth_connection = Some(auth_connection);
        self
    }
}

// =============================================================================
// GET /v1/convai/mcp-servers — List MCP Servers
// =============================================================================

/// Lists the workspace's MCP servers.
///
/// See [List MCP Servers API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/list)
#[derive(Clone, Debug, Default)]
pub struct ListMcpServers;

impl crate::endpoints::sealed::Sealed for ListMcpServers {}

impl ElevenLabsEndpoint for ListMcpServers {
    const PATH: &'static str = "/v1/convai/mcp-servers";

    const METHOD: Method = Method::GET;

    type ResponseBody = McpServersResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`ListMcpServers`].
#[derive(Clone, Debug, Deserialize)]
pub struct McpServersResponse {
    pub mcp_servers: Vec<McpServer>,
}

impl IntoIterator for McpServersResponse {
    type Item = McpServer;
    type IntoIter = std::vec::IntoIter<McpServer>;

    fn into_iter(self) -> Self::IntoIter {
        self.mcp_servers.into_iter()
    }
}

// =============================================================================
// GET /v1/convai/mcp-servers/{mcp_server_id} — Get MCP Server
// =============================================================================

/// Retrieves an MCP server by ID.
///
/// See [Get MCP Server API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/get)
#[derive(Clone, Debug)]
pub struct GetMcpServer {
    mcp_server_id: String,
}

impl GetMcpServer {
    pub fn new(mcp_server_id: impl Into<String>) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetMcpServer {}

impl ElevenLabsEndpoint for GetMcpServer {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.mcp_server_id.and_param(PathParam::McpServerID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/convai/mcp-servers/{mcp_server_id} — Delete MCP Server
// =============================================================================

/// Deletes an MCP server.
///
/// See [Delete MCP Server API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/delete)
#[derive(Clone, Debug)]
pub struct DeleteMcpServer {
    mcp_server_id: String,
}

impl DeleteMcpServer {
    pub fn new(mcp_server_id: impl Into<String>) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteMcpServer {}

impl ElevenLabsEndpoint for DeleteMcpServer {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.mcp_server_id.and_param(PathParam::McpServerID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

// =============================================================================
// PATCH /v1/convai/mcp-servers/{mcp_server_id} — Update MCP Server Config
// =============================================================================

/// Updates an MCP server's configuration. All fields are optional.
///
/// See [Update MCP Server API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/update)
#[derive(Clone, Debug)]
pub struct UpdateMcpServerConfig {
    mcp_server_id: String,
    body: McpServerConfigUpdate,
}

impl UpdateMcpServerConfig {
    pub fn new(mcp_server_id: impl Into<String>, body: McpServerConfigUpdate) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            body,
        }
    }
}

/// Body for [`UpdateMcpServerConfig`]. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct McpServerConfigUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_policy: Option<McpApprovalPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_timeout_secs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_compression: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_interruptions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interruption_mode: Option<ToolInterruptionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_token: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_connection: Option<Value>,
}

impl McpServerConfigUpdate {
    pub fn with_approval_policy(mut self, approval_policy: McpApprovalPolicy) -> Self {
        self.approval_policy = Some(approval_policy);
        self
    }

    pub fn with_response_timeout_secs(mut self, response_timeout_secs: u32) -> Self {
        self.response_timeout_secs = Some(response_timeout_secs);
        self
    }

    pub fn with_request_headers(mut self, request_headers: HashMap<String, String>) -> Self {
        self.request_headers = Some(request_headers);
        self
    }

    pub fn with_disable_compression(mut self, disable_compression: bool) -> Self {
        self.disable_compression = Some(disable_compression);
        self
    }

    #[deprecated(
        note = "ElevenLabs deprecated disable_interruptions; use with_interruption_mode instead"
    )]
    pub fn with_disable_interruptions(mut self, disable_interruptions: bool) -> Self {
        self.disable_interruptions = Some(disable_interruptions);
        self
    }

    pub fn with_interruption_mode(mut self, interruption_mode: ToolInterruptionMode) -> Self {
        self.interruption_mode = Some(interruption_mode);
        self
    }

    pub fn with_secret_token(mut self, secret_token: Value) -> Self {
        self.secret_token = Some(secret_token);
        self
    }

    pub fn with_auth_connection(mut self, auth_connection: Value) -> Self {
        self.auth_connection = Some(auth_connection);
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateMcpServerConfig {}

impl ElevenLabsEndpoint for UpdateMcpServerConfig {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.mcp_server_id.and_param(PathParam::McpServerID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// PATCH /v1/convai/mcp-servers/{mcp_server_id}/approval-policy
// =============================================================================

/// Updates an MCP server's approval policy.
///
/// See [Update Approval Policy API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/update-approval-policy)
#[derive(Clone, Debug)]
pub struct UpdateMcpApprovalPolicy {
    mcp_server_id: String,
    approval_policy: McpApprovalPolicy,
}

impl UpdateMcpApprovalPolicy {
    pub fn new(mcp_server_id: impl Into<String>, approval_policy: McpApprovalPolicy) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            approval_policy,
        }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateMcpApprovalPolicy {}

impl ElevenLabsEndpoint for UpdateMcpApprovalPolicy {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/approval-policy";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.mcp_server_id.and_param(PathParam::McpServerID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "approval_policy": self.approval_policy }),
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/convai/mcp-servers/{mcp_server_id}/tool-approvals
// =============================================================================

/// Adds (or updates) the approval policy for a single MCP tool.
///
/// See [Add Tool Approval API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/add-tool-approval)
#[derive(Clone, Debug)]
pub struct AddMcpToolApproval {
    mcp_server_id: String,
    body: AddMcpToolApprovalBody,
}

impl AddMcpToolApproval {
    pub fn new(mcp_server_id: impl Into<String>, body: AddMcpToolApprovalBody) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            body,
        }
    }
}

/// Body for [`AddMcpToolApproval`].
#[derive(Clone, Debug, Serialize)]
pub struct AddMcpToolApprovalBody {
    tool_name: String,
    tool_description: String,
    approval_policy: McpToolApprovalPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_schema: Option<Value>,
}

impl AddMcpToolApprovalBody {
    pub fn new(
        tool_name: impl Into<String>,
        tool_description: impl Into<String>,
        approval_policy: McpToolApprovalPolicy,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            tool_description: tool_description.into(),
            approval_policy,
            input_schema: None,
        }
    }

    /// The tool's JSON input schema, supplied as raw JSON.
    pub fn with_input_schema(mut self, input_schema: Value) -> Self {
        self.input_schema = Some(input_schema);
        self
    }
}

impl crate::endpoints::sealed::Sealed for AddMcpToolApproval {}

impl ElevenLabsEndpoint for AddMcpToolApproval {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/tool-approvals";

    const METHOD: Method = Method::POST;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.mcp_server_id.and_param(PathParam::McpServerID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/convai/mcp-servers/{mcp_server_id}/tool-approvals/{tool_name}
// =============================================================================

/// Removes the approval policy for a single MCP tool.
///
/// See [Delete Tool Approval API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/delete-tool-approval)
#[derive(Clone, Debug)]
pub struct DeleteMcpToolApproval {
    mcp_server_id: String,
    tool_name: String,
}

impl DeleteMcpToolApproval {
    pub fn new(mcp_server_id: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            tool_name: tool_name.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteMcpToolApproval {}

impl ElevenLabsEndpoint for DeleteMcpToolApproval {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/tool-approvals/:tool_name";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.mcp_server_id.and_param(PathParam::McpServerID),
            self.tool_name.and_param(PathParam::ToolName),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/convai/mcp-servers/{mcp_server_id}/tool-configs — Create Tool Config
// =============================================================================

/// Creates a per-tool config override for an MCP server.
///
/// See [Create Tool Config API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/create-tool-config)
#[derive(Clone, Debug)]
pub struct CreateMcpToolConfig {
    mcp_server_id: String,
    body: McpToolConfigCreate,
    environment: Option<String>,
}

impl CreateMcpToolConfig {
    pub fn new(mcp_server_id: impl Into<String>, body: McpToolConfigCreate) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            body,
            environment: None,
        }
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }
}

/// Body for [`CreateMcpToolConfig`].
#[derive(Clone, Debug, Serialize)]
pub struct McpToolConfigCreate {
    tool_name: String,
    #[serde(flatten)]
    overrides: McpToolConfigOverrides,
}

impl McpToolConfigCreate {
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            overrides: McpToolConfigOverrides::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: McpToolConfigOverrides) -> Self {
        self.overrides = overrides;
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateMcpToolConfig {}

impl ElevenLabsEndpoint for CreateMcpToolConfig {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/tool-configs";

    const METHOD: Method = Method::POST;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.mcp_server_id.and_param(PathParam::McpServerID)]
    }

    fn query_params(&self) -> Option<QueryValues> {
        self.environment
            .as_ref()
            .map(|environment| vec![("environment", environment.clone())])
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The overridable fields of a per-tool config. Deeply nested fields
/// (`assignments`, `input_overrides`, `response_mocks`) are accepted as raw JSON.
#[derive(Clone, Debug, Default, Serialize)]
pub struct McpToolConfigOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    force_pre_tool_speech: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_interruptions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interruption_mode: Option<ToolInterruptionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_timeout_secs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignments: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_overrides: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_mocks: Option<Value>,
}

impl McpToolConfigOverrides {
    pub fn with_force_pre_tool_speech(mut self, force_pre_tool_speech: bool) -> Self {
        self.force_pre_tool_speech = Some(force_pre_tool_speech);
        self
    }

    #[deprecated(
        note = "ElevenLabs deprecated disable_interruptions; use with_interruption_mode instead"
    )]
    pub fn with_disable_interruptions(mut self, disable_interruptions: bool) -> Self {
        self.disable_interruptions = Some(disable_interruptions);
        self
    }

    pub fn with_interruption_mode(mut self, interruption_mode: ToolInterruptionMode) -> Self {
        self.interruption_mode = Some(interruption_mode);
        self
    }

    pub fn with_response_timeout_secs(mut self, response_timeout_secs: u32) -> Self {
        self.response_timeout_secs = Some(response_timeout_secs);
        self
    }

    pub fn with_assignments(mut self, assignments: Value) -> Self {
        self.assignments = Some(assignments);
        self
    }

    pub fn with_input_overrides(mut self, input_overrides: Value) -> Self {
        self.input_overrides = Some(input_overrides);
        self
    }

    pub fn with_response_mocks(mut self, response_mocks: Value) -> Self {
        self.response_mocks = Some(response_mocks);
        self
    }
}

// =============================================================================
// GET /v1/convai/mcp-servers/{mcp_server_id}/tool-configs/{tool_name}
// =============================================================================

/// Retrieves a single per-tool config override. The override is returned as raw
/// JSON because of its many nested shapes.
///
/// See [Get Tool Config API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/get-tool-config)
#[derive(Clone, Debug)]
pub struct GetMcpToolConfig {
    mcp_server_id: String,
    tool_name: String,
}

impl GetMcpToolConfig {
    pub fn new(mcp_server_id: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            tool_name: tool_name.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetMcpToolConfig {}

impl ElevenLabsEndpoint for GetMcpToolConfig {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/tool-configs/:tool_name";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.mcp_server_id.and_param(PathParam::McpServerID),
            self.tool_name.and_param(PathParam::ToolName),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// PATCH /v1/convai/mcp-servers/{mcp_server_id}/tool-configs/{tool_name}
// =============================================================================

/// Updates a per-tool config override.
///
/// See [Update Tool Config API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/update-tool-config)
#[derive(Clone, Debug)]
pub struct UpdateMcpToolConfig {
    mcp_server_id: String,
    tool_name: String,
    body: McpToolConfigOverrides,
    environment: Option<String>,
}

impl UpdateMcpToolConfig {
    pub fn new(
        mcp_server_id: impl Into<String>,
        tool_name: impl Into<String>,
        body: McpToolConfigOverrides,
    ) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            tool_name: tool_name.into(),
            body,
            environment: None,
        }
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateMcpToolConfig {}

impl ElevenLabsEndpoint for UpdateMcpToolConfig {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/tool-configs/:tool_name";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.mcp_server_id.and_param(PathParam::McpServerID),
            self.tool_name.and_param(PathParam::ToolName),
        ]
    }

    fn query_params(&self) -> Option<QueryValues> {
        self.environment
            .as_ref()
            .map(|environment| vec![("environment", environment.clone())])
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/convai/mcp-servers/{mcp_server_id}/tool-configs/{tool_name}
// =============================================================================

/// Deletes a per-tool config override.
///
/// See [Delete Tool Config API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/delete-tool-config)
#[derive(Clone, Debug)]
pub struct DeleteMcpToolConfig {
    mcp_server_id: String,
    tool_name: String,
}

impl DeleteMcpToolConfig {
    pub fn new(mcp_server_id: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            tool_name: tool_name.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteMcpToolConfig {}

impl ElevenLabsEndpoint for DeleteMcpToolConfig {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/tool-configs/:tool_name";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = McpServer;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.mcp_server_id.and_param(PathParam::McpServerID),
            self.tool_name.and_param(PathParam::ToolName),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// GET /v1/convai/mcp-servers/{mcp_server_id}/tools — List MCP Tools
// =============================================================================

/// Lists the tools exposed by an MCP server.
///
/// See [List MCP Tools API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/mcp/list-tools)
#[derive(Clone, Debug)]
pub struct ListMcpTools {
    mcp_server_id: String,
    environment: Option<String>,
}

impl ListMcpTools {
    pub fn new(mcp_server_id: impl Into<String>) -> Self {
        Self {
            mcp_server_id: mcp_server_id.into(),
            environment: None,
        }
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListMcpTools {}

impl ElevenLabsEndpoint for ListMcpTools {
    const PATH: &'static str = "/v1/convai/mcp-servers/:mcp_server_id/tools";

    const METHOD: Method = Method::GET;

    type ResponseBody = ListMcpToolsResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.mcp_server_id.and_param(PathParam::McpServerID)]
    }

    fn query_params(&self) -> Option<QueryValues> {
        self.environment
            .as_ref()
            .map(|environment| vec![("environment", environment.clone())])
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`ListMcpTools`]. Each tool definition is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct ListMcpToolsResponse {
    pub success: bool,
    pub tools: Vec<Value>,
    pub error_message: Option<String>,
}
