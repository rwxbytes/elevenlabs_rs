//! The voice library endpoints
//!
//! This module contains endpoints related to the voice library.
//! The voice library is a collection of shared voices that can be used by users.
//! Users can add shared voices to their collection of voices in VoiceLab.
//! The shared voices can be filtered by various criteria such as:
//!
//! - page size
//! - category
//! - gender
//! - age
//! - accent
//! - language
//! - search
//! - use cases
//! - descriptives
//! - featured
//! - rendered app enabled
//! - owner ID
//! - sort
//! - page
//!
//! # Example
//! ```no_run
//! use elevenlabs_rs::{ElevenLabsClient, Result};
//! use elevenlabs_rs::endpoints::admin::voice_library::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let c = ElevenLabsClient::from_env()?;
//!
//!     let mut query = SharedVoicesQuery::default();
//!
//!     query = query
//!         .with_page_size(1)
//!         .with_category(SharedVoiceCategory::HighQuality)
//!         .with_gender(Gender::Female)
//!         .with_age(Age::Young)
//!         .with_language(Language::English)
//!         .with_accent("indian")
//!         .with_use_cases("social_media");
//!
//!     let resp = c.hit(GetSharedVoices::with_query(query)).await?;
//!
//!     if let Some(shared_voice) = resp.voices.first() {
//!         let public_user_id = &shared_voice.public_owner_id;
//!         let voice_id = &shared_voice.voice_id;
//!         let add_shared_voice = AddSharedVoice::new(public_user_id, voice_id, "Maya");
//!         let resp = c.hit(add_shared_voice).await?;
//!         println!("{:#?}", resp);
//!     } else {
//!         println!("no shared voices found with query")
//!     }
//!     Ok(())
//! }
//! ```

use super::*;
pub use crate::shared::{Age, Language, VerifiedLanguage, SharedVoice};

/// Gets a list of shared voices.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice_library::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let mut query = SharedVoicesQuery::default();
///     query = query
///         .with_page_size(1)
///         .with_category(SharedVoiceCategory::Professional)
///         .with_gender(Gender::Male)
///         .with_age(Age::MiddleAged)
///         .with_accent("irish")
///         .with_language(Language::English)
///         .with_use_cases("narrative_story")
///         .with_descriptives("confident");
///     let resp = c.hit(GetSharedVoices::with_query(query)).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See [Get Shared Voices API reference](https://elevenlabs.io/docs/api-reference/voice-library/get-shared)
#[derive(Clone, Debug, Default)]
pub struct GetSharedVoices {
    query: Option<SharedVoicesQuery>,
}

impl GetSharedVoices {
    pub fn with_query(query: SharedVoicesQuery) -> Self {
        GetSharedVoices { query: Some(query) }
    }
}

impl ElevenLabsEndpoint for GetSharedVoices {
    const PATH: &'static str = "/v1/shared-voices";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetSharedVoicesResponse;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Shared voices response
#[derive(Clone, Debug, Deserialize)]
pub struct GetSharedVoicesResponse {
    pub voices: Vec<SharedVoice>,
    pub has_more: bool,
    pub last_sort_id: Option<String>,
}

/// Shared voices query
///
/// See [ElevenLabs API documentation](https://elevenlabs.io/docs/api-reference/query-library) for more information
#[derive(Clone, Debug, Default)]
pub struct SharedVoicesQuery {
    params: QueryValues,
}

impl SharedVoicesQuery {
    pub fn with_page_size(mut self, page_size: u16) -> Self {
        self.params.push(("page_size", page_size.to_string()));
        self
    }
    pub fn with_category(mut self, category: SharedVoiceCategory) -> Self {
        self.params.push(("category", category.to_string()));
        self
    }
    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.params.push(("gender", gender.to_string()));
        self
    }
    pub fn with_age(mut self, age: Age) -> Self {
        self.params.push(("age", age.as_str().to_string()));
        self
    }
    pub fn with_accent(mut self, accent: &str) -> Self {
        self.params.push(("accent", accent.to_string()));
        self
    }
    pub fn with_language(mut self, language: Language) -> Self {
        let language = serde_json::to_string(&language).unwrap();
        self.params.push(("language", language));
        self
    }
    pub fn with_search(mut self, search: &str) -> Self {
        self.params.push(("search", search.to_string()));
        self
    }
    pub fn with_use_cases(mut self, use_cases: &str) -> Self {
        self.params.push(("use_cases", use_cases.to_string()));
        self
    }
    pub fn with_descriptives(mut self, descriptives: &str) -> Self {
        self.params.push(("descriptives", descriptives.to_string()));
        self
    }
    pub fn with_featured(mut self, featured: bool) -> Self {
        self.params.push(("featured", featured.to_string()));
        self
    }
    pub fn with_rendered_app_enabled(mut self, rendered_app_enabled: bool) -> Self {
        self.params
            .push(("rendered_app_enabled", rendered_app_enabled.to_string()));
        self
    }
    pub fn with_owner_id(mut self, owner_id: &str) -> Self {
        self.params.push(("owner_id", owner_id.to_string()));
        self
    }
    pub fn with_sort(mut self, sort: &str) -> Self {
        self.params.push(("sort", sort.to_string()));
        self
    }
    pub fn with_page(mut self, page: u16) -> Self {
        self.params.push(("page", page.to_string()));
        self
    }

    pub fn with_min_notice_period_days(mut self, days: u32) -> Self {
        self.params.push(("min_notice_period_days", days.to_string()));
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
    Neutral,
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Gender::Female => "female",
            Gender::Male => "male",
            Gender::Neutral => "neutral",
        };
        write!(f, "{}", value)
    }
}

/// Add a sharing voice to your collection of voices in VoiceLab.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::voice_library::AddSharedVoice;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let public_user_id = "some_public_user_id";
///     let voice_id = "some_voice_id";
///     let name = "new_voice_name";
///     let endpoint = AddSharedVoice::new(public_user_id, voice_id, name);
///     let resp = c.hit(endpoint).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See [Add Sharing Voice API reference](https://elevenlabs.io/docs/api-reference/voice-library/add-sharing-voice)
#[derive(Clone, Debug)]
pub struct AddSharedVoice {
    public_user_id: String,
    voice_id: String,
    body: AddSharedVoiceBody,
}

impl AddSharedVoice {
    pub fn new(
        public_user_id: impl Into<String>,
        voice_id: impl Into<String>,
        new_name: &str,
    ) -> Self {
        let body = AddSharedVoiceBody::new(new_name);
        AddSharedVoice {
            public_user_id: public_user_id.into(),
            voice_id: voice_id.into(),
            body,
        }
    }
}

impl ElevenLabsEndpoint for AddSharedVoice {
    const PATH: &'static str = "/v1/voices/add/:public_user_id/:voice_id";

    const METHOD: Method = Method::POST;

    type ResponseBody = AddSharedVoiceResponse;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![
            self.public_user_id.and_param(PathParam::PublicUserID),
            self.voice_id.and_param(PathParam::VoiceID),
        ]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// Response for adding a shared voice
#[derive(Clone, Debug, Deserialize)]
pub struct AddSharedVoiceResponse {
    pub voice_id: String,
}

/// The name that identifies this voice. This will be displayed in the dropdown of the website.
#[derive(Clone, Debug, Serialize)]
pub struct AddSharedVoiceBody {
    pub new_name: String,
}

impl AddSharedVoiceBody {
    pub fn new(new_name: &str) -> Self {
        AddSharedVoiceBody {
            new_name: new_name.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedVoiceCategory {
    Generated,
    HighQuality,
    Professional,
    Famous,
}

impl std::fmt::Display for SharedVoiceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            SharedVoiceCategory::Generated => "generated",
            SharedVoiceCategory::HighQuality => "high_quality",
            SharedVoiceCategory::Professional => "professional",
            SharedVoiceCategory::Famous => "famous",
        };
        write!(f, "{}", value)
    }
}
