//! The pronunciation dictionaries endpoints.
//!
//! See examples [here](https://www.github.com/rwxbytes/elevenlabs_rs/tree/main/examples/pronunciation_dictionaries).
#[allow(dead_code)]
use super::*;

const PRONUNCIATION_PATH: &str = "/v1/pronunciation-dictionaries";
const ADD_FROM_FILE_PATH: &str = "/add-from-file";
const ADD_RULES_PATH: &str = "/add-rules";
const REMOVE_RULES_PATH: &str = "/remove-rules";

const CURSOR_QUERY: &str = "cursor";
const PAGE_SIZE_QUERY: &str = "page_size";

/// Creates a new pronunciation dictionary from a lexicon .PLS file
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::pronunciation::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let body = AddFromFileBody::new("acronyms.pls", "acronyms");
///    let resp = c.hit(AddFromFile::new(body)).await?;
///    println!("{:?}", resp);
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct AddFromFile(AddFromFileBody);

impl AddFromFile {
    pub fn new(body: AddFromFileBody) -> Self {
        Self(body)
    }
}

/// Add from file body
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::pronunciation::*;
///
/// let mut body = AddFromFileBody::new("acronyms.pls", "acronyms")
///     .with_description("A list of acronyms")
///     .with_workspace_access("editor");
/// ```
#[derive(Clone, Debug)]
pub struct AddFromFileBody {
    file: String,
    name: String,
    description: Option<String>,
    workspace_access: Option<String>,
}

impl AddFromFileBody {
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

    fn to_form(&self) -> Result<Form> {
        let mut form = Form::new();
        let file = std::fs::read(self.file.clone())?;
        form = form.part("file", Part::bytes(file).file_name("file"));
        form = form.text("name", self.name.clone());
        if let Some(description) = &self.description {
            form = form.text("description", description.clone());
        }
        if let Some(workspace_access) = &self.workspace_access {
            form = form.text("workspace_access", workspace_access.clone());
        }
        Ok(form)
    }
}

impl Endpoint for AddFromFile {
    type ResponseBody = AddFromFileResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Multipart(self.0.to_form()?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}{}", PRONUNCIATION_PATH, ADD_FROM_FILE_PATH));
        url
    }
}

/// Add from file response
#[derive(Clone, Debug, Deserialize)]
pub struct AddFromFileResponse {
    id: String,
    name: String,
    created_by: String,
    creation_time_unix: i64,
    version_id: String,
    description: Option<String>,
}

impl AddFromFileResponse {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    pub fn creation_time_unix(&self) -> i64 {
        self.creation_time_unix
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Add pronunciation rules to a pronunciation dictionary
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::pronunciation::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::default()?;
///   let rules = vec![
///      Rule::new_alias("TTS", "text to speech"),
///      Rule::new_alias("API", "application programming interface"),
///
///   ];
///   let resp = c.hit(AddRules::new("dictionary_id", rules)).await?;
///   println!("{:?}", resp);
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct AddRules {
    param: DictionaryID,
    body: AddRulesBody,
}

impl AddRules {
    pub fn new(dictionary_id: &str, rules: Vec<Rule>) -> Self {
        let body = AddRulesBody::new(rules);
        Self {
            param: DictionaryID(dictionary_id.to_string()),
            body,
        }
    }
}

/// This struct is used to build the request body for the AddRules endpoint.
#[derive(Clone, Debug, Serialize)]
struct AddRulesBody {
    rules: Vec<Rule>,
}

impl AddRulesBody {
    fn new(rules: Vec<Rule>) -> Self {
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

impl Endpoint for AddRules {
    type ResponseBody = RulesResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            PRONUNCIATION_PATH, self.param.0, ADD_RULES_PATH
        ));
        url
    }
}

/// Add rules response
#[derive(Clone, Debug, Deserialize)]
pub struct RulesResponse {
    id: String,
    version_id: String,
}

impl RulesResponse {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

/// Get the list of pronunciation dictionaries
///
/// This endpoint returns the list of pronunciation dictionaries.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::pronunciation::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let resp = c.hit(GetDictionaries::new()).await?;
///    println!("{:?}", resp);
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetDictionaries(pub GetDictionariesQuery);

impl GetDictionaries {
    pub fn new() -> Self {
        Self(GetDictionariesQuery::default())
    }

    pub fn with_query(query: GetDictionariesQuery) -> Self {
        Self(query)
    }
}

/// Get dictionaries query
///
/// This struct is used to build the query parameters for the GetDictionaries endpoint.
#[derive(Clone, Debug, Default)]
pub struct GetDictionariesQuery {
    pub page_size: Option<String>,
    pub cursor: Option<String>,
}

impl GetDictionariesQuery {
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(format!("{}={}", PAGE_SIZE_QUERY, page_size));
        self
    }

    pub fn with_cursor(mut self, cursor: &str) -> Self {
        self.cursor = Some(format!("{}={}", CURSOR_QUERY, cursor));
        self
    }

    fn to_string(&self) -> String {
        let mut query = String::new();

        if let Some(page_size) = &self.page_size {
            query.push_str(page_size);
        }
        if let Some(cursor) = &self.cursor {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(cursor);
        }
        query
    }
}

impl Endpoint for GetDictionaries {
    type ResponseBody = GetDictionariesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(PRONUNCIATION_PATH);
        if !self.0.to_string().is_empty() {
            url.set_query(Some(&self.0.to_string()));
        }
        url
    }
}

/// Get dictionaries response
#[derive(Clone, Debug, Deserialize)]
pub struct GetDictionariesResponse {
    pronunciation_dictionaries: Vec<PronunciationDictionary>,
    next_cursor: Option<String>,
    has_more: bool,
}

impl GetDictionariesResponse {
    pub fn pronunciation_dictionaries(&self) -> &Vec<PronunciationDictionary> {
        &self.pronunciation_dictionaries
    }

    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    pub fn has_more(&self) -> bool {
        self.has_more
    }
}

/// Pronunciation dictionary
#[derive(Clone, Debug, Deserialize)]
pub struct PronunciationDictionary {
    id: String,
    latest_version_id: String,
    name: String,
    created_by: String,
    creation_time_unix: i64,
    description: Option<String>,
}

impl PronunciationDictionary {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn latest_version_id(&self) -> &str {
        &self.latest_version_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    pub fn creation_time_unix(&self) -> i64 {
        self.creation_time_unix
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// This endpoint returns the pronunciation dictionary with the specified ID.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::pronunciation::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::default()?;
///   let resp = c.hit(GetDictionary::new("dictionary_id")).await?;
///   println!("{:?}", resp);
///  Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetDictionary(DictionaryID);

impl GetDictionary {
    pub fn new(id: &str) -> Self {
        Self(DictionaryID::from(id.to_string()))
    }
}

#[derive(Clone, Debug)]
struct DictionaryID(String);

impl From<String> for DictionaryID {
    fn from(id: String) -> Self {
        Self(id)
    }

}

impl Endpoint for GetDictionary {
    type ResponseBody = PronunciationDictionary;

    fn method(&self) -> Method {
        Method::GET
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", PRONUNCIATION_PATH, self.0 .0));
        url
    }
}

/// This endpoint returns the PLS file with the pronunciation dictionary version rules.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::pronunciation::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let c = ElevenLabsClient::default()?;
///   let resp_bytes = c.hit(DownloadVersionByID::new("dictionary_id", "version_id")).await?;
///   save("dictionary_rules.pls", resp_bytes)?;
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct DownloadVersionByID {
    dictionary_id: DictionaryID,
    version_id: VersionID,
}

impl DownloadVersionByID {
    pub fn new(dictionary_id: &str, version_id: &str) -> Self {
        Self {
            dictionary_id: DictionaryID::from(dictionary_id.to_string()),
            version_id: VersionID::from(version_id.to_string()),
        }
    }
}

/// This struct represents the version ID of a pronunciation dictionary.
#[derive(Clone, Debug)]
struct VersionID(String);

impl From<String> for VersionID {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl Endpoint for DownloadVersionByID {
    type ResponseBody = Bytes;

    fn method(&self) -> Method {
        Method::GET
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}/{}{}",
            PRONUNCIATION_PATH, self.dictionary_id.0, self.version_id.0, DOWNLOAD_PATH
        ));
        url
    }
}

/// This endpoint removes the specified rules from the pronunciation dictionary.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::pronunciation::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let remove_rules = vec!["rule_string_1", "rule_string_2"];
///    let resp = c.hit(RemoveRules::new("dictionary_id", remove_rules)).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct RemoveRules {
    param: DictionaryID,
    body: RemoveRulesBody,
}

impl RemoveRules {
    pub fn new(dictionary_id: &str, rules: Vec<&str>) -> Self {
        let body = RemoveRulesBody::new(rules);
        Self {
            param: DictionaryID::from(dictionary_id.to_string()),
            body,
        }
    }
}

/// This struct is used to build the request body for the RemoveRules endpoint.
/// The rules are removed from the latest version of the pronunciation dictionary.
#[derive(Clone, Debug, Serialize)]
struct RemoveRulesBody {
    rule_strings: Vec<String>,
}

impl RemoveRulesBody {
    fn new(rules: Vec<&str>) -> Self {
        Self {
            rule_strings: rules.iter().map(|r| r.to_string()).collect(),
        }
    }
}

impl Endpoint for RemoveRules {
    type ResponseBody = RulesResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            PRONUNCIATION_PATH, self.param.0, REMOVE_RULES_PATH
        ));
        url
    }
}
