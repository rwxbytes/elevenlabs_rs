use crate::{api::ClientBuilder, prelude::*};
use http_body_util::Empty;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

const ACCEPT_HEADER: &str = "ACCEPT";
const ACCEPT_VALUE: &str = "application/json";
const GET: &str = "GET";
const BASE_PATH: &str = "/user";
const SUBSCRIPTION_PATH: &str = "/subscription";

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    subscription: Subscription,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    tier: String,
    character_count: i64,
    character_limit: i64,
    can_extend_character_limit: bool,
    allowed_to_extend_character_limit: bool,
    next_character_count_reset_unix: i64,
    voice_limit: i64,
    professional_voice_limit: i64,
    can_extend_voice_limit: bool,
    can_use_instant_voice_cloning: bool,
    can_use_professional_voice_cloning: bool,
    currency: String,
    status: String,
    //next_invoice: Invoice,
}

//#[derive(Debug, Serialize, Deserialize)]
//struct Invoice {
//    amount_due_cents: i64,
//    next_payment_attempt_unix: i64,
//}

pub async fn get_user_subscription() -> Result<Subscription> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(format!("{}{}", BASE_PATH, SUBSCRIPTION_PATH))?
        .method(GET)?
        .header(ACCEPT_HEADER, ACCEPT_VALUE)?
        .build()?;
    let data = c.send_request(Empty::<Bytes>::new()).await?;
    let user_subscription = serde_json::from_slice::<Subscription>(&data)?;
    Ok(user_subscription)
}

//pub async fn get_user() -> Result<User> {
//    let cb = ClientBuilder::new()?;
//    let c = cb
//        .path(PATH)?
//        .method(GET)?
//        .header(ACCEPT_HEADER, ACCEPT_VALUE)?
//        .build()?;
//    let data = c.send_request(Empty::<Bytes>::new()).await?;
//    let user = serde_json::from_slice::<User>(&data)?;
//    Ok(user)
//}
