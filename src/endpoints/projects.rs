// TODO: Get API access to the projects endpoint
use super::*;

const PROJECTS_PATH: &str = "/v1/projects";

/// Projects endpoint
///
/// # Example
///
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::projects::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let resp = c.hit(GetProjects::new()).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GetProjects;

impl GetProjects {
    pub fn new() -> Self {
        GetProjects
    }
}

impl Endpoint for GetProjects {
    type ResponseBody = ProjectsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)

    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(PROJECTS_PATH);
        url
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProjectsResponse {
    projects: Vec<Project>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Project {
    project_id: String,
    name: String,
    create_date_unix: u64,
    default_title_voice_id: String,
    default_paragraph_voice_id: String,
    default_model_id: String,
    last_conversion_date_unix: u64,
    can_be_downloaded: bool,
    title: String,
    author: String,
    isbn_number: String,
    volume_normalization: bool,
    state: String,
}
