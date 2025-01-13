use std::string::ToString;
use strum::IntoStaticStr;

pub(crate) trait AndPathParam {
    fn and_param(&self, id: PathParam) -> (&'static str, &str);
}

impl AndPathParam for String {
    fn and_param(&self, param: PathParam) -> (&'static str, &str) {
        (param.into(), self)
    }
}


#[derive(IntoStaticStr)]
pub(crate) enum PathParam {
    #[strum(serialize = ":agent_id")]
    AgentID,
    #[strum(serialize = ":conversation_id")]
    ConversationID,
    #[strum(serialize = ":documentation_id")]
    DocumentationID
}