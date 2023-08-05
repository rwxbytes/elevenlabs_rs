use crate::{api::ClientBuilder, prelude::*};
use http_body_util::Empty;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

const BASE_PATH: &str = "/user";
const SUBSCRIPTION_PATH: &str = "/subscription";

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{Any, TypeId};

    #[tokio::test]
    #[ignore]
    async fn get_user_subscription_is_returning_a_subscription_checked_with_typeid() {
        let want = TypeId::of::<Subscription>();
        let got = get_user_subscription().await.unwrap();
        assert_eq!(want, got.type_id());
    }
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
}

pub async fn get_user_subscription() -> Result<Subscription> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(format!("{}{}", BASE_PATH, SUBSCRIPTION_PATH))?
        .method(GET)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let data = c.send_request(Empty::<Bytes>::new()).await?;
    let user_subscription = serde_json::from_slice::<Subscription>(&data)?;
    Ok(user_subscription)
}
