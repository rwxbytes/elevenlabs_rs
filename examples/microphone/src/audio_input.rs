use crate::prelude::*;

pub struct DefaultMicrophoneManager {
    audio_tx: mpsc::UnboundedSender<Vec<i16>>,
    stop_recording: Arc<AtomicBool>,
}

impl DefaultMicrophoneManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Vec<i16>>) {
        let (audio_tx, audio_rx) = mpsc::unbounded_channel();

        let manager = Self {
            audio_tx,
            stop_recording: Arc::new(AtomicBool::new(false)),
        };

        (manager, audio_rx)
    }

    pub async fn build_input_stream(&self) -> (CpalStream, SampleRate) {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("Failed to get default input device");

        let config = device.default_input_config()
            .expect("Failed to get default input config");

        let sample_rate = config.sample_rate();

        let audio_tx = self.audio_tx.clone();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                // Stereo to mono conversion
                let mono_samples: Vec<i16> = data
                    .chunks_exact(2)
                    .map(|chunk| {
                        let mono = (chunk[0] + chunk[1]) / 2.0;
                        i16::from_sample(mono)
                    })
                    .collect();

                let _ = audio_tx
                    .send(mono_samples)
                    .expect("Failed to send audio samples");
            },
            |err| eprintln!("Input stream error: {}", err),
            None,
        ).expect("Failed to build input stream");

        (stream, sample_rate)
    }

    pub fn stop(&self) {
        self.stop_recording.store(true, Ordering::SeqCst);
    }
}