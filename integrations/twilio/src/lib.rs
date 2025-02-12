use axum::extract::ws::{Message, WebSocket};
use elevenlabs_convai::{client::ElevenLabsAgentClient, messages::server_messages::ServerMessage};
use futures_util::{SinkExt, StreamExt};
pub use rusty_twilio::{
    endpoints::{
        accounts::*,
        applications::*,
        voice::{call::*, stream::*},
    },
    TwilioClient,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(async_fn_in_trait)]
trait PhoneCall {
    async fn handle_call(&self, socket: WebSocket) -> Result<(), &'static str>;

}



#[derive(Clone, Debug)]
pub struct ElevenLabsOutboundCallAgent {
    pub elevenlabs_client: Arc<Mutex<ElevenLabsAgentClient>>,
    pub twilio_client: TwilioClient,
    pub to_number: String,
    pub from_number: String,
    //pub create_call_body: CreateCallBody,
    pub create_call_endpoint: CreateCall,
}

impl ElevenLabsOutboundCallAgent {
    pub fn from_env() -> Result<Self, &'static str> {
        let elevenlabs_client = ElevenLabsAgentClient::from_env().unwrap();
        let twilio_client = TwilioClient::from_env().unwrap();
        let account_sid = twilio_client.account_sid();
        let to_number = std::env::var("TO_NUMBER").expect("TO_NUMBER must be set");
        let from_number = std::env::var("FROM_NUMBER").expect("FROM_NUMBER must be set");
        let body = CreateCallBody::new(
            to_number.clone(),
            from_number.clone(),
            TwilmlSrc::url("http://demo.twilio.com/docs/voice.xml".to_string()),
        );
        Ok(ElevenLabsOutboundCallAgent {
            elevenlabs_client: Arc::new(Mutex::new(elevenlabs_client)),
            twilio_client,
            to_number,
            from_number,
            create_call_endpoint: CreateCall::new(account_sid, body),
            //create_call_body: CreateCallBody::default(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct ElevenLabsTelephonyAgent {
    pub elevenlabs_client: Arc<Mutex<ElevenLabsAgentClient>>,
    pub twilio_client: TwilioClient,
    pub msg_tx: Option<tokio::sync::mpsc::UnboundedSender<ServerMessage>>,
}

impl ElevenLabsTelephonyAgent {
    pub fn from_env() -> Result<Self, &'static str> {
        let eleven_client =
            ElevenLabsAgentClient::from_env().expect("Failed to create eleven client");
        let twilio_client = TwilioClient::from_env().expect("Failed to create twilio client");
        Ok(ElevenLabsTelephonyAgent {
            elevenlabs_client: Arc::new(Mutex::new(eleven_client)),
            twilio_client,
            msg_tx: None,
        })
    }

    pub async fn handle_call(&self, mut socket: WebSocket) -> Result<(), &'static str> {
        socket.next().await;

        // Get stream_sid
        let stream_sid = match socket.next().await {
            Some(Ok(Message::Text(msg))) => {
                let start_msg = serde_json::from_str::<StartMessage>(&msg)
                    .expect("Failed to parse start message");
                start_msg.stream_sid
            }
            _ => panic!("Expected start message with stream sid"),
        };

        let (twilio_payload_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let twilio_payload_rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

        let (mut twilio_sink, mut twilio_stream) = socket.split();

        let client_for_stop = Arc::clone(&self.elevenlabs_client);
        tokio::spawn(async move {
            while let Some(result) = twilio_stream.next().await {
                match result {
                    Ok(Message::Text(txt)) => {
                        match TwilioMessage::try_from(txt.as_str()) {
                            Ok(TwilioMessage::Media(media_msg)) => {
                                let payload = media_msg.media.payload;
                                if twilio_payload_tx.send(payload).is_err() {
                                    eprintln!("failed to send Twilio payload to conversation");
                                    break;
                                }
                            }
                            Ok(TwilioMessage::Stop(_)) => {
                                if let Err(e) =
                                    client_for_stop.lock().await.stop_conversation().await
                                {
                                    eprintln!("failed to stop conversation: {}", e);
                                    break;
                                }
                                break;
                            }
                            Ok(TwilioMessage::Mark(mark_msg)) => {}
                            Ok(TwilioMessage::Dtmf(dtmf)) => {}
                            Err(e) => {
                                eprintln!("failed to parse twilio message: {}", e);
                            }
                            // `Connected` and `Start` messages are only sent once at the beginning of the call
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });

        let client_for_convo = Arc::clone(&self.elevenlabs_client);
        tokio::spawn(async move {
            let mut convai_stream = client_for_convo
                .lock()
                .await
                .start_conversation(twilio_payload_rx)
                .await
                .expect("Failed to start conversation");


            while let Some(msg_result) = convai_stream.next().await {
                let server_msg = msg_result.map_err(|_| "Failed to get message")?;
                match server_msg {
                    ServerMessage::Audio(audio) => {
                        let payload = audio.audio_event.audio_base_64;
                        let media_msg = MediaMessage::new(&stream_sid, &payload);
                        let json = serde_json::to_string(&media_msg)
                            .expect("Failed to serialize media message");
                        twilio_sink
                            .send(Message::Text(json.into()))
                            .await
                            .expect("Failed to send message");
                    }
                    ServerMessage::Interruption(_) => {
                        let clear_msg = ClearMessage::new(&stream_sid);
                        let json = serde_json::to_string(&clear_msg)
                            .expect("Failed to serialize clear message");
                        twilio_sink
                            .send(Message::Text(json.into()))
                            .await
                            .expect("Failed to send message");
                    }
                    x => if self.msg_tx.is_some() {
                        self.msg_tx
                            .as_ref()
                            .unwrap()
                            .send(x)
                            .expect("Failed to send message");
                    }
                }
            }
            Ok::<(), &'static str>(())
        });
        Ok(())
    }
}
