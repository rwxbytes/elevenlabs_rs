// TODO: Get API access to the projects endpoint
#![allow(dead_code)]
use super::*;

/// Returns a list of your projects together and its metadata.
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::projects::GetProjects;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let resp = c.hit(GetProjects).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GetProjects;


impl ElevenLabsEndpoint for GetProjects {
    const PATH: &'static str = "/v1/projects";

    const METHOD: Method = Method::GET;
    type ResponseBody = ProjectsResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

}

#[derive(Clone, Debug, Deserialize)]
pub struct ProjectsResponse {
    projects: Vec<Project>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub create_date_unix: u64,
    pub default_title_voice_id: String,
    pub default_paragraph_voice_id: String,
    pub default_model_id: String,
    pub last_conversion_date_unix: u64,
    pub can_be_downloaded: bool,
    pub title: String,
    pub author: String,
    pub isbn_number: String,
    pub volume_normalization: bool,
    pub state: String,
}
