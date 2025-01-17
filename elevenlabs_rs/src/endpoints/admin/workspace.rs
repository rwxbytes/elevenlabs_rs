//! The workspace endpoints
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

