use crate::db_types::{Customer, DB_SETUP};
use axum::routing::post;
use axum::Router;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::Mutex;

mod agent;
mod db_types;
mod handlers;

#[derive(Clone)]
pub struct AppState {
    revisiting_customer: Option<Customer>,
    db: Surreal<Client>,
    ngrok_url: String,
    caller_id: Arc<Mutex<Option<String>>>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Ws>("localhost:8000").await?;

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("ns").use_db("db").await?;

    let _resp = db.query(DB_SETUP).await?;

    let ngrok_url = "some_ngrok_url";
    let phone_number_id = "some_phone_number_id";

    // Creates an agent, assigns it the phone number, and sets up the webhook
    let _agent = agent::agent_setup(ngrok_url, phone_number_id).await?;

    let app_state = AppState {
        revisiting_customer: None,
        ngrok_url: ngrok_url.to_string(),
        db,
        caller_id: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/inbound-call", post(handlers::personalization))
        .route("/tables", post(handlers::list_available_tables))
        .route("/reservation", post(handlers::create_reservation))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give Rustalicious a call");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}