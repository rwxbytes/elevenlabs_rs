use std::ops::DerefMut;
use axum::extract::{FromRef, Request, State};
use axum::response::IntoResponse;
use axum::{extract::ws::{WebSocket, WebSocketUpgrade}, response::Response, routing::{get, post}, Form, Router};
use elevenlabs_twilio::{handle_outbound_call_twiml, Gather, GatherDigits, InboundAgent, TelephonyAgent};



#[derive(Clone)]
pub struct AppState {
    agent: InboundAgent,
}

impl FromRef<AppState> for InboundAgent {
    fn from_ref(app_state: &AppState) -> InboundAgent {
        app_state.agent.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ngrok_url = "https://130a-86-18-8-153.ngrok-free.app";

    let inbound_agent = InboundAgent::from_url(ngrok_url)?;

    let app_state = AppState {
        agent: inbound_agent,
    };

    let app = Router::new()
        .route("/dtmf", post(handle_dtmf::<InboundAgent>))
        .route("/gather", post(gather_action::<InboundAgent>))
        .route("/ws", get(ws_handler::<InboundAgent>))
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    println!("Give us a call");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}

async fn ws_handler<S>(state: State<S>, ws: WebSocketUpgrade) -> Response
where
    S: TelephonyAgent + Send + Sync + 'static,
{
    ws.on_upgrade(move |socket| async {
        match elevenlabs_twilio::handle_phone_call(socket, state).await {
            Ok(_) => println!("phone call started"),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    })
}




async fn gather_action<S>(state: State<S>, Form(gather): Form<GatherDigits>) -> impl IntoResponse
where
    S: TelephonyAgent + Send + Sync + 'static,
{
    //println!("{:?}", body);
    //let digits = gather.digits;

    let resp: Response<String> = Response::builder()
        .header("Content-Type", "application/xml")
        .body(STREAM_TWIML.to_string())
        .unwrap();
    resp
}

async fn handle_dtmf<S>(state: State<S>, req: Request) -> impl IntoResponse
where
    S: TelephonyAgent + Send + Sync + 'static,
{
    println!("{:?}", req);
    //DTMF_TWIML

    let resp: Response<String> = Response::builder()
        .header("Content-Type", "application/xml")
        .body(DTMF_TWIML.to_string())
        .unwrap();
    resp
}

const EXAMPLE : &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Response>
    <Gather action="/gather" method="GET">
        <Say>
            Please enter your account number,
            followed by the pound sign
        </Say>
    </Gather>
    <Say>We didn't receive any input. Goodbye!</Say>
</Response>
"#;



const DTMF_TWIML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Response>
    <Gather action="/gather" method="POST">
        <Say>
            For an English Assistant, press 1. For a Spanish Assistant, press 2,
            For a French Assistant, press 3. For a German Assistant, press 4.
        </Say>
    </Gather>
    <Say>We didn't receive any input. Goodbye!</Say>
</Response>
"#;

const STREAM_TWIML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Response>
    <Connect>
        <Stream url="wss://130a-86-18-8-153.ngrok-free.app/ws" track="inbound_track" />
    </Connect>
</Response>
"#;
