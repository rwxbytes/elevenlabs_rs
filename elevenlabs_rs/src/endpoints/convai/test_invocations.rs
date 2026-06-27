//! The Conversational AI test-invocations endpoints.
//!
//! A test invocation is a run of an agent test suite. List invocations, fetch a
//! single invocation's details, and resubmit selected test runs.
//!
//! The nested test-run and result structures are large and evolving, so they
//! are preserved as raw JSON.
//!
//! See the [Test Invocations API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/test-invocations).

use super::*;

// =============================================================================
// GET /v1/convai/test-invocations — List Test Invocations
// =============================================================================

/// Lists agent test-suite invocations.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::test_invocations::{
///     ListTestInvocations, TestInvocationsQuery,
/// };
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint = ListTestInvocations::default()
///         .with_query(TestInvocationsQuery::default().with_agent_id("agent_id"));
///     let resp = client.hit(endpoint).await?;
///     println!("{} invocations", resp.results.len());
///     Ok(())
/// }
/// ```
/// See [List Test Invocations API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/test-invocations/list)
#[derive(Clone, Debug, Default)]
pub struct ListTestInvocations {
    query: Option<TestInvocationsQuery>,
}

impl ListTestInvocations {
    pub fn with_query(mut self, query: TestInvocationsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`ListTestInvocations`].
#[derive(Clone, Debug, Default)]
pub struct TestInvocationsQuery {
    params: QueryValues,
}

impl TestInvocationsQuery {
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.params.push(("agent_id", agent_id.into()));
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListTestInvocations {}

impl ElevenLabsEndpoint for ListTestInvocations {
    const PATH: &'static str = "/v1/convai/test-invocations";

    const METHOD: Method = Method::GET;

    type ResponseBody = TestInvocationsPage;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of test-invocation summaries. Each summary is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct TestInvocationsPage {
    pub results: Vec<Value>,
    pub meta: Option<Value>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

// =============================================================================
// GET /v1/convai/test-invocations/{test_invocation_id} — Get Test Invocation
// =============================================================================

/// Retrieves a single test-suite invocation.
///
/// See [Get Test Invocation API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/test-invocations/get)
#[derive(Clone, Debug)]
pub struct GetTestInvocation {
    test_invocation_id: String,
}

impl GetTestInvocation {
    pub fn new(test_invocation_id: impl Into<String>) -> Self {
        Self {
            test_invocation_id: test_invocation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetTestInvocation {}

impl ElevenLabsEndpoint for GetTestInvocation {
    const PATH: &'static str = "/v1/convai/test-invocations/:test_invocation_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = TestSuiteInvocation;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .test_invocation_id
            .and_param(PathParam::TestInvocationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A test-suite invocation. The test runs and result groups are preserved as
/// raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct TestSuiteInvocation {
    pub id: String,
    pub agent_id: Option<String>,
    pub branch_id: Option<String>,
    pub folder_id: Option<String>,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub repeat_count: i64,
    pub bucketing_status: Option<Value>,
    #[serde(default)]
    pub result_groups: Vec<Value>,
    pub test_runs: Vec<Value>,
}

// =============================================================================
// POST /v1/convai/test-invocations/{test_invocation_id}/resubmit — Resubmit
// =============================================================================

/// Resubmits selected test runs of an invocation.
///
/// See [Resubmit Tests API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/test-invocations/resubmit)
#[derive(Clone, Debug)]
pub struct ResubmitTests {
    test_invocation_id: String,
    body: ResubmitTestsBody,
}

impl ResubmitTests {
    pub fn new(test_invocation_id: impl Into<String>, body: ResubmitTestsBody) -> Self {
        Self {
            test_invocation_id: test_invocation_id.into(),
            body,
        }
    }
}

/// Resubmit-tests body.
#[derive(Clone, Debug, Serialize)]
pub struct ResubmitTestsBody {
    agent_id: String,
    test_run_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_config_override: Option<Value>,
}

impl ResubmitTestsBody {
    pub fn new<I, S>(agent_id: impl Into<String>, test_run_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            agent_id: agent_id.into(),
            test_run_ids: test_run_ids.into_iter().map(Into::into).collect(),
            branch_id: None,
            agent_config_override: None,
        }
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.branch_id = Some(branch_id.into());
        self
    }

    /// An ad-hoc agent-config override to apply for the resubmitted runs,
    /// supplied as raw JSON.
    pub fn with_agent_config_override(mut self, agent_config_override: Value) -> Self {
        self.agent_config_override = Some(agent_config_override);
        self
    }
}

impl crate::endpoints::sealed::Sealed for ResubmitTests {}

impl ElevenLabsEndpoint for ResubmitTests {
    const PATH: &'static str = "/v1/convai/test-invocations/:test_invocation_id/resubmit";

    const METHOD: Method = Method::POST;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .test_invocation_id
            .and_param(PathParam::TestInvocationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}
