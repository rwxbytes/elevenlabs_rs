use super::*;
use crate::endpoints::convai::agents::{AccessInfo, AccessLevel};
use crate::error::Error;
use std::path::Path;
use std::string::ToString;
use strum::Display;

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
    pub r#type: KnowledgeBaseType,
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
pub enum KnowledgeBaseType {
    File,
    Url,
}

///
/// # Example
///
/// ```no_run
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
#[derive(Debug, Clone)]
pub struct CreateKnowledgeBaseDoc {
    body: CreateKnowledgeBaseDocBody,
}

impl CreateKnowledgeBaseDoc {
    pub fn new(body: impl Into<CreateKnowledgeBaseDocBody>) -> Self {
        Self { body: body.into() }
    }
}
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
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &CreateKnowledgeBaseDocBody) -> Result<Self> {
        let mut form = Form::new();

        if let Some(name) = &body.name {
            form = form.text("name", name.clone());
        }

        match body.knowledge_base_doc.clone() {
            KnowledgeBaseDoc::File(path) => {
                let path = Path::new(&path);

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or(Error::PathNotValidUTF8)?;

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .ok_or(Error::FileExtensionNotFound)?;

                let file_type = FileType::from_extension(ext)?;

                let content = std::fs::read(path)?;

                let part = Part::bytes(content)
                    .file_name(filename.to_string())
                    .mime_str(file_type.mime_type())?;

                form = form.part("file", part);
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
            _ => Err(Error::FileExtensionNotSupported.into()),
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
    File(String),
    Url(String),
}

impl KnowledgeBaseDoc {
    pub fn file(path: impl Into<String>) -> Self {
        Self::File(path.into())
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
    pub r#type: KnowledgeBaseType,
    pub name: String,
    pub access_info: AccessInfo,
    pub dependent_agents: Vec<DependentAgent>,
    pub prompt_injectable: bool,
    pub metadata: DocMetadata,
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
    type Error = Box<dyn std::error::Error + Send + Sync>;

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
#[derive(Clone,Debug, Deserialize,Display, Serialize, )]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingModel {
    #[strum(to_string = "e5_mistral_7b_instruct")]
    #[serde(rename = "e5_mistral_7b_instruct")]
    E5Mistral7BInstruct,
    #[strum(to_string = "multilingual_e5_large_instruct")]
    #[serde(rename = "multilingual_e5_large_instruct")]
    MultilingualE5LargeInstruct,

}

impl From<EmbeddingModel> for ComputeRAGIndexBody {
    fn from(model: EmbeddingModel) -> Self {
        Self {
            model: model.to_string(),
        }
    }
}

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

