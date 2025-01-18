use super::*;
use crate::error::Error;
use std::path::Path;

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
///    let endpoint = GetKnowledgeBase::new("agent_id", "documentation_id");
///
///    let resp = client.hit(endpoint).await?;
///
///    println!("{:#?}", resp);
///
///    Ok(())
/// }
/// ```
/// See [Get Knowledge Base Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/get-conversational-ai-knowledge-base-document).
#[derive(Debug, Clone)]
pub struct GetKnowledgeBase {
    agent_id: String,
    documentation_id: String,
}

impl GetKnowledgeBase {
    pub fn new<T: Into<String>>(agent_id: T, documentation_id: T) -> Self {
        Self {
            agent_id: agent_id.into(),
            documentation_id: documentation_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetKnowledgeBase {
    const PATH: &'static str = "v1/convai/agents/:agent_id/knowledge-base/:documentation_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetKnowledgeBaseResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.agent_id.and_param(PathParam::AgentID),
            self.documentation_id.and_param(PathParam::DocumentationID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetKnowledgeBaseResponse {
    pub id: String,
    pub r#type: KnowledgeBaseType,
    pub extracted_inner_html: String,
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
/// use elevenlabs_rs::endpoints::convai::knowledge_base::{CreateKnowledgeBase, KnowledgeBaseDoc};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let client = ElevenLabsClient::from_env()?;
///   let kb = KnowledgeBaseDoc::url("https://elevenlabs.io/blog");
///   // Or KnowledgeBaseDoc::file("some_file.pdf");
///   let endpoint = CreateKnowledgeBase::new("agent_id", kb);
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
/// See [Create Knowledge Base Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/post-conversational-ai-knowledge-base-document).
#[derive(Debug, Clone)]
pub struct CreateKnowledgeBase {
    agent_id: String,
    body: CreateKnowledgeBaseBody,
}

impl CreateKnowledgeBase {
    pub fn new(agent_id: impl Into<String>, body: impl Into<CreateKnowledgeBaseBody>) -> Self {
        Self {
            agent_id: agent_id.into(),
            body: body.into(),
        }
    }
}
impl ElevenLabsEndpoint for CreateKnowledgeBase {
    const PATH: &'static str = "v1/convai/agents/:agent_id/add-to-knowledge-base";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateKnowledgeBaseResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone)]
pub struct CreateKnowledgeBaseBody {
    knowledge_base_doc: KnowledgeBaseDoc,
}

impl CreateKnowledgeBaseBody {
    pub fn new(knowledge_base_doc: KnowledgeBaseDoc) -> Self {
        Self { knowledge_base_doc }
    }
}

impl TryFrom<&CreateKnowledgeBaseBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &CreateKnowledgeBaseBody) -> Result<Self> {
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

                Ok(RequestBody::Multipart(Form::new().part("file", part)))
            }

            KnowledgeBaseDoc::Url(url) => Ok(RequestBody::Multipart(Form::new().text("url", url))),
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
pub struct CreateKnowledgeBaseResponse {
    pub id: String,
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

impl From<KnowledgeBaseDoc> for CreateKnowledgeBaseBody {
    fn from(knowledge_base_doc: KnowledgeBaseDoc) -> Self {
        Self { knowledge_base_doc }
    }
}
