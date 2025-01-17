//! The usage endpoint.

use super::*;
use std::collections::HashMap;
use std::string::ToString;
use strum_macros::Display;

/// Returns the credit usage metrics for the current user or the entire workspace they are part of.
///
/// # Example
/// ``` no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::usage::{BreakdownType, GetUsage, GetUsageQuery};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///
///    let query = GetUsageQuery::new(1730419200, 1735912295)
///      .with_workspace_metrics(true)
///      .with_breakdown_type(BreakdownType::Voice);
///
///    let endpoint = GetUsage::new(query);
///    let resp = c.hit(endpoint).await?;
///
///    println!("{:#?}", resp);
///
///    Ok(())
/// }
/// ```
/// See [Get Usage API reference](https://elevenlabs.io/docs/api-reference/usage/get-characters-usage-metrics)
#[derive(Debug, Clone)]
pub struct GetUsage {
    query: GetUsageQuery,
}

impl GetUsage {
    pub fn new(query: GetUsageQuery) -> Self {
        Self { query }
    }
}

#[derive(Debug, Clone)]
pub struct GetUsageQuery {
    params: QueryValues,
}

impl GetUsageQuery {
    pub fn new(start_unix: u64, end_unix: u64) -> Self {
        let params = vec![
            ("start_unix", start_unix.to_string()),
            ("end_unix", end_unix.to_string()),
        ];
        Self { params }
    }

    pub fn with_workspace_metrics(mut self, workspace_metrics: bool) -> Self {
        self.params
            .push(("workspace_metrics", workspace_metrics.to_string()));
        self
    }

    pub fn with_breakdown_type(mut self, breakdown_type: BreakdownType) -> Self {
        self.params
            .push(("breakdown_type", breakdown_type.to_string()));
        self
    }
}

impl ElevenLabsEndpoint for GetUsage {

    const PATH: &'static str = "/v1/usage/character-stats";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetUsageResponseBody;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUsageResponseBody {
    pub time: Vec<u64>,
    pub usage: HashMap<String, Vec<u64>>,
}

#[derive(Debug, Display, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum BreakdownType {
    None,
    Voice,
    VoiceMultiplier,
    User,
    Groups,
    ApiKeys,
    AllApiKeys,
    ProductType,
    Model,
    Resource,
}
