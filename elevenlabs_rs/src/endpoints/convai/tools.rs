//! Tools endpoints

use super::*;
use crate::endpoints::convai::agents::Tool;
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

///// Add a new tool to the available tools in the workspace.
//#[derive(Clone, Debug)]
//pub struct CreateTool {
//    body: CreateToolBody,
//}
//
//impl CreateTool {
//    pub fn new(body: impl Into<CreateToolBody>) -> Self {
//        Self { body: body.into() }
//    }
//}
//
//#[derive(Clone, Debug, Serialize)]
//pub struct CreateToolBody {
//    pub tool_config: Tool,
//}
//
//impl CreateToolBody {
//    pub fn new(tool_config: Tool) -> Self {
//        Self { tool_config }
//    }
//}
//
//
//impl ElevenLabsEndpoint for CreateTool {
//    const PATH: &'static str = "/v1/convai/add-tool";
//
//    const METHOD: Method = Method::POST;
//
//    type ResponseBody = CreateToolResponse;
//
//    async fn request_body(&self) -> Result<RequestBody> {
//        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
//
//    }
//
//    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
//        Ok(resp.json().await?)
//    }
//}
//
//type CreateToolResponse = GetToolResponse;