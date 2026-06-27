//! Advanced Conversational AI agent-management endpoints.
//!
//! Beyond the core agent CRUD in [`agents`](super::agents), these endpoints
//! cover branches, deployments, drafts, duplication, simulations, test runs,
//! topics, versions, the widget embed, and the avatar.
//!
//! Branch configs, simulation transcripts, and merge previews are large and
//! evolving, so the heaviest request/response payloads are modeled as raw JSON.
//!
//! See the [Agents API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents).

use super::*;
use crate::endpoints::convai::test_invocations::TestSuiteInvocation;
use futures_util::{Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;

// =============================================================================
// GET /v1/convai/agents/summaries — Get Agent Summaries
// =============================================================================

/// Retrieves summaries for the given agent IDs.
///
/// See [Get Agent Summaries API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/summaries)
#[derive(Clone, Debug)]
pub struct GetAgentSummaries {
    agent_ids: Vec<String>,
}

impl GetAgentSummaries {
    pub fn new<I, S>(agent_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            agent_ids: agent_ids.into_iter().map(Into::into).collect(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetAgentSummaries {}

impl ElevenLabsEndpoint for GetAgentSummaries {
    const PATH: &'static str = "/v1/convai/agents/summaries";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn query_params(&self) -> Option<QueryValues> {
        Some(
            self.agent_ids
                .iter()
                .map(|id| ("agent_ids", id.clone()))
                .collect(),
        )
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// Branches
// =============================================================================

/// Create a new branch for an agent.
///
/// See [Create Branch API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/create-branch)
#[derive(Clone, Debug)]
pub struct CreateAgentBranch {
    agent_id: String,
    body: CreateAgentBranchBody,
}

impl CreateAgentBranch {
    pub fn new(agent_id: impl Into<String>, body: CreateAgentBranchBody) -> Self {
        Self {
            agent_id: agent_id.into(),
            body,
        }
    }
}

/// Body for [`CreateAgentBranch`]. `conversation_config`, `platform_settings`,
/// and `workflow` are deep configs, supplied as raw JSON.
#[derive(Clone, Debug, Serialize)]
pub struct CreateAgentBranchBody {
    parent_version_id: String,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_settings: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow: Option<Value>,
}

impl CreateAgentBranchBody {
    pub fn new(
        parent_version_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            parent_version_id: parent_version_id.into(),
            name: name.into(),
            description: description.into(),
            conversation_config: None,
            platform_settings: None,
            workflow: None,
        }
    }

    pub fn with_conversation_config(mut self, conversation_config: Value) -> Self {
        self.conversation_config = Some(conversation_config);
        self
    }

    pub fn with_platform_settings(mut self, platform_settings: Value) -> Self {
        self.platform_settings = Some(platform_settings);
        self
    }

    pub fn with_workflow(mut self, workflow: Value) -> Self {
        self.workflow = Some(workflow);
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateAgentBranch {}

impl ElevenLabsEndpoint for CreateAgentBranch {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/branches";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateAgentBranchResponse;

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

/// The response of [`CreateAgentBranch`].
#[derive(Clone, Debug, Deserialize)]
pub struct CreateAgentBranchResponse {
    pub created_branch_id: String,
    pub created_version_id: String,
}

/// List the branches of an agent.
///
/// See [List Branches API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/list-branches)
#[derive(Clone, Debug)]
pub struct ListAgentBranches {
    agent_id: String,
    query: Option<ListAgentBranchesQuery>,
}

impl ListAgentBranches {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: ListAgentBranchesQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`ListAgentBranches`].
#[derive(Clone, Debug, Default)]
pub struct ListAgentBranchesQuery {
    params: QueryValues,
}

impl ListAgentBranchesQuery {
    pub fn with_include_archived(mut self, include_archived: bool) -> Self {
        self.params
            .push(("include_archived", include_archived.to_string()));
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.params.push(("limit", limit.to_string()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListAgentBranches {}

impl ElevenLabsEndpoint for ListAgentBranches {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/branches";

    const METHOD: Method = Method::GET;

    type ResponseBody = AgentBranchList;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A list of agent-branch summaries. Each summary is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentBranchList {
    pub results: Vec<Value>,
    pub meta: Option<Value>,
}

/// Get a single agent branch. The branch detail is returned as raw JSON.
///
/// See [Get Branch API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/get-branch)
#[derive(Clone, Debug)]
pub struct GetAgentBranch {
    agent_id: String,
    branch_id: String,
}

impl GetAgentBranch {
    pub fn new(agent_id: impl Into<String>, branch_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            branch_id: branch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetAgentBranch {}

impl ElevenLabsEndpoint for GetAgentBranch {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/branches/:branch_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.branch_id.and_param(PathParam::BranchID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Update an agent branch (rename, archive, or change protection status). The
/// updated branch is returned as raw JSON.
///
/// See [Update Branch API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/update-branch)
#[derive(Clone, Debug)]
pub struct UpdateAgentBranch {
    agent_id: String,
    branch_id: String,
    body: UpdateAgentBranchBody,
}

impl UpdateAgentBranch {
    pub fn new(
        agent_id: impl Into<String>,
        branch_id: impl Into<String>,
        body: UpdateAgentBranchBody,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            branch_id: branch_id.into(),
            body,
        }
    }
}

/// Body for [`UpdateAgentBranch`]. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateAgentBranchBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_archived: Option<bool>,
    /// The branch protection status, e.g. `"protected"` or `"unprotected"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    protection_status: Option<String>,
}

impl UpdateAgentBranchBody {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_is_archived(mut self, is_archived: bool) -> Self {
        self.is_archived = Some(is_archived);
        self
    }

    pub fn with_protection_status(mut self, protection_status: impl Into<String>) -> Self {
        self.protection_status = Some(protection_status.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateAgentBranch {}

impl ElevenLabsEndpoint for UpdateAgentBranch {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/branches/:branch_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.branch_id.and_param(PathParam::BranchID),
        ]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Rebase a branch onto its parent. The result is returned as raw JSON.
///
/// See [Rebase Branch API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/rebase-branch)
#[derive(Clone, Debug)]
pub struct RebaseAgentBranch {
    agent_id: String,
    branch_id: String,
}

impl RebaseAgentBranch {
    pub fn new(agent_id: impl Into<String>, branch_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            branch_id: branch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for RebaseAgentBranch {}

impl ElevenLabsEndpoint for RebaseAgentBranch {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/branches/:branch_id/rebase";

    const METHOD: Method = Method::POST;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.branch_id.and_param(PathParam::BranchID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Preview the result of rebasing a branch. The preview is returned as raw JSON.
///
/// See [Rebase Preview API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/rebase-preview)
#[derive(Clone, Debug)]
pub struct GetRebasePreview {
    agent_id: String,
    branch_id: String,
}

impl GetRebasePreview {
    pub fn new(agent_id: impl Into<String>, branch_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            branch_id: branch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetRebasePreview {}

impl ElevenLabsEndpoint for GetRebasePreview {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/branches/:branch_id/rebase-preview";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.branch_id.and_param(PathParam::BranchID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Merge a source branch into a target branch. The result is returned as raw JSON.
///
/// See [Merge Branch API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/merge-branch)
#[derive(Clone, Debug)]
pub struct MergeAgentBranch {
    agent_id: String,
    source_branch_id: String,
    target_branch_id: String,
    body: MergeAgentBranchBody,
}

impl MergeAgentBranch {
    pub fn new(
        agent_id: impl Into<String>,
        source_branch_id: impl Into<String>,
        target_branch_id: impl Into<String>,
        body: MergeAgentBranchBody,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            source_branch_id: source_branch_id.into(),
            target_branch_id: target_branch_id.into(),
            body,
        }
    }
}

/// Body for [`MergeAgentBranch`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct MergeAgentBranchBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    archive_source_branch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    force: Option<bool>,
}

impl MergeAgentBranchBody {
    pub fn with_archive_source_branch(mut self, archive_source_branch: bool) -> Self {
        self.archive_source_branch = Some(archive_source_branch);
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }
}

impl crate::endpoints::sealed::Sealed for MergeAgentBranch {}

impl ElevenLabsEndpoint for MergeAgentBranch {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/branches/:source_branch_id/merge";

    const METHOD: Method = Method::POST;

    type ResponseBody = Value;

    fn query_params(&self) -> Option<QueryValues> {
        Some(vec![("target_branch_id", self.target_branch_id.clone())])
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.source_branch_id.and_param(PathParam::SourceBranchID),
        ]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Preview the result of merging a source branch into a target branch. The
/// preview is returned as raw JSON.
///
/// See [Merge Preview API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/merge-preview)
#[derive(Clone, Debug)]
pub struct GetMergePreview {
    agent_id: String,
    source_branch_id: String,
    target_branch_id: String,
    force: Option<bool>,
}

impl GetMergePreview {
    pub fn new(
        agent_id: impl Into<String>,
        source_branch_id: impl Into<String>,
        target_branch_id: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            source_branch_id: source_branch_id.into(),
            target_branch_id: target_branch_id.into(),
            force: None,
        }
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetMergePreview {}

impl ElevenLabsEndpoint for GetMergePreview {
    const PATH: &'static str =
        "/v1/convai/agents/:agent_id/branches/:source_branch_id/merge-preview";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn query_params(&self) -> Option<QueryValues> {
        let mut params = vec![("target_branch_id", self.target_branch_id.clone())];
        if let Some(force) = self.force {
            params.push(("force", force.to_string()));
        }
        Some(params)
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.source_branch_id.and_param(PathParam::SourceBranchID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/convai/agents/{agent_id}/deployments — Create/Update Deployments
// =============================================================================

/// Create or update an agent's branch traffic deployments.
///
/// `traffic_percentage_branch_id_map` maps branch ID to traffic percentage.
///
/// See [Create Deployments API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/deployments)
#[derive(Clone, Debug)]
pub struct CreateAgentDeployments {
    agent_id: String,
    traffic_percentage_branch_id_map: HashMap<String, Value>,
}

impl CreateAgentDeployments {
    pub fn new(
        agent_id: impl Into<String>,
        traffic_percentage_branch_id_map: HashMap<String, Value>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            traffic_percentage_branch_id_map,
        }
    }
}

impl crate::endpoints::sealed::Sealed for CreateAgentDeployments {}

impl ElevenLabsEndpoint for CreateAgentDeployments {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/deployments";

    const METHOD: Method = Method::POST;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::json!({
            "traffic_percentage_branch_id_map": self.traffic_percentage_branch_id_map,
        })))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// Drafts
// =============================================================================

/// Create a draft for an agent branch from a (raw JSON) draft config. The result
/// is returned as raw JSON.
///
/// See [Create Draft API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/create-draft)
#[derive(Clone, Debug)]
pub struct CreateAgentDraft {
    agent_id: String,
    branch_id: String,
    draft: Value,
}

impl CreateAgentDraft {
    pub fn new(agent_id: impl Into<String>, branch_id: impl Into<String>, draft: Value) -> Self {
        Self {
            agent_id: agent_id.into(),
            branch_id: branch_id.into(),
            draft,
        }
    }
}

impl crate::endpoints::sealed::Sealed for CreateAgentDraft {}

impl ElevenLabsEndpoint for CreateAgentDraft {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/drafts";

    const METHOD: Method = Method::POST;

    type ResponseBody = Value;

    fn query_params(&self) -> Option<QueryValues> {
        Some(vec![("branch_id", self.branch_id.clone())])
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(self.draft.clone()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Delete the draft of an agent branch. The result is returned as raw JSON.
///
/// See [Delete Draft API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/delete-draft)
#[derive(Clone, Debug)]
pub struct DeleteAgentDraft {
    agent_id: String,
    branch_id: String,
}

impl DeleteAgentDraft {
    pub fn new(agent_id: impl Into<String>, branch_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            branch_id: branch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteAgentDraft {}

impl ElevenLabsEndpoint for DeleteAgentDraft {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/drafts";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = Value;

    fn query_params(&self) -> Option<QueryValues> {
        Some(vec![("branch_id", self.branch_id.clone())])
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/convai/agents/{agent_id}/duplicate — Duplicate Agent
// =============================================================================

/// Duplicates an agent.
///
/// See [Duplicate Agent API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/duplicate)
#[derive(Clone, Debug)]
pub struct DuplicateAgent {
    agent_id: String,
    name: Option<String>,
}

impl DuplicateAgent {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for DuplicateAgent {}

impl ElevenLabsEndpoint for DuplicateAgent {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/duplicate";

    const METHOD: Method = Method::POST;

    type ResponseBody = AgentIdResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::json!({ "name": self.name })))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A response carrying a single agent ID.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentIdResponse {
    pub agent_id: String,
}

// =============================================================================
// POST /v1/convai/agents/{agent_id}/run-tests — Run Tests
// =============================================================================

/// Runs the given tests against an agent. The request body is a raw JSON
/// run-tests payload.
///
/// See [Run Tests API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/run-tests)
#[derive(Clone, Debug)]
pub struct RunAgentTests {
    agent_id: String,
    body: Value,
}

impl RunAgentTests {
    pub fn new(agent_id: impl Into<String>, body: Value) -> Self {
        Self {
            agent_id: agent_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for RunAgentTests {}

impl ElevenLabsEndpoint for RunAgentTests {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/run-tests";

    const METHOD: Method = Method::POST;

    type ResponseBody = TestSuiteInvocation;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(self.body.clone()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// Simulate conversation
// =============================================================================

/// Simulates a conversation against an agent. The request body is a raw JSON
/// simulation specification.
///
/// See [Simulate Conversation API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/simulate-conversation)
#[derive(Clone, Debug)]
pub struct SimulateConversation {
    agent_id: String,
    body: Value,
}

impl SimulateConversation {
    pub fn new(agent_id: impl Into<String>, body: Value) -> Self {
        Self {
            agent_id: agent_id.into(),
            body,
        }
    }
}

impl crate::endpoints::sealed::Sealed for SimulateConversation {}

impl ElevenLabsEndpoint for SimulateConversation {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/simulate-conversation";

    const METHOD: Method = Method::POST;

    type ResponseBody = SimulateConversationResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(self.body.clone()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`SimulateConversation`]. The transcript and analysis are
/// preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct SimulateConversationResponse {
    pub simulated_conversation: Vec<Value>,
    pub analysis: Value,
}

/// Streams a simulated conversation against an agent. The request body is a raw
/// JSON simulation specification; the response is a stream of raw bytes.
///
/// See [Simulate Conversation Stream API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/simulate-conversation-stream)
#[derive(Clone, Debug)]
pub struct SimulateConversationStream {
    agent_id: String,
    body: Value,
}

impl SimulateConversationStream {
    pub fn new(agent_id: impl Into<String>, body: Value) -> Self {
        Self {
            agent_id: agent_id.into(),
            body,
        }
    }
}

type SimulateConversationStreamResponse = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
impl crate::endpoints::sealed::Sealed for SimulateConversationStream {}

impl ElevenLabsEndpoint for SimulateConversationStream {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/simulate-conversation/stream";

    const METHOD: Method = Method::POST;

    type ResponseBody = SimulateConversationStreamResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(self.body.clone()))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        let stream = resp.bytes_stream().map(|r| r.map_err(Into::into));
        Ok(Box::pin(stream))
    }
}

// =============================================================================
// GET /v1/convai/agents/{agent_id}/topics — Get Agent Topics
// =============================================================================

/// Gets an agent's conversation topics over a time window.
///
/// See [Get Agent Topics API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/topics)
#[derive(Clone, Debug)]
pub struct GetAgentTopics {
    agent_id: String,
    query: Option<AgentTopicsQuery>,
}

impl GetAgentTopics {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: AgentTopicsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetAgentTopics`].
#[derive(Clone, Debug, Default)]
pub struct AgentTopicsQuery {
    params: QueryValues,
}

impl AgentTopicsQuery {
    pub fn with_from_unix_secs(mut self, from_unix_secs: i64) -> Self {
        self.params
            .push(("from_unix_secs", from_unix_secs.to_string()));
        self
    }

    pub fn with_to_unix_secs(mut self, to_unix_secs: i64) -> Self {
        self.params.push(("to_unix_secs", to_unix_secs.to_string()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetAgentTopics {}

impl ElevenLabsEndpoint for GetAgentTopics {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/topics";

    const METHOD: Method = Method::GET;

    type ResponseBody = AgentTopicsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`GetAgentTopics`]. Each topic is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentTopicsResponse {
    pub topics: Vec<Value>,
    pub window_start_unix_secs: i64,
    pub window_end_unix_secs: i64,
}

// =============================================================================
// GET /v1/convai/agents/{agent_id}/versions/{version_id} — Get Agent Version
// =============================================================================

/// Gets metadata for a specific agent version. The metadata is returned as raw
/// JSON.
///
/// See [Get Agent Version API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/get-version)
#[derive(Clone, Debug)]
pub struct GetAgentVersion {
    agent_id: String,
    version_id: String,
}

impl GetAgentVersion {
    pub fn new(agent_id: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            version_id: version_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetAgentVersion {}

impl ElevenLabsEndpoint for GetAgentVersion {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/versions/:version_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.version_id.and_param(PathParam::VersionID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// The agent avatar (`POST .../avatar`) and widget (`GET .../widget`) endpoints
// live in the [`widget`](super::widget) module (`CreateWidgetAvatar`,
// `GetWidget`).
