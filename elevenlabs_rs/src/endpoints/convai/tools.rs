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

#[derive(Clone, Debug, Deserialize)]
pub struct GetToolResponse {
    pub id: String,
    pub tool_config: Tool,
    pub dependent_agents: Vec<DependentAgent>,
}
