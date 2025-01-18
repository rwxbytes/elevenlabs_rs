use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use cpal::SampleRate;
use dasp::interpolate::linear::Linear;
use dasp::{self, signal, Signal};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub trait UpmixMonoToStereo {
    fn upmix_mono_to_stereo(&self) -> Vec<i16>;
}

impl UpmixMonoToStereo for Vec<i16> {
    fn upmix_mono_to_stereo(&self) -> Vec<i16> {
        self.iter()
            .flat_map(|&sample| vec![sample, sample])
            .collect()
    }
}

pub struct AudioProcessor {
    audio_rx: mpsc::UnboundedReceiver<Vec<i16>>,
    encoded_audio_tx: mpsc::UnboundedSender<String>,
}

impl AudioProcessor {
    pub fn new(
        audio_rx: mpsc::UnboundedReceiver<Vec<i16>>,
    ) -> (Self, UnboundedReceiverStream<String>) {
        let (encoded_audio_tx, encoded_audio_rx) = mpsc::unbounded_channel();
        let encoded_audio_rx = UnboundedReceiverStream::new(encoded_audio_rx);
        (
            Self {
                audio_rx,
                encoded_audio_tx,
            },
            encoded_audio_rx,
        )
    }

    pub async fn start(&mut self, sample_rate: SampleRate) {
        let chunk_size = 896;
        let mut buffer = Vec::with_capacity(chunk_size);

        while let Some(samples) = self.audio_rx.recv().await {
            buffer.extend(samples);
            dbg!(buffer.len());
            if buffer.len() >= chunk_size {
                let chunk = buffer.drain(..).collect::<Vec<i16>>();
                if !chunk.is_empty() {
                    let encoded_audio = resample_and_encode_audio_to_b64(&chunk, sample_rate);
                    self.encoded_audio_tx
                        .send(encoded_audio)
                        .expect("Failed to send encoded audio");
                }
            }
        }
        ()
    }
}

pub fn decode_base64_pcm(base64_data: &str) -> Vec<i16> {
    let decoded_bytes = BASE64_STANDARD
        .decode(base64_data)
        .expect("Failed to decode base64 data");
    let pcm_samples: &[i16] = bytemuck::cast_slice(&decoded_bytes);
    pcm_samples.to_vec()
}

pub fn resample_hz(data: &[i16], source_hz: SampleRate, target_hz: SampleRate) -> Vec<i16> {
    let mut source = signal::from_iter(data.iter().cloned());
    let interp = Linear::new(source.next(), source.next());
    source
        .from_hz_to_hz(interp, source_hz.0 as f64, target_hz.0 as f64)
        .until_exhausted()
        .collect::<Vec<i16>>()
}

pub fn resample_to_16khz(data: &[i16], source_hz: SampleRate) -> Vec<i16> {
    let mut source = signal::from_iter(data.iter().cloned());
    let interp = Linear::new(source.next(), source.next());
    let target_hz = 16000.0;
    source
        .from_hz_to_hz(interp, source_hz.0 as f64, target_hz)
        .until_exhausted()
        .collect::<Vec<i16>>()
}

pub fn resample_and_encode_audio_to_b64(data: &[i16], source_hz: SampleRate) -> String {
    let resampled_audio = resample_to_16khz(data, source_hz);
    let resampled_bytes: &[u8] = bytemuck::cast_slice(&resampled_audio);
    let base64_chunk = BASE64_STANDARD.encode(&resampled_bytes);
    base64_chunk
}
