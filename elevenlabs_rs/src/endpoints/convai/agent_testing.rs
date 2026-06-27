//! The Conversational AI agent-testing endpoints.
//!
//! Agent tests verify an agent's behavior (LLM response, tool call, or full
//! simulation). These endpoints list, create, fetch, update, and delete tests,
//! and organize them into folders.
//!
//! A test definition is a 3-way discriminated union (`llm`, `tool`,
//! `simulation`) with large nested config, so individual test definitions and
//! summaries are modeled as raw JSON. Build the JSON for `CreateAgentTest` /
//! `UpdateAgentTest` per the API reference.
//!
//! See the [Agent Testing API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing).

use super::*;

// =============================================================================
// GET /v1/convai/agent-testing — List Tests
// =============================================================================

/// Lists agent tests (and, optionally, folders).
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agent_testing::{ListAgentTests, AgentTestsQuery};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint =
///         ListAgentTests::default().with_query(AgentTestsQuery::default().with_page_size(30));
///     let resp = client.hit(endpoint).await?;
///     println!("{} tests", resp.tests.len());
///     Ok(())
/// }
/// ```
/// See [List Tests API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/list)
#[derive(Clone, Debug, Default)]
pub struct ListAgentTests {
    query: Option<AgentTestsQuery>,
}

impl ListAgentTests {
    pub fn with_query(mut self, query: AgentTestsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`ListAgentTests`].
#[derive(Clone, Debug, Default)]
pub struct AgentTestsQuery {
    params: QueryValues,
}

impl AgentTestsQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.params.push(("search", search.into()));
        self
    }

    pub fn with_parent_folder_id(mut self, parent_folder_id: impl Into<String>) -> Self {
        self.params
            .push(("parent_folder_id", parent_folder_id.into()));
        self
    }

    pub fn with_include_folders(mut self, include_folders: bool) -> Self {
        self.params
            .push(("include_folders", include_folders.to_string()));
        self
    }

    pub fn with_sort_mode(mut self, sort_mode: impl Into<String>) -> Self {
        self.params.push(("sort_mode", sort_mode.into()));
        self
    }

    pub fn with_sharing_mode(mut self, sharing_mode: impl Into<String>) -> Self {
        self.params.push(("sharing_mode", sharing_mode.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListAgentTests {}

impl ElevenLabsEndpoint for ListAgentTests {
    const PATH: &'static str = "/v1/convai/agent-testing";

    const METHOD: Method = Method::GET;

    type ResponseBody = AgentTestsPage;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of agent-test summaries. Each summary is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentTestsPage {
    pub tests: Vec<Value>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

// =============================================================================
// POST /v1/convai/agent-testing/create — Create Test
// =============================================================================

/// Creates an agent test from a JSON test definition.
///
/// The test definition is a 3-way discriminated union (`llm`, `tool`,
/// `simulation`); build it as JSON per the API reference.
///
/// See [Create Test API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/create)
#[derive(Clone, Debug)]
pub struct CreateAgentTest {
    test: Value,
}

impl CreateAgentTest {
    pub fn new(test: Value) -> Self {
        Self { test }
    }
}

impl crate::endpoints::sealed::Sealed for CreateAgentTest {}

impl ElevenLabsEndpoint for CreateAgentTest {
    const PATH: &'static str = "/v1/convai/agent-testing/create";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateAgentTestResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(self.test.clone()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`CreateAgentTest`].
#[derive(Clone, Debug, Deserialize)]
pub struct CreateAgentTestResponse {
    pub id: String,
}

// =============================================================================
// GET /v1/convai/agent-testing/{test_id} — Get Test
// =============================================================================

/// Retrieves an agent test by ID. The test definition is returned as raw JSON.
///
/// See [Get Test API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/get)
#[derive(Clone, Debug)]
pub struct GetAgentTest {
    test_id: String,
}

impl GetAgentTest {
    pub fn new(test_id: impl Into<String>) -> Self {
        Self {
            test_id: test_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetAgentTest {}

impl ElevenLabsEndpoint for GetAgentTest {
    const PATH: &'static str = "/v1/convai/agent-testing/:test_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.test_id.and_param(PathParam::TestID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// PUT /v1/convai/agent-testing/{test_id} — Update Test
// =============================================================================

/// Updates an agent test from a JSON test definition. The updated test is
/// returned as raw JSON.
///
/// See [Update Test API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/update)
#[derive(Clone, Debug)]
pub struct UpdateAgentTest {
    test_id: String,
    test: Value,
}

impl UpdateAgentTest {
    pub fn new(test_id: impl Into<String>, test: Value) -> Self {
        Self {
            test_id: test_id.into(),
            test,
        }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateAgentTest {}

impl ElevenLabsEndpoint for UpdateAgentTest {
    const PATH: &'static str = "/v1/convai/agent-testing/:test_id";

    const METHOD: Method = Method::PUT;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.test_id.and_param(PathParam::TestID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(self.test.clone()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/convai/agent-testing/{test_id} — Delete Test
// =============================================================================

/// Deletes an agent test.
///
/// See [Delete Test API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/delete)
#[derive(Clone, Debug)]
pub struct DeleteAgentTest {
    test_id: String,
}

impl DeleteAgentTest {
    pub fn new(test_id: impl Into<String>) -> Self {
        Self {
            test_id: test_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteAgentTest {}

impl ElevenLabsEndpoint for DeleteAgentTest {
    const PATH: &'static str = "/v1/convai/agent-testing/:test_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.test_id.and_param(PathParam::TestID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

// =============================================================================
// POST /v1/convai/agent-testing/summaries — Get Test Summaries By IDs
// =============================================================================

/// Retrieves summaries for the given test IDs.
///
/// See [Get Test Summaries API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/summaries)
#[derive(Clone, Debug)]
pub struct GetAgentTestSummaries {
    test_ids: Vec<String>,
}

impl GetAgentTestSummaries {
    pub fn new<I, S>(test_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            test_ids: test_ids.into_iter().map(Into::into).collect(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetAgentTestSummaries {}

impl ElevenLabsEndpoint for GetAgentTestSummaries {
    const PATH: &'static str = "/v1/convai/agent-testing/summaries";

    const METHOD: Method = Method::POST;

    type ResponseBody = AgentTestSummariesResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "test_ids": self.test_ids }),
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`GetAgentTestSummaries`]: a map of test ID to summary,
/// preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentTestSummariesResponse {
    pub tests: Value,
}

// =============================================================================
// POST /v1/convai/agent-testing/bulk-move — Bulk Move Tests
// =============================================================================

/// Moves several tests (and/or folders) into a folder.
///
/// See [Bulk Move Tests API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/bulk-move)
#[derive(Clone, Debug)]
pub struct BulkMoveTests {
    body: BulkMoveTestsBody,
}

impl BulkMoveTests {
    pub fn new(body: BulkMoveTestsBody) -> Self {
        Self { body }
    }
}

/// Body for [`BulkMoveTests`].
#[derive(Clone, Debug, Serialize)]
pub struct BulkMoveTestsBody {
    entity_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    move_to: Option<String>,
}

impl BulkMoveTestsBody {
    pub fn new<I, S>(entity_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            entity_ids: entity_ids.into_iter().map(Into::into).collect(),
            move_to: None,
        }
    }

    /// The destination folder ID. Omit (or `None`) to move to the root.
    pub fn with_move_to(mut self, move_to: impl Into<String>) -> Self {
        self.move_to = Some(move_to.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for BulkMoveTests {}

impl ElevenLabsEndpoint for BulkMoveTests {
    const PATH: &'static str = "/v1/convai/agent-testing/bulk-move";

    const METHOD: Method = Method::POST;

    type ResponseBody = ();

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

// =============================================================================
// POST /v1/convai/agent-testing/folders — Create Folder
// =============================================================================

/// Creates an agent-test folder.
///
/// See [Create Folder API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/create-folder)
#[derive(Clone, Debug)]
pub struct CreateAgentTestFolder {
    body: CreateAgentTestFolderBody,
}

impl CreateAgentTestFolder {
    pub fn new(body: CreateAgentTestFolderBody) -> Self {
        Self { body }
    }
}

/// Body for [`CreateAgentTestFolder`].
#[derive(Clone, Debug, Serialize)]
pub struct CreateAgentTestFolderBody {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_folder_id: Option<String>,
}

impl CreateAgentTestFolderBody {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent_folder_id: None,
        }
    }

    pub fn with_parent_folder_id(mut self, parent_folder_id: impl Into<String>) -> Self {
        self.parent_folder_id = Some(parent_folder_id.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateAgentTestFolder {}

impl ElevenLabsEndpoint for CreateAgentTestFolder {
    const PATH: &'static str = "/v1/convai/agent-testing/folders";

    const METHOD: Method = Method::POST;

    type ResponseBody = AgentTestFolderRef;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A reference to an agent-test folder (its ID and name).
#[derive(Clone, Debug, Deserialize)]
pub struct AgentTestFolderRef {
    pub id: String,
    pub name: String,
}

// =============================================================================
// GET /v1/convai/agent-testing/folders/{folder_id} — Get Folder
// =============================================================================

/// Retrieves an agent-test folder by ID.
///
/// See [Get Folder API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/get-folder)
#[derive(Clone, Debug)]
pub struct GetAgentTestFolder {
    folder_id: String,
}

impl GetAgentTestFolder {
    pub fn new(folder_id: impl Into<String>) -> Self {
        Self {
            folder_id: folder_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetAgentTestFolder {}

impl ElevenLabsEndpoint for GetAgentTestFolder {
    const PATH: &'static str = "/v1/convai/agent-testing/folders/:folder_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = AgentTestFolder;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.folder_id.and_param(PathParam::FolderID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// An agent-test folder. The `folder_path` breadcrumb segments are preserved as
/// raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentTestFolder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub folder_path: Vec<Value>,
    #[serde(default)]
    pub children_count: i64,
}

// =============================================================================
// PATCH /v1/convai/agent-testing/folders/{folder_id} — Update Folder
// =============================================================================

/// Updates (renames) an agent-test folder.
///
/// See [Update Folder API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/update-folder)
#[derive(Clone, Debug)]
pub struct UpdateAgentTestFolder {
    folder_id: String,
    name: String,
}

impl UpdateAgentTestFolder {
    pub fn new(folder_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            folder_id: folder_id.into(),
            name: name.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateAgentTestFolder {}

impl ElevenLabsEndpoint for UpdateAgentTestFolder {
    const PATH: &'static str = "/v1/convai/agent-testing/folders/:folder_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = AgentTestFolder;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.folder_id.and_param(PathParam::FolderID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::json!({ "name": self.name })))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/convai/agent-testing/folders/{folder_id} — Delete Folder
// =============================================================================

/// Deletes an agent-test folder.
///
/// See [Delete Folder API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agent-testing/delete-folder)
#[derive(Clone, Debug)]
pub struct DeleteAgentTestFolder {
    folder_id: String,
    force: Option<bool>,
}

impl DeleteAgentTestFolder {
    pub fn new(folder_id: impl Into<String>) -> Self {
        Self {
            folder_id: folder_id.into(),
            force: None,
        }
    }

    /// Force-delete the folder even if it is not empty.
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }
}

impl crate::endpoints::sealed::Sealed for DeleteAgentTestFolder {}

impl ElevenLabsEndpoint for DeleteAgentTestFolder {
    const PATH: &'static str = "/v1/convai/agent-testing/folders/:folder_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn query_params(&self) -> Option<QueryValues> {
        self.force.map(|force| vec![("force", force.to_string())])
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.folder_id.and_param(PathParam::FolderID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}
