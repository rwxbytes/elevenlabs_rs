//! The workspace endpoints

use std::collections::HashMap;
use strum::Display;
use std::string::ToString;
use crate::endpoints::convai::agents::AccessLevel;
use super::*;

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
    pub fn new(body: impl Into<InviteUserBody> ) -> Self {
        Self { body: body.into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InviteUserBody {
    email: String,
}

impl InviteUserBody {
    pub fn new(email: &str) -> Self {
        Self { email: email.to_string()}
    }
}

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
    pub fn new(body: impl Into<DeleteInvitationBody> ) -> Self {
        Self { body: body.into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteInvitationBody {
    email: String,
}

impl DeleteInvitationBody {
    pub fn new(email: &str) -> Self {
        Self { email: email.to_string()}
    }
}

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
pub struct  UpdateMember {
    body: UpdateMemberBody
}

impl UpdateMember {
    pub fn new(body: impl Into<UpdateMemberBody>) -> Self {
        Self { body: body.into() }
    }
}


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
        Self { email: email.to_string(), is_locked: None, workspace_role: None }
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
        Self { resource_id: resource_id.into(), query: query.into() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetResourceQuery {
    pub params: QueryValues
}

impl GetResourceQuery {
    pub fn with_resource_type(mut self, resource_type: ResourceType) -> Self {
        self.params.push(("resource_type", resource_type.to_string()));
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
    ConvaiPhoneNumbers
}

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
    Key
}

/// See [Share Workspace Resource API reference](https://elevenlabs.io/docs/api-reference/workspace/share-workspace-resource)
#[derive(Debug, Clone)]
pub struct ShareWorkspaceResource {
    resource_id: String,
    body: ShareWorkspaceResourceBody,
}

impl ShareWorkspaceResource {
    pub fn new(resource_id: impl Into<String>, body: ShareWorkspaceResourceBody) -> Self {
        Self { resource_id: resource_id.into(), body: body.into() }
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
        Self { role, resource_type, user_email: None, group_email: None, workspace_api_key_id: None }
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
        Self { resource_id: resource_id.into(), body: body.into() }
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
        Self { resource_type, user_email: None, group_email: None, workspace_api_key_id: None }
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




