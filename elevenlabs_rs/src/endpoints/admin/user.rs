//! The user endpoints
use super::*;

/// Gets extended information about the users subscription
///
/// # Examples
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::user::GetUserSubscriptionInfo;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let resp = c.hit(GetUserSubscriptionInfo).await?;
///     println!("{:#?}", resp);
///     Ok(())
/// }
/// ```
/// See the [Get User Subscription API reference](https://elevenlabs.io/docs/api-reference/user/get-subscription)
#[derive(Clone, Debug)]
pub struct GetUserSubscriptionInfo;

impl ElevenLabsEndpoint for GetUserSubscriptionInfo {
    const PATH: &'static str = "/v1/user/subscription";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetUserSubscriptionInfoResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetUserSubscriptionInfoResponse {
    pub tier: String,
    pub character_count: i64,
    pub character_limit: i64,
    pub can_extend_character_limit: bool,
    pub allowed_to_extend_character_limit: bool,
    pub next_character_count_reset_unix: i64,
    pub voice_limit: i64,
    pub max_voice_add_edits: Option<i64>,
    pub voice_add_edit_counter: Option<i64>,
    pub professional_voice_limit: i64,
    pub can_extend_voice_limit: bool,
    pub can_use_instant_voice_cloning: bool,
    pub can_use_professional_voice_cloning: bool,
    pub currency: Option<Currency>,
    pub status: Option<Status>,
    pub billing_period: Option<BillingPeriod>,
    pub character_refresh_period: Option<BillingPeriod>,
    pub next_invoice: Option<NextInvoice>,
    pub has_open_invoices: Option<bool>,
    pub voice_slots_used: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NextInvoice {
    pub amount_due_cents: i64,
    pub next_payment_attempt_unix: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    Usd,
    Eur,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Trailing,
    Active,
    Incomplete,
    IncompleteExpired,
    Cancelled,
    PastDue,
    Unpaid,
    Free,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPeriod {
    MonthlyPeriod,
    AnnualPeriod,
}

/// Gets information about the user
///
/// # Examples
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::user::GetUserInfo;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let resp = c.hit(GetUserInfo).await?;
///    println!("{:#?}", resp);
///   Ok(())
/// }
/// ```
/// See the [Get User API reference](https://elevenlabs.io/docs/api-reference/user/get)
#[derive(Clone, Debug)]
pub struct GetUserInfo;

impl ElevenLabsEndpoint for GetUserInfo {
    const PATH: &'static str = "/v1/user";

    const METHOD: Method = Method::GET;

    type ResponseBody = GetUserInfoResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct GetUserInfoResponse {
    pub subscription: GetUserSubscriptionInfoResponse,
    pub subscription_extras: Value,
    pub is_new_user: bool,
    pub xi_api_key: String,
    pub can_use_delayed_payment_methods: bool,
    pub is_onboarding_completed: bool,
    pub is_onboarding_checklist_completed: bool,
    pub first_name: Option<String>,
    pub is_api_key_hashed: Option<bool>,
    pub xi_api_key_preview: Option<String>,
    pub referral_link_code: Option<String>,
    pub partnerstack_partner_default_link: Option<String>,
    pub user_id: String,
}
