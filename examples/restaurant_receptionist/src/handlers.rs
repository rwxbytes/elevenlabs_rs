use crate::db_types::{
    revisiting_customer, Customer, Reservation, ReservationData, Table, AVAILABLE_TABLES,
};
use crate::{agent, AppState};
use axum::extract::State;
use axum::Json;
use chrono::Utc;
use elevenlabs_twilio::agents::DynamicVar;
use elevenlabs_twilio::{
    AgentOverrideData, ConversationInitiationClientData, OverrideData,
    Personalization, PromptOverrideData,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

pub async fn personalization(
    State(state): State<AppState>,
    Json(data): Personalization,
) -> Json<ConversationInitiationClientData> {
    info!("personalization called");
    let caller_id = data.caller_id.clone();
    *state.caller_id.lock().await = Some(caller_id.clone());

    let mut dyn_vars = HashMap::new();
    let dyn_var = DynamicVar::new_string(Utc::now().to_rfc2822());
    dyn_vars.insert("datetime".to_string(), dyn_var);
    let mut init_client_data =
        ConversationInitiationClientData::default().with_dynamic_variables(dyn_vars);

    if let Some(customer) = revisiting_customer(&state.db, &caller_id).await {
        let dyn_var = DynamicVar::new_string(&customer.name);

        init_client_data
            .dynamic_variables
            .as_mut()
            .unwrap()
            .insert("customer".to_string(), dyn_var);

        let prompt_override_data = PromptOverrideData::default()
            .override_prompt(agent::ATTITUDE_TOWARDS_REVISITING_CUSTOMER);

        let agent_override = AgentOverrideData::default()
            .override_first_message(agent::FIRST_MSG_FOR_REVISITING_CUSTOMER)
            .with_prompt_override_data(prompt_override_data);

        let override_data = OverrideData::default().with_agent_override_data(agent_override);

        init_client_data.with_override_data(override_data);
    };

    Json(init_client_data)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReservationSuccess {
    message: String,
}

pub async fn create_reservation(
    State(state): State<AppState>,
    Json(data): Json<ReservationData>,
) -> Json<ReservationSuccess> {
    let caller_id = state.caller_id.lock().await.clone();

    let customer = match state.revisiting_customer {
        None => {
            let resp = state
                .db
                .create("customer")
                .content(Customer {
                    name: data.name.clone(),
                    email: None,
                    number: caller_id.unwrap(),
                    id: None,
                })
                .await
                .expect("Failed to create customer");

            resp.unwrap()
        }
        Some(customer) => customer,
    };

    let _: Vec<Reservation> = state
        .db
        .insert("reservation")
        .relation(Reservation {
            customer: customer.id.clone().unwrap(),
            table: ("table", data.table_id).into(),
            party_size: Some(data.party_size),
            time: Some(data.datetime.into()),
        })
        .await
        .expect("Failed to create reservation");

    Json(ReservationSuccess {
        message: "Reservation created".to_string(),
    })
}

#[derive(Deserialize)]
pub struct CheckAvailabilityData {
    datetime: chrono::DateTime<Utc>,
    party_size: u32,
}

pub async fn list_available_tables(
    State(state): State<AppState>,
    Json(data): Json<CheckAvailabilityData>,
) -> Json<Vec<Table>> {
    let mut result = state
        .db
        .query(AVAILABLE_TABLES)
        .bind(("datetime", data.datetime))
        .bind(("party_size", data.party_size))
        .await
        .expect("failed to query for available tables");

    let free_tables: Vec<Table> = result.take(0).unwrap();

    Json(free_tables)
}
