//use crate::prelude::*;
use chrono::Utc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::{RecordId, Surreal};
use serde::{Deserialize, Serialize};

pub const AVAILABLE_TABLES: &str = "SELECT * FROM table WHERE capacity >= $party_size \
AND {out: id } NOTINSIDE (SELECT out FROM reservation WHERE time == <datetime>$datetime);";

pub const REVISITING_CUSTOMER: &str =
    "SELECT * FROM ONLY customer WHERE number == $customer_number LIMIT 1";

pub async fn revisiting_customer(
    db: &Surreal<Client>,
    customer_number: &str,
) -> Option<Customer> {
    let mut resp = db
        .query(REVISITING_CUSTOMER)
        .bind(("customer_number", customer_number.to_string()))
        .await
        .expect("is getting customer query");

    let customer: Option<Customer> = resp.take(0).expect("is getting customer");
    customer
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Table {
    pub capacity: u32,
    pub location: String,
    pub id: RecordId,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Customer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub name: String,
    pub email: Option<String>,
    pub number: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Reservation {
    #[serde(rename = "in")]
    pub customer: RecordId,
    #[serde(rename = "out")]
    pub table: RecordId,
    pub party_size: Option<u32>,
    pub time: Option<surrealdb::sql::Datetime>,
}

#[derive(Deserialize)]
pub struct ReservationData {
    pub table_id: i64,
    pub party_size: u32,
    pub datetime: chrono::DateTime<Utc>,
    pub name: String,
}

pub const DB_SETUP: &str = r#"
BEGIN TRANSACTION;

DEFINE TABLE IF NOT EXISTS customer;
DEFINE FIELD IF NOT EXISTS name ON TABLE customer TYPE string;
DEFINE FIELD IF NOT EXISTS number ON TABLE customer TYPE string;

DEFINE TABLE IF NOT EXISTS table;
DEFINE FIELD IF NOT EXISTS capacity ON TABLE table TYPE number;
DEFINE FIELD IF NOT EXISTS location ON TABLE table TYPE string;

DEFINE TABLE IF NOT EXISTS reservation TYPE RELATION FROM customer TO table;
DEFINE FIELD IF NOT EXISTS time ON TABLE reservation TYPE datetime;
DEFINE FIELD IF NOT EXISTS party_size ON TABLE reservation TYPE int;
DEFINE INDEX IF NOT EXISTS unique_reservation_time ON TABLE reservation FIELDS out, time UNIQUE;

LET $_ = CREATE |table:1..15| CONTENT {
    capacity: rand::enum(2, 3, 4, 5, 6, 7, 8),
    location: rand::enum('outdoor', 'indoor', 'bar', 'window'),
};

LET $_ = CREATE |customer:1..15| CONTENT {
    name: string::lowercase(rand::string(5)),
    number: '+44' + <string>rand::int(1000000000, 9999999999)
};

LET $tables = (SELECT VALUE id FROM table);
LET $customers = (SELECT VALUE id FROM customer);

LET $start_timestamp = time::unix(time::now());
LET $end_timestamp = time::unix(time::now() + 10h);

FOR $i IN 0..15 {
    LET $customer = array::at($customers, $i);
    LET $table = array::at($tables, $i);
    LET $reservation_time = time::ceil(rand::time($start_timestamp, $end_timestamp), 30m);

    RELATE $customer -> reservation -> $table
    SET
        time = $reservation_time,
        party_size = rand::int(1, (SELECT capacity FROM ONLY $table).capacity);

};
COMMIT TRANSACTION;
"#;
