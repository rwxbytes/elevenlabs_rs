//! Widget endpoints

use std::path::Path;
use crate::endpoints::convai::agents::Widget;
use crate::error::Error;
use super::*;

/// Retrieve the widget configuration for an agent
///
/// # Example
///
/// ```no_run
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

/// Sets the avatar for an agent displayed in the widget
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::endpoints::convai::widget::{CreateWidgetAvatar, CreateWidgetAvatarBody};
/// use elevenlabs_rs::{ElevenLabsClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let client = ElevenLabsClient::from_env()?;
///    let body = CreateWidgetAvatarBody::new("avatar_file");
///    let resp = client.hit(CreateWidgetAvatar::new("agent_id", body)).await?;
///   println!("{:?}", resp);
///   Ok(())
/// }
/// ```
/// See [Create Widget Avatar API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/widget/post-agent-avatar)
#[derive(Clone, Debug)]
pub struct CreateWidgetAvatar {
    pub agent_id: String,
    pub body: CreateWidgetAvatarBody
}

impl CreateWidgetAvatar {
    pub fn new(agent_id: impl Into<String>, body: CreateWidgetAvatarBody) -> Self {
        Self {
            agent_id: agent_id.into(),
            body
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreateWidgetAvatarBody {
    pub avatar_file: String
}

impl CreateWidgetAvatarBody {
    pub fn new(avatar_file: impl Into<String>) -> Self {
        Self {
            avatar_file: avatar_file.into()
        }
    }
}

impl TryFrom<&CreateWidgetAvatarBody> for RequestBody
{
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(body: &CreateWidgetAvatarBody) -> Result<Self> {
        let path = Path::new(&body.avatar_file);
        let bytes = std::fs::read(path)?;
        let mut part = Part::bytes(bytes);
        let file_path_str = path.to_str().ok_or(Box::new(Error::PathNotValidUTF8))?;
        part = part.file_name(file_path_str.to_string());
        let mime_subtype = path
            .extension()
            .ok_or(Box::new(Error::FileExtensionNotFound))?
            .to_str()
            .ok_or(Box::new(Error::FileExtensionNotValidUTF8))?;
        let mime = format!("image/{}", mime_subtype);
        part = part.mime_str(&mime)?;
        Ok(RequestBody::Multipart(Form::new().part("avatar_file", part)))
    }
}

impl ElevenLabsEndpoint for CreateWidgetAvatar {
    const PATH: &'static str = "/v1/convai/agents/:agent_id/avatar";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateWidgetAvatarResponse;

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

#[derive(Clone, Debug, Deserialize)]
pub struct CreateWidgetAvatarResponse {
    pub agent_id: String,
    pub avatar_url: String
}

