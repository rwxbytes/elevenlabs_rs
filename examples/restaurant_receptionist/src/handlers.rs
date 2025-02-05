use crate::db_types::{Customer, Reservation, ReservationData, Table, AVAILABLE_TABLES};
use crate::prelude::*;
use crate::{twilio, AppState};
use axum::Form;
use url::Url;

pub async fn twiml(State(state): State<AppState>, Form(caller): Form<twilio::Caller>) -> String {
    let url = Url::parse(&state.ngrok_url).expect("invalid ngrok URL");
    let domain = url.domain().unwrap();
    *state.caller.lock().await = Some(caller);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Connect>
            <Stream url="wss://{}/ws" track="inbound_track" />
        </Connect>
    </Response>
    "#,
        domain
    )
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reservation200 {
    message: String,
}

pub async fn create_reservation(
    State(state): State<AppState>,
    Json(data): Json<ReservationData>,
) -> Json<Reservation200> {
    let caller = state.caller.lock().await.clone();

    let customer = match state.revisiting_customer.lock().await.clone() {
        None => {
            let resp = state
                .db
                .create("customer")
                .content(Customer {
                    name: data.name.clone(),
                    email: None,
                    number: caller.unwrap().caller,
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

    Json(Reservation200 {
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
