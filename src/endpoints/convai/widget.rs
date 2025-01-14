//! Widget endpoints

use crate::endpoints::convai::agents::Widget;
use super::*;

/// Retrieve the widget configuration for an agent
///
/// # Example
///
/// ```
/// use elevenlabs_rs::endpoints::convai::widget::GetWidget;
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let resp = client.hit(GetWidget::new("agent_id")).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// See [Get Widget API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/widget/get-agent-widget)
#[derive(Clone, Debug)]
pub struct GetWidget {
    pub agent_id: String,
    pub query: Option<GetWidgetQuery>
}

impl GetWidget {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            query: None
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetWidgetQuery {
    params: QueryValues
}


impl GetWidgetQuery {
    pub fn with_conversation_signature(&mut self, conversation_signature: impl Into<String>) -> &mut Self {
        self.params.push(("conversation_signature", conversation_signature.into()));
        self
    }

}

impl ElevenLabsEndpoint for GetWidget {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/widget/";

    const METHOD: Method = Method::GET;

    type ResponseBody = WidgetResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self.agent_id.and_param(PathParam::AgentID)]
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct WidgetResponse {
    pub agent_id: String,
    pub widget_config: Widget
}

