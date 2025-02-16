use crate::audio_helpers::{resample_hz, UpmixMonoToStereo};
use crate::prelude::{SampleRate, StreamTrait};
use elevenlabs_convai::client::AgentWebSocket;
use elevenlabs_convai::messages::server_messages::ServerMessage;
use futures_util::StreamExt;

mod audio_helpers;
mod audio_input;
mod audio_output;
mod prelude;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let mut client = AgentWebSocket::new("apikey", "agent_id");
    //let mut client = ElevenLabsAgentClient::from_env()?;

    // Set up microphone and play input audio stream
    let (mic, audio_rx) = audio_input::DefaultMicrophoneManager::new();
    let (microphone_stream, input_sample_rate) = mic.build_input_stream().await;

    microphone_stream
        .play()
        .expect("microphone stream failed to play");

    // Start audio processing task to encode audio samples
    let (mut audio_processor, encoded_audio_tx) = audio_helpers::AudioProcessor::new(audio_rx);
    tokio::spawn(async move {
        audio_processor.start(input_sample_rate).await;
    });

    // Set up speaker and play output audio stream
    let (decoded_audio_tx, decoded_audio_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut speaker = audio_output::DefaultSpeakersManager::new(decoded_audio_rx);
    let (speaker_stream, output_sample_rate) = speaker.build_output_stream().await;
    speaker_stream
        .play()
        .expect("speaker stream failed to play");

    // Have a chinwag
    let mut convo = client.start_session(encoded_audio_tx).await?;

    // change the sample rate according to your agent's TTS output format
    let tts_output_format = SampleRate(16000);

    while let Some(msg_result) = convo.next().await {
        let server_msg = match msg_result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        };

        match server_msg {
            ServerMessage::Audio(audio) => {
                let audio_b64 = audio.audio_event.audio_base_64;
                let mut decoded_bytes = audio_helpers::decode_base64_pcm(&audio_b64);
                decoded_bytes = decoded_bytes.upmix_mono_to_stereo();
                decoded_bytes = resample_hz(&decoded_bytes, tts_output_format, output_sample_rate);

                decoded_audio_tx
                    .send(decoded_bytes)
                    .expect("Failed to send audio samples");
            }
            ServerMessage::ConversationInitiationMetadata(_) => {
                println!("Conversation started");
            }
            ServerMessage::Interruption(_) => {
                println!("Conversation interrupted");
                speaker.inner.lock().unwrap().clear();
            }
            _ => {}
        }
    }

    Ok(())
}
