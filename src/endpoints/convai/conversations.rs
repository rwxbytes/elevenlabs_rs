#![allow(dead_code)]
//! Conversations endpoints

use super::*;

const GET_SIGNED_PATH: &str = "/v1/convai/conversation/get_signed_url";
const CONVERSATION_PATH: &str = "/v1/conversational_ai/conversations";
const CONVERSATION_AUDIO_PATH: &str = "/audio";

#[derive(Clone, Debug, Serialize)]
struct ConversationID(String);

/// See the [Get Conversations API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/get-conversational-ai-conversation)
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::{
///     CallSuccessful, GetConversations, GetConversationsQuery,
/// };
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let query = GetConversationsQuery {
///         page_size: 10,
///         call_successful: CallSuccessful::Success,
///         ..Default::default()
///     };
///
///     let endpoint = GetConversations::new(query);
///
///     let resp = client.hit(endpoint).await?;
///
///     let avg_convo_duration = (&resp)
///         .into_iter()
///         .map(|convo| convo.call_duration_secs())
///         .sum::<u32>()
///         / resp.conversations().len() as u32;
///
///     println!(
///         "Average duration of the last {} successful convos : {}s",
///         resp.conversations().len(),
///         avg_convo_duration
///     );
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct GetConversations {
    query: GetConversationsQuery,
}

#[derive(Clone, Debug, Serialize, Validate)]
pub struct GetConversationsQuery {
    /// The id of the agent you're taking the action on.
    pub agent_id: String,
    /// The result of the success evaluation. Can be `failure`, `success`, or `unknown`.
    pub call_successful: CallSuccessful,
    /// Used for fetching next page. Cursor is returned in the response.
    pub cursor: String,
    /// How many conversations to return at maximum. Can not exceed 100, defaults to 30.
    #[validate(range(min = 1, max = 100))]
    pub page_size: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallSuccessful {
    Failure,
    Success,
    Unknown,
    // Not an actual value, used for convenience
    // to avoid Option<CallSuccessful> in `GetConversationsQuery`
    All,
}

impl CallSuccessful {
    pub fn is_failure(&self) -> bool {
        matches!(*self, CallSuccessful::Failure)
    }
    pub fn is_success(&self) -> bool {
        matches!(*self, CallSuccessful::Success)
    }
    pub fn is_unknown(&self) -> bool {
        matches!(*self, CallSuccessful::Unknown)
    }
}

/// See the [Get Conversation Audio API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/get-conversational-ai-conversation-audio)
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::{
///     GetConversationAudio, GetConversations, GetConversationsQuery};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::utils::play;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = GetConversations::new(GetConversationsQuery::default());
///    let resp = client.hit(endpoint).await?;
///    let last_convo = resp.into_iter().next().unwrap();
///    let last_convo_id = last_convo.conversation_id();
///    let endpoint = GetConversationAudio::new(last_convo_id);
///    let bytes = client.hit(endpoint).await?;
///    play(bytes)?;
///    Ok(())
/// }
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct GetConversationAudio {
    conversation_id: ConversationID,
}

/// See the [Get Conversation Details API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/get-conversational-ai-conversations)
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::conversations::{
///     GetConversationDetails, GetConversations, GetConversationsQuery,
/// };
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use futures_util::TryFutureExt;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///     let endpoint = GetConversations::new(GetConversationsQuery::default());
///
///     let convo_details = client
///         .clone()
///         .hit(endpoint)
///         .and_then(|convos| {
///             let latest_convo = convos
///                 .conversations()
///                 .first()
///                 .ok_or("No conversations found".into())?;
///             let endpoint = GetConversationDetails::new(latest_convo.conversation_id());
///             client.hit(endpoint)
///         })
///         .await?;
///
///     let potential_keywords = convo_details
///         .into_iter()
///         .flat_map(|transcript| {
///             transcript
///                 .message()
///                 .split_whitespace()
///                 .filter(|word| word.len() > 3)
///                 .map(|word| word.to_lowercase())
///                 .collect::<Vec<String>>()
///         })
///         .fold(HashMap::new(), |mut acc, word| {
///             *acc.entry(word).or_insert(0) += 1;
///             acc
///         })
///         .into_iter()
///         .filter(|&(_, count)| count > 1)
///         .collect::<HashMap<String, i32>>();
///
///     for (word, count) in potential_keywords {
///         println!("{}: {}", word, count);
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct GetConversationDetails {
    conversation_id: ConversationID,
}

impl GetConversationDetails {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: ConversationID(conversation_id.into()),
        }
    }
}

impl Endpoint for GetConversationDetails {
    type ResponseBody = GetConversationDetailsResponse;

    const METHOD: Method = Method::GET;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Result<Url> {
        let mut url = Url::parse(BASE_URL).unwrap();
        url.set_path(&format!("{}/{}", CONVERSATION_PATH, self.conversation_id));
        Ok(url)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetConversationDetailsResponse {
    agent_id: String,
    conversation_id: String,
    status: ConvoStatus,
    transcript: Vec<Transcript>,
    metadata: Metadata,
    analysis: Option<Analysis>,
}

impl GetConversationDetailsResponse {
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }
    pub fn status(&self) -> &ConvoStatus {
        &self.status
    }
    pub fn transcript(&self) -> &[Transcript] {
        self.transcript.as_slice()
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn analysis(&self) -> Option<&Analysis> {
        self.analysis.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Analysis {
    call_successful: CallSuccessful,
    data_collection_results: Option<DataCollectionResult>,
    evaluation_criteria_results: Option<EvaluationResult>,
    transcript_summary: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataCollectionResult {
    #[serde(flatten)]
    key: std::collections::HashMap<String, DataCollectionData>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataCollectionData {
    data_collection_id: String,
    json_schema: JsonSchema,
    value: Value,
    rationale: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsonSchema {
    description: String,
    r#type: JsonSchemaType,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonSchemaType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EvaluationResult {
    #[serde(flatten)]
    key: std::collections::HashMap<String, EvaluationData>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EvaluationData {
    criteria_id: String,
    result: CallSuccessful,
    rationale: String,
}

impl Analysis {
    pub fn evaluation_criteria_results(&self) -> Option<&EvaluationResult> {
        self.evaluation_criteria_results.as_ref()
    }

    pub fn data_collection_results(&self) -> Option<&DataCollectionResult> {
        self.data_collection_results.as_ref()
    }
    pub fn call_successful(&self) -> &CallSuccessful {
        &self.call_successful
    }
    pub fn transcript_summary(&self) -> &str {
        &self.transcript_summary
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    start_time_unix_secs: u64,
    call_duration_secs: u32,
    cost: Option<u32>,
}

impl Metadata {
    pub fn start_time_unix_secs(&self) -> u64 {
        self.start_time_unix_secs
    }
    pub fn call_duration_secs(&self) -> u32 {
        self.call_duration_secs
    }
    pub fn cost(&self) -> Option<u32> {
        self.cost
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transcript {
    role: Role,
    message: String,
    time_in_call_secs: u32,
}

impl Transcript {
    pub fn role(&self) -> &Role {
        &self.role
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn time_in_call_secs(&self) -> u32 {
        self.time_in_call_secs
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Agent,
    User,
}

#[derive(Clone, Debug, Serialize)]
pub struct GetSignedUrl {
    agent_id: AgentID,
}

impl GetSignedUrl {
    pub fn new(agent_id: impl Into<String>) -> Self {
        GetSignedUrl {
            agent_id: AgentID(agent_id.into()),
        }
    }
}

impl Endpoint for GetSignedUrl {
    type ResponseBody = SignedUrlResponse;

    const METHOD: Method = Method::GET;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Result<Url> {
        let mut url = Url::parse(BASE_URL).unwrap();
        url.set_path(GET_SIGNED_PATH);
        url.set_query(Some(&format!("agent_id={}", self.agent_id)));
        Ok(url)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SignedUrlResponse {
    signed_url: String,
}

impl SignedUrlResponse {
    pub fn as_str(&self) -> &str {
        self.signed_url.as_str()
    }
}

impl std::fmt::Display for GetConversationsQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut params = vec![];
        if !self.agent_id.is_empty() {
            params.push(format!("agent_id={}", self.agent_id));
        }
        if let CallSuccessful::All = self.call_successful {
            // Do nothing
        } else {
            params.push(format!("call_successful={}", self.call_successful));
        }
        if !self.cursor.is_empty() {
            params.push(format!("cursor={}", self.cursor));
        }
        if self.page_size > 0 {
            params.push(format!("page_size={}", self.page_size));
        }
        write!(f, "{}", params.join("&"))
    }
}

impl GetConversations {
    pub fn new(query: GetConversationsQuery) -> Self {
        Self { query }
    }
}

impl GetConversationAudio {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: ConversationID(conversation_id.into()),
        }
    }
}



#[derive(Clone, Debug, Deserialize)]
pub struct GetConversationsResponse {
    conversations: Vec<Conversation>,
    next_cursor: Option<String>,
    has_more: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Conversation {
    agent_id: String,
    agent_name: Option<String>,
    conversation_id: String,
    start_time_unix_secs: u64,
    call_duration_secs: u32,
    message_count: u32,
    status: ConvoStatus,
    call_successful: CallSuccessful,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvoStatus {
    Done,
    Processing,
}

impl ConvoStatus {
    pub fn is_done(&self) -> bool {
        matches!(*self, ConvoStatus::Done)
    }
    pub fn is_processing(&self) -> bool {
        matches!(*self, ConvoStatus::Processing)
    }
}

impl Conversation {
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
    pub fn agent_name(&self) -> Option<&str> {
        self.agent_name.as_deref()
    }
    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }
    pub fn start_time_unix_secs(&self) -> u64 {
        self.start_time_unix_secs
    }
    pub fn call_duration_secs(&self) -> u32 {
        self.call_duration_secs
    }
    pub fn message_count(&self) -> u32 {
        self.message_count
    }
    pub fn status(&self) -> &ConvoStatus {
        &self.status
    }
    pub fn call_successful(&self) -> &CallSuccessful {
        &self.call_successful
    }
}

impl GetConversationsResponse {
    pub fn conversations(&self) -> &[Conversation] {
        self.conversations.as_slice()
    }
    pub fn has_more(&self) -> bool {
        self.has_more
    }
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }
}

impl Default for GetConversationsQuery {
    /// Default values for the query parameters
    ///
    /// - `agent_id`: `String::new()`
    /// - `call_successful`: `None`
    /// - `cursor`: `String::new()`
    /// - `page_size`: `30`
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            call_successful: CallSuccessful::All,
            cursor: String::new(),
            page_size: 30,
        }
    }
}

impl std::fmt::Display for CallSuccessful {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallSuccessful::Failure => write!(f, "failure"),
            CallSuccessful::Success => write!(f, "success"),
            CallSuccessful::Unknown => write!(f, "unknown"),
            CallSuccessful::All => write!(f, ""),
        }
    }
}

impl std::fmt::Display for ConversationID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Endpoint for GetConversations {
    type ResponseBody = GetConversationsResponse;

    const METHOD: Method = Method::GET;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Result<Url> {
        self.query.validate()?;
        let mut url = Url::parse(BASE_URL)?;
        url.set_path(CONVERSATION_PATH);
        url.set_query(Some(&self.query.to_string()));
        Ok(url)
    }
}

impl Endpoint for GetConversationAudio {
    type ResponseBody = Bytes;

    const METHOD: Method = Method::GET;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }

    fn url(&self) -> Result<Url> {
        let mut url = Url::parse(BASE_URL).unwrap();
        url.set_path(&format!(
            "{}/{}{}",
            CONVERSATION_PATH, self.conversation_id, CONVERSATION_AUDIO_PATH
        ));
        Ok(url)
    }
}

impl IntoIterator for GetConversationsResponse {
    type Item = Conversation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.conversations.into_iter()
    }
}

impl<'a> IntoIterator for &'a GetConversationsResponse {
    type Item = &'a Conversation;
    type IntoIter = std::slice::Iter<'a, Conversation>;

    fn into_iter(self) -> Self::IntoIter {
        self.conversations.iter()
    }
}

impl IntoIterator for GetConversationDetailsResponse {
    type Item = Transcript;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.transcript.into_iter()
    }
}

impl<'a> IntoIterator for &'a GetConversationDetailsResponse {
    type Item = &'a Transcript;
    type IntoIter = std::slice::Iter<'a, Transcript>;

    fn into_iter(self) -> Self::IntoIter {
        self.transcript.iter()
    }
}
