//! The workspace endpoints

use super::*;
use crate::shared::AccessLevel;
use std::collections::HashMap;
use std::string::ToString;
use strum::Display;

/// Sends an email invitation to join your workspace to the provided email.
///
/// If the user doesn’t have an account they will be prompted to create one.
/// If the user accepts this invite they will be added as a user to your workspace
/// and your subscription using one of your seats.
/// This endpoint may only be called by workspace administrators.
///
/// # Example
/// ``` no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::workspace::InviteUser;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///
///    let endpoint = InviteUser::new("undecim@laboratorium.com");
///    let resp = c.hit(endpoint).await?;
///
///    println!("{:#?}", resp);
///
///  Ok(())
/// }
/// ```
/// See [Invite User API reference](https://elevenlabs.io/docs/api-reference/workspace/invite-user)
#[derive(Debug, Clone)]
pub struct InviteUser {
    body: InviteUserBody,
}

impl InviteUser {
    pub fn new(body: impl Into<InviteUserBody>) -> Self {
        Self { body: body.into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InviteUserBody {
    email: String,
}

impl InviteUserBody {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for InviteUser {}

impl ElevenLabsEndpoint for InviteUser {
    const PATH: &'static str = "/v1/workspace/invites/add";

    const METHOD: Method = Method::POST;

    type ResponseBody = InvitationResponseBody;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvitationResponseBody {
    pub key: String,
}

impl From<&str> for InviteUserBody {
    fn from(email: &str) -> Self {
        Self::new(email)
    }
}

/// Invalidates an existing email invitation.
///
/// The invitation will still show up in the inbox it has been delivered to,
/// but activating it to join the workspace won’t work.
/// This endpoint may only be called by workspace administrators.
///
/// # Example
/// ``` no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::workspace::DeleteInvitation;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///
///   let endpoint = DeleteInvitation::new("foo@baz.com");
///   let resp = c.hit(endpoint).await?;
///
///   println!("{:#?}", resp);
///
///   Ok(())
/// }
/// ```
/// See [Delete Invitation API reference](https://elevenlabs.io/docs/api-reference/workspace/delete-existing-invitation)
#[derive(Debug, Clone)]
pub struct DeleteInvitation {
    body: DeleteInvitationBody,
}

impl DeleteInvitation {
    pub fn new(body: impl Into<DeleteInvitationBody>) -> Self {
        Self { body: body.into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteInvitationBody {
    email: String,
}

impl DeleteInvitationBody {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteInvitation {}

impl ElevenLabsEndpoint for DeleteInvitation {
    const PATH: &'static str = "/v1/workspace/invites";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = InvitationResponseBody;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

impl From<&str> for DeleteInvitationBody {
    fn from(email: &str) -> Self {
        Self::new(email)
    }
}

/// Updates attributes of a workspace member.
///
/// Apart from the email identifier, all parameters will remain unchanged unless specified.
/// This endpoint may only be called by workspace administrators.
///
/// # Example
/// ``` no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::workspace::{UpdateMember, UpdateMemberBody, WorkspaceRole};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///
///    let body = UpdateMemberBody::new("undecim@laboratorium.com")
///      .with_is_locked(true)
///      .with_workspace_role(WorkspaceRole::WorkspaceAdmin);
///
///    let resp = c.hit(UpdateMember::new(body)).await?;
///
///    println!("{:#?}", resp);
///
///    Ok(())
/// }
/// ```
/// See [Update Member API reference](https://elevenlabs.io/docs/api-reference/workspace/update-member)
#[derive(Debug, Clone)]
pub struct UpdateMember {
    body: UpdateMemberBody,
}

impl UpdateMember {
    pub fn new(body: impl Into<UpdateMemberBody>) -> Self {
        Self { body: body.into() }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateMember {}

impl ElevenLabsEndpoint for UpdateMember {
    const PATH: &'static str = "/v1/workspace/members";

    const METHOD: Method = Method::POST;

    type ResponseBody = InvitationResponseBody;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMemberBody {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_locked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_role: Option<WorkspaceRole>,
}

impl UpdateMemberBody {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
            is_locked: None,
            workspace_role: None,
        }
    }

    pub fn with_is_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = Some(is_locked.to_string());
        self
    }

    pub fn with_workspace_role(mut self, workspace_role: WorkspaceRole) -> Self {
        self.workspace_role = Some(workspace_role);
        self
    }
}

impl From<&str> for UpdateMemberBody {
    fn from(email: &str) -> Self {
        Self::new(email)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceRole {
    WorkspaceAdmin,
    WorkspaceMember,
}

/// Gets the metadata of a resource by ID.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::workspace::{GetResource, GetResourceQuery, ResourceType};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let q = GetResourceQuery::default().with_resource_type(ResourceType::Voice);
///    let endpoint = GetResource::new("id", q);
///
///    let resp = c.hit(endpoint).await?;
///
///   println!("{:#?}", resp);
///
///   Ok(())
/// }
/// ```
/// See [Get Resource API reference](https://elevenlabs.io/docs/api-reference/workspace/get-resource)
#[derive(Debug, Clone)]
pub struct GetResource {
    resource_id: String,
    query: GetResourceQuery,
}

impl GetResource {
    pub fn new(resource_id: impl Into<String>, query: impl Into<GetResourceQuery>) -> Self {
        Self {
            resource_id: resource_id.into(),
            query: query.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetResourceQuery {
    pub params: QueryValues,
}

impl GetResourceQuery {
    pub fn with_resource_type(mut self, resource_type: ResourceType) -> Self {
        self.params
            .push(("resource_type", resource_type.to_string()));
        self
    }
}

#[derive(Debug, Clone, Deserialize, Display, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ResourceType {
    Voice,
    PronunciationDictionary,
    Dubbing,
    Project,
    ConvaiAgents,
    ConvaiKnowledgeBaseDocuments,
    ConvaiTools,
    ConvaiSettings,
    ConvaiSecrets,
    MusicLatent,
    ConvaiPhoneNumbers,
}

impl crate::endpoints::sealed::Sealed for GetResource {}

impl ElevenLabsEndpoint for GetResource {
    const PATH: &'static str = "/v1/workspace/resources/:resource_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = ResourceResponseBody;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.resource_id.and_param(PathParam::ResourceID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceResponseBody {
    pub resource_id: String,
    pub resource_type: ResourceType,
    pub role_to_group_ids: HashMap<String, Vec<String>>,
    pub share_options: Vec<ShareOption>,
    pub creator_user_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShareOption {
    pub name: String,
    pub id: String,
    pub r#type: PrincipalRole,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PrincipalRole {
    User,
    Group,
    Key,
}

/// See [Share Workspace Resource API reference](https://elevenlabs.io/docs/api-reference/workspace/share-workspace-resource)
#[derive(Debug, Clone)]
pub struct ShareWorkspaceResource {
    resource_id: String,
    body: ShareWorkspaceResourceBody,
}

impl ShareWorkspaceResource {
    pub fn new(resource_id: impl Into<String>, body: ShareWorkspaceResourceBody) -> Self {
        Self {
            resource_id: resource_id.into(),
            body,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ShareWorkspaceResourceBody {
    pub role: AccessLevel,
    pub resource_type: ResourceType,
    user_email: Option<String>,
    group_email: Option<String>,
    workspace_api_key_id: Option<String>,
}

impl ShareWorkspaceResourceBody {
    pub fn new(role: AccessLevel, resource_type: ResourceType) -> Self {
        Self {
            role,
            resource_type,
            user_email: None,
            group_email: None,
            workspace_api_key_id: None,
        }
    }

    pub fn with_user_email(mut self, user_email: &str) -> Self {
        self.user_email = Some(user_email.to_string());
        self
    }

    pub fn with_group_email(mut self, group_email: &str) -> Self {
        self.group_email = Some(group_email.to_string());
        self
    }

    pub fn with_workspace_api_key_id(mut self, workspace_api_key_id: &str) -> Self {
        self.workspace_api_key_id = Some(workspace_api_key_id.to_string());
        self
    }
}

impl crate::endpoints::sealed::Sealed for ShareWorkspaceResource {}

impl ElevenLabsEndpoint for ShareWorkspaceResource {
    const PATH: &'static str = "/v1/workspace/resources/:resource_id/share";

    const METHOD: Method = Method::POST;

    type ResponseBody = ShareWorkspaceResourceResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.resource_id.and_param(PathParam::ResourceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShareWorkspaceResourceResponse {
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct UnshareWorkspaceResource {
    resource_id: String,
    body: UnshareWorkspaceResourceBody,
}

impl UnshareWorkspaceResource {
    pub fn new(resource_id: impl Into<String>, body: UnshareWorkspaceResourceBody) -> Self {
        Self {
            resource_id: resource_id.into(),
            body,
        }
    }
}

/// See [Unshare Workspace Resource API reference](https://elevenlabs.io/docs/api-reference/workspace/unshare-workspace-resource)
#[derive(Debug, Clone, Serialize)]
pub struct UnshareWorkspaceResourceBody {
    pub resource_type: ResourceType,
    pub user_email: Option<String>,
    pub group_email: Option<String>,
    pub workspace_api_key_id: Option<String>,
}

impl UnshareWorkspaceResourceBody {
    pub fn new(resource_type: ResourceType) -> Self {
        Self {
            resource_type,
            user_email: None,
            group_email: None,
            workspace_api_key_id: None,
        }
    }

    pub fn with_user_email(mut self, user_email: &str) -> Self {
        self.user_email = Some(user_email.to_string());
        self
    }

    pub fn with_group_email(mut self, group_email: &str) -> Self {
        self.group_email = Some(group_email.to_string());
        self
    }

    pub fn with_workspace_api_key_id(mut self, workspace_api_key_id: &str) -> Self {
        self.workspace_api_key_id = Some(workspace_api_key_id.to_string());
        self
    }
}

impl crate::endpoints::sealed::Sealed for UnshareWorkspaceResource {}

impl ElevenLabsEndpoint for UnshareWorkspaceResource {
    const PATH: &'static str = "/v1/workspace/resources/:resource_id/unshare";

    const METHOD: Method = Method::POST;

    type ResponseBody = ShareWorkspaceResourceResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.resource_id.and_param(PathParam::ResourceID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// GET /v1/workspace/audit-logs — Get Workspace Audit Logs
// =============================================================================

/// Retrieves a paginated list of workspace audit-log entries.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::workspace::{GetWorkspaceAuditLogs, AuditLogsQuery};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let endpoint = GetWorkspaceAuditLogs::default().with_query(AuditLogsQuery::default().with_limit(50));
///     let page = c.hit(endpoint).await?;
///     for entry in &page.entries {
///         println!("{}: {}", entry.activity_name, entry.message);
///     }
///     Ok(())
/// }
/// ```
/// See [Get Workspace Audit Logs API reference](https://elevenlabs.io/docs/api-reference/workspace/get-audit-logs).
#[derive(Clone, Debug, Default)]
pub struct GetWorkspaceAuditLogs {
    query: Option<AuditLogsQuery>,
}

impl GetWorkspaceAuditLogs {
    pub fn with_query(mut self, query: AuditLogsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`GetWorkspaceAuditLogs`].
#[derive(Clone, Debug, Default)]
pub struct AuditLogsQuery {
    params: QueryValues,
}

impl AuditLogsQuery {
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.params.push(("limit", limit.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    pub fn with_time_from_unix_ms(mut self, time_from_unix_ms: i64) -> Self {
        self.params
            .push(("time_from_unix_ms", time_from_unix_ms.to_string()));
        self
    }

    pub fn with_time_to_unix_ms(mut self, time_to_unix_ms: i64) -> Self {
        self.params
            .push(("time_to_unix_ms", time_to_unix_ms.to_string()));
        self
    }

    pub fn with_actor_uid(mut self, actor_uid: impl Into<String>) -> Self {
        self.params.push(("actor_uid", actor_uid.into()));
        self
    }

    pub fn with_class_name(mut self, class_name: impl Into<String>) -> Self {
        self.params.push(("class_name", class_name.into()));
        self
    }

    pub fn with_activity_name(mut self, activity_name: impl Into<String>) -> Self {
        self.params.push(("activity_name", activity_name.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetWorkspaceAuditLogs {}

impl ElevenLabsEndpoint for GetWorkspaceAuditLogs {
    const PATH: &'static str = "/v1/workspace/audit-logs";

    const METHOD: Method = Method::GET;

    type ResponseBody = WorkspaceAuditLogsPage;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of workspace audit-log entries.
#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceAuditLogsPage {
    pub entries: Vec<AuditLogEntry>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// A single audit-log entry. The entry follows the OCSF schema; the commonly
/// used fields are typed and the full payload is preserved in `extra`.
#[derive(Clone, Debug, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub time: Option<i64>,
    pub activity_name: String,
    pub category_name: Option<String>,
    pub class_name: Option<String>,
    pub message: String,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// =============================================================================
// GET /v1/workspace/groups — Get All Groups
// =============================================================================

/// Retrieves all workspace groups, keyed by group ID.
///
/// See [Get Workspace Groups API reference](https://elevenlabs.io/docs/api-reference/workspace/get-groups).
#[derive(Clone, Debug, Default)]
pub struct GetWorkspaceGroups;

impl crate::endpoints::sealed::Sealed for GetWorkspaceGroups {}

impl ElevenLabsEndpoint for GetWorkspaceGroups {
    const PATH: &'static str = "/v1/workspace/groups";

    const METHOD: Method = Method::GET;

    type ResponseBody = HashMap<String, WorkspaceGroup>;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A workspace group.
#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceGroup {
    pub name: String,
    pub id: String,
    pub members: Vec<String>,
    pub permissions: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// =============================================================================
// GET /v1/workspace/groups/search — Search User Groups
// =============================================================================

/// Searches workspace groups by name.
///
/// See [Search Workspace Groups API reference](https://elevenlabs.io/docs/api-reference/workspace/search-groups).
#[derive(Clone, Debug)]
pub struct SearchWorkspaceGroups {
    name: String,
}

impl SearchWorkspaceGroups {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl crate::endpoints::sealed::Sealed for SearchWorkspaceGroups {}

impl ElevenLabsEndpoint for SearchWorkspaceGroups {
    const PATH: &'static str = "/v1/workspace/groups/search";

    const METHOD: Method = Method::GET;

    type ResponseBody = Vec<WorkspaceGroupByName>;

    fn query_params(&self) -> Option<QueryValues> {
        Some(vec![("name", self.name.clone())])
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A workspace group returned by a name search.
#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceGroupByName {
    pub name: String,
    pub id: String,
    pub members_emails: Vec<String>,
}

// =============================================================================
// POST /v1/workspace/groups/{group_id}/members — Add Member To Group
// =============================================================================

/// Adds a workspace member to a group.
///
/// See [Add Member To Group API reference](https://elevenlabs.io/docs/api-reference/workspace/add-group-member).
#[derive(Clone, Debug)]
pub struct AddMemberToGroup {
    group_id: String,
    email: String,
}

impl AddMemberToGroup {
    pub fn new(group_id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            email: email.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for AddMemberToGroup {}

impl ElevenLabsEndpoint for AddMemberToGroup {
    const PATH: &'static str = "/v1/workspace/groups/:group_id/members";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.group_id.and_param(PathParam::GroupID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "email": self.email }),
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/workspace/groups/{group_id}/members/remove — Remove Member From Group
// =============================================================================

/// Removes a workspace member from a group.
///
/// See [Remove Member From Group API reference](https://elevenlabs.io/docs/api-reference/workspace/remove-group-member).
#[derive(Clone, Debug)]
pub struct RemoveMemberFromGroup {
    group_id: String,
    email: String,
}

impl RemoveMemberFromGroup {
    pub fn new(group_id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            email: email.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for RemoveMemberFromGroup {}

impl ElevenLabsEndpoint for RemoveMemberFromGroup {
    const PATH: &'static str = "/v1/workspace/groups/:group_id/members/remove";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.group_id.and_param(PathParam::GroupID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "email": self.email }),
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// POST /v1/workspace/invites/add-bulk — Invite Multiple Users
// =============================================================================

/// Seat type assigned to invited workspace members.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeatType {
    WorkspaceAdmin,
    WorkspaceMember,
    WorkspaceLiteMember,
}

/// Invites multiple users to the workspace in a single request.
///
/// See [Invite Multiple Users API reference](https://elevenlabs.io/docs/api-reference/workspace/invite-users).
#[derive(Clone, Debug)]
pub struct InviteUsers {
    body: InviteUsersBody,
}

impl InviteUsers {
    pub fn new(body: InviteUsersBody) -> Self {
        Self { body }
    }
}

/// Bulk-invite body.
#[derive(Clone, Debug, Serialize)]
pub struct InviteUsersBody {
    emails: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seat_type: Option<SeatType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_ids: Option<Vec<String>>,
}

impl InviteUsersBody {
    pub fn new<I, S>(emails: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            emails: emails.into_iter().map(Into::into).collect(),
            seat_type: None,
            group_ids: None,
        }
    }

    pub fn with_seat_type(mut self, seat_type: SeatType) -> Self {
        self.seat_type = Some(seat_type);
        self
    }

    pub fn with_group_ids<I, S>(mut self, group_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.group_ids = Some(group_ids.into_iter().map(Into::into).collect());
        self
    }
}

impl crate::endpoints::sealed::Sealed for InviteUsers {}

impl ElevenLabsEndpoint for InviteUsers {
    const PATH: &'static str = "/v1/workspace/invites/add-bulk";

    const METHOD: Method = Method::POST;

    type ResponseBody = StatusResponseBody;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// GET /v1/workspace/webhooks — List Workspace Webhooks
// =============================================================================

/// Lists the webhooks configured for the workspace.
///
/// See [List Workspace Webhooks API reference](https://elevenlabs.io/docs/api-reference/workspace/get-webhooks).
#[derive(Clone, Debug, Default)]
pub struct GetWorkspaceWebhooks {
    include_usages: Option<bool>,
}

impl GetWorkspaceWebhooks {
    /// Include, for each webhook, the list of products configured to trigger it.
    pub fn with_include_usages(mut self, include_usages: bool) -> Self {
        self.include_usages = Some(include_usages);
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetWorkspaceWebhooks {}

impl ElevenLabsEndpoint for GetWorkspaceWebhooks {
    const PATH: &'static str = "/v1/workspace/webhooks";

    const METHOD: Method = Method::GET;

    type ResponseBody = WorkspaceWebhookList;

    fn query_params(&self) -> Option<QueryValues> {
        self.include_usages
            .map(|include| vec![("include_usages", include.to_string())])
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A list of workspace webhooks.
#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceWebhookList {
    pub webhooks: Vec<WorkspaceWebhook>,
}

/// A configured workspace webhook.
#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceWebhook {
    pub name: String,
    pub webhook_id: String,
    pub webhook_url: String,
    pub is_disabled: bool,
    pub is_auto_disabled: bool,
    pub created_at_unix: i64,
    pub auth_type: String,
    pub usage: Option<Vec<WorkspaceWebhookUsage>>,
    pub most_recent_failure_error_code: Option<i64>,
    pub most_recent_failure_timestamp: Option<i64>,
}

/// A product configured to trigger a workspace webhook.
#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceWebhookUsage {
    pub usage_type: String,
}

// =============================================================================
// POST /v1/workspace/webhooks — Create Workspace Webhook
// =============================================================================

/// Creates an HMAC-authenticated workspace webhook.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::workspace::{CreateWorkspaceWebhook, WebhookHmacSettings};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let settings = WebhookHmacSettings::new("My Webhook", "https://example.com/callback");
///     let resp = c.hit(CreateWorkspaceWebhook::new(settings)).await?;
///     println!("{}", resp.webhook_id);
///     Ok(())
/// }
/// ```
/// See [Create Workspace Webhook API reference](https://elevenlabs.io/docs/api-reference/workspace/create-webhook).
#[derive(Clone, Debug)]
pub struct CreateWorkspaceWebhook {
    settings: WebhookHmacSettings,
}

impl CreateWorkspaceWebhook {
    pub fn new(settings: WebhookHmacSettings) -> Self {
        Self { settings }
    }
}

/// Settings for creating an HMAC-authenticated webhook.
#[derive(Clone, Debug, Serialize)]
pub struct WebhookHmacSettings {
    auth_type: &'static str,
    name: String,
    webhook_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_headers: Option<HashMap<String, String>>,
}

impl WebhookHmacSettings {
    pub fn new(name: impl Into<String>, webhook_url: impl Into<String>) -> Self {
        Self {
            auth_type: "hmac",
            name: name.into(),
            webhook_url: webhook_url.into(),
            request_headers: None,
        }
    }

    /// Custom request headers to include with each webhook delivery.
    pub fn with_request_headers(mut self, request_headers: HashMap<String, String>) -> Self {
        self.request_headers = Some(request_headers);
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateWorkspaceWebhook {}

impl ElevenLabsEndpoint for CreateWorkspaceWebhook {
    const PATH: &'static str = "/v1/workspace/webhooks";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateWorkspaceWebhookResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "settings": self.settings }),
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`CreateWorkspaceWebhook`].
#[derive(Clone, Debug, Deserialize)]
pub struct CreateWorkspaceWebhookResponse {
    pub webhook_id: String,
    pub webhook_secret: Option<String>,
}

// =============================================================================
// PATCH /v1/workspace/webhooks/{webhook_id} — Update Workspace Webhook
// =============================================================================

/// Updates a workspace webhook, e.g. to disable it or rename it.
///
/// See [Update Workspace Webhook API reference](https://elevenlabs.io/docs/api-reference/workspace/update-webhook).
#[derive(Clone, Debug)]
pub struct UpdateWorkspaceWebhook {
    webhook_id: String,
    body: UpdateWorkspaceWebhookBody,
}

impl UpdateWorkspaceWebhook {
    pub fn new(webhook_id: impl Into<String>, body: UpdateWorkspaceWebhookBody) -> Self {
        Self {
            webhook_id: webhook_id.into(),
            body,
        }
    }
}

/// Update-webhook body. `is_disabled` and `name` are required by the API.
#[derive(Clone, Debug, Serialize)]
pub struct UpdateWorkspaceWebhookBody {
    is_disabled: bool,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_headers: Option<HashMap<String, String>>,
}

impl UpdateWorkspaceWebhookBody {
    pub fn new(name: impl Into<String>, is_disabled: bool) -> Self {
        Self {
            is_disabled,
            name: name.into(),
            retry_enabled: None,
            request_headers: None,
        }
    }

    /// Enable automatic retries for transient failures (5xx, 429, timeout).
    pub fn with_retry_enabled(mut self, retry_enabled: bool) -> Self {
        self.retry_enabled = Some(retry_enabled);
        self
    }

    pub fn with_request_headers(mut self, request_headers: HashMap<String, String>) -> Self {
        self.request_headers = Some(request_headers);
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateWorkspaceWebhook {}

impl ElevenLabsEndpoint for UpdateWorkspaceWebhook {
    const PATH: &'static str = "/v1/workspace/webhooks/:webhook_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.webhook_id.and_param(PathParam::WebhookID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/workspace/webhooks/{webhook_id} — Delete Workspace Webhook
// =============================================================================

/// Deletes a workspace webhook.
///
/// See [Delete Workspace Webhook API reference](https://elevenlabs.io/docs/api-reference/workspace/delete-webhook).
#[derive(Clone, Debug)]
pub struct DeleteWorkspaceWebhook {
    webhook_id: String,
}

impl DeleteWorkspaceWebhook {
    pub fn new(webhook_id: impl Into<String>) -> Self {
        Self {
            webhook_id: webhook_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteWorkspaceWebhook {}

impl ElevenLabsEndpoint for DeleteWorkspaceWebhook {
    const PATH: &'static str = "/v1/workspace/webhooks/:webhook_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.webhook_id.and_param(PathParam::WebhookID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}
