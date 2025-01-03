use super::*;
use crate::endpoints::convai::agents::{KnowledgeBaseType, AGENTS_PATH};
use crate::error::Error;
use std::path::{Path, PathBuf};

const KNOWLEDGE_BASE_PATH: &str = "/knowledge-base";
const ADD_KNOWLEDGE_BASE_PATH: &str = "/add-to-knowledge-base";

/// See the [Create Knowledge Base Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/post-conversational-ai-knowledge-base-document).
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::knowledge_base::{CreateKnowledgeBase, KnowledgeBaseDoc};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   use elevenlabs_rs::endpoints::Url;
///   let client = ElevenLabsClient::from_env()?;
///   let kb = KnowledgeBaseDoc::url("https://elevenlabs.io/blog");
///   //let kb_file = KnowledgeBaseDoc::file("path/to/file");
///   let endpoint = CreateKnowledgeBase::new("some_agent_id", vec![kb]);
///   let resp = client.hit(endpoint).await?;
///   println!("{:#?}", resp);
/// Ok(())
/// }///
#[derive(Debug, Clone)]
pub struct CreateKnowledgeBase {
    agent_id: AgentID,
    // TODO: No longer a Vec!
    knowledge_base_doc: Vec<KnowledgeBaseDoc>,
}

impl CreateKnowledgeBase {
    pub fn new<T: Into<String>>(agent_id: T, knowledge_base_doc: Vec<KnowledgeBaseDoc>) -> Self {
        Self {
            agent_id: AgentID(agent_id.into()),
            knowledge_base_doc,
        }
    }

    async fn build_form(&self) -> Result<Form> {
        let mut form = Form::new();

        for doc in &self.knowledge_base_doc {
            match doc {
                KnowledgeBaseDoc::File(path) => {
                    let content = tokio::fs::read(path).await?;
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .ok_or(Error::PathNotValidUTF8)?;
                        //.ok_or(Error::InvalidPath)?;

                    let ext = path.extension()
                        .and_then(|e| e.to_str())
                        .ok_or(Error::FileExtensionNotFound)?;
                        //.ok_or(Error::MissingExtension)?;

                    let file_type = FileType::from_extension(ext)?;

                    let part = Part::bytes(content)
                        .file_name(filename.to_string())
                        .mime_str(file_type.mime_type())?;

                    form = form.part("file", part);
                }
                KnowledgeBaseDoc::Url(url) => {
                    form = form.text("url", url.clone());
                }
            }
        }
        Ok(form)
    }
}

/// File type definitions
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
            _=> Err(Error::FileExtensionNotSupported.into()),
            //_ => Err(Error::UnsupportedFileType(ext.to_string())),
        }
    }
}
impl Endpoint for CreateKnowledgeBase {
    type ResponseBody = CreateKnowledgeBaseResponse;

    const METHOD: Method = Method::POST;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(self.build_form().await?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Result<Url> {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            AGENTS_PATH, self.agent_id, ADD_KNOWLEDGE_BASE_PATH
        ));
        Ok(url)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeBaseResponse {
    id: String,
}

impl CreateKnowledgeBaseResponse {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub enum KnowledgeBaseDoc {
    //Docx(PathBuf),
    //Epub(PathBuf),
    //Html(PathBuf),
    //Pdf(PathBuf),
    //Txt(PathBuf),
    File(PathBuf),
    Url(String),
}

impl KnowledgeBaseDoc {
    pub fn file<T: Into<PathBuf>>(path: T) -> Self {
        Self::File(path.into())
    }
    pub fn url<T: Into<String>>(url: T) -> Self {
        Self::Url(url.into())
    }
}

// TODO: Finish example after implementing `AddKnowledgeBase` endpoint
/// See the [Get Knowledge Base Document API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/get-conversational-ai-knowledge-base-document).
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::knowledge_base::GetKnowledgeBase;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = GetKnowledgeBase::new("some_agent_id", "some_documentation_id");
///    let resp = client.hit(endpoint).await?;
///    println!("{:#?}", resp.extracted_inner_html());
///    Ok(())
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct GetKnowledgeBase {
    agent_id: AgentID,
    documentation_id: DocumentationID,
}

impl GetKnowledgeBase {
    pub fn new<T: Into<String>>(agent_id: T, documentation_id: T) -> Self {
        Self {
            agent_id: AgentID(agent_id.into()),
            documentation_id: DocumentationID(documentation_id.into()),
        }
    }
}

impl Endpoint for GetKnowledgeBase {
    type ResponseBody = GetKnowledgeBaseResponse;

    const METHOD: Method = Method::GET;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Result<Url> {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_path(&format!(
            "{}/{}{}/{}",
            AGENTS_PATH, self.agent_id, KNOWLEDGE_BASE_PATH, self.documentation_id
        ));
        Ok(url)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetKnowledgeBaseResponse {
    id: String,
    r#type: KnowledgeBaseType,
    extracted_inner_html: String,
}

impl GetKnowledgeBaseResponse {
    pub fn extracted_inner_html(&self) -> &str {
        &self.extracted_inner_html
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn file_type(&self) -> &KnowledgeBaseType {
        &self.r#type
    }
}
