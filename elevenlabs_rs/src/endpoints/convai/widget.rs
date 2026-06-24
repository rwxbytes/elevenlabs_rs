//! Widget endpoints

use super::*;
use crate::endpoints::convai::agents::Widget;
use crate::shared::{image_mime_from_extension, FilePart};

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
    pub query: Option<GetWidgetQuery>,
}

impl GetWidget {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            query: None,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetWidgetQuery {
    params: QueryValues,
}

impl GetWidgetQuery {
    pub fn with_conversation_signature(
        &mut self,
        conversation_signature: impl Into<String>,
    ) -> &mut Self {
        self.params
            .push(("conversation_signature", conversation_signature.into()));
        self
    }
}

impl crate::endpoints::sealed::Sealed for GetWidget {}

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
    pub widget_config: Widget,
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
    pub body: CreateWidgetAvatarBody,
}

impl CreateWidgetAvatar {
    pub fn new(agent_id: impl Into<String>, body: CreateWidgetAvatarBody) -> Self {
        Self {
            agent_id: agent_id.into(),
            body,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreateWidgetAvatarBody {
    pub avatar_file: FilePart,
}

impl CreateWidgetAvatarBody {
    pub fn new(avatar_file: impl Into<FilePart>) -> Self {
        Self {
            avatar_file: avatar_file.into(),
        }
    }

    pub fn from_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self::new(FilePart::bytes(file_name, mime, bytes))
    }
}

impl TryFrom<&CreateWidgetAvatarBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &CreateWidgetAvatarBody) -> Result<Self> {
        let inferred_mime = inferred_image_mime(&body.avatar_file)?;
        Ok(RequestBody::Multipart(Form::new().part(
            "avatar_file",
            body.avatar_file.clone().into_part(inferred_mime)?,
        )))
    }
}

fn inferred_image_mime(file: &FilePart) -> Result<Option<String>> {
    if file.mime().is_some() {
        return Ok(None);
    }

    let extension = file.extension()?;
    Ok(Some(image_mime_from_extension(&extension)?.to_owned()))
}

impl crate::endpoints::sealed::Sealed for CreateWidgetAvatar {}

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
    pub avatar_url: String,
}
