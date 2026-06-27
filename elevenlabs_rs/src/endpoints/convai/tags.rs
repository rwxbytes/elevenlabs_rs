//! The Conversational AI conversation-tags endpoints.
//!
//! Conversation tags are workspace-level labels that can be applied to
//! conversations. List, create, fetch, update, and delete them.
//!
//! See the [Conversation Tags API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tags).

use super::*;

/// A conversation tag.
#[derive(Clone, Debug, Deserialize)]
pub struct ConversationTag {
    pub tag_id: String,
    pub workspace_id: String,
    pub owner_user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at_unix_secs: i64,
}

// =============================================================================
// GET /v1/convai/tags — List Conversation Tags
// =============================================================================

/// Lists the workspace's conversation tags.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::tags::{ListConversationTags, ConversationTagsQuery};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint =
///         ListConversationTags::default().with_query(ConversationTagsQuery::default().with_page_size(50));
///     let resp = client.hit(endpoint).await?;
///     for tag in &resp.conversation_tags {
///         println!("{}: {}", tag.tag_id, tag.title);
///     }
///     Ok(())
/// }
/// ```
/// See [List Conversation Tags API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tags/list)
#[derive(Clone, Debug, Default)]
pub struct ListConversationTags {
    query: Option<ConversationTagsQuery>,
}

impl ListConversationTags {
    pub fn with_query(mut self, query: ConversationTagsQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// Query parameters for [`ListConversationTags`].
#[derive(Clone, Debug, Default)]
pub struct ConversationTagsQuery {
    params: QueryValues,
}

impl ConversationTagsQuery {
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListConversationTags {}

impl ElevenLabsEndpoint for ListConversationTags {
    const PATH: &'static str = "/v1/convai/tags";

    const METHOD: Method = Method::GET;

    type ResponseBody = ConversationTagsPage;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of conversation tags.
#[derive(Clone, Debug, Deserialize)]
pub struct ConversationTagsPage {
    pub conversation_tags: Vec<ConversationTag>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl IntoIterator for ConversationTagsPage {
    type Item = ConversationTag;
    type IntoIter = std::vec::IntoIter<ConversationTag>;

    fn into_iter(self) -> Self::IntoIter {
        self.conversation_tags.into_iter()
    }
}

// =============================================================================
// POST /v1/convai/tags — Create Conversation Tag
// =============================================================================

/// Creates a conversation tag.
///
/// See [Create Conversation Tag API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tags/create)
#[derive(Clone, Debug)]
pub struct CreateConversationTag {
    body: CreateConversationTagBody,
}

impl CreateConversationTag {
    pub fn new(body: CreateConversationTagBody) -> Self {
        Self { body }
    }
}

/// Create-conversation-tag body.
#[derive(Clone, Debug, Serialize)]
pub struct CreateConversationTagBody {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl CreateConversationTagBody {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateConversationTag {}

impl ElevenLabsEndpoint for CreateConversationTag {
    const PATH: &'static str = "/v1/convai/tags";

    const METHOD: Method = Method::POST;

    type ResponseBody = ConversationTag;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// GET /v1/convai/tags/{tag_id} — Get Conversation Tag
// =============================================================================

/// Retrieves a conversation tag by ID.
///
/// See [Get Conversation Tag API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tags/get)
#[derive(Clone, Debug)]
pub struct GetConversationTag {
    tag_id: String,
}

impl GetConversationTag {
    pub fn new(tag_id: impl Into<String>) -> Self {
        Self {
            tag_id: tag_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetConversationTag {}

impl ElevenLabsEndpoint for GetConversationTag {
    const PATH: &'static str = "/v1/convai/tags/:tag_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = ConversationTag;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tag_id.and_param(PathParam::TagID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// PATCH /v1/convai/tags/{tag_id} — Update Conversation Tag
// =============================================================================

/// Updates a conversation tag.
///
/// See [Update Conversation Tag API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tags/update)
#[derive(Clone, Debug)]
pub struct UpdateConversationTag {
    tag_id: String,
    body: UpdateConversationTagBody,
}

impl UpdateConversationTag {
    pub fn new(tag_id: impl Into<String>, body: UpdateConversationTagBody) -> Self {
        Self {
            tag_id: tag_id.into(),
            body,
        }
    }
}

/// Update-conversation-tag body. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateConversationTagBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl UpdateConversationTagBody {
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateConversationTag {}

impl ElevenLabsEndpoint for UpdateConversationTag {
    const PATH: &'static str = "/v1/convai/tags/:tag_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = ConversationTag;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tag_id.and_param(PathParam::TagID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// =============================================================================
// DELETE /v1/convai/tags/{tag_id} — Delete Conversation Tag
// =============================================================================

/// Deletes a conversation tag.
///
/// See [Delete Conversation Tag API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/tags/delete)
#[derive(Clone, Debug)]
pub struct DeleteConversationTag {
    tag_id: String,
}

impl DeleteConversationTag {
    pub fn new(tag_id: impl Into<String>) -> Self {
        Self {
            tag_id: tag_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteConversationTag {}

impl ElevenLabsEndpoint for DeleteConversationTag {
    const PATH: &'static str = "/v1/convai/tags/:tag_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.tag_id.and_param(PathParam::TagID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}
