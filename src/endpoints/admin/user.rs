#![allow(dead_code)]
//! The user endpoints
use super::*;

const USER_PATH: &str = "v1/user";
const SUBSCRIPTION_PATH: &str = "v1/user/subscription";

/// Gets extended information about the users subscription
/// # Examples
/// ```no_run
///use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let resp = c.hit(GetUserSubscriptionInfo).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetUserSubscriptionInfo;

impl Endpoint for GetUserSubscriptionInfo {
    type ResponseBody = Subscription;

    const METHOD: Method = Method::GET;
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Result<Url> {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(SUBSCRIPTION_PATH);
        Ok(url)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Subscription {
    tier: String,
    character_count: i64,
    character_limit: i64,
    can_extend_character_limit: bool,
    allowed_to_extend_character_limit: bool,
    next_character_count_reset_unix: i64,
    voice_limit: i64,
    max_voice_add_edits: i64,
    voice_add_edit_counter: i64,
    professional_voice_limit: i64,
    can_extend_voice_limit: bool,
    can_use_instant_voice_cloning: bool,
    can_use_professional_voice_cloning: bool,
    currency: String,
    status: String,
    billing_period: String,
    character_refresh_period: String,
    next_invoice: Option<NextInvoice>,
    has_open_invoices: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NextInvoice {
    amount_due_cents: i64,
    next_payment_attempt_unix: i64,
}

/// Gets information about the user
/// # Examples
/// ```no_run
/// use elevenlabs_rs::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let resp = c.hit(GetUserInfo).await?;
///    println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct GetUserInfo;

impl Endpoint for GetUserInfo {
    type ResponseBody = UserInfo;

    const METHOD: Method = Method::GET;
    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
    fn url(&self) -> Result<Url> {
        let mut url = BASE_URL.parse::<Url>().unwrap();
        url.set_path(USER_PATH);
        Ok(url)
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct UserInfo {
    subscription: Subscription,
    is_new_user: bool,
    xi_api_key: String,
    can_use_delayed_payment_methods: bool,
    is_onboarding_completed: bool,
    is_onboarding_checklist_completed: bool,
    first_name: Option<String>,
}
