#![allow(dead_code)]
//! Agents endpoints
use super::*;

const AGENTS_PATH: &str = "/v1/convai/agents";
const AGENT_ID_QUERY: &str = "agent_id";
const CURSOR_QUERY: &str = "cursor";
const SEARCH_QUERY: &str = "search";
const PAGE_SIZE_QUERY: &str = "page_size";

/// See the official [Delete Agent API reference](https://elevenlabs.io/docs/api-reference/delete-conversational-ai-agent)
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agents::{DeleteAgent, GetAgentsQuery, GetAgents};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::default()?;
///    let agent_query = GetAgentsQuery::default().with_search("Foo");
///    let agents_resp = client.hit(GetAgents::new(agent_query)).await?;
///    let Some(foo_agent) = agents_resp.agents().first() else {
///         return Err("Agent named Foo not found".into());
///   };
///    let endpoint = DeleteAgent::new(foo_agent.agent_id());
///    let _ = client.hit(endpoint).await?;
///    Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct DeleteAgent(AgentID);

impl DeleteAgent {
    pub fn new<T: Into<String>>(agent_id: T) -> Self {
        DeleteAgent(AgentID(agent_id.into()))
    }
}

impl Endpoint for DeleteAgent {
    type ResponseBody = ();

    fn method(&self) -> Method {
        Method::DELETE
    }

    async fn response_body(self, _: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!("{}/{}", AGENTS_PATH, self.0.0));
        url
    }
}



/// see Elevenlabs' docs on [Get Agents](https://elevenlabs.io/docs/api-reference/get-conversational-ai-agents)
///
/// This endpoint retrieves a list of agents that are available for use in the Conversational AI API.
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
/// - `agents`: A list of agents.
/// - `has_more`: A boolean indicating if there are more agents to retrieve.
/// - `next_cursor`: A cursor to paginate to the next page of agents.
///
/// # Example
///
/// // TODO: do we stop using pub use, and instead use the full path?
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::agents::{GetAgents, GetAgentsQuery};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::default()?;
///    let query = GetAgentsQuery::default();
///    let agents = client.hit(GetAgents::new(query)).await?;
///    for agent in agents {
///         println!("{:?}", agent);
///   }
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct GetAgents(GetAgentsQuery);

impl GetAgents {
    pub fn new(query: GetAgentsQuery) -> Self {
        GetAgents(query)
    }
}

impl Endpoint for GetAgents {
    type ResponseBody = GetAgentsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(AGENTS_PATH);
        url.set_query(self.0.join_query().as_deref());
        url
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetAgentsResponse {
    agents: Vec<Agent>,
    has_more: bool,
    next_cursor: Option<String>,
}

impl GetAgentsResponse {
    pub fn agents(&self) -> &[Agent] {self.agents.as_slice()}
    pub fn is_more(&self) -> bool {self.has_more}
    pub fn cursor(&self) -> Option<&str> {self.next_cursor.as_deref()}
}

impl Iterator for GetAgentsResponse {
    type Item = Agent;

    fn next(&mut self) -> Option<Self::Item> {
        self.agents.pop()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Agent {
    agent_id: String,
    name: String,
    created_at_unix_secs: u64,
}

impl Agent {
    pub fn agent_id(&self) -> &str {self.agent_id.as_str()}
    pub fn name(&self) -> &str {self.name.as_str()}
    pub fn created_at(&self) -> u64 {self.created_at_unix_secs}
}


#[derive(Clone, Debug, Default, Serialize)]
pub struct GetAgentsQuery {
    search: Option<String>,
    page_size: Option<String>,
    cursor: Option<String>,
}

impl GetAgentsQuery {
    pub fn with_search<T: Into<String>>(mut self, search: T) -> Self {
        self.search = Some(format!("{}={}", SEARCH_QUERY, search.into()));
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(format!("{}={}", PAGE_SIZE_QUERY, page_size));
        self
    }

    pub fn with_cursor<T: Into<String>>(mut self, cursor: T) -> Self {
        self.cursor = Some(format!("{}={}", CURSOR_QUERY, cursor.into()));
        self
    }


    fn join_query(&self) -> Option<String> {
        let mut query = String::new();
        if let Some(search) = &self.search {
            query.push_str(&search);
        }
        if let Some(page_size) = &self.page_size {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(&page_size);
        }
        if let Some(cursor) = &self.cursor {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(&cursor);
        }
        if query.is_empty() {
            None
        } else {
            Some(query)
        }
    }
}