//! The pronunciation dictionaries endpoints.
//!
//! See an extensive example [here](https://www.github.com/rwxbytes/elevenlabs_rs/blob/master/examples/pronunciation_dictionaries.rs).
use super::*;
pub use crate::shared::DictionaryLocator;

/// The level of workspace access granted on a pronunciation dictionary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceAccess {
    Admin,
    Editor,
    Commenter,
    Viewer,
}

impl WorkspaceAccess {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Commenter => "commenter",
            Self::Viewer => "viewer",
        }
    }
}

impl std::fmt::Display for WorkspaceAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Creates a new pronunciation dictionary from a lexicon .PLS file
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{CreateDictionary, CreateDictionaryBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let body = CreateDictionaryBody::new("acronyms.pls", "acronyms");
///    let resp = c.hit(CreateDictionary::new(body)).await?;
///    println!("{:?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Create A Pronunciation Dictionary API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/add-from-file)
#[derive(Clone, Debug)]
pub struct CreateDictionary {
    body: CreateDictionaryBody,
}

impl CreateDictionary {
    pub fn new(body: CreateDictionaryBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for CreateDictionary {}

impl ElevenLabsEndpoint for CreateDictionary {
    const PATH: &'static str = "/v1/pronunciation-dictionaries/add-from-file";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateDictionaryResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        let form = Form::try_from(self.body.clone())?;
        Ok(RequestBody::Multipart(form))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Create dictionary body
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::admin::pronunciation::CreateDictionaryBody;
///
/// let mut body = CreateDictionaryBody::new("acronyms.pls", "acronyms")
///     .with_description("A list of acronyms")
///     .with_workspace_access("editor");
/// ```
#[derive(Clone, Debug)]
pub struct CreateDictionaryBody {
    file: String,
    name: String,
    description: Option<String>,
    workspace_access: Option<String>,
}

impl CreateDictionaryBody {
    pub fn new(file: &str, name: &str) -> Self {
        Self {
            file: file.to_string(),
            name: name.to_string(),
            description: None,
            workspace_access: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_workspace_access(mut self, workspace_access: &str) -> Self {
        self.workspace_access = Some(workspace_access.to_string());
        self
    }
}

impl TryFrom<CreateDictionaryBody> for Form {
    type Error = crate::error::Error;

    fn try_from(body: CreateDictionaryBody) -> Result<Self> {
        let mut form = Form::new();
        let file = std::fs::read(body.file.clone())?;
        form = form.part("file", Part::bytes(file).file_name("file"));
        form = form.text("name", body.name.clone());
        if let Some(description) = body.description {
            form = form.text("description", description);
        }
        if let Some(workspace_access) = body.workspace_access {
            form = form.text("workspace_access", workspace_access);
        }
        Ok(form)
    }
}

/// Create dictionary response
#[derive(Clone, Debug, Deserialize)]
pub struct CreateDictionaryResponse {
    pub id: String,
    pub name: String,
    pub created_by: String,
    pub creation_time_unix: u64,
    pub version_id: String,
    pub version_rules_num: u32,
    pub description: Option<String>,
    pub permission_on_resource: Option<WorkspaceAccess>,
}

/// Creates a new pronunciation dictionary from a list of rules.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{
///     AddDictionaryFromRules, AddDictionaryFromRulesBody, Rule,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let rules = vec![
///         Rule::new_alias("TTS", "text to speech"),
///         Rule::new_phoneme("Apple", "ˈæpəl", "ipa"),
///     ];
///     let body = AddDictionaryFromRulesBody::new("acronyms", rules)
///         .with_description("A list of acronyms");
///     let resp = c.hit(AddDictionaryFromRules::new(body)).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See the [Add From Rules API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/add-from-rules)
#[derive(Clone, Debug)]
pub struct AddDictionaryFromRules {
    body: AddDictionaryFromRulesBody,
}

impl AddDictionaryFromRules {
    pub fn new(body: AddDictionaryFromRulesBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for AddDictionaryFromRules {}

impl ElevenLabsEndpoint for AddDictionaryFromRules {
    const PATH: &'static str = "/v1/pronunciation-dictionaries/add-from-rules";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateDictionaryResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Add-from-rules body
#[derive(Clone, Debug, Serialize)]
pub struct AddDictionaryFromRulesBody {
    /// The name of the pronunciation dictionary, used for identification only.
    name: String,
    /// The list of pronunciation rules.
    rules: Vec<Rule>,
    /// A description of the pronunciation dictionary, used for identification only.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// The workspace access level to grant. Defaults to no access.
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_access: Option<WorkspaceAccess>,
}

impl AddDictionaryFromRulesBody {
    pub fn new(name: impl Into<String>, rules: Vec<Rule>) -> Self {
        Self {
            name: name.into(),
            rules,
            description: None,
            workspace_access: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_workspace_access(mut self, workspace_access: WorkspaceAccess) -> Self {
        self.workspace_access = Some(workspace_access);
        self
    }
}

/// Add rules to the pronunciation dictionary
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{AddRules, AddRulesBody, Rule};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::from_env()?;
///
///   let rules = vec![
///      Rule::new_alias("TTS", "text to speech"),
///      Rule::new_alias("API", "application programming interface"),
///   ];
///
///   let body = AddRulesBody::new(rules);
///
///   let resp = c.hit(AddRules::new("dictionary_id", body)).await?;
///
///   println!("{:?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Add Rules API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/add-rules)
#[derive(Clone, Debug)]
pub struct AddRules {
    dictionary_id: String,
    body: AddRulesBody,
}

impl AddRules {
    pub fn new(dictionary_id: impl Into<String>, body: AddRulesBody) -> Self {
        Self {
            dictionary_id: dictionary_id.into(),
            body,
        }
    }
}
impl crate::endpoints::sealed::Sealed for AddRules {}

impl ElevenLabsEndpoint for AddRules {
    const PATH: &'static str =
        "/v1/pronunciation-dictionaries/:pronunciation_dictionary_id/add-rules";

    const METHOD: Method = Method::POST;

    type ResponseBody = RulesResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .dictionary_id
            .and_param(PathParam::PronunciationDictionaryID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Add rules body
#[derive(Clone, Debug, Serialize)]
pub struct AddRulesBody {
    rules: Vec<Rule>,
}

impl AddRulesBody {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }
}

/// This enum represents the rules that can be added to a pronunciation dictionary.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Rule {
    Alias {
        r#type: String,
        string_to_replace: String,
        alias: String,
    },
    Phoneme {
        r#type: String,
        string_to_replace: String,
        phoneme: String,
        alphabet: String,
    },
}

impl Rule {
    pub fn new_alias(string_to_replace: &str, alias: &str) -> Self {
        Self::Alias {
            r#type: "alias".to_string(),
            string_to_replace: string_to_replace.to_string(),
            alias: alias.to_string(),
        }
    }
    pub fn new_phoneme(string_to_replace: &str, phoneme: &str, alphabet: &str) -> Self {
        Self::Phoneme {
            r#type: "phoneme".to_string(),
            string_to_replace: string_to_replace.to_string(),
            phoneme: phoneme.to_string(),
            alphabet: alphabet.to_string(),
        }
    }
}

/// Add rules response
#[derive(Clone, Debug, Deserialize)]
pub struct RulesResponse {
    pub id: String,
    pub version_id: String,
    pub version_rules_num: u32,
}

///  Remove rules from the pronunciation dictionary
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{RemoveRules, RemoveRulesBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let rules = vec!["rule_string_1", "rule_string_2"];
///    let body = RemoveRulesBody::new(rules);
///    let resp = c.hit(RemoveRules::new("dictionary_id", body)).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See the [Remove Rules API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/remove-rules)
#[derive(Clone, Debug)]
pub struct RemoveRules {
    dictionary_id: String,
    body: RemoveRulesBody,
}

impl RemoveRules {
    pub fn new(dictionary_id: impl Into<String>, body: RemoveRulesBody) -> Self {
        Self {
            dictionary_id: dictionary_id.into(),
            body,
        }
    }
}

/// The rules are removed from the latest version of the pronunciation dictionary.
#[derive(Clone, Debug, Serialize)]
pub struct RemoveRulesBody {
    rule_strings: Vec<String>,
}

impl RemoveRulesBody {
    pub fn new<'a, I>(rules: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        Self {
            rule_strings: rules.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for RemoveRules {}

impl ElevenLabsEndpoint for RemoveRules {
    const PATH: &'static str =
        "/v1/pronunciation-dictionaries/:pronunciation_dictionary_id/remove-rules";

    const METHOD: Method = Method::POST;

    type ResponseBody = RulesResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .dictionary_id
            .and_param(PathParam::PronunciationDictionaryID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Replaces all rules on the latest version of the pronunciation dictionary
/// with the provided rules.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{SetRules, SetRulesBody, Rule};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let rules = vec![Rule::new_alias("TTS", "text to speech")];
///     let resp = c.hit(SetRules::new("dictionary_id", SetRulesBody::new(rules))).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See the [Set Rules API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/set-rules)
#[derive(Clone, Debug)]
pub struct SetRules {
    dictionary_id: String,
    body: SetRulesBody,
}

impl SetRules {
    pub fn new(dictionary_id: impl Into<String>, body: SetRulesBody) -> Self {
        Self {
            dictionary_id: dictionary_id.into(),
            body,
        }
    }
}

/// Set-rules body
#[derive(Clone, Debug, Serialize)]
pub struct SetRulesBody {
    rules: Vec<Rule>,
}

impl SetRulesBody {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }
}

impl crate::endpoints::sealed::Sealed for SetRules {}

impl ElevenLabsEndpoint for SetRules {
    const PATH: &'static str =
        "/v1/pronunciation-dictionaries/:pronunciation_dictionary_id/set-rules";

    const METHOD: Method = Method::POST;

    type ResponseBody = RulesResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .dictionary_id
            .and_param(PathParam::PronunciationDictionaryID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Get PLS file with a pronunciation dictionary version rules
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{GetPLSFile, GetDictionaries};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let dictionaries = c.hit(GetDictionaries::default()).await?;
///
///     for dict in dictionaries {
///         let id = dict.id.as_str();
///         let version_id = dict.latest_version_id.as_str();
///
///         let bytes = c.hit(GetPLSFile::new(id, version_id)).await?;
///         println!("{}", std::str::from_utf8(&bytes)?);
///         println!("\n---\n");
///     }
///   Ok(())
/// }
/// ```
/// See the [Get PLS File API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/download)
#[derive(Clone, Debug)]
pub struct GetPLSFile {
    dictionary_id: String,
    version_id: String,
}

impl GetPLSFile {
    pub fn new(dictionary_id: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            dictionary_id: dictionary_id.into(),
            version_id: version_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetPLSFile {}

impl ElevenLabsEndpoint for GetPLSFile {
    const PATH: &'static str = "/v1/pronunciation-dictionaries/:dictionary_id/:version_id/download";

    const METHOD: Method = Method::GET;

    type ResponseBody = Bytes;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.dictionary_id
                .and_param(PathParam::PronunciationDictionaryID),
            self.version_id.and_param(PathParam::VersionID),
        ]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// Get metadata for a pronunciation dictionary
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::GetDictionaryMetaData;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let resp = c.hit(GetDictionaryMetaData::new("dictionary_id")).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See the [Get Dictionary Metadata API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/get)
#[derive(Clone, Debug)]
pub struct GetDictionaryMetaData {
    dictionary_id: String,
}

impl GetDictionaryMetaData {
    pub fn new(dictionary_id: impl Into<String>) -> Self {
        Self {
            dictionary_id: dictionary_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for GetDictionaryMetaData {}

impl ElevenLabsEndpoint for GetDictionaryMetaData {
    const PATH: &'static str = "/v1/pronunciation-dictionaries/:pronunciation_dictionary_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = DictionaryMetadataResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .dictionary_id
            .and_param(PathParam::PronunciationDictionaryID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Update the metadata of a pronunciation dictionary, e.g. its name or whether
/// it is archived.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{UpdateDictionary, UpdateDictionaryBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let body = UpdateDictionaryBody::default().with_name("Renamed Dictionary");
///     let resp = c.hit(UpdateDictionary::new("dictionary_id", body)).await?;
///     println!("{:?}", resp);
///     Ok(())
/// }
/// ```
/// See the [Update Pronunciation Dictionary API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/update)
#[derive(Clone, Debug)]
pub struct UpdateDictionary {
    dictionary_id: String,
    body: UpdateDictionaryBody,
}

impl UpdateDictionary {
    pub fn new(dictionary_id: impl Into<String>, body: UpdateDictionaryBody) -> Self {
        Self {
            dictionary_id: dictionary_id.into(),
            body,
        }
    }
}

/// Update-dictionary body
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateDictionaryBody {
    /// Whether to archive the pronunciation dictionary.
    #[serde(skip_serializing_if = "Option::is_none")]
    archived: Option<bool>,
    /// The name of the pronunciation dictionary, used for identification only.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl UpdateDictionaryBody {
    pub fn with_archived(mut self, archived: bool) -> Self {
        self.archived = Some(archived);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl crate::endpoints::sealed::Sealed for UpdateDictionary {}

impl ElevenLabsEndpoint for UpdateDictionary {
    const PATH: &'static str = "/v1/pronunciation-dictionaries/:pronunciation_dictionary_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = DictionaryMetadataResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .dictionary_id
            .and_param(PathParam::PronunciationDictionaryID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Get a list of the pronunciation dictionaries you have access to and their metadata
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::pronunciation::{GetDictionaries, GetDictionariesQuery};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let resp = c.hit(GetDictionaries::default()).await?;
///    println!("{:?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Get Dictionaries API reference](https://elevenlabs.io/docs/api-reference/pronunciation-dictionary/get-all)
#[derive(Clone, Debug, Default)]
pub struct GetDictionaries {
    query: Option<GetDictionariesQuery>,
}

impl GetDictionaries {
    pub fn with_query(query: GetDictionariesQuery) -> Self {
        Self { query: Some(query) }
    }
}

/// # Query Parameters
///
/// - `cursor`: Used for fetching next page. Cursor is returned in the response.
///
///
/// - `page_size`: How many pronunciation dictionaries to return at maximum. Can not exceed 100, defaults to 30.
#[derive(Clone, Debug, Default)]
pub struct GetDictionariesQuery {
    params: QueryValues,
}

impl GetDictionariesQuery {
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }

    pub fn with_cursor(mut self, cursor: &str) -> Self {
        self.params.push(("cursor", cursor.to_string()));
        self
    }
    pub fn with_sort(mut self, sort: &str) -> Self {
        self.params.push(("sort", sort.to_string()));
        self
    }

    pub fn with_sort_direction(mut self, sort_direction: &str) -> Self {
        self.params
            .push(("sort_direction", sort_direction.to_string()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetDictionaries {}

impl ElevenLabsEndpoint for GetDictionaries {
    const PATH: &'static str = "/v1/pronunciation-dictionaries";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetDictionariesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Get dictionaries response
#[derive(Clone, Debug, Deserialize)]
pub struct GetDictionariesResponse {
    pub pronunciation_dictionaries: Vec<DictionaryMetadataResponse>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Pronunciation dictionary metadata
#[derive(Clone, Debug, Deserialize)]
pub struct DictionaryMetadataResponse {
    pub id: String,
    pub latest_version_id: String,
    pub latest_version_rules_num: u32,
    pub name: String,
    pub created_by: String,
    pub creation_time_unix: u64,
    pub archived_time_unix: Option<u64>,
    pub description: Option<String>,
    pub permission_on_resource: Option<WorkspaceAccess>,
}

impl IntoIterator for GetDictionariesResponse {
    type Item = DictionaryMetadataResponse;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pronunciation_dictionaries.into_iter()
    }
}

impl<'a> IntoIterator for &'a GetDictionariesResponse {
    type Item = &'a DictionaryMetadataResponse;
    type IntoIter = std::slice::Iter<'a, DictionaryMetadataResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.pronunciation_dictionaries.iter()
    }
}
