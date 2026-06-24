//! Realtime speech-to-text from the system microphone.
//!
//! Captures audio from the default input device, resamples it to 16 kHz mono
//! PCM, streams it to the ElevenLabs realtime Scribe WebSocket, and prints
//! partial and committed transcripts as they arrive.
//!
//! Run with your API key in the environment:
//!
//! ```bash
//! ELEVENLABS_API_KEY=... cargo run -p realtime_stt
//! ```
//!
//! Speak into your microphone; press Ctrl-C to stop.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SampleRate};
use dasp::{signal, Signal};
use elevenlabs_rs::endpoints::genai::speech_to_text::ws::{
    RealtimeCommitStrategy, RealtimeSpeechToText, RealtimeSpeechToTextInput,
    RealtimeSpeechToTextQuery, RealtimeSpeechToTextResponse,
};
use elevenlabs_rs::{ElevenLabsClient, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Sample rate the realtime endpoint expects when using `pcm_16000`.
const TARGET_HZ: u32 = 16_000;
const DEFAULT_MODEL_ID: &str = "scribe_v2_realtime";

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let client = ElevenLabsClient::from_env()?;

    // The cpal callback (a plain, non-async closure) forwards captured mono
    // samples here; the resampler task downstream turns them into 16 kHz chunks.
    let (raw_tx, raw_rx) = mpsc::unbounded_channel::<Vec<i16>>();

    // Keep the cpal stream alive for the lifetime of the program. Dropping it
    // stops capture, so it must outlive the transcription loop below.
    let (_input_stream, source_hz) = build_input_stream(raw_tx)?;
    _input_stream.play()?;
    println!(
        "Capturing at {} Hz, streaming as {} Hz mono PCM.",
        source_hz.0, TARGET_HZ
    );

    // Resample captured audio and wrap each chunk as a realtime input message.
    // This channel becomes the outbound WebSocket stream.
    let (input_tx, input_rx) = mpsc::unbounded_channel::<RealtimeSpeechToTextInput>();
    tokio::spawn(resample_task(raw_rx, input_tx, source_hz));
    let input_stream = UnboundedReceiverStream::new(input_rx);

    // Let server-side voice activity detection decide where transcripts commit,
    // so we don't have to flag commits manually on the audio chunks.
    let query = RealtimeSpeechToTextQuery::default()
        .with_audio_format(format!("pcm_{TARGET_HZ}"))
        .with_commit_strategy(RealtimeCommitStrategy::Vad);

    let model_id =
        std::env::var("ELEVENLABS_STT_MODEL").unwrap_or_else(|_| DEFAULT_MODEL_ID.to_string());
    let endpoint = RealtimeSpeechToText::new(model_id, input_stream).with_query(query);

    let mut session = client.connect_realtime_speech_to_text(endpoint).await?;
    println!("Connected. Speak into your microphone (Ctrl-C to quit).\n");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nClosing session...");
                session.close().await?;
                break;
            }
            message = session.next() => {
                let Some(message) = message else { break };
                handle_response(message?);
            }
        }
    }

    let report = session.join().await;
    eprintln!(
        "WebSocket tasks finished: reader={:?}, writer={:?}",
        report.reader, report.writer
    );

    Ok(())
}

/// Print transcripts and surface any server-side errors.
fn handle_response(response: RealtimeSpeechToTextResponse) {
    use RealtimeSpeechToTextResponse::*;

    match response {
        SessionStarted(started) => {
            println!("[session started: {}]", started.session_id);
        }
        PartialTranscript(transcript) => {
            // Partials are overwritten as more audio arrives; redraw the line.
            print!("\r… {}", transcript.text);
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
        CommittedTranscript(transcript) => {
            println!("\r✓ {}", transcript.text);
        }
        CommittedTranscriptWithTimestamps(transcript) => {
            println!("\r✓ {}", transcript.text);
        }
        other if other.is_error() => {
            if let Some(error) = other.error() {
                let detail = error.message.as_deref().unwrap_or("(no message)");
                eprintln!("\n[{}] {}", other.message_type(), detail);
            }
        }
        other => {
            eprintln!("\n[{}]", other.message_type());
        }
    }
}

/// Drain raw mono samples, resample each batch to 16 kHz, and forward the bytes.
async fn resample_task(
    mut raw_rx: mpsc::UnboundedReceiver<Vec<i16>>,
    input_tx: mpsc::UnboundedSender<RealtimeSpeechToTextInput>,
    source_hz: SampleRate,
) {
    // Buffer ~100 ms of source audio before resampling so each WebSocket frame
    // carries a meaningful slice of speech.
    let batch_samples = (source_hz.0 / 10).max(1) as usize;
    let mut buffer: Vec<i16> = Vec::with_capacity(batch_samples);

    while let Some(samples) = raw_rx.recv().await {
        buffer.extend(samples);
        if buffer.len() < batch_samples {
            continue;
        }

        let chunk = std::mem::take(&mut buffer);
        let resampled = resample_to_16khz(&chunk, source_hz);
        let bytes: &[u8] = bytemuck::cast_slice(&resampled);

        // `audio` base64-encodes the bytes for the JSON message internally.
        if input_tx
            .send(RealtimeSpeechToTextInput::audio(bytes))
            .is_err()
        {
            break; // Session closed; stop resampling.
        }
    }
}

/// Linearly resample 16-bit mono PCM from `source_hz` to 16 kHz.
fn resample_to_16khz(data: &[i16], source_hz: SampleRate) -> Vec<i16> {
    if source_hz.0 == TARGET_HZ {
        return data.to_vec();
    }
    let mut source = signal::from_iter(data.iter().copied());
    let interp = dasp::interpolate::linear::Linear::new(source.next(), source.next());
    source
        .from_hz_to_hz(interp, source_hz.0 as f64, TARGET_HZ as f64)
        .until_exhausted()
        .collect()
}

/// Open the default input device and forward mono `i16` samples over `raw_tx`.
fn build_input_stream(
    raw_tx: mpsc::UnboundedSender<Vec<i16>>,
) -> Result<(cpal::Stream, SampleRate), BoxError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("no default input device available")?;
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate();
    let channels = config.channels() as usize;
    let err_fn = |err| eprintln!("input stream error: {err}");

    // Average interleaved frames down to a single mono channel.
    let downmix = move |frame: &[i16]| -> i16 {
        let sum: i32 = frame.iter().map(|&s| s as i32).sum();
        (sum / channels as i32) as i16
    };

    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let stream_config = config.into();
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    let mono: Vec<i16> = data
                        .chunks_exact(channels)
                        .map(|frame| {
                            let sum: f32 = frame.iter().copied().sum();
                            i16::from_sample(sum / channels as f32)
                        })
                        .collect();
                    let _ = raw_tx.send(mono);
                },
                err_fn,
                None,
            )?
        }
        SampleFormat::I16 => {
            let stream_config = config.into();
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let mono: Vec<i16> = data.chunks_exact(channels).map(downmix).collect();
                    let _ = raw_tx.send(mono);
                },
                err_fn,
                None,
            )?
        }
        SampleFormat::U16 => {
            let stream_config = config.into();
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _| {
                    let mono: Vec<i16> = data
                        .chunks_exact(channels)
                        .map(|frame| {
                            let sum: i32 = frame.iter().map(|&s| i16::from_sample(s) as i32).sum();
                            (sum / channels as i32) as i16
                        })
                        .collect();
                    let _ = raw_tx.send(mono);
                },
                err_fn,
                None,
            )?
        }
        other => return Err(format!("unsupported sample format: {other:?}").into()),
    };

    Ok((stream, sample_rate))
}
