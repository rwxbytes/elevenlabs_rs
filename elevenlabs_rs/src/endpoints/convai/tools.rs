//! Tools endpoints

use super::*;
use crate::endpoints::convai::agents::{ClientTool, SystemTool, Tool, WebHook};
use crate::endpoints::convai::knowledge_base::DependentAgent;

/// Get all available tools available in the workspace.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::tools::ListTools;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let resp = c.hit(ListTools).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See [List Tools API reference](https://elevenlabs.io/docs/api-reference/tools/get-tools).
pub struct ListTools;

impl crate::endpoints::sealed::Sealed for ListTools {}

impl ElevenLabsEndpoint for ListTools {
    const PATH: &'static str = "/v1/convai/tools";

    const METHOD: Method = Method::GET;

    type ResponseBody = ListToolsResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Response from the ListTools endpoint
#[derive(Clone, Debug, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<GetToolResponse>,
}

/// Get tool that is available in the workspace.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::tools::GetTool;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let resp = c.hit(GetTool::new("tool_id")).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See [Get Tool API reference](https://elevenlabs.io/docs/api-reference/tools/get-tool).
#[derive(Clone, Debug)]
pub struct GetTool {
    tool_id: String,
    environment: Option<String>,
}

impl GetTool {
    pub fn new(tool_id: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
            environment: None,
        }
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetTool {}

impl ElevenLabsEndpoint for GetTool {
    const PATH: &'static str = "/v1/convai/tools/:tool_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetToolResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.environment
            .as_ref()
            .map(|environment| vec![("environment", environment.clone())])
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tool_id.and_param(PathParam::ToolID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Response from the GetTool endpoint
#[derive(Clone, Debug, Deserialize)]
pub struct GetToolResponse {
    pub id: String,
    pub tool_config: Tool,
    pub dependent_agents: Vec<DependentAgent>,
}

/// Add a new tool to the available tools in the workspace.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::agents::{ApiSchema, WebHook};
/// use elevenlabs_rs::endpoints::convai::tools::CreateTool;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let api_schema = ApiSchema::new("https://example.com");
///    let webhook = WebHook::new("name", "description", api_schema);
///    let resp = c.hit(CreateTool::new(webhook)).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See [Create Tool API reference](https://elevenlabs.io/docs/api-reference/tools/add-tool).
#[derive(Clone, Debug)]
pub struct CreateTool {
    body: CreateToolBody,
}

impl CreateTool {
    pub fn new(body: impl Into<CreateToolBody>) -> Self {
        Self { body: body.into() }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateToolBody {
    pub tool_config: Tool,
}

impl CreateToolBody {
    pub fn new(tool_config: Tool) -> Self {
        Self { tool_config }
    }
}

impl From<Tool> for CreateToolBody {
    fn from(tool_config: Tool) -> Self {
        Self::new(tool_config)
    }
}

impl From<WebHook> for CreateToolBody {
    fn from(webhook: WebHook) -> Self {
        Self::new(Tool::new_webhook(webhook))
    }
}

impl From<ClientTool> for CreateToolBody {
    fn from(client_tool: ClientTool) -> Self {
        Self::new(Tool::new_client(client_tool))
    }
}

impl From<SystemTool> for CreateToolBody {
    fn from(system_tool: SystemTool) -> Self {
        Self::new(Tool::new_system(system_tool))
    }
}

pub type CreateToolResponse = GetToolResponse;

impl crate::endpoints::sealed::Sealed for CreateTool {}

impl ElevenLabsEndpoint for CreateTool {
    const PATH: &'static str = "/v1/convai/tools";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateToolResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Update tool that is available in the workspace.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::tools::UpdateTool;
/// use elevenlabs_rs::endpoints::convai::agents::{ApiSchema, WebHook};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let api_schema = ApiSchema::new("https://example.com/api/update");
///    let webhook = WebHook::new("name", "description", api_schema);
///    let resp = c.hit(UpdateTool::new("tool_id", webhook)).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See [Update Tool API reference](https://elevenlabs.io/docs/api-reference/tools/update-tool).
#[derive(Clone, Debug)]
pub struct UpdateTool {
    tool_id: String,
    body: UpdateToolBody,
}

pub type UpdateToolBody = CreateToolBody;

impl UpdateTool {
    pub fn new(tool_id: impl Into<String>, body: impl Into<UpdateToolBody>) -> Self {
        Self {
            tool_id: tool_id.into(),
            body: body.into(),
        }
    }
}

pub type UpdateToolResponse = GetToolResponse;

impl crate::endpoints::sealed::Sealed for UpdateTool {}

impl ElevenLabsEndpoint for UpdateTool {
    const PATH: &'static str = "/v1/convai/tools/:tool_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = UpdateToolResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tool_id.and_param(PathParam::ToolID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Delete tool from the workspace.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::tools::DeleteTool;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let resp = c.hit(DeleteTool::new("tool_id")).await?;
///    println!("{:#?}", resp);
///    Ok(())
///
/// }
/// ```
/// See [Delete Tool API reference](https://elevenlabs.io/docs/api-reference/tools/remove-tool).
#[derive(Clone, Debug)]
pub struct DeleteTool {
    tool_id: String,
}

impl DeleteTool {
    pub fn new(tool_id: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteTool {}

impl ElevenLabsEndpoint for DeleteTool {
    const PATH: &'static str = "/v1/convai/tools/:tool_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tool_id.and_param(PathParam::ToolID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

impl IntoIterator for ListToolsResponse {
    type Item = GetToolResponse;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.tools.into_iter()
    }
}

impl<'a> IntoIterator for &'a ListToolsResponse {
    type Item = &'a GetToolResponse;
    type IntoIter = std::slice::Iter<'a, GetToolResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.tools.iter()
    }
}

impl IntoIterator for GetToolResponse {
    type Item = DependentAgent;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.dependent_agents.into_iter()
    }
}

impl<'a> IntoIterator for &'a GetToolResponse {
    type Item = &'a DependentAgent;
    type IntoIter = std::slice::Iter<'a, DependentAgent>;

    fn into_iter(self) -> Self::IntoIter {
        self.dependent_agents.iter()
    }
}

/// Get the list of agents that depend on a tool.
///
/// See [Get Dependent Agents API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tools/get-dependent-agents)
#[derive(Clone, Debug)]
pub struct GetToolDependentAgents {
    tool_id: String,
    query: Option<ToolDependentAgentsQuery>,
}

impl GetToolDependentAgents {
    pub fn new(tool_id: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: ToolDependentAgentsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetToolDependentAgents`].
#[derive(Clone, Debug, Default)]
pub struct ToolDependentAgentsQuery {
    params: QueryValues,
}

impl ToolDependentAgentsQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetToolDependentAgents {}

impl ElevenLabsEndpoint for GetToolDependentAgents {
    const PATH: &'static str = "/v1/convai/tools/:tool_id/dependent-agents";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetToolDependentAgentsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tool_id.and_param(PathParam::ToolID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of agents that depend on a tool.
#[derive(Clone, Debug, Deserialize)]
pub struct GetToolDependentAgentsResponse {
    pub agents: Vec<DependentAgent>,
    /// Branch information for the dependent agents, preserved as raw JSON.
    #[serde(default)]
    pub branches: Vec<Value>,
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

/// Get the execution history of a tool.
///
/// See [Get Tool Executions API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tools/get-executions)
#[derive(Clone, Debug)]
pub struct GetToolExecutions {
    tool_id: String,
    query: Option<ToolExecutionsQuery>,
}

impl GetToolExecutions {
    pub fn new(tool_id: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: ToolExecutionsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetToolExecutions`].
#[derive(Clone, Debug, Default)]
pub struct ToolExecutionsQuery {
    params: QueryValues,
}

impl ToolExecutionsQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    /// Only return executions that resulted in an error (or, with `false`, only successful ones).
    pub fn with_is_error(mut self, is_error: bool) -> Self {
        self.params.push(("is_error", is_error.to_string()));
        self
    }

    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.params.push(("agent_id", agent_id.into()));
        self
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.params.push(("branch_id", branch_id.into()));
        self
    }

    pub fn with_start_time(mut self, start_time: i64) -> Self {
        self.params.push(("start_time", start_time.to_string()));
        self
    }

    pub fn with_end_time(mut self, end_time: i64) -> Self {
        self.params.push(("end_time", end_time.to_string()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetToolExecutions {}

impl ElevenLabsEndpoint for GetToolExecutions {
    const PATH: &'static str = "/v1/convai/tools/:tool_id/executions";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetToolExecutionsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tool_id.and_param(PathParam::ToolID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of tool executions.
#[derive(Clone, Debug, Deserialize)]
pub struct GetToolExecutionsResponse {
    pub executions: Vec<ToolExecution>,
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

/// A single tool execution. Request/response payloads and tool-call details are
/// preserved as raw JSON because their shape depends on the tool.
#[derive(Clone, Debug, Deserialize)]
pub struct ToolExecution {
    pub id: String,
    pub tool_id: String,
    pub tool_request_id: String,
    pub conversation_id: String,
    pub agent_id: String,
    pub branch_id: Option<String>,
    pub timestamp: f64,
    pub latency_secs: f64,
    #[serde(default)]
    pub is_error: bool,
    pub request_payload: Option<Value>,
    pub response_payload: Option<Value>,
    pub error_message: Option<String>,
    pub error_type: Option<String>,
    pub tool_call_details: Option<Value>,
}
