//! Agents endpoints

use super::*;
use crate::shared::DictionaryLocator;
use std::collections::HashMap;

/// Create an agent from a config object
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::convai::agents::{CreateAgent, CreateAgentBody};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let body = CreateAgentBody::new("some_system_prompt");
///
///     let endpoint = CreateAgent::new(body);
///
///     let resp = client.hit(endpoint).await?;
///
///     println!("{:?}", resp);
///
///     Ok(())
/// }
/// ```
/// See [Create Agent API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/create-agent)
#[derive(Clone, Debug)]
pub struct CreateAgent {
    body: CreateAgentBody,
}

impl CreateAgent {
    pub fn new(body: CreateAgentBody) -> Self {
        CreateAgent { body }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateAgentBody {
    pub conversation_config: ConversationConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_settings: Option<PlatformSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl CreateAgentBody {
    pub fn new(prompt: impl Into<String>) -> Self {
        CreateAgentBody {
            conversation_config: ConversationConfig::default().with_agent_config(
                AgentConfig::default()
                    .with_prompt(PromptConfig::default().with_prompt(prompt.into())),
            ),
            platform_settings: None,
            name: None,
        }
    }

    pub fn with_conversation_config(mut self, conversation_config: ConversationConfig) -> Self {
        self.conversation_config = conversation_config;
        self
    }

    pub fn with_platform_settings(mut self, platform_settings: PlatformSettings) -> Self {
        self.platform_settings = Some(platform_settings);
        self
    }
}

impl ElevenLabsEndpoint for CreateAgent {
    const PATH: &'static str = "/v1/convai/agents/create";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateAgentResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryInto::try_into(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateAgentResponse {
    pub agent_id: String,
}

impl TryFrom<&CreateAgentBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &CreateAgentBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(body)?))
    }
}

/// See the official [Delete Agent API reference](https://elevenlabs.io/docs/api-reference/delete-conversational-ai-agent)
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agents::DeleteAgent;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let endpoint = DeleteAgent::new("agent_id");
///    let resp = client.hit(endpoint).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See [Delete Agent API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/delete-agent)
#[derive(Clone, Debug)]
pub struct DeleteAgent {
    agent_id: String,
}

impl DeleteAgent {
    pub fn new(agent_id: impl Into<String>) -> Self {
        DeleteAgent {
            agent_id: agent_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for DeleteAgent {
    const PATH: &'static str = "/v1/convai/agents/:agent_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = StatusResponseBody;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConversationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asr: Option<ASR>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<Conversation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<TTSConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<Turn>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_presets: Option<HashMap<String, LanguagePreset>>,
}

impl ConversationConfig {
    pub fn with_agent_config(mut self, agent_config: AgentConfig) -> Self {
        self.agent = Some(agent_config);
        self
    }

    pub fn with_asr(mut self, asr: ASR) -> Self {
        self.asr = Some(asr);
        self
    }

    pub fn with_conversation(mut self, conversation: Conversation) -> Self {
        self.conversation = Some(conversation);
        self
    }

    pub fn with_tts_config(mut self, tts: TTSConfig) -> Self {
        self.tts = Some(tts);
        self
    }

    pub fn with_turn(mut self, turn: Turn) -> Self {
        self.turn = Some(turn);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AgentConfig {
    //pub server: Option<ServerConfig>,
    /// The system prompt is used to determine the persona of the agent and the context of the conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<PromptConfig>,
    /// The first message the agent will say.
    ///
    /// If empty the agent will wait for the user to start the conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
    /// The language of the agent.
    ///
    /// The agent will use English as the default language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl AgentConfig {
    pub fn new(
        prompt: PromptConfig,
        first_message: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        AgentConfig {
            prompt: Some(prompt),
            first_message: Some(first_message.into()),
            language: Some(language.into()),
        }
    }

    //pub fn with_server(mut self, server: ServerConfig) -> Self {
    //    self.server = Some(server);
    //    self
    //}

    pub fn with_prompt(mut self, prompt: PromptConfig) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn with_first_message(mut self, first_message: impl Into<String>) -> Self {
        self.first_message = Some(first_message.into());
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
}

//#[derive(Clone, Debug, Default, Deserialize, Serialize)]
//pub struct ServerConfig {
//    pub server_events: Vec<ServerEvent>,
//    pub url: String,
//    pub secret: String,
//    pub timeout: u32,
//    pub num_retries: u32,
//    pub error_message: String,
//}
//
//#[derive(Clone, Debug, Deserialize, Serialize)]
//#[serde(rename_all = "snake_case")]
//pub enum ServerEvent {
//    Interruption,
//    Turn,
//    TurnAbandoned,
//}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PromptConfig {
    /// Provide the LLM with domain-specific information to help it answer questions more accurately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge_base: Option<Vec<KnowledgeBase>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<LLM>,
    /// Configure the maximum number of tokens that the LLM can predict.
    /// A limit will be applied if the value is greater than 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    /// System prompt is used to determine the persona of the agent and the context of the conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Temperature is a parameter that controls the creativity
    /// or randomness of the responses generated by the LLM.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_llm: Option<CustomLLM>,
}

impl PromptConfig {
    pub fn with_knowledge_base(mut self, knowledge_base: Vec<KnowledgeBase>) -> Self {
        self.knowledge_base = Some(knowledge_base);
        self
    }

    pub fn with_llm(mut self, llm: LLM) -> Self {
        self.llm = Some(llm);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_custom_llm(mut self, custom_llm: CustomLLM) -> Self {
        self.custom_llm = Some(custom_llm);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnowledgeBase {
    pub id: String,
    pub name: String,
    pub r#type: KnowledgeBaseType,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeBaseType {
    File,
    Url,
}

impl KnowledgeBase {
    pub fn new_file(id: impl Into<String>, name: impl Into<String>) -> Self {
        KnowledgeBase {
            id: id.into(),
            name: name.into(),
            r#type: KnowledgeBaseType::File,
        }
    }

    pub fn new_url(id: impl Into<String>, name: impl Into<String>) -> Self {
        KnowledgeBase {
            id: id.into(),
            name: name.into(),
            r#type: KnowledgeBaseType::Url,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LLM {
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    #[serde(rename = "gpt-4")]
    Gpt4,
    #[serde(rename = "gpt-4-turbo")]
    Gpt4Turbo,
    #[serde(rename = "gpt-3.5-turbo")]
    Gpt3_5Turbo,
    #[serde(rename = "gemini-1.5-pro")]
    #[default]
    Gemini1_5Pro,
    #[serde(rename = "gemini-1.5-flash")]
    Gemini1_5Flash,
    #[serde(rename = "gemini-1.0-pro")]
    Gemini1_0Pro,
    #[serde(rename = "claude-3-5-sonnet")]
    Claude3_5Sonnet,
    #[serde(rename = "claude-3-haiku")]
    Claude3Haiku,
    #[serde(rename = "grok-beta")]
    GrokBeta,
    #[serde(rename = "custom-llm")]
    CustomLLM,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum Tool {
    WebHook(WebHook),
    Client(ClientTool),
}

/// A webhook tool is a tool that calls an external webhook from ElevenLabs' server
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebHook {
    api_schema: ApiSchema,
    description: String,
    name: String,
    r#type: ToolType,
}

impl WebHook {
    pub fn new<T: Into<String>>(name: T, description: T, api_schema: ApiSchema) -> Self {
        WebHook {
            api_schema,
            description: description.into(),
            name: name.into(),
            r#type: ToolType::Webhook,
        }
    }
}

impl From<WebHook> for Tool {
    fn from(webhook: WebHook) -> Self {
        Tool::WebHook(webhook)
    }
}

impl From<ClientTool> for Tool {
    fn from(client_tool: ClientTool) -> Self {
        Tool::Client(client_tool)
    }
}

/// A client tool is one that sends an event to the user’s client to trigger something client side
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientTool {
    pub description: String,
    pub name: String,
    pub expects_response: Option<bool>,
    pub parameters: Option<ClientToolParams>,
    pub response_timeout_secs: Option<u32>,
    r#type: ToolType,
}

impl ClientTool {
    pub fn new<T: Into<String>>(name: T, description: T) -> Self {
        ClientTool {
            description: description.into(),
            name: name.into(),
            expects_response: None,
            parameters: None,
            response_timeout_secs: None,
            r#type: ToolType::Client,
        }
    }

    pub fn with_expects_response(mut self, expects_response: bool) -> Self {
        self.expects_response = Some(expects_response);
        self
    }

    pub fn with_parameters(mut self, parameters: ClientToolParams) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn with_response_timeout_secs(mut self, response_timeout_secs: u32) -> Self {
        self.response_timeout_secs = Some(response_timeout_secs);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToolParams {
    r#type: DataType,
    pub properties: Option<HashMap<String, Schema>>,
    pub required: Option<Vec<String>>,
    pub description: Option<String>,
}

impl ClientToolParams {
    pub fn with_properties(mut self, properties: HashMap<String, Schema>) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl Default for ClientToolParams {
    fn default() -> Self {
        ClientToolParams {
            r#type: DataType::Object,
            properties: None,
            required: None,
            description: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Schema {
    Literal(LiteralJsonSchema),
    Object(ObjectJsonSchema),
    Array(ArrayJsonSchema),
}

impl Schema {
    pub fn new_boolean(description: impl Into<String>) -> Self {
        Schema::Literal(LiteralJsonSchema {
            r#type: DataType::Boolean,
            description: description.into(),
        })
    }

    pub fn new_integer(description: impl Into<String>) -> Self {
        Schema::Literal(LiteralJsonSchema {
            r#type: DataType::Integer,
            description: description.into(),
        })
    }

    pub fn new_number(description: impl Into<String>) -> Self {
        Schema::Literal(LiteralJsonSchema {
            r#type: DataType::Number,
            description: description.into(),
        })
    }

    pub fn new_string(description: impl Into<String>) -> Self {
        Schema::Literal(LiteralJsonSchema {
            r#type: DataType::String,
            description: description.into(),
        })
    }

    pub fn new_object(properties: HashMap<String, Schema>) -> Self {
        Schema::Object(ObjectJsonSchema {
            r#type: DataType::Object,
            properties: Some(properties),
            required: None,
            description: None,
        })
    }

    pub fn new_array(items: Schema) -> Self {
        Schema::Array(ArrayJsonSchema {
            r#type: DataType::Array,
            items: Box::new(items),
            description: None,
        })
    }

    pub fn with_required(mut self, required: Vec<String>) -> Self {
        if let Schema::Object(obj) = &mut self {
            obj.required = Some(required);
        }
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        match &mut self {
            Schema::Literal(literal) => {
                literal.description = description.into();
            }
            Schema::Object(obj) => {
                obj.description = Some(description.into());
            }
            Schema::Array(array) => {
                array.description = Some(description.into());
            }
        }
        self
    }

    pub fn with_properties(mut self, properties: HashMap<String, Schema>) -> Self {
        if let Schema::Object(obj) = &mut self {
            obj.properties = Some(properties);
        }
        self
    }

    pub fn with_items(mut self, items: Schema) -> Self {
        if let Schema::Array(array) = &mut self {
            array.items = Box::new(items);
        }
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LiteralJsonSchema {
    pub r#type: DataType,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectJsonSchema {
    r#type: DataType,
    pub properties: Option<HashMap<String, Schema>>,
    pub required: Option<Vec<String>>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArrayJsonSchema {
    r#type: DataType,
    items: Box<Schema>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomLLM {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apikey: Option<SecretType>,
}

impl CustomLLM {
    pub fn new(url: impl Into<String>) -> Self {
        CustomLLM {
            url: url.into(),
            model_id: None,
            apikey: None,
        }
    }

    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn with_apikey(mut self, apikey: SecretType) -> Self {
        self.apikey = Some(apikey);
        self
    }
}

/// Configuration for a webhook that will be called by an LLM tool.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApiSchema {
    url: String,
    method: ApiMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    path_params_schema: Option<HashMap<String, ParamSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_params_schema: Option<QueryParamsSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_body_schema: Option<RequestBodySchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_headers: Option<RequestHeaders>,
}

pub type RequestHeaders = HashMap<String, ConvAIHeaderValue>;

impl ApiSchema {
    pub fn new(url: &str) -> Self {
        ApiSchema {
            url: url.to_string(),
            ..Default::default()
        }
    }

    pub fn with_method(mut self, method: ApiMethod) -> Self {
        self.method = method;
        self
    }
    pub fn with_path_params(mut self, path_params_schema: HashMap<String, ParamSchema>) -> Self {
        self.path_params_schema = Some(path_params_schema);
        self
    }
    pub fn with_query_params(mut self, query_params_schema: QueryParamsSchema) -> Self {
        self.query_params_schema = Some(query_params_schema);
        self
    }

    pub fn with_request_body(mut self, request_body_schema: RequestBodySchema) -> Self {
        self.request_body_schema = Some(request_body_schema);
        self
    }

    pub fn with_request_headers(mut self, request_headers: RequestHeaders) -> Self {
        self.request_headers = Some(request_headers);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConvAIHeaderValue {
    String(String),
    Secret(Secret),
}

impl ConvAIHeaderValue {
    pub fn new_string(value: &str) -> Self {
        ConvAIHeaderValue::String(value.to_string())
    }

    //pub fn new_secret(secret_id: &str) -> Self {
    //    ConvAIHeaderValue::Secret(Secret::new(secret_id))
    //}
}

impl From<String> for ConvAIHeaderValue {
    fn from(value: String) -> Self {
        ConvAIHeaderValue::String(value)
    }
}

impl From<Secret> for ConvAIHeaderValue {
    fn from(secret: Secret) -> Self {
        ConvAIHeaderValue::Secret(secret)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum Secret {
    New {
        name: String,
        value: String,
        #[serde(default = "SecretType::new")]
        r#type: SecretType,
    },
    Stored {
        name: String,
        secret_id: String,
        #[serde(default = "SecretType::stored")]
        r#type: SecretType,
    },
}

impl Secret {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Secret::New {
            name: name.into(),
            value: value.into(),
            r#type: SecretType::New,
        }
    }

    pub fn new_stored(name: impl Into<String>, secret_id: impl Into<String>) -> Self {
        Secret::Stored {
            name: name.into(),
            secret_id: secret_id.into(),
            r#type: SecretType::Stored,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SecretType {
    New,
    Stored,
}

impl SecretType {
    fn new() -> Self {
        SecretType::New
    }

    fn stored() -> Self {
        SecretType::Stored
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ApiMethod {
    #[default]
    GET,
    POST,
    PATCH,
    DELETE,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParamSchema {
    description: String,
    r#type: DataType,
}

impl ParamSchema {
    pub fn new_bool(description: &str) -> Self {
        ParamSchema {
            description: description.to_string(),
            r#type: DataType::Boolean,
        }
    }

    pub fn new_integer(description: &str) -> Self {
        ParamSchema {
            description: description.to_string(),
            r#type: DataType::Integer,
        }
    }

    pub fn new_number(description: &str) -> Self {
        ParamSchema {
            description: description.to_string(),
            r#type: DataType::Number,
        }
    }

    pub fn new_string(description: &str) -> Self {
        ParamSchema {
            description: description.to_string(),
            r#type: DataType::String,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Boolean,
    Integer,
    Number,
    String,
    Object,
    Array,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryParamsSchema {
    properties: HashMap<String, ParamSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
}

impl QueryParamsSchema {
    pub fn new(properties: HashMap<String, ParamSchema>) -> Self {
        QueryParamsSchema {
            properties,
            required: None,
        }
    }

    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestBodySchema {
    r#type: DataType,
    properties: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Webhook,
    Client,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ASR {
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<AsrQuality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<AsrProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_input_audio_format: Option<ConvAIAudioFormat>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    keywords: Vec<String>,
}

// impl `with_quality` and `with_provider` methods
// when enums have more than one variant
impl ASR {
    //pub fn with_quality(mut self, quality: AsrQuality) -> Self {
    //    self.quality = quality;
    //    self
    //}

    //pub fn with_provider(mut self, provider: AsrProvider) -> Self {
    //    self.provider = provider;
    //    self
    //}
    pub fn with_user_input_audio_format(
        mut self,
        user_input_audio_format: ConvAIAudioFormat,
    ) -> Self {
        self.user_input_audio_format = Some(user_input_audio_format);
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AsrQuality {
    #[default]
    High,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AsrProvider {
    #[default]
    ElevenLabs,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ConvAIAudioFormat {
    #[default]
    #[serde(rename = "pcm_16000")]
    Pcm16000hz,
    #[serde(rename = "pcm_22050")]
    Pcm22050hz,
    #[serde(rename = "pcm_24000")]
    Pcm24000hz,
    #[serde(rename = "pcm_44100")]
    Pcm44100hz,
    #[serde(rename = "ulaw_8000")]
    Ulaw8000hz,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Conversation {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub client_events: Vec<ClientEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_seconds: Option<u32>,
}

impl Conversation {
    pub fn with_client_events(mut self, client_events: Vec<ClientEvent>) -> Self {
        self.client_events = client_events;
        self
    }

    pub fn with_max_duration_seconds(mut self, max_duration_seconds: u32) -> Self {
        self.max_duration_seconds = Some(max_duration_seconds);
        self
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Conversation {
            client_events: vec![ClientEvent::Audio, ClientEvent::Interruption],
            max_duration_seconds: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TTSConfig {
    /// The voice model to use for the agent.
    ///
    /// Default: `ConvAIModel::ElevenTurboV2`
    ///
    /// #### Additional Variants
    /// - `ConvAIModel::ElevenTurboV2_5`
    /// - `ConvAIModel::ElevenFlashV2`
    /// - `ConvAIModel::ElevenFlashV2_5`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<ConvAIModel>,
    /// The voice ID to use for the agent.
    ///
    ///  Default: `DefaultVoice::Eric` i.e. `cjVigY5qzO86Huf0OWal`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    /// The output format you want to use for ElevenLabs text to speech
    ///
    ///  Default: `ConvAIAudioFormat::Pcm16000hz`
    ///
    /// #### Additional Variants
    /// - `ConvAIAudioFormat::Pcm22050hz`
    /// - `ConvAIAudioFormat::Pcm24000hz`
    /// - `ConvAIAudioFormat::Pcm44100hz`
    /// - `ConvAIAudioFormat::Ulaw8000hz`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_output_audio_format: Option<ConvAIAudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimize_streaming_latency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_boost: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pronunciation_dictionary_locators: Vec<DictionaryLocator>,
}

impl TTSConfig {
    pub fn with_model_id(mut self, model_id: ConvAIModel) -> Self {
        self.model_id = Some(model_id);
        self
    }

    pub fn with_voice_id(mut self, voice_id: impl Into<String>) -> Self {
        self.voice_id = Some(voice_id.into());
        self
    }

    pub fn with_agent_output_audio_format(
        mut self,
        agent_output_audio_format: ConvAIAudioFormat,
    ) -> Self {
        self.agent_output_audio_format = Some(agent_output_audio_format);
        self
    }

    pub fn with_optimize_streaming_latency(mut self, optimize_streaming_latency: u32) -> Self {
        self.optimize_streaming_latency = Some(optimize_streaming_latency);
        self
    }

    pub fn with_stability(mut self, stability: f32) -> Self {
        self.stability = Some(stability);
        self
    }

    pub fn with_similarity_boost(mut self, similarity_boost: f32) -> Self {
        self.similarity_boost = Some(similarity_boost);
        self
    }

    pub fn with_pronunciation_dictionary_locators(
        mut self,
        pronunciation_dictionary_locators: Vec<DictionaryLocator>,
    ) -> Self {
        self.pronunciation_dictionary_locators = pronunciation_dictionary_locators;
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientEvent {
    AgentResponse,
    AgentResponseCorrection,
    AsrInitiationMetadata,
    Audio,
    ClientToolCall,
    ConversationInitiationMetadata,
    InternalTentativeAgentResponse,
    InternalTurnProbability,
    InternalVadScore,
    Interruption,
    Ping,
    UserTranscript,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ConvAIModel {
    #[default]
    #[serde(rename = "eleven_turbo_v2")]
    ElevenTurboV2,
    #[serde(rename = "eleven_turbo_v2_5")]
    ElevenTurboV2_5,
    #[serde(rename = "eleven_flash_v2")]
    ElevenFlashV2,
    #[serde(rename = "eleven_flash_v2_5")]
    ElevenFlashV2_5,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Turn {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_timeout: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<TurnMode>,
}

impl Turn {
    pub fn with_mode(mut self, mode: TurnMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn with_turn_timeout(mut self, turn_timeout: f32) -> Self {
        self.turn_timeout = Some(turn_timeout);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnMode {
    Silence,
    Turn,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LanguagePreset {
    pub overrides: ConversationConfigOverride,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message_translation: Option<FirstMessageTranslation>,
}

impl LanguagePreset {
    pub fn new(overrides: ConversationConfigOverride) -> Self {
        LanguagePreset {
            overrides,
            first_message_translation: None,
        }
    }

    pub fn with_first_message_translation(
        mut self,
        first_message_translation: FirstMessageTranslation,
    ) -> Self {
        self.first_message_translation = Some(first_message_translation);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FirstMessageTranslation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PlatformSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation: Option<Evaluation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget: Option<Widget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<DataCollection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Overrides>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban: Option<Ban>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety: Option<Safety>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<Privacy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_limits: Option<CallLimits>,
}

impl PlatformSettings {
    pub fn with_auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_evaluation(mut self, evaluation: Evaluation) -> Self {
        self.evaluation = Some(evaluation);
        self
    }

    pub fn with_widget(mut self, widget: Widget) -> Self {
        self.widget = Some(widget);
        self
    }

    pub fn with_data_collection(mut self, data_collection: DataCollection) -> Self {
        self.data_collection = Some(data_collection);
        self
    }

    pub fn with_overrides(mut self, overrides: Overrides) -> Self {
        self.overrides = Some(overrides);
        self
    }

    pub fn with_ban(mut self, ban: Ban) -> Self {
        self.ban = Some(ban);
        self
    }

    pub fn with_safety(mut self, safety: Safety) -> Self {
        self.safety = Some(safety);
        self
    }

    pub fn with_privacy(mut self, privacy: Privacy) -> Self {
        self.privacy = Some(privacy);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Auth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_auth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowlist: Option<Vec<AllowHost>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shareable_token: Option<String>,
}

impl Auth {
    pub fn with_enable_auth(mut self, enable_auth: bool) -> Self {
        self.enable_auth = Some(enable_auth);
        self
    }

    pub fn with_allowlist<'a, I: IntoIterator<Item = &'a str>>(mut self, allowlist: I) -> Self {
        let allowlist = allowlist.into_iter().map(AllowHost::new).collect();

        self.allowlist = Some(allowlist);
        self
    }

    pub fn with_shareable_token(mut self, shareable_token: impl Into<String>) -> Self {
        self.shareable_token = Some(shareable_token.into());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AllowHost {
    hostname: String,
}

impl AllowHost {
    fn new(hostname: &str) -> Self {
        AllowHost {
            hostname: hostname.to_string(),
        }
    }
}

pub type DataCollection = HashMap<String, CustomData>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomData {
    description: String,
    r#type: CustomDataType,
}

// TODO: not needed if `DataType` enum remains private and internally used for all
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum CustomDataType {
    Boolean,
    Integer,
    Number,
    String,
}

impl CustomData {
    pub fn new_boolean(description: impl Into<String>) -> Self {
        CustomData {
            description: description.into(),
            r#type: CustomDataType::Boolean,
        }
    }

    pub fn new_integer(description: impl Into<String>) -> Self {
        CustomData {
            description: description.into(),
            r#type: CustomDataType::Integer,
        }
    }

    pub fn new_number(description: impl Into<String>) -> Self {
        CustomData {
            description: description.into(),
            r#type: CustomDataType::Number,
        }
    }

    pub fn new_string(description: impl Into<String>) -> Self {
        CustomData {
            description: description.into(),
            r#type: CustomDataType::String,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Evaluation {
    pub criteria: Vec<Criterion>,
}

impl Evaluation {
    pub fn new(criteria: Vec<Criterion>) -> Self {
        Evaluation { criteria }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Criterion {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    r#type: CriterionType,
    pub conversation_goal_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_knowledge_base: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum CriterionType {
    #[default]
    Prompt,
}

impl Criterion {
    pub fn new(id: impl Into<String>, conversation_goal_prompt: impl Into<String>) -> Self {
        Criterion {
            id: id.into(),
            name: None,
            r#type: CriterionType::Prompt,
            conversation_goal_prompt: conversation_goal_prompt.into(),
            use_knowledge_base: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_use_knowledge_base(mut self, use_knowledge_base: bool) -> Self {
        self.use_knowledge_base = Some(use_knowledge_base);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Overrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_config_override: Option<ConversationConfigOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_llm_extra_body: Option<bool>,
}

impl Overrides {
    pub fn with_conversation_config_override(
        mut self,
        conversation_config_override: ConversationConfigOverride,
    ) -> Self {
        self.conversation_config_override = Some(conversation_config_override);
        self
    }

    pub fn override_custom_llm_extra_body(mut self, custom_llm_extra_body: bool) -> Self {
        self.custom_llm_extra_body = Some(custom_llm_extra_body);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConversationConfigOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<TTSOverride>,
}

impl ConversationConfigOverride {
    pub fn with_agent_override(mut self, agent: AgentOverride) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn with_tts_override(mut self, tts: TTSOverride) -> Self {
        self.tts = Some(tts);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AgentOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<PromptOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<bool>,
}

impl AgentOverride {
    pub fn with_prompt_override(mut self, prompt: PromptOverride) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn override_first_message(mut self, first_message: bool) -> Self {
        self.first_message = Some(first_message);
        self
    }

    pub fn override_language(mut self, language: bool) -> Self {
        self.language = Some(language);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PromptOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<bool>,
}

impl PromptOverride {
    pub fn override_prompt(mut self, prompt: bool) -> Self {
        self.prompt = Some(prompt);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TTSOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<bool>,
}

impl TTSOverride {
    pub fn override_voice_id(mut self, voice_id: bool) -> Self {
        self.voice_id = Some(voice_id);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ban {
    pub at_unix: u64,
    pub reason_type: BanReasonType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BanReasonType {
    Safety,
    Manual,
}

impl Ban {
    pub fn new_safety(at_unix: u64) -> Self {
        Ban {
            at_unix,
            reason_type: BanReasonType::Safety,
            reason: None,
        }
    }

    pub fn new_manual(at_unix: u64) -> Self {
        Ban {
            at_unix,
            reason_type: BanReasonType::Manual,
            reason: None,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Safety {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ivc: Option<IVC>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_ivc: Option<NonIVC>,
}

impl Safety {
    pub fn with_ivc(mut self, ivc: IVC) -> Self {
        self.ivc = Some(ivc);
        self
    }

    pub fn with_non_ivc(mut self, non_ivc: NonIVC) -> Self {
        self.non_ivc = Some(non_ivc);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IVC {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_unsafe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_prompt_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_rule_id: Option<Vec<MatchedRule>>,
}

impl IVC {
    pub fn with_is_unsafe(mut self, is_unsafe: bool) -> Self {
        self.is_unsafe = Some(is_unsafe);
        self
    }

    pub fn with_llm_reason(mut self, llm_reason: impl Into<String>) -> Self {
        self.llm_reason = Some(llm_reason.into());
        self
    }

    pub fn with_safety_prompt_version(mut self, safety_prompt_version: u32) -> Self {
        self.safety_prompt_version = Some(safety_prompt_version);
        self
    }

    pub fn with_matched_rule_ids<'a, I: IntoIterator<Item = &'a str>>(
        mut self,
        matched_rule_ids: I,
    ) -> Self {
        let matched_rule_ids = matched_rule_ids.into_iter().map(MatchedRule::new).collect();

        self.matched_rule_id = Some(matched_rule_ids);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NonIVC {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_unsafe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_prompt_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_rule_id: Option<Vec<MatchedRule>>,
}

impl NonIVC {
    pub fn with_is_unsafe(mut self, is_unsafe: bool) -> Self {
        self.is_unsafe = Some(is_unsafe);
        self
    }

    pub fn with_llm_reason(mut self, llm_reason: impl Into<String>) -> Self {
        self.llm_reason = Some(llm_reason.into());
        self
    }

    pub fn with_safety_prompt_version(mut self, safety_prompt_version: u32) -> Self {
        self.safety_prompt_version = Some(safety_prompt_version);
        self
    }

    pub fn with_matched_rule_ids<'a, I: IntoIterator<Item = &'a str>>(
        mut self,
        matched_rule_ids: I,
    ) -> Self {
        let matched_rule_ids = matched_rule_ids.into_iter().map(MatchedRule::new).collect();

        self.matched_rule_id = Some(matched_rule_ids);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchedRule {
    SexualMinors,
    ForgetModeration,
    Extremism,
    ScamFraud,
    Political,
    SelfHarm,
    IllegalDistributionMedical,
    SexualAdults,
    Unknown,
}

impl MatchedRule {
    pub fn new(id: &str) -> Self {
        match id {
            "sexual_minors" => MatchedRule::SexualMinors,
            "forget_moderation" => MatchedRule::ForgetModeration,
            "extremism" => MatchedRule::Extremism,
            "scam_fraud" => MatchedRule::ScamFraud,
            "political" => MatchedRule::Political,
            "self_harm" => MatchedRule::SelfHarm,
            "illegal_distribution_medical" => MatchedRule::IllegalDistributionMedical,
            "sexual_adults" => MatchedRule::SexualAdults,
            "unknown" => MatchedRule::Unknown,
            _ => MatchedRule::Unknown,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Privacy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_voice: Option<bool>,
}

impl Privacy {
    pub fn record_voice(mut self, record_voice: bool) -> Self {
        self.record_voice = Some(record_voice);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CallLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_concurrency_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_limit: Option<u32>,
}

impl CallLimits {
    pub fn with_agent_concurrency_limit(mut self, agent_concurrency_limit: i32) -> Self {
        self.agent_concurrency_limit = Some(agent_concurrency_limit);
        self
    }

    pub fn with_daily_limit(mut self, daily_limit: u32) -> Self {
        self.daily_limit = Some(daily_limit);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Widget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<WidgetVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Avatar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_mode: Option<FeedBackMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_avatar_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_text_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_radius: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_call_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_call_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listening_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaking_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shareable_page_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_keys: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_language_overrides: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WidgetVariant {
    Compact,
    Full,
    Expandable,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Avatar {
    Image {
        r#type: AvatarType,
        url: Option<String>,
    },
    Orb {
        r#type: AvatarType,
        color_1: Option<String>,
        color_2: Option<String>,
    },
    Url {
        r#type: AvatarType,
        custom_url: Option<String>,
    },
}

impl Avatar {
    pub fn default_image() -> Self {
        Avatar::Image {
            r#type: AvatarType::Image,
            url: None,
        }
    }

    pub fn default_orb() -> Self {
        Avatar::Orb {
            r#type: AvatarType::Orb,
            color_1: None,
            color_2: None,
        }
    }

    pub fn default_url() -> Self {
        Avatar::Url {
            r#type: AvatarType::Url,
            custom_url: None,
        }
    }

    pub fn with_custom_url(mut self, custom_url: &str) -> Self {
        if let Avatar::Image { ref mut url, .. } = self {
            *url = Some(custom_url.to_string());
        }
        self
    }

    pub fn with_color_1(mut self, color: &str) -> Self {
        if let Avatar::Orb { ref mut color_1, .. } = self {
            *color_1 = Some(color.to_string());
        }
        self
    }

    pub fn with_color_2(mut self, color: &str) -> Self {
        if let Avatar::Orb { ref mut color_2, .. } = self {
            *color_2 = Some(color.to_string());
        }
        self
    }

    pub fn with_url(mut self, url: &str) -> Self {
        if let Avatar::Url { ref mut custom_url, .. } = self {
            *custom_url = Some(url.to_string());
        }
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AvatarType {
    Image,
    #[default]
    Orb,
    Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedBackMode {
    None,
    During,
    End,
}

impl Widget {
    pub fn with_variant(mut self, variant: WidgetVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn with_avatar(mut self, avatar: Avatar) -> Self {
        self.avatar = Some(avatar);
        self
    }

    pub fn with_feedback_mode(mut self, feedback_mode: FeedBackMode) -> Self {
        self.feedback_mode = Some(feedback_mode);
        self
    }

    pub fn with_custom_avatar_path(mut self, custom_avatar_path: impl Into<String>) -> Self {
        self.custom_avatar_path = Some(custom_avatar_path.into());
        self
    }

    pub fn with_bg_color(mut self, bg_color: impl Into<String>) -> Self {
        self.bg_color = Some(bg_color.into());
        self
    }

    pub fn with_text_color(mut self, text_color: impl Into<String>) -> Self {
        self.text_color = Some(text_color.into());
        self
    }

    pub fn with_btn_color(mut self, btn_color: impl Into<String>) -> Self {
        self.btn_color = Some(btn_color.into());
        self
    }

    pub fn with_btn_text_color(mut self, btn_text_color: impl Into<String>) -> Self {
        self.btn_text_color = Some(btn_text_color.into());
        self
    }

    pub fn with_border_color(mut self, border_color: impl Into<String>) -> Self {
        self.border_color = Some(border_color.into());
        self
    }

    pub fn with_focus_color(mut self, focus_color: impl Into<String>) -> Self {
        self.focus_color = Some(focus_color.into());
        self
    }

    pub fn with_border_radius(mut self, border_radius: i64) -> Self {
        self.border_radius = Some(border_radius);
        self
    }

    pub fn with_btn_radius(mut self, btn_radius: i64) -> Self {
        self.btn_radius = Some(btn_radius);
        self
    }

    pub fn with_action_text(mut self, action_text: impl Into<String>) -> Self {
        self.action_text = Some(action_text.into());
        self
    }

    pub fn with_start_call_text(mut self, start_call_text: impl Into<String>) -> Self {
        self.start_call_text = Some(start_call_text.into());
        self
    }

    pub fn with_end_call_text(mut self, end_call_text: impl Into<String>) -> Self {
        self.end_call_text = Some(end_call_text.into());
        self
    }

    pub fn with_expand_text(mut self, expand_text: impl Into<String>) -> Self {
        self.expand_text = Some(expand_text.into());
        self
    }

    pub fn with_listening_text(mut self, listening_text: impl Into<String>) -> Self {
        self.listening_text = Some(listening_text.into());
        self
    }

    pub fn with_speaking_text(mut self, speaking_text: impl Into<String>) -> Self {
        self.speaking_text = Some(speaking_text.into());
        self
    }

    pub fn with_shareable_page_text(mut self, shareable_page_text: impl Into<String>) -> Self {
        self.shareable_page_text = Some(shareable_page_text.into());
        self
    }
}

/// Retrieve config for an agent
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agents::GetAgent;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let resp = client.hit(GetAgent::new("agent_id")).await?;
///    println!("{:?}", resp);
/// Ok(())
/// }
/// ```
///
/// See [Get Agent API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/get-agent)
#[derive(Clone, Debug, Serialize)]
pub struct GetAgent {
    agent_id: String,
}

impl GetAgent {
    pub fn new(agent_id: impl Into<String>) -> Self {
        GetAgent {
            agent_id: agent_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetAgent {
    const PATH: &'static str = "/v1/convai/agents/:agent_id";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetAgentResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAgentResponse {
    pub agent_id: String,
    pub name: String,
    pub conversation_config: ConversationConfig,
    pub platform_settings: Option<PlatformSettings>,
    pub metadata: Metadata,
    pub secrets: Vec<Secret>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Metadata {
    pub created_at_unix_secs: u64,
}

/// Returns a page of your agents and their metadata.
///
///
///
/// # Query Parameters
///
/// - `search` (optional): A search term to filter agents by name.
/// - `page_size` (optional): The number of agents to return per page. Can not exceed 100, default is 30.
/// - `cursor` (optional): A cursor to paginate through the list of agents.
///
/// # Response
///
/// The response will contain a list of agents and metadata about the list.
///
/// - `agents`: A `Vec<Agent>`.
/// - `has_more`: A boolean indicating if there are more agents to retrieve.
/// - `next_cursor`: A cursor to paginate to the next page of agents.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agents::{GetAgents, GetAgentsQuery};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let query = GetAgentsQuery::default().with_page_size(3);
///    let agents = client.hit(GetAgents::with_query(query)).await?;
///    for agent in agents {
///         println!("{:?}", agent);
///   }
///   Ok(())
/// }
/// ```
/// See [Get Agents API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/get-agents)
#[derive(Clone, Debug, Default, Serialize)]
pub struct GetAgents {
    query: Option<GetAgentsQuery>,
}

impl GetAgents {
    pub fn with_query(query: GetAgentsQuery) -> Self {
        GetAgents { query: Some(query) }
    }
}

impl ElevenLabsEndpoint for GetAgents {
    const PATH: &'static str = "/v1/convai/agents";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetAgentsResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAgentsResponse {
    pub agents: Vec<Agent>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Agent {
    pub agent_id: String,
    pub name: String,
    pub created_at_unix_secs: u64,
    pub access_level: AccessLevel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    Admin,
    Editor,
    Viewer,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetAgentsQuery {
    params: QueryValues,
}

impl GetAgentsQuery {
    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.params.push(("search", search.into()));
        self
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
/// Patches an Agent settings
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agents::*;
/// use elevenlabs_rs::{DefaultVoice, ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ElevenLabsClient::from_env()?;
///
///     let new_secret = Secret::new("new_secret", "new_secret_value");
///     let new_stored_secret = Secret::new("another_secret", "another_value");
///     let secrets = vec![new_secret];
///
///     let updated_config = ConversationConfig::default()
///         .with_agent_config(AgentConfig::default().with_first_message("updated first message"))
///         .with_tts_config(TTSConfig::default().with_voice_id(DefaultVoice::Matilda))
///         .with_conversation(Conversation::default().with_max_duration_seconds(60));
///
///     let body = UpdateAgentBody::default()
///         .with_conversation_config(updated_config)
///         .with_name("updated agent")
///         .with_secrets(secrets);
///
///     let endpoint = UpdateAgent::new("agent_id", body);
///
///     let resp = client.hit(endpoint).await?;
///
///     println!("{:?}", resp);
///
///     Ok(())
/// }
/// ```
/// See [Update Agent API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/update-agent)
#[derive(Clone, Debug)]
pub struct UpdateAgent {
    agent_id: String,
    body: UpdateAgentBody,
}

impl UpdateAgent {
    pub fn new(agent_id: &str, body: UpdateAgentBody) -> Self {
        UpdateAgent {
            agent_id: agent_id.to_string(),
            body,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateAgentBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_config: Option<ConversationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_settings: Option<PlatformSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secrets: Option<Vec<Secret>>,
}

impl UpdateAgentBody {
    pub fn with_conversation_config(mut self, conversation_config: ConversationConfig) -> Self {
        self.conversation_config = Some(conversation_config);
        self
    }
    pub fn with_platform_settings(mut self, platform_settings: PlatformSettings) -> Self {
        self.platform_settings = Some(platform_settings);
        self
    }
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Add a secret to the agent.
    ///
    /// # Example
    ///
    ///
    /// ```
    /// use elevenlabs_rs::endpoints::convai::agents::{UpdateAgent, UpdateAgentBody, Secret};
    ///
    /// let body = UpdateAgentBody::default().with_secrets(vec![
    ///     Secret::new("secret_name", "secret_value"),
    ///     Secret::new("other_secret_name", "other_secret_value"),
    /// ]);
    ///
    /// let endpoint = UpdateAgent::new("my_agent_id", body);
    ///
    /// ```
    pub fn with_secrets(mut self, secrets: Vec<Secret>) -> Self {
        self.secrets = Some(secrets);
        self
    }
}

type UpdateAgentResponse = GetAgentResponse;

impl ElevenLabsEndpoint for UpdateAgent {
    const PATH: &'static str = "/v1/convai/agents/:agent_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = UpdateAgentResponse;

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

impl TryInto<RequestBody> for &UpdateAgentBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_into(self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(self)?))
    }
}

/// Get the current link used to share the agent with others
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agents::GetLink;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let resp = client.hit(GetLink::new("agent_id")).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See [Get Link API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/agents/get-agent-link)
#[derive(Clone, Debug)]
pub struct GetLink {
    agent_id: String,
}

impl GetLink {
    pub fn new(agent_id: impl Into<String>) -> Self {
        GetLink {
            agent_id: agent_id.into(),
        }
    }
}

impl ElevenLabsEndpoint for GetLink {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/link";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetLinkResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetLinkResponse {
    pub agent_id: String,
    pub token: Option<Token>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Token {
    pub agent_id: String,
    pub conversation_token: String,
    pub expiration_time_unix_secs: Option<u64>,
    pub purpose: Option<Purpose>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    SignedUrl,
    ShareableLink,
}

impl IntoIterator for GetAgentsResponse {
    type Item = Agent;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.agents.into_iter()
    }
}

impl<'a> IntoIterator for &'a GetAgentsResponse {
    type Item = &'a Agent;
    type IntoIter = std::slice::Iter<'a, Agent>;

    fn into_iter(self) -> Self::IntoIter {
        self.agents.iter()
    }
}
