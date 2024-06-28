#![allow(dead_code)]
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
//! use elevenlabs_rs::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let c = ElevenLabsClient::default()?;
//!
//!     let mut query = SharedVoicesQuery::default();
//!     query = query
//!         .with_page_size(1)
//!         .with_category(Category::HighQuality)
//!         .with_gender(Gender::Female)
//!         .with_age(Age::Young)
//!         .with_language("en")
//!         .with_accent("indian")
//!         .with_use_cases(vec!["social_media".to_string()]);
//!
//!     let resp = c.hit(GetSharedVoices::new(query)).await?;
//!
//!     if let Some(shared_voice) = resp.voices().first() {
//!         let public_user_id = shared_voice.public_owner_id();
//!         let voice_id = shared_voice.voice_id();
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
pub use crate::endpoints::voice_generation::Age;
const SHARED_VOICES_PATH: &str = "/v1/shared-voices";
const PAGE_SIZE_QUERY: &str = "page_size";
const CATEGORY_QUERY: &str = "category";
const GENDER_QUERY: &str = "gender";
const AGE_QUERY: &str = "age";
const ACCENT_QUERY: &str = "accent";
const LANGUAGE_QUERY: &str = "language";
const SEARCH_QUERY: &str = "search";
const USE_CASES_QUERY: &str = "use_cases";
const DESCRIPTIVES_QUERY: &str = "descriptives";
const FEATURED_QUERY: &str = "featured";
const RENDERED_APP_ENABLED_QUERY: &str = "rendered_app_enabled";
const OWNER_ID_QUERY: &str = "owner_id";
const SORT_QUERY: &str = "sort";
const PAGE_QUERY: &str = "page";

/// Get shared voices
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::endpoints::voice_library::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let mut query = SharedVoicesQuery::default();
///     query = query
///         .with_page_size(1)
///         .with_category(Category::Professional)
///         .with_gender(Gender::Male)
///         .with_age(Age::MiddleAged)
///         .with_accent("irish")
///         .with_language("en")
///         .with_use_cases(vec!["narrative_story".to_string()])
///         .with_descriptives(vec!["confident".to_string()]);
///     let resp = c.hit(GetSharedVoices::new(query)).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See [ElevenLabs API documentation](https://elevenlabs.io/docs/api-reference/query-library) for more information
#[derive(Clone, Debug)]
pub struct GetSharedVoices(SharedVoicesQuery);

impl GetSharedVoices {
    pub fn new(query: SharedVoicesQuery) -> Self {
        GetSharedVoices(query)
    }
}

impl Endpoint for GetSharedVoices {
    type ResponseBody = SharedVoicesResponse;

    fn method(&self) -> Method {
        Method::GET
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(SHARED_VOICES_PATH);
        url.set_query(Some(&self.0.to_string()));
        url
    }
}

/// Shared voices response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedVoicesResponse {
    voices: Vec<SharedVoice>,
    has_more: bool,
    last_sort_id: Option<String>,
}

impl SharedVoicesResponse {
    pub fn voices(&self) -> &Vec<SharedVoice> {
        &self.voices
    }
    pub fn has_more(&self) -> bool {
        self.has_more
    }
    pub fn last_sort_id(&self) -> Option<&str> {
        self.last_sort_id.as_deref()
    }
}

/// Shared voice
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedVoice {
    public_owner_id: String,
    voice_id: String,
    date_unix: f32,
    name: String,
    accent: String,
    gender: String,
    age: String,
    descriptive: String,
    use_case: String,
    category: String,
    language: String,
    description: String,
    preview_url: String,
    usage_character_count_1y: f32,
    usage_character_count_7d: f32,
    play_api_usage_character_count_1y: f32,
    cloned_by_count: f32,
    rate: f32,
    free_users_allowed: bool,
    live_moderation_enabled: bool,
    featured: bool,
    notice_period: Option<f32>,
    instagram_username: Option<String>,
    twitter_username: Option<String>,
    youtube_username: Option<String>,
    tiktok_username: Option<String>,
}

impl SharedVoice {
    pub fn public_owner_id(&self) -> &str {
        &self.public_owner_id
    }
    pub fn voice_id(&self) -> &str {
        &self.voice_id
    }
    pub fn date_unix(&self) -> f32 {
        self.date_unix
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn accent(&self) -> &str {
        &self.accent
    }
    pub fn language(&self) -> &str {
        &self.language
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn preview_url(&self) -> &str {
        &self.preview_url
    }
    pub fn usage_character_count_1y(&self) -> f32 {
        self.usage_character_count_1y
    }
    pub fn usage_character_count_7d(&self) -> f32 {
        self.usage_character_count_7d
    }
    pub fn play_api_usage_character_count_1y(&self) -> f32 {
        self.play_api_usage_character_count_1y
    }
    pub fn cloned_by_count(&self) -> f32 {
        self.cloned_by_count
    }
    pub fn rate(&self) -> f32 {
        self.rate
    }
    pub fn free_users_allowed(&self) -> bool {
        self.free_users_allowed
    }
    pub fn live_moderation_enabled(&self) -> bool {
        self.live_moderation_enabled
    }
    pub fn featured(&self) -> bool {
        self.featured
    }
    pub fn notice_period(&self) -> Option<f32> {
        self.notice_period
    }
    pub fn instagram_username(&self) -> Option<&str> {
        self.instagram_username.as_deref()
    }
    pub fn twitter_username(&self) -> Option<&str> {
        self.twitter_username.as_deref()
    }
    pub fn youtube_username(&self) -> Option<&str> {
        self.youtube_username.as_deref()
    }
    pub fn tiktok_username(&self) -> Option<&str> {
        self.tiktok_username.as_deref()
    }
}

/// Shared voices query
///
/// See [ElevenLabs API documentation](https://elevenlabs.io/docs/api-reference/query-library) for more information
#[derive(Clone, Debug, Default)]
pub struct SharedVoicesQuery {
    pub page_size: Option<String>,
    pub category: Option<String>,
    pub gender: Option<String>,
    pub age: Option<String>,
    pub accent: Option<String>,
    pub language: Option<String>,
    pub search: Option<String>,
    pub use_cases: Option<String>,
    pub descriptives: Option<String>,
    pub featured: Option<String>,
    pub rendered_app_enabled: Option<String>,
    pub owner_id: Option<String>,
    pub sort: Option<String>,
    pub page: Option<String>,
}

impl SharedVoicesQuery {
    pub fn with_page_size(mut self, page_size: u16) -> Self {
        self.page_size = Some(format!("{}={}", PAGE_SIZE_QUERY, page_size));
        self
    }
    pub fn with_category(mut self, category: Category) -> Self {
        self.category = Some(format!("{}={}", CATEGORY_QUERY, category.as_str()));
        self
    }
    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.gender = Some(format!("{}={}", GENDER_QUERY, gender.as_str()));
        self
    }
    pub fn with_age(mut self, age: Age) -> Self {
        self.age = Some(format!("{}={}", AGE_QUERY, age.as_str()));
        self
    }
    pub fn with_accent(mut self, accent: &str) -> Self {
        self.accent = Some(format!("{}={}", ACCENT_QUERY, accent));
        self
    }
    pub fn with_language(mut self, language: &str) -> Self {
        self.language = Some(format!("{}={}", LANGUAGE_QUERY, language));
        self
    }
    pub fn with_search(mut self, search: &str) -> Self {
        self.search = Some(format!("{}={}", SEARCH_QUERY, search));
        self
    }
    pub fn with_use_cases(mut self, use_cases: Vec<String>) -> Self {
        let use_cases_formatted = use_cases
            .iter()
            .map(|use_case| format!("{}={}", USE_CASES_QUERY, use_case))
            .collect::<Vec<String>>()
            .join("&");
        self.use_cases = Some(use_cases_formatted);
        self
    }
    pub fn with_descriptives(mut self, descriptives: Vec<String>) -> Self {
        let descriptives_formatted = descriptives
            .iter()
            .map(|descriptive| format!("{}={}", DESCRIPTIVES_QUERY, descriptive))
            .collect::<Vec<String>>()
            .join("&");
        self.descriptives = Some(descriptives_formatted);
        self
    }
    pub fn with_featured(mut self, featured: bool) -> Self {
        self.featured = Some(format!("{}={}", FEATURED_QUERY, featured));
        self
    }
    pub fn with_rendered_app_enabled(mut self, rendered_app_enabled: bool) -> Self {
        self.rendered_app_enabled = Some(format!(
            "{}={}",
            RENDERED_APP_ENABLED_QUERY, rendered_app_enabled
        ));
        self
    }
    pub fn with_owner_id(mut self, owner_id: &str) -> Self {
        self.owner_id = Some(format!("{}={}", OWNER_ID_QUERY, owner_id));
        self
    }
    pub fn with_sort(mut self, sort: &str) -> Self {
        self.sort = Some(format!("{}={}", SORT_QUERY, sort));
        self
    }
    pub fn with_page(mut self, page: u16) -> Self {
        self.page = Some(format!("{}={}", PAGE_QUERY, page));
        self
    }

    fn to_string(&self) -> String {
        let mut result = String::new();

        if let Some(value) = self.page_size.as_ref() {
            result.push_str(&value);
        }
        if let Some(value) = self.category.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.gender.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.age.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.accent.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.language.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.search.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.use_cases.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.descriptives.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.featured.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.rendered_app_enabled.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.owner_id.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.sort.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        if let Some(value) = self.page.as_ref() {
            if !result.is_empty() {
                result.push('&');
            }
            result.push_str(&value);
        }
        result
    }
}

#[derive(Clone, Debug)]
pub enum Gender {
    Female,
    Male,
    Neutral,
}

impl Gender {
    pub fn as_str(&self) -> &str {
        match self {
            Gender::Female => "female",
            Gender::Male => "male",
            Gender::Neutral => "neutral",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Category {
    Generated,
    HighQuality,
    Professional,
}

impl Category {
    pub fn as_str(&self) -> &str {
        match self {
            Category::Generated => "generated",
            Category::HighQuality => "high_quality",
            Category::Professional => "professional",
        }
    }
}

/// Add a sharing voice to your collection of voices in VoiceLab.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let public_user_id = "some_public_user_id";
///     let voice_id = "some_voice_id";
///     let name = "new_voice_name";
///     let endpoint = AddSharedVoice::new(public_user_id, voice_id, name);
///     let resp = c.hit(endpoint).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See [ElevenLabs API documentation](https://elevenlabs.io/docs/api-reference/add-shared-voice) for more information
#[derive(Clone, Debug)]
pub struct AddSharedVoice {
    pub params: AddSharedVoiceParams,
    pub body: AddSharedVoiceBody,
}

impl AddSharedVoice {
    pub fn new(public_user_id: &str, voice_id: &str, new_name: &str) -> Self {
        let params = AddSharedVoiceParams::new(public_user_id, voice_id);
        let body = AddSharedVoiceBody::new(new_name);
        AddSharedVoice { params, body }
    }
    /// If you don't care about changing the name of the voice, use `from_shared_voice`
    ///
    /// # Example
    /// ```no_run
    /// use elevenlabs_rs::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let c = ElevenLabsClient::default()?;
    ///     let mut query = SharedVoicesQuery::default();
    ///     query = query
    ///         .with_page_size(1)
    ///         .with_use_cases(vec!["characters_animation".to_string()])
    ///         .with_descriptives(vec!["deep".to_string()]);
    ///     let resp = c.hit(GetSharedVoices::new(query)).await?;
    ///     println!("{:#?}", resp);
    ///     if let Some(shared_voice) = resp.voices().first() {
    ///         let resp = c.hit(AddSharedVoice::from_shared_voice(shared_voice)).await?;
    ///         println!("{:#?}", resp);
    ///     } else {
    ///         println!("no shared voices found with query")
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn from_shared_voice(v: &SharedVoice) -> Self {
        AddSharedVoice::new(v.public_owner_id(), v.voice_id(), v.name())
    }
}

impl Endpoint for AddSharedVoice {
    type ResponseBody = AddSharedVoiceResponse;

    fn method(&self) -> Method {
        Method::POST
    }
    fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Url {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(&format!(
            "{}{}/{}/{}",
            VOICES_PATH, ADD_VOICE_PATH, self.params.public_user_id.0, self.params.voice_id.0
        ));
        url
    }
}

/// Response for adding a shared voice
#[derive(Clone, Debug, Deserialize)]
pub struct AddSharedVoiceResponse {
    voice_id: String,
}

/// Parameters for adding a shared voice
#[derive(Clone, Debug)]
pub struct AddSharedVoiceParams {
    public_user_id: PublicUserID,
    voice_id: VoiceID,
}

impl AddSharedVoiceParams {
    pub fn new(public_user_id: &str, voice_id: &str) -> Self {
        let public_user_id = PublicUserID::from(public_user_id);
        let voice_id = VoiceID::from(voice_id.to_string());
        AddSharedVoiceParams {
            public_user_id,
            voice_id,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PublicUserID(pub(crate) String);

impl From<&str> for PublicUserID {
    fn from(id: &str) -> Self {
        PublicUserID(id.to_string())
    }
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
