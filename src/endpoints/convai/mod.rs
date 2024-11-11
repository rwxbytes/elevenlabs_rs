pub use super::*;
pub mod agents;
pub mod conversations;


#[derive(Clone, Debug, Serialize)]
pub(crate) struct AgentID(String);

impl From<String> for AgentID {
    fn from(id: String) -> Self {
        AgentID(id)
    }
}