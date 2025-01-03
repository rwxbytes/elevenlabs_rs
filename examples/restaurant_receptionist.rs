use axum::body::Body;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
    routing::{get, post},
    Router,
};
use elevenlabs_rs::conversational_ai::client::ElevenLabsConversationalClient;
use elevenlabs_rs::conversational_ai::server_messages::ServerMessage;
use futures_util::{SinkExt, StreamExt};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::{engine::local::Mem, RecordId, Surreal};
use surrealdb::sql::Thing;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    db: Surreal<Db>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Table {
    capacity: u32,
    location: String,
    bookings: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Customer {
    name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Reservation {
    id: RecordId,
    guests: u32,
    time: String,
    r#in: Thing,
    out: Thing,
}

#[derive(Deserialize, Debug)]
struct Record {
    #[allow(dead_code)]
    id: RecordId,
}

//const DUMMY_TABLE_QUERY: &str = "CREATE |table:10| Content {
//    capacity: rand::enum(2, 3, 4, 5, 6, 8),
//    location: rand::enum('outdoor', 'indoor', 'window', 'bar'),
//    is_available: rand::bool(),
//    notes: Null
//};";


const DUMMY_DB_SETUP: &str = "BEGIN TRANSACTION;
DEFINE TABLE customer;
DEFINE TABLE table;

DEFINE TABLE reservation
TYPE RELATION FROM customer TO table;

DEFINE FIELD capacity ON TABLE table TYPE number;
DEFINE FIELD location ON TABLE table TYPE string;
DEFINE FIELD bookings ON TABLE table TYPE set<datetime>;

DEFINE FIELD time ON TABLE reservation TYPE datetime;
DEFINE FIELD guests ON TABLE reservation TYPE int;

-- Create a set number of tables
LET $_ = CREATE |table:1..10| CONTENT {
    capacity: rand::enum(2, 3, 4, 5, 6, 7, 8),
    location: rand::enum('outdoor', 'indoor', 'bar', 'window'),
    bookings: []
};

-- Create a set number of customers
LET $_ = CREATE |customer:1..50| CONTENT {
    name: 'customer_' + string::lowercase(rand::string(5))
};

LET $tables = (SELECT VALUE id FROM table);
LET $customers = (SELECT VALUE id FROM customer);

-- Define a range for random reservation times (Unix timestamps)
LET $start_timestamp = time::unix(time::now());
LET $end_timestamp = time::unix(time::now() + 7d);


-- Create 50 reservations with random times, customers, and tables
FOR $_ IN 1..1000 {
    LET $random_customer = array::at($customers, rand::int(0, 20));
    LET $random_table = array::at($tables, rand::int(0, 9));
    LET $reservation_time = time::ceil(rand::time($start_timestamp, $end_timestamp), 30m);


    RELATE $random_customer -> reservation -> $random_table
    SET
        time = $reservation_time,
        guests = rand::int(1, (SELECT capacity FROM ONLY $random_table).capacity - 1); -- Guests up to table capacity minus the customer

    UPDATE $random_table SET bookings += $reservation_time;

};

COMMIT TRANSACTION;
";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create SurrealDB");

    db.use_ns("example")
        .use_db("example")
        .await
        .expect("Failed to use namespace and database");

    let _result = db
        .query(DUMMY_DB_SETUP)
        .await
        .expect("Failed to create dummy tables");

    let mut result = db
        .query("SELECT * FROM type::table($table) WHERE guests = 5")
        .bind(("table", "reservation"))
        .await
        .expect("Failed to query table");

    let reservation: Vec<Reservation> = result.take(0).expect("Failed to take reservations");
    let first_reservation = reservation.first().expect("Failed to get first reservation");
    dbg!(first_reservation);

    let app_state = AppState { db };

    let app = Router::new()
        .route("/call/incoming", post(twiml))
        .route("/call/connection", get(handler))
        //.route("/availability", get(check_availability))
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give us a call");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
async fn check_availability(State(state): State<AppState>) -> Response {
    let mut resp = state
        .db
        .query("SELECT * FROM type::table($table) WHERE is_available = true")
        .bind(("table", "table"))
        .await
        .expect("Failed to query table");

    //let table: Vec<Table> = resp.take(0).expect("Failed to take tables");

    let builder = Response::builder();
    let response = builder
        .header("Content-Type", "application/json")
        .body(())
        //.body(Body::from(serde_json::to_string(&table).unwrap()))
        .unwrap();
    response
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut ws_stream: WebSocket) {
    let mut client =
        ElevenLabsConversationalClient::from_env().expect("Failed to create ConvAIClient");
    let client = Arc::new(Mutex::new(client));
    let client_two = Arc::clone(&client);

    // Skip connected message
    ws_stream.next().await;

    // Get stream sid
    let stream_sid = if let Some(msg_result) = ws_stream.next().await {
        let msg = msg_result.unwrap();
        let msg_json = msg.to_text().unwrap();
        let start_msg = serde_json::from_str::<StartMessage>(msg_json).unwrap();
        start_msg.stream_sid
    } else {
        panic!("no stream sid")
    };

    let (twilio_payload_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let twilio_encoded_audio_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let (mut twilio_sink, mut twilio_stream) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(msg_result) = twilio_stream.next().await {
            let msg = msg_result.unwrap();
            match msg {
                Message::Close(_) => {
                    break;
                }
                Message::Text(txt) => {
                    let twilio_msg = TwilioMessage::try_from(txt.as_str()).unwrap();
                    match twilio_msg {
                        TwilioMessage::Media(media_msg) => {
                            let payload = media_msg.media.payload().to_string();
                            twilio_payload_tx.send(payload).unwrap()
                        }
                        TwilioMessage::Mark(mark_msg) => {
                            println!("Mark: {:?}", mark_msg)
                        }
                        TwilioMessage::Stop(stop_msg) => {
                            println!("Stop: {:?}", stop_msg);
                            client_two.lock().await.stop_conversation().await.unwrap();
                            break;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        let mut convai_stream = client
            .lock()
            .await
            .start_conversation(twilio_encoded_audio_stream)
            .await
            .unwrap();

        while let Some(resp_result) = convai_stream.next().await {
            let convai_msg = resp_result.unwrap(); // TODO: errs after max duration
            match convai_msg {
                ServerMessage::Audio(audio) => {
                    let payload = audio.event().base_64();
                    let media_msg = MediaMessage::new(&stream_sid, payload);
                    let json = serde_json::to_string(&media_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();

                    let mark_msg = MarkMessage {
                        event: "mark".to_string(),
                        stream_sid: stream_sid.to_string(),
                        sequence_number: None,
                        mark: Mark {
                            name: "elabs_audio".to_string(),
                        },
                    };
                    let json = serde_json::to_string(&mark_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();
                }
                ServerMessage::Interruption(_) => {
                    let clear_msg = ClearMessage::new(&stream_sid);
                    let json = serde_json::to_string(&clear_msg).unwrap();
                    twilio_sink.send(Message::Text(json)).await.unwrap();
                    println!("Sent clear message")
                }
                _ => {}
            }
        }
    });
}

// TODO: add your ngrok domain
async fn twiml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Connect>
            <Stream url="wss://a66a-86-18-8-153.ngrok-free.app/call/connection" track="inbound_track" />
        </Connect>
    </Response>
    "#
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TwilioMessage {
    Connected(ConnectedMessage),
    Start(StartMessage),
    Media(MediaMessage),
    Mark(MarkMessage),
    Stop(StopMessage),
    Dtmf(DtmfMessage),
}

impl TryFrom<&str> for TwilioMessage {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let twilio_message: TwilioMessage = serde_json::from_str(value)?;
        Ok(twilio_message)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedMessage {
    event: String,
    protocol: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartMessage {
    event: String,
    sequence_number: String,
    start: StartMetadata,
    stream_sid: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopMessage {
    event: String,
    stream_sid: String,
    sequence_number: String,
    stop: Stop,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    account_sid: String,
    call_sid: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearMessage {
    event: String,
    stream_sid: String,
}

impl ClearMessage {
    fn new(sid: &str) -> Self {
        ClearMessage {
            event: "clear".to_string(),
            stream_sid: sid.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarkMessage {
    event: String,
    stream_sid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sequence_number: Option<String>,
    mark: Mark,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mark {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartMetadata {
    stream_sid: String,
    account_sid: String,
    call_sid: String,
    tracks: Vec<Track>,
    custom_parameters: serde_json::Value,
    media_format: MediaFormat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaFormat {
    encoding: String,
    sample_rate: u32,
    channels: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Track {
    #[serde(rename = "inbound")]
    Inbound,
    #[serde(rename = "outbound")]
    Outbound,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaMessage {
    event: String,
    stream_sid: String,
    media: Media,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    payload: String,
}

impl Media {
    fn payload(&self) -> &str {
        self.payload.as_str()
    }
}

impl MediaMessage {
    pub fn new(stream_sid: &str, payload: &str) -> Self {
        MediaMessage {
            event: "media".to_string(),
            stream_sid: stream_sid.to_string(),
            media: Media {
                payload: payload.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DtmfMessage {
    event: String,
    stream_sid: String,
    sequence_number: u32,
    dtmf: Dtmf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dtmf {
    digit: String,
    track: Track,
}
