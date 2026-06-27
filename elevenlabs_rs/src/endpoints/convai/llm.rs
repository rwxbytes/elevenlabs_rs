//! The Conversational AI LLM endpoints: model listing and usage-cost calculation.
//!
//! See the [LLM Usage API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/llm-usage).

use super::*;

// =============================================================================
// GET /v1/convai/llm/list — List LLMs
// =============================================================================

/// Lists the LLMs available for Conversational AI agents.
///
/// See [List LLMs API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/llm/list)
#[derive(Clone, Debug, Default)]
pub struct ListLlms;

impl crate::endpoints::sealed::Sealed for ListLlms {}

impl ElevenLabsEndpoint for ListLlms {
    const PATH: &'static str = "/v1/convai/llm/list";

    const METHOD: Method = Method::GET;

    type ResponseBody = LlmListResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`ListLlms`].
///
/// The individual LLM entries and deprecation config are preserved as raw JSON
/// because their shape is large and provider-specific.
#[derive(Clone, Debug, Deserialize)]
pub struct LlmListResponse {
    pub llms: Vec<Value>,
    pub default_deprecation_config: Value,
}

// =============================================================================
// POST /v1/convai/llm-usage/calculate — Calculate LLM Usage
// =============================================================================

/// Calculates the per-LLM token price for the given usage parameters.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::llm::{CalculateLlmUsage, LlmUsageBody};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let body = LlmUsageBody::new(800, 4, true);
///     let resp = client.hit(CalculateLlmUsage::new(body)).await?;
///     println!("{} prices", resp.llm_prices.len());
///     Ok(())
/// }
/// ```
/// See [Calculate LLM Usage API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/llm-usage/calculate)
#[derive(Clone, Debug)]
pub struct CalculateLlmUsage {
    body: LlmUsageBody,
}

impl CalculateLlmUsage {
    pub fn new(body: LlmUsageBody) -> Self {
        Self { body }
    }
}

/// Body for [`CalculateLlmUsage`].
#[derive(Clone, Debug, Serialize)]
pub struct LlmUsageBody {
    prompt_length: u32,
    number_of_pages: u32,
    rag_enabled: bool,
}

impl LlmUsageBody {
    pub fn new(prompt_length: u32, number_of_pages: u32, rag_enabled: bool) -> Self {
        Self {
            prompt_length,
            number_of_pages,
            rag_enabled,
        }
    }
}

impl crate::endpoints::sealed::Sealed for CalculateLlmUsage {}

impl ElevenLabsEndpoint for CalculateLlmUsage {
    const PATH: &'static str = "/v1/convai/llm-usage/calculate";

    const METHOD: Method = Method::POST;

    type ResponseBody = LlmUsageResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of the LLM-usage calculator. Per-LLM prices are preserved as raw
/// JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct LlmUsageResponse {
    pub llm_prices: Vec<Value>,
}

// =============================================================================
// POST /v1/convai/agent/{agent_id}/llm-usage/calculate — Calculate Agent LLM Usage
// =============================================================================

/// Calculates the per-LLM token price for a specific agent. All parameters are
/// optional and default to the agent's current configuration.
///
/// See [Calculate Agent LLM Usage API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/llm-usage/calculate-agent)
#[derive(Clone, Debug)]
pub struct CalculateAgentLlmUsage {
    agent_id: String,
    body: AgentLlmUsageBody,
}

impl CalculateAgentLlmUsage {
    pub fn new(agent_id: impl Into<String>, body: AgentLlmUsageBody) -> Self {
        Self {
            agent_id: agent_id.into(),
            body,
        }
    }
}

/// Body for [`CalculateAgentLlmUsage`]. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AgentLlmUsageBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    number_of_pages: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rag_enabled: Option<bool>,
}

impl AgentLlmUsageBody {
    pub fn with_prompt_length(mut self, prompt_length: u32) -> Self {
        self.prompt_length = Some(prompt_length);
        self
    }

    pub fn with_number_of_pages(mut self, number_of_pages: u32) -> Self {
        self.number_of_pages = Some(number_of_pages);
        self
    }

    pub fn with_rag_enabled(mut self, rag_enabled: bool) -> Self {
        self.rag_enabled = Some(rag_enabled);
        self
    }
}

impl crate::endpoints::sealed::Sealed for CalculateAgentLlmUsage {}

impl ElevenLabsEndpoint for CalculateAgentLlmUsage {
    const PATH: &'static str = "/v1/convai/agent/:agent_id/llm-usage/calculate";

    const METHOD: Method = Method::POST;

    type ResponseBody = LlmUsageResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}
