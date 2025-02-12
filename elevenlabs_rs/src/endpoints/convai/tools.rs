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
pub struct GetTool {
    tool_id: String,
}

impl GetTool {
    pub fn new(tool_id: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetTool {
    const PATH: &'static str = "/v1/convai/tools/:tool_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetToolResponse;

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
