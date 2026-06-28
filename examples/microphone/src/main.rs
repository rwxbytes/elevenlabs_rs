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
    let mut client = AgentWebSocket::from_env()?;

    // Set up microphone input.
    let (mic, audio_rx) = audio_input::DefaultMicrophoneManager::new();
    let (microphone_stream, input_sample_rate) = mic.build_input_stream().await;

    // Prepare audio processing task to encode audio samples.
    let (mut audio_processor, encoded_audio_rx) = audio_helpers::AudioProcessor::new(audio_rx);

    // Set up speaker output.
    let (decoded_audio_tx, decoded_audio_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut speaker = audio_output::DefaultSpeakersManager::new(decoded_audio_rx);
    let (speaker_stream, output_sample_rate) = speaker.build_output_stream().await;

    // Start the WebSocket before starting CPAL streams. If signed URL creation
    // fails, the audio tasks never start and callbacks cannot panic on closed
    // channels.
    let mut convo = match client.start_conversation(encoded_audio_rx).await {
        Ok(convo) => convo,
        Err(error) => {
            eprintln!("failed to start conversation: {error}");
            eprintln!(
                "check ELEVENLABS_AGENT_ID belongs to the same workspace as ELEVENLABS_API_KEY"
            );
            return Err(Box::new(error) as BoxError);
        }
    };
    tokio::spawn(async move {
        audio_processor.start(input_sample_rate).await;
    });
    microphone_stream
        .play()
        .expect("microphone stream failed to play");
    speaker_stream
        .play()
        .expect("speaker stream failed to play");

    // change the sample rate according to your agent's TTS output format
    let tts_output_format = SampleRate(16000);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Closing conversation...");
                convo.close().await?;
                break;
            }
            msg_result = convo.next() => {
                let Some(msg_result) = msg_result else {
                    break;
                };

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

                        if decoded_audio_tx.send(decoded_bytes).is_err() {
                            eprintln!("speaker output channel closed");
                            break;
                        }
                    }
                    ServerMessage::ConversationInitiationMetadata(_) => {
                        println!("Conversation started");
                    }
                    ServerMessage::Interruption(_) => {
                        println!("Conversation interrupted");
                        if let Ok(mut buffer) = speaker.inner.lock() {
                            buffer.clear();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let report = convo.join().await;
    eprintln!(
        "WebSocket tasks finished: reader={:?}, writer={:?}, audio={:?}",
        report.reader, report.writer, report.audio
    );

    Ok(())
}
