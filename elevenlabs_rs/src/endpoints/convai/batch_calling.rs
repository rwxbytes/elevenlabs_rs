//! The Conversational AI batch-calling endpoints.
//!
//! Batch calls dispatch an outbound conversational-AI call to many recipients.
//! Submit a batch, list a workspace's batches, fetch a batch's details, and
//! cancel, retry, or delete a batch.
//!
//! See the [Batch Calling API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/batch-calling).

use super::*;
use crate::endpoints::convai::conversations::ConversationInitiationClientData;
use std::collections::HashMap;

// =============================================================================
// Shared types
// =============================================================================

/// The status of a batch call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchCallStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// The status of an individual recipient within a batch call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchCallRecipientStatus {
    Pending,
    Dispatched,
    Initiated,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Voicemail,
}

/// The telephony provider used to dispatch a batch call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelephonyProvider {
    Twilio,
    SipTrunk,
    Exotel,
}

/// Provider-agnostic telephony call configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TelephonyCallConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ringing_timeout_secs: Option<u32>,
}

impl TelephonyCallConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ringing_timeout_secs(mut self, ringing_timeout_secs: u32) -> Self {
        self.ringing_timeout_secs = Some(ringing_timeout_secs);
        self
    }
}

/// WhatsApp parameters for a batch call.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchCallWhatsAppParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whatsapp_phone_number_id: Option<String>,
    pub whatsapp_call_permission_request_template_name: String,
    pub whatsapp_call_permission_request_template_language_code: String,
}

impl BatchCallWhatsAppParams {
    pub fn new(
        template_name: impl Into<String>,
        template_language_code: impl Into<String>,
    ) -> Self {
        Self {
            whatsapp_phone_number_id: None,
            whatsapp_call_permission_request_template_name: template_name.into(),
            whatsapp_call_permission_request_template_language_code: template_language_code.into(),
        }
    }

    pub fn with_whatsapp_phone_number_id(
        mut self,
        whatsapp_phone_number_id: impl Into<String>,
    ) -> Self {
        self.whatsapp_phone_number_id = Some(whatsapp_phone_number_id.into());
        self
    }
}

/// A recipient of a batch call.
#[derive(Clone, Debug, Default, Serialize)]
pub struct OutboundCallRecipient {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whatsapp_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}

impl OutboundCallRecipient {
    /// A recipient identified by phone number.
    pub fn phone_number(phone_number: impl Into<String>) -> Self {
        Self {
            phone_number: Some(phone_number.into()),
            ..Default::default()
        }
    }

    /// A recipient identified by WhatsApp user ID.
    pub fn whatsapp_user(whatsapp_user_id: impl Into<String>) -> Self {
        Self {
            whatsapp_user_id: Some(whatsapp_user_id.into()),
            ..Default::default()
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Per-recipient overrides applied when the conversation is initiated.
    pub fn with_conversation_initiation_client_data(
        mut self,
        data: ConversationInitiationClientData,
    ) -> Self {
        self.conversation_initiation_client_data = Some(data);
        self
    }
}

// =============================================================================
// POST /v1/convai/batch-calling/submit — Submit Batch Call
// =============================================================================

/// Submits a batch call request.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::batch_calling::{
///     OutboundCallRecipient, SubmitBatchCall, SubmitBatchCallBody,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = SubmitBatchCallBody::new(
///         "Spring campaign",
///         "agent_id",
///         [OutboundCallRecipient::phone_number("+15551234567")],
///     )
///     .with_agent_phone_number_id("phone_number_id");
///     let batch = c.hit(SubmitBatchCall::new(body)).await?;
///     println!("{}: {:?}", batch.id, batch.status);
///     Ok(())
/// }
/// ```
/// See [Submit Batch Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/batch-calling/create).
#[derive(Clone, Debug)]
pub struct SubmitBatchCall {
    body: SubmitBatchCallBody,
}

impl SubmitBatchCall {
    pub fn new(body: SubmitBatchCallBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for SubmitBatchCall {}

impl ElevenLabsEndpoint for SubmitBatchCall {
    const PATH: &'static str = "/v1/convai/batch-calling/submit";

    const METHOD: Method = Method::POST;

    type ResponseBody = BatchCall;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Submit-batch-call body.
#[derive(Clone, Debug, Serialize)]
pub struct SubmitBatchCallBody {
    call_name: String,
    agent_id: String,
    recipients: Vec<OutboundCallRecipient>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_time_unix: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_phone_number_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    whatsapp_params: Option<BatchCallWhatsAppParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    telephony_call_config: Option<TelephonyCallConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_concurrency_limit: Option<u32>,
}

impl SubmitBatchCallBody {
    pub fn new(
        call_name: impl Into<String>,
        agent_id: impl Into<String>,
        recipients: impl IntoIterator<Item = OutboundCallRecipient>,
    ) -> Self {
        Self {
            call_name: call_name.into(),
            agent_id: agent_id.into(),
            recipients: recipients.into_iter().collect(),
            scheduled_time_unix: None,
            agent_phone_number_id: None,
            whatsapp_params: None,
            timezone: None,
            branch_id: None,
            environment: None,
            telephony_call_config: None,
            target_concurrency_limit: None,
        }
    }

    /// The phone number, by ID, to place the calls from.
    pub fn with_agent_phone_number_id(mut self, agent_phone_number_id: impl Into<String>) -> Self {
        self.agent_phone_number_id = Some(agent_phone_number_id.into());
        self
    }

    /// Schedule the batch to start at the given Unix time (seconds).
    pub fn with_scheduled_time_unix(mut self, scheduled_time_unix: i64) -> Self {
        self.scheduled_time_unix = Some(scheduled_time_unix);
        self
    }

    pub fn with_whatsapp_params(mut self, whatsapp_params: BatchCallWhatsAppParams) -> Self {
        self.whatsapp_params = Some(whatsapp_params);
        self
    }

    pub fn with_timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.branch_id = Some(branch_id.into());
        self
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    pub fn with_telephony_call_config(
        mut self,
        telephony_call_config: TelephonyCallConfig,
    ) -> Self {
        self.telephony_call_config = Some(telephony_call_config);
        self
    }

    pub fn with_target_concurrency_limit(mut self, target_concurrency_limit: u32) -> Self {
        self.target_concurrency_limit = Some(target_concurrency_limit);
        self
    }
}

// =============================================================================
// GET /v1/convai/batch-calling/workspace — List Workspace Batch Calls
// =============================================================================

/// Lists all batch calls for the workspace.
///
/// See [List Workspace Batch Calls API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/batch-calling/list).
#[derive(Clone, Debug, Default)]
pub struct ListWorkspaceBatchCalls {
    query: Option<ListWorkspaceBatchCallsQuery>,
}

impl ListWorkspaceBatchCalls {
    pub fn with_query(mut self, query: ListWorkspaceBatchCallsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`ListWorkspaceBatchCalls`].
#[derive(Clone, Debug, Default)]
pub struct ListWorkspaceBatchCallsQuery {
    params: QueryValues,
}

impl ListWorkspaceBatchCallsQuery {
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.params.push(("limit", limit.to_string()));
        self
    }

    /// Cursor for pagination, from a previous response's `next_doc`.
    pub fn with_last_doc(mut self, last_doc: impl Into<String>) -> Self {
        self.params.push(("last_doc", last_doc.into()));
        self
    }

    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.params.push(("agent_id", agent_id.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListWorkspaceBatchCalls {}

impl ElevenLabsEndpoint for ListWorkspaceBatchCalls {
    const PATH: &'static str = "/v1/convai/batch-calling/workspace";

    const METHOD: Method = Method::GET;

    type ResponseBody = WorkspaceBatchCalls;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of a workspace's batch calls.
#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceBatchCalls {
    pub batch_calls: Vec<BatchCall>,
    pub next_doc: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

impl IntoIterator for WorkspaceBatchCalls {
    type Item = BatchCall;
    type IntoIter = std::vec::IntoIter<BatchCall>;

    fn into_iter(self) -> Self::IntoIter {
        self.batch_calls.into_iter()
    }
}

// =============================================================================
// GET /v1/convai/batch-calling/{batch_id} — Get Batch Call
// =============================================================================

/// Retrieves a batch call by ID, including its recipients.
///
/// See [Get Batch Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/batch-calling/get).
#[derive(Clone, Debug)]
pub struct GetBatchCall {
    batch_id: String,
}

impl GetBatchCall {
    pub fn new(batch_id: impl Into<String>) -> Self {
        Self {
            batch_id: batch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetBatchCall {}

impl ElevenLabsEndpoint for GetBatchCall {
    const PATH: &'static str = "/v1/convai/batch-calling/:batch_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = BatchCallDetailed;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.batch_id.and_param(PathParam::BatchID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/convai/batch-calling/{batch_id}/cancel — Cancel Batch Call
// =============================================================================

/// Cancels a batch call.
///
/// See [Cancel Batch Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/batch-calling/cancel).
#[derive(Clone, Debug)]
pub struct CancelBatchCall {
    batch_id: String,
}

impl CancelBatchCall {
    pub fn new(batch_id: impl Into<String>) -> Self {
        Self {
            batch_id: batch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for CancelBatchCall {}

impl ElevenLabsEndpoint for CancelBatchCall {
    const PATH: &'static str = "/v1/convai/batch-calling/:batch_id/cancel";

    const METHOD: Method = Method::POST;

    type ResponseBody = BatchCall;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.batch_id.and_param(PathParam::BatchID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/convai/batch-calling/{batch_id}/retry — Retry Batch Call
// =============================================================================

/// Retries the failed/no-answer recipients of a batch call.
///
/// See [Retry Batch Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/batch-calling/retry).
#[derive(Clone, Debug)]
pub struct RetryBatchCall {
    batch_id: String,
}

impl RetryBatchCall {
    pub fn new(batch_id: impl Into<String>) -> Self {
        Self {
            batch_id: batch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for RetryBatchCall {}

impl ElevenLabsEndpoint for RetryBatchCall {
    const PATH: &'static str = "/v1/convai/batch-calling/:batch_id/retry";

    const METHOD: Method = Method::POST;

    type ResponseBody = BatchCall;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.batch_id.and_param(PathParam::BatchID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/convai/batch-calling/{batch_id} — Delete Batch Call
// =============================================================================

/// Deletes a batch call.
///
/// See [Delete Batch Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/batch-calling/delete).
#[derive(Clone, Debug)]
pub struct DeleteBatchCall {
    batch_id: String,
}

impl DeleteBatchCall {
    pub fn new(batch_id: impl Into<String>) -> Self {
        Self {
            batch_id: batch_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteBatchCall {}

impl ElevenLabsEndpoint for DeleteBatchCall {
    const PATH: &'static str = "/v1/convai/batch-calling/:batch_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.batch_id.and_param(PathParam::BatchID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

// =============================================================================
// Response models
// =============================================================================

/// A batch call's metadata and aggregate progress counters.
#[derive(Clone, Debug, Deserialize)]
pub struct BatchCall {
    pub id: String,
    pub name: String,
    pub agent_id: String,
    pub agent_name: String,
    pub phone_number_id: Option<String>,
    pub phone_provider: Option<TelephonyProvider>,
    pub whatsapp_params: Option<BatchCallWhatsAppParams>,
    pub branch_id: Option<String>,
    pub branch_name: Option<String>,
    pub environment: Option<String>,
    pub created_at_unix: i64,
    pub scheduled_time_unix: Option<i64>,
    pub timezone: Option<String>,
    pub last_updated_at_unix: i64,
    pub status: BatchCallStatus,
    pub retry_count: u32,
    pub total_calls_dispatched: u32,
    pub total_calls_scheduled: u32,
    pub total_calls_finished: u32,
    pub telephony_call_config: TelephonyCallConfig,
    pub target_concurrency_limit: Option<u32>,
}

/// A batch call together with its recipients.
#[derive(Clone, Debug, Deserialize)]
pub struct BatchCallDetailed {
    #[serde(flatten)]
    pub batch_call: BatchCall,
    pub recipients: Vec<OutboundCallRecipientResponse>,
}

/// A recipient of a batch call, as returned by the API.
#[derive(Clone, Debug, Deserialize)]
pub struct OutboundCallRecipientResponse {
    pub id: String,
    pub phone_number: Option<String>,
    pub whatsapp_user_id: Option<String>,
    pub status: BatchCallRecipientStatus,
    pub created_at_unix: i64,
    pub updated_at_unix: i64,
    pub conversation_id: Option<String>,
    /// The per-recipient conversation overrides, preserved as raw JSON.
    pub conversation_initiation_client_data: Option<HashMap<String, Value>>,
}
