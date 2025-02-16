pub use std::sync::{Arc, Mutex};
pub use tokio::sync::mpsc;
pub use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub use cpal::{Sample, Stream as CpalStream, SampleRate};
pub struct DefaultSpeakersManager {
    audio_rx: Option<mpsc::UnboundedReceiver<Vec<i16>>>,
    pub inner: Arc<Mutex<Vec<i16>>>,
}

impl DefaultSpeakersManager {
    pub fn new(audio_rx: mpsc::UnboundedReceiver<Vec<i16>>) -> Self {
        Self {
            audio_rx: Some(audio_rx),
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn build_output_stream(&mut self) -> (CpalStream, SampleRate)  {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No output device available");

        let config = device.default_output_config()
            .expect("Failed to get default output config");

        let sample_rate = config.sample_rate();

        let mut audio_rx = self.audio_rx.take().unwrap();

        let buffer = Arc::clone(&self.inner);
        let buffer_for_processing = Arc::clone(&self.inner);

        tokio::spawn(async move {
            while let Some(samples) = audio_rx.recv().await {
                let mut buffer = buffer.lock().unwrap();
                buffer.extend(samples);
            }
        });

        let stream = device.build_output_stream(
            &config.into(),
            move |output: &mut [i16], _| {
                let mut buffer = buffer_for_processing.lock().unwrap();
                if buffer.len() >= output.len() {
                    let chunk: Vec<i16> = buffer.drain(..output.len()).collect();
                    output.copy_from_slice(&chunk);
                } else {
                    output.fill(0);
                }
            },
            |err| eprintln!("Output stream error: {}", err),
            None,
        ).expect("Failed to build output stream");

        (stream, sample_rate)
    }
}