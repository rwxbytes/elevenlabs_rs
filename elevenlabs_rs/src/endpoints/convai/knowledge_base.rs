use super::*;
use crate::endpoints::convai::agents::AccessInfo;
use crate::error::Error;
use crate::shared::{AccessLevel, FilePart};
use std::string::ToString;

/// Get details about a specific documentation making up the agent’s knowledge base.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::knowledge_base::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///
///    let endpoint = GetKnowledgeBaseDoc::new("documentation_id");
///
///    let resp = client.hit(endpoint).await?;
///
///    println!("{:#?}", resp);
///
///    Ok(())
/// }
/// ```
/// See [Get Knowledge Base Document API reference](https://elevenlabs.io/docs/api-reference/knowledge-base/get-knowledge-base-document-by-id).
#[derive(Debug, Clone)]
pub struct GetKnowledgeBaseDoc {
    documentation_id: String,
}

impl GetKnowledgeBaseDoc {
    pub fn new(documentation_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetKnowledgeBaseDoc {}

impl ElevenLabsEndpoint for GetKnowledgeBaseDoc {
    const PATH: &'static str = "v1/convai/knowledge-base/:documentation_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetKnowledgeBaseDocResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetKnowledgeBaseDocResponse {
    pub id: String,
    pub r#type: KnowledgeBaseDocType,
    pub extracted_inner_html: String,
    pub name: String,
    pub access_info: AccessInfo,
    pub prompt_injectable: bool,
    pub metadata: DocMetadata,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocMetadata {
    pub created_at_unix_secs: u64,
    pub last_updated_at_unix_secs: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DependentAgent {
    pub id: String,
    pub name: String,
    pub r#type: DependentAgentType,
    pub created_at_unix_secs: u64,
    pub access_level: AccessLevel,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependentAgentType {
    Available,
    /// A model that represents an agent dependent on a knowledge base/tools to which the user has no direct access.
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeBaseDocType {
    File,
    Url,
    Text,
}

///
/// # Example
///
/// ```no_run
/// # #![allow(deprecated)]
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::agents::*;
/// use elevenlabs_rs::endpoints::convai::knowledge_base::{CreateKnowledgeBaseDoc, KnowledgeBaseDoc};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let client = ElevenLabsClient::from_env()?;
///   let kb = KnowledgeBaseDoc::url("https://elevenlabs.io/blog");
///   // Or KnowledgeBaseDoc::file("some_file.pdf");
///   let endpoint = CreateKnowledgeBaseDoc::new(kb);
///   let resp = client.hit(endpoint).await?;
///
///   // You must now patch the agent to include the knowledge base
///   let kb = KnowledgeBase::new_url(resp.id, "ElevenLabs' Blog");
///
///   let prompt_config = PromptConfig::default().with_knowledge_base(vec![kb]);
///
///   let agent_config = AgentConfig::default().with_prompt(prompt_config);
///
///   let config = ConversationConfig::default().with_agent_config(agent_config);
///
///   let body = UpdateAgentBody::default().with_conversation_config(config);
///
///   let endpoint = UpdateAgent::new("agent_id", body);
///
///   let resp = client.hit(endpoint).await?;
///
///   println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See [Create Knowledge Base Document API reference](https://elevenlabs.io/docs/api-reference/knowledge-base/add-to-knowledge-base).
#[deprecated(
    since = "0.7.0",
    note = "POST /v1/convai/knowledge-base is deprecated; use CreateFileDocument, CreateUrlDocument, or CreateTextDocument"
)]
#[derive(Debug, Clone)]
pub struct CreateKnowledgeBaseDoc {
    body: CreateKnowledgeBaseDocBody,
}

#[allow(deprecated)]
impl CreateKnowledgeBaseDoc {
    pub fn new(body: impl Into<CreateKnowledgeBaseDocBody>) -> Self {
        Self { body: body.into() }
    }
}
#[allow(deprecated)]
impl crate::endpoints::sealed::Sealed for CreateKnowledgeBaseDoc {}

#[allow(deprecated)]
impl ElevenLabsEndpoint for CreateKnowledgeBaseDoc {
    const PATH: &'static str = "v1/convai/knowledge-base";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateKnowledgeBaseDocResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone)]
pub struct CreateKnowledgeBaseDocBody {
    knowledge_base_doc: KnowledgeBaseDoc,
    name: Option<String>,
}

impl CreateKnowledgeBaseDocBody {
    pub fn new(knowledge_base_doc: KnowledgeBaseDoc) -> Self {
        Self {
            knowledge_base_doc,
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl TryFrom<&CreateKnowledgeBaseDocBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &CreateKnowledgeBaseDocBody) -> Result<Self> {
        let mut form = Form::new();

        if let Some(name) = &body.name {
            form = form.text("name", name.clone());
        }

        match body.knowledge_base_doc.clone() {
            KnowledgeBaseDoc::File(file) => {
                let inferred_mime = if file.mime().is_some() {
                    None
                } else {
                    let ext = file.extension()?;
                    Some(FileType::from_extension(&ext)?.mime_type().to_owned())
                };

                form = form.part("file", file.into_part(inferred_mime)?);
                Ok(RequestBody::Multipart(form))
            }

            KnowledgeBaseDoc::Url(url) => {
                form = form.text("url", url);
                Ok(RequestBody::Multipart(form))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Docx,
    Epub,
    Html,
    Pdf,
    Txt,
}

impl FileType {
    fn mime_type(&self) -> &'static str {
        match self {
            FileType::Docx => "application/docx",
            FileType::Epub => "application/epub",
            FileType::Html => "text/html",
            FileType::Pdf => "application/pdf",
            FileType::Txt => "text/plain",
        }
    }

    fn from_extension(ext: &str) -> Result<Self> {
        match ext.to_lowercase().as_str() {
            "docx" => Ok(FileType::Docx),
            "epub" => Ok(FileType::Epub),
            "html" => Ok(FileType::Html),
            "pdf" => Ok(FileType::Pdf),
            "txt" => Ok(FileType::Txt),
            _ => Err(Error::FileExtensionNotSupported),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateKnowledgeBaseDocResponse {
    pub id: String,
    pub prompt_injectable: bool,
}

#[derive(Debug, Clone)]
pub enum KnowledgeBaseDoc {
    File(FilePart),
    Url(String),
}

impl KnowledgeBaseDoc {
    pub fn file(path: impl Into<FilePart>) -> Self {
        Self::File(path.into())
    }
    pub fn file_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::File(FilePart::bytes(file_name, mime, bytes))
    }
    pub fn url(url: impl Into<String>) -> Self {
        Self::Url(url.into())
    }
}

impl From<KnowledgeBaseDoc> for CreateKnowledgeBaseDocBody {
    fn from(knowledge_base_doc: KnowledgeBaseDoc) -> Self {
        Self {
            knowledge_base_doc,
            name: None,
        }
    }
}

/// Get a list of available knowledge base documents.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::knowledge_base::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = ListKnowledgeBaseDocs::new();
///    let resp = client.hit(endpoint).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See [List Knowledge Base Documents API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/get-knowledge-base-list).
#[derive(Clone, Debug, Default)]
pub struct ListKnowledgeBaseDocs {
    query: Option<KnowledgeBaseQuery>,
}

impl ListKnowledgeBaseDocs {
    pub fn new() -> Self {
        Self { query: None }
    }

    pub fn with_query(mut self, query: KnowledgeBaseQuery) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct KnowledgeBaseQuery {
    pub params: QueryValues,
}

impl KnowledgeBaseQuery {
    /// Used for fetching next page. Cursor is returned in the response.
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    /// How many documents to return at maximum. Can not exceed 100, defaults to 30.
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    /// If specified, the endpoint returns only such knowledge base documents whose names start with this string.
    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.params.push(("search", search.into()));
        self
    }

    /// If set to true, the endpoint will return only documents owned by you (and not shared from somebody else).
    /// Defaults to false.
    pub fn show_only_owned_documents(mut self) -> Self {
        self.params
            .push(("show_only_owned_documents", true.to_string()));
        self
    }
    /// If set to true, the endpoint will use typesense DB to search for the documents.
    /// Defaults to false.
    pub fn use_typesense(mut self) -> Self {
        self.params.push(("use_typesense", true.to_string()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListKnowledgeBaseDocs {}

impl ElevenLabsEndpoint for ListKnowledgeBaseDocs {
    const PATH: &'static str = "v1/convai/knowledge-base";

    const METHOD: Method = Method::GET;

    type ResponseBody = ListKnowledgeBaseDocsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListKnowledgeBaseDocsResponse {
    pub documents: Vec<Document>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    pub id: String,
    pub r#type: KnowledgeBaseDocType,
    pub name: String,
    pub access_info: AccessInfo,
    pub dependent_agents: Vec<DependentAgent>,
    pub prompt_injectable: bool,
    pub metadata: DocMetadata,
    pub url: Option<String>,
}

/// Get a list of agents depending on this knowledge base document.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::knowledge_base::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = ListDependentAgents::new("documentation_id");
///    let resp = client.hit(endpoint).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// See [Get Dependent Agents API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/get-dependent-agents)
#[derive(Clone, Debug)]
pub struct ListDependentAgents {
    documentation_id: String,
    query: Option<KnowledgeBaseQuery>,
}

impl ListDependentAgents {
    pub fn new(documentation_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
            query: None,
        }
    }

    pub fn with_query(mut self, query: KnowledgeBaseQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl crate::endpoints::sealed::Sealed for ListDependentAgents {}

impl ElevenLabsEndpoint for ListDependentAgents {
    const PATH: &'static str = "v1/convai/knowledge-base/:documentation_id/dependent-agents";

    const METHOD: Method = Method::GET;

    type ResponseBody = ListDependentAgentsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListDependentAgentsResponse {
    pub agents: Vec<DependentAgent>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Delete a document from the knowledge base.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::knowledge_base::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = DeleteKnowledgeBaseDoc::new("documentation_id");
///    let resp = client.hit(endpoint).await?;
///    println!("{:#?}", resp);
///    Ok(())
/// }
/// ```
/// # Note
/// A 422 error will be returned if the document is still being used by an agent.
///
/// See [Delete Knowledge Base Document API reference](https://elevenlabs.io/docs/api-reference/knowledge-base/delete-knowledge-base-document).
#[derive(Clone, Debug)]
pub struct DeleteKnowledgeBaseDoc {
    documentation_id: String,
}

impl DeleteKnowledgeBaseDoc {
    pub fn new(documentation_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteKnowledgeBaseDoc {}

impl ElevenLabsEndpoint for DeleteKnowledgeBaseDoc {
    const PATH: &'static str = "v1/convai/knowledge-base/:documentation_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

/// Compute a RAG index for a knowledge base document.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::knowledge_base::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let client = ElevenLabsClient::from_env()?;
///   let endpoint = ComputeRAGIndex::new("documentation_id", EmbeddingModel::E5Mistral7BInstruct);
///   let resp = client.hit(endpoint).await?;
///   println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See [Compute RAG Index API reference](https://elevenlabs.io/docs/api-reference/knowledge-base/rag-index-status).
/// # Note
/// In case the document is not RAG indexed, it triggers rag indexing task,
/// otherwise it just returns the current status.
#[derive(Debug, Clone)]
pub struct ComputeRAGIndex {
    documentation_id: String,
    body: ComputeRAGIndexBody,
}

impl ComputeRAGIndex {
    pub fn new(documentation_id: impl Into<String>, body: impl Into<ComputeRAGIndexBody>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
            body: body.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ComputeRAGIndexBody {
    pub model: String,
}

impl ComputeRAGIndexBody {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

impl TryFrom<&ComputeRAGIndexBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &ComputeRAGIndexBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

impl From<&str> for ComputeRAGIndexBody {
    fn from(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }
}

impl From<String> for ComputeRAGIndexBody {
    fn from(model: String) -> Self {
        Self { model }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum EmbeddingModel {
    E5Mistral7BInstruct,
    MultilingualE5LargeInstruct,
    Custom(String),
}

impl EmbeddingModel {
    pub fn custom(model: impl Into<String>) -> Self {
        Self::Custom(model.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::E5Mistral7BInstruct => "e5_mistral_7b_instruct",
            Self::MultilingualE5LargeInstruct => "multilingual_e5_large_instruct",
            Self::Custom(model) => model,
        }
    }
}

impl From<String> for EmbeddingModel {
    fn from(model: String) -> Self {
        match model.as_str() {
            "e5_mistral_7b_instruct" => Self::E5Mistral7BInstruct,
            "multilingual_e5_large_instruct" => Self::MultilingualE5LargeInstruct,
            _ => Self::Custom(model),
        }
    }
}

impl From<&str> for EmbeddingModel {
    fn from(model: &str) -> Self {
        Self::from(model.to_owned())
    }
}

impl std::fmt::Display for EmbeddingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<EmbeddingModel> for String {
    fn from(model: EmbeddingModel) -> Self {
        model.to_string()
    }
}

impl From<EmbeddingModel> for ComputeRAGIndexBody {
    fn from(model: EmbeddingModel) -> Self {
        Self {
            model: model.to_string(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for ComputeRAGIndex {}

impl ElevenLabsEndpoint for ComputeRAGIndex {
    const PATH: &'static str = "v1/convai/knowledge-base/:documentation_id/rag-index";

    const METHOD: Method = Method::POST;

    type ResponseBody = ComputeRAGIndexResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeRAGIndexResponse {
    pub status: RAGIndexStatus,
    pub progress_percentage: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RAGIndexStatus {
    Created,
    Processing,
    Failed,
    Succeeded,
}

/// See [Get Document Content API reference](https://elevenlabs.io/docs/api-reference/knowledge-base/get-knowledge-base-document-content).
#[derive(Debug, Clone)]
pub struct GetDocumentContent {
    pub documentation_id: String,
}

impl GetDocumentContent {
    pub fn new(documentation_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetDocumentContent {}

impl ElevenLabsEndpoint for GetDocumentContent {
    const PATH: &'static str = "v1/convai/knowledge-base/:documentation_id/content";

    const METHOD: Method = Method::GET;

    type ResponseBody = String;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.text().await?)
    }
}

/// See [Get Document Chunk API reference](https://elevenlabs.io/docs/api-reference/knowledge-base/get-knowledge-base-document-part-by-id).
#[derive(Debug, Clone)]
pub struct GetDocumentChunk {
    pub documentation_id: String,
    pub chunk_id: String,
}

impl GetDocumentChunk {
    pub fn new(documentation_id: impl Into<String>, chunk_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
            chunk_id: chunk_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetDocumentChunk {}

impl ElevenLabsEndpoint for GetDocumentChunk {
    const PATH: &'static str = "v1/convai/knowledge-base/:documentation_id/chunk/:chunk_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetDocumentChunkResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.documentation_id.and_param(PathParam::DocumentationID),
            self.chunk_id.and_param(PathParam::ChunkID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDocumentChunkResponse {
    pub id: String,
    pub name: String,
    pub content: String,
}

/// The response of the typed document-creation endpoints
/// ([`CreateTextDocument`], [`CreateUrlDocument`], [`CreateFileDocument`]) and
/// [`CreateKnowledgeBaseFolder`].
#[derive(Clone, Debug, Deserialize)]
pub struct AddKnowledgeBaseResponse {
    pub id: String,
    pub name: String,
    /// The folder breadcrumb of the created entity, preserved as raw JSON.
    #[serde(default)]
    pub folder_path: Vec<Value>,
}

/// Create a text document in the knowledge base.
///
/// See [Create Text Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/create-from-text)
#[derive(Clone, Debug)]
pub struct CreateTextDocument {
    body: CreateTextDocumentBody,
}

impl CreateTextDocument {
    pub fn new(body: CreateTextDocumentBody) -> Self {
        Self { body }
    }
}

/// Body for [`CreateTextDocument`].
#[derive(Clone, Debug, Serialize)]
pub struct CreateTextDocumentBody {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_folder_id: Option<String>,
}

impl CreateTextDocumentBody {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            name: None,
            parent_folder_id: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_parent_folder_id(mut self, parent_folder_id: impl Into<String>) -> Self {
        self.parent_folder_id = Some(parent_folder_id.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateTextDocument {}

impl ElevenLabsEndpoint for CreateTextDocument {
    const PATH: &'static str = "/v1/convai/knowledge-base/text";

    const METHOD: Method = Method::POST;

    type ResponseBody = AddKnowledgeBaseResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Create a document in the knowledge base from a URL.
///
/// See [Create URL Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/create-from-url)
#[derive(Clone, Debug)]
pub struct CreateUrlDocument {
    body: CreateUrlDocumentBody,
}

impl CreateUrlDocument {
    pub fn new(body: CreateUrlDocumentBody) -> Self {
        Self { body }
    }
}

/// Body for [`CreateUrlDocument`].
#[derive(Clone, Debug, Serialize)]
pub struct CreateUrlDocumentBody {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_auto_sync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_remove: Option<bool>,
}

impl CreateUrlDocumentBody {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            name: None,
            parent_folder_id: None,
            enable_auto_sync: None,
            auto_remove: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_parent_folder_id(mut self, parent_folder_id: impl Into<String>) -> Self {
        self.parent_folder_id = Some(parent_folder_id.into());
        self
    }

    pub fn with_enable_auto_sync(mut self, enable_auto_sync: bool) -> Self {
        self.enable_auto_sync = Some(enable_auto_sync);
        self
    }

    pub fn with_auto_remove(mut self, auto_remove: bool) -> Self {
        self.auto_remove = Some(auto_remove);
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateUrlDocument {}

impl ElevenLabsEndpoint for CreateUrlDocument {
    const PATH: &'static str = "/v1/convai/knowledge-base/url";

    const METHOD: Method = Method::POST;

    type ResponseBody = AddKnowledgeBaseResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Create a document in the knowledge base from an uploaded file.
///
/// See [Create File Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/create-from-file)
#[derive(Clone, Debug)]
pub struct CreateFileDocument {
    file: FilePart,
    name: Option<String>,
    parent_folder_id: Option<String>,
}

impl CreateFileDocument {
    pub fn new(file: impl Into<FilePart>) -> Self {
        Self {
            file: file.into(),
            name: None,
            parent_folder_id: None,
        }
    }

    pub fn from_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(FilePart::bytes(file_name, mime, bytes))
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_parent_folder_id(mut self, parent_folder_id: impl Into<String>) -> Self {
        self.parent_folder_id = Some(parent_folder_id.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for CreateFileDocument {}

impl ElevenLabsEndpoint for CreateFileDocument {
    const PATH: &'static str = "/v1/convai/knowledge-base/file";

    const METHOD: Method = Method::POST;

    type ResponseBody = AddKnowledgeBaseResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        let mut form = Form::new().part("file", kb_file_part(&self.file)?);
        if let Some(name) = &self.name {
            form = form.text("name", name.clone());
        }
        if let Some(parent_folder_id) = &self.parent_folder_id {
            form = form.text("parent_folder_id", parent_folder_id.clone());
        }
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Create a folder in the knowledge base.
///
/// See [Create Folder API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/create-folder)
#[derive(Clone, Debug)]
pub struct CreateKnowledgeBaseFolder {
    body: CreateKnowledgeBaseFolderBody,
}

impl CreateKnowledgeBaseFolder {
    pub fn new(body: CreateKnowledgeBaseFolderBody) -> Self {
        Self { body }
    }
}

/// Body for [`CreateKnowledgeBaseFolder`].
#[derive(Clone, Debug, Serialize)]
pub struct CreateKnowledgeBaseFolderBody {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_folder_id: Option<String>,
}

impl CreateKnowledgeBaseFolderBody {
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

impl crate::endpoints::sealed::Sealed for CreateKnowledgeBaseFolder {}

impl ElevenLabsEndpoint for CreateKnowledgeBaseFolder {
    const PATH: &'static str = "/v1/convai/knowledge-base/folder";

    const METHOD: Method = Method::POST;

    type ResponseBody = AddKnowledgeBaseResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Move several knowledge-base entities into a folder.
///
/// See [Bulk Move API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/bulk-move)
#[derive(Clone, Debug)]
pub struct BulkMoveKnowledgeBase {
    body: BulkMoveKnowledgeBaseBody,
}

impl BulkMoveKnowledgeBase {
    pub fn new(body: BulkMoveKnowledgeBaseBody) -> Self {
        Self { body }
    }
}

/// Body for [`BulkMoveKnowledgeBase`].
#[derive(Clone, Debug, Serialize)]
pub struct BulkMoveKnowledgeBaseBody {
    document_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    move_to: Option<String>,
}

impl BulkMoveKnowledgeBaseBody {
    pub fn new<I, S>(document_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            document_ids: document_ids.into_iter().map(Into::into).collect(),
            move_to: None,
        }
    }

    /// The destination folder ID. Omit to move to the root.
    pub fn with_move_to(mut self, move_to: impl Into<String>) -> Self {
        self.move_to = Some(move_to.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for BulkMoveKnowledgeBase {}

impl ElevenLabsEndpoint for BulkMoveKnowledgeBase {
    const PATH: &'static str = "/v1/convai/knowledge-base/bulk-move";

    const METHOD: Method = Method::POST;

    type ResponseBody = ();

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

/// Move a single knowledge-base entity into a folder.
///
/// See [Move Entity API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/move)
#[derive(Clone, Debug)]
pub struct MoveKnowledgeBaseEntity {
    document_id: String,
    move_to: Option<String>,
}

impl MoveKnowledgeBaseEntity {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            move_to: None,
        }
    }

    /// The destination folder ID. Omit to move to the root.
    pub fn with_move_to(mut self, move_to: impl Into<String>) -> Self {
        self.move_to = Some(move_to.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for MoveKnowledgeBaseEntity {}

impl ElevenLabsEndpoint for MoveKnowledgeBaseEntity {
    const PATH: &'static str = "/v1/convai/knowledge-base/:document_id/move";

    const METHOD: Method = Method::POST;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.document_id.and_param(PathParam::DocumentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "move_to": self.move_to }),
        ))
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

/// Get an overview of the workspace's RAG indexes (usage and per-model totals).
///
/// See [Get RAG Index Overview API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/rag-index-overview)
#[derive(Clone, Debug, Default)]
pub struct GetRagIndexOverview;

impl crate::endpoints::sealed::Sealed for GetRagIndexOverview {}

impl ElevenLabsEndpoint for GetRagIndexOverview {
    const PATH: &'static str = "/v1/convai/knowledge-base/rag-index";

    const METHOD: Method = Method::GET;

    type ResponseBody = RagIndexOverview;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`GetRagIndexOverview`]. Per-model entries are preserved as
/// raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct RagIndexOverview {
    pub total_used_bytes: i64,
    pub total_max_bytes: i64,
    pub models: Vec<Value>,
}

/// Compute (or fetch) RAG indexes for several documents in a batch.
///
/// See [Compute RAG Indexes API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/rag-index-batch)
#[derive(Clone, Debug)]
pub struct ComputeRagIndexesBatch {
    items: Vec<RagIndexItem>,
}

impl ComputeRagIndexesBatch {
    pub fn new(items: impl IntoIterator<Item = RagIndexItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }
}

/// A single document's RAG-index request within [`ComputeRagIndexesBatch`].
#[derive(Clone, Debug, Serialize)]
pub struct RagIndexItem {
    document_id: String,
    model: String,
    create_if_missing: bool,
}

impl RagIndexItem {
    pub fn new(document_id: impl Into<String>, model: EmbeddingModel) -> Self {
        Self {
            document_id: document_id.into(),
            model: model.to_string(),
            create_if_missing: true,
        }
    }

    pub fn with_create_if_missing(mut self, create_if_missing: bool) -> Self {
        self.create_if_missing = create_if_missing;
        self
    }
}

impl crate::endpoints::sealed::Sealed for ComputeRagIndexesBatch {}

impl ElevenLabsEndpoint for ComputeRagIndexesBatch {
    const PATH: &'static str = "/v1/convai/knowledge-base/rag-index";

    const METHOD: Method = Method::POST;

    type ResponseBody = Value;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(
            serde_json::json!({ "items": self.items }),
        ))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Delete a single RAG index of a document.
///
/// See [Delete RAG Index API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/delete-rag-index)
#[derive(Clone, Debug)]
pub struct DeleteRagIndex {
    documentation_id: String,
    rag_index_id: String,
}

impl DeleteRagIndex {
    pub fn new(documentation_id: impl Into<String>, rag_index_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
            rag_index_id: rag_index_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteRagIndex {}

impl ElevenLabsEndpoint for DeleteRagIndex {
    const PATH: &'static str =
        "/v1/convai/knowledge-base/:documentation_id/rag-index/:rag_index_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.documentation_id.and_param(PathParam::DocumentationID),
            self.rag_index_id.and_param(PathParam::RagIndexID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Search the knowledge base by content.
///
/// See [Search Knowledge Base API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/search)
#[derive(Clone, Debug)]
pub struct SearchKnowledgeBase {
    query: SearchKnowledgeBaseQuery,
}

impl SearchKnowledgeBase {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: SearchKnowledgeBaseQuery::new(query),
        }
    }

    pub fn with_query(mut self, query: SearchKnowledgeBaseQuery) -> Self {
        self.query = query;
        self
    }
}

/// Query parameters for [`SearchKnowledgeBase`].
#[derive(Clone, Debug)]
pub struct SearchKnowledgeBaseQuery {
    params: QueryValues,
}

impl SearchKnowledgeBaseQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            params: vec![("query", query.into())],
        }
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.params.push(("cursor", cursor.into()));
        self
    }

    /// Filter by document type. May be called multiple times.
    pub fn with_type(mut self, doc_type: impl Into<String>) -> Self {
        self.params.push(("types", doc_type.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for SearchKnowledgeBase {}

impl ElevenLabsEndpoint for SearchKnowledgeBase {
    const PATH: &'static str = "/v1/convai/knowledge-base/search";

    const METHOD: Method = Method::GET;

    type ResponseBody = KnowledgeBaseSearchResponse;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`SearchKnowledgeBase`]. Each result is preserved as raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct KnowledgeBaseSearchResponse {
    pub results: Vec<Value>,
    pub next_cursor: Option<String>,
}

/// Get summaries for the given knowledge-base document IDs.
///
/// See [Get Summaries API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/summaries)
#[derive(Clone, Debug)]
pub struct GetKnowledgeBaseSummaries {
    document_ids: Vec<String>,
}

impl GetKnowledgeBaseSummaries {
    pub fn new<I, S>(document_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            document_ids: document_ids.into_iter().map(Into::into).collect(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetKnowledgeBaseSummaries {}

impl ElevenLabsEndpoint for GetKnowledgeBaseSummaries {
    const PATH: &'static str = "/v1/convai/knowledge-base/summaries";

    const METHOD: Method = Method::GET;

    type ResponseBody = Value;

    fn query_params(&self) -> Option<QueryValues> {
        Some(
            self.document_ids
                .iter()
                .map(|id| ("document_ids", id.clone()))
                .collect(),
        )
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// List the chunks of a knowledge-base document.
///
/// See [Get Document Chunks API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/get-chunks)
#[derive(Clone, Debug)]
pub struct GetDocumentChunks {
    documentation_id: String,
    query: DocumentChunksQuery,
}

impl GetDocumentChunks {
    pub fn new(documentation_id: impl Into<String>, embedding_model: EmbeddingModel) -> Self {
        Self {
            documentation_id: documentation_id.into(),
            query: DocumentChunksQuery::new(embedding_model),
        }
    }

    pub fn with_query(mut self, query: DocumentChunksQuery) -> Self {
        self.query = query;
        self
    }
}

/// Query parameters for [`GetDocumentChunks`].
#[derive(Clone, Debug)]
pub struct DocumentChunksQuery {
    params: QueryValues,
}

impl DocumentChunksQuery {
    pub fn new(embedding_model: EmbeddingModel) -> Self {
        Self {
            params: vec![("embedding_model", embedding_model.to_string())],
        }
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

impl crate::endpoints::sealed::Sealed for GetDocumentChunks {}

impl ElevenLabsEndpoint for GetDocumentChunks {
    const PATH: &'static str = "/v1/convai/knowledge-base/:documentation_id/chunks";

    const METHOD: Method = Method::GET;

    type ResponseBody = DocumentChunksResponse;

    fn query_params(&self) -> Option<QueryValues> {
        Some(self.query.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A page of a document's chunks.
#[derive(Clone, Debug, Deserialize)]
pub struct DocumentChunksResponse {
    pub chunks: Vec<GetDocumentChunkResponse>,
    pub next_cursor: Option<String>,
}

/// Refresh a knowledge-base document (re-fetch its source).
///
/// See [Refresh Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/refresh)
#[derive(Clone, Debug)]
pub struct RefreshDocument {
    documentation_id: String,
}

impl RefreshDocument {
    pub fn new(documentation_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for RefreshDocument {}

impl ElevenLabsEndpoint for RefreshDocument {
    const PATH: &'static str = "/v1/convai/knowledge-base/:documentation_id/refresh";

    const METHOD: Method = Method::POST;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Get a signed URL to download a document's source file.
///
/// See [Get Source File URL API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/source-file-url)
#[derive(Clone, Debug)]
pub struct GetSourceFileUrl {
    documentation_id: String,
}

impl GetSourceFileUrl {
    pub fn new(documentation_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetSourceFileUrl {}

impl ElevenLabsEndpoint for GetSourceFileUrl {
    const PATH: &'static str = "/v1/convai/knowledge-base/:documentation_id/source-file-url";

    const METHOD: Method = Method::GET;

    type ResponseBody = SourceFileUrlResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`GetSourceFileUrl`].
#[derive(Clone, Debug, Deserialize)]
pub struct SourceFileUrlResponse {
    pub signed_url: String,
}

/// Replace the file of a file-backed knowledge-base document.
///
/// See [Update File Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/update-file)
#[derive(Clone, Debug)]
pub struct UpdateFileDocument {
    documentation_id: String,
    file: FilePart,
}

impl UpdateFileDocument {
    pub fn new(documentation_id: impl Into<String>, file: impl Into<FilePart>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
            file: file.into(),
        }
    }

    pub fn from_bytes(
        documentation_id: impl Into<String>,
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(documentation_id, FilePart::bytes(file_name, mime, bytes))
    }
}

impl crate::endpoints::sealed::Sealed for UpdateFileDocument {}

impl ElevenLabsEndpoint for UpdateFileDocument {
    const PATH: &'static str = "/v1/convai/knowledge-base/:documentation_id/update-file";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        let form = Form::new().part("file", kb_file_part(&self.file)?);
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Build a multipart part for a knowledge-base document file. The [`FilePart`]'s
/// own MIME type is used when present, otherwise `application/octet-stream`.
fn kb_file_part(file: &FilePart) -> Result<reqwest::multipart::Part> {
    file.clone()
        .into_part(Some("application/octet-stream".to_owned()))
}

/// Update a knowledge-base document's name and/or text content. The updated
/// document is returned as raw JSON.
///
/// See [Update Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/update)
#[derive(Clone, Debug)]
pub struct UpdateKnowledgeBaseDocument {
    documentation_id: String,
    body: UpdateKnowledgeBaseDocumentBody,
}

impl UpdateKnowledgeBaseDocument {
    pub fn new(documentation_id: impl Into<String>, body: UpdateKnowledgeBaseDocumentBody) -> Self {
        Self {
            documentation_id: documentation_id.into(),
            body,
        }
    }
}

/// Body for [`UpdateKnowledgeBaseDocument`]. All fields are optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateKnowledgeBaseDocumentBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

impl UpdateKnowledgeBaseDocumentBody {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateKnowledgeBaseDocument {}

impl ElevenLabsEndpoint for UpdateKnowledgeBaseDocument {
    const PATH: &'static str = "/v1/convai/knowledge-base/:documentation_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = Value;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// List the RAG indexes of a knowledge-base document.
///
/// See [Get Document RAG Indexes API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/knowledge-base/get-document-rag-indexes)
#[derive(Clone, Debug)]
pub struct GetDocumentRagIndexes {
    documentation_id: String,
}

impl GetDocumentRagIndexes {
    pub fn new(documentation_id: impl Into<String>) -> Self {
        Self {
            documentation_id: documentation_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetDocumentRagIndexes {}

impl ElevenLabsEndpoint for GetDocumentRagIndexes {
    const PATH: &'static str = "/v1/convai/knowledge-base/:documentation_id/rag-index";

    const METHOD: Method = Method::GET;

    type ResponseBody = DocumentRagIndexes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.documentation_id.and_param(PathParam::DocumentationID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`GetDocumentRagIndexes`]. Each index entry is preserved as
/// raw JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct DocumentRagIndexes {
    pub indexes: Vec<Value>,
}
