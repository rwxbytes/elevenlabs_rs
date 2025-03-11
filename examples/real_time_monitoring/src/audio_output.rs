// Iterations of LLMs
// Audio quality still poor
pub use std::sync::{Arc, Mutex};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
pub use tokio::sync::mpsc;
pub use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub use cpal::{Sample, Stream as CpalStream, SampleRate};

use dasp::{signal, Signal};
use dasp::interpolate::Interpolator;
use dasp::interpolate::linear::Linear;
use dasp::signal::interpolate::Converter;
use tracing::info;
use crate::{audio_output, AudioDecodeError};

pub struct DefaultSpeakersManager {
    audio_rx: Option<mpsc::UnboundedReceiver<Vec<i16>>>,
    pub inner: Arc<Mutex<Vec<i16>>>,
    //processor: Option<AudioProcessor>,
}

#[derive(Clone)]
pub struct AudioProcessor {
    input_sample_rate: u32,
    output_sample_rate: u32,
    prev_samples: [f32; 3],
    // Store the last input sample to improve interpolation continuity.
    last_input_sample: Option<f32>,
}

impl AudioProcessor {
    pub fn new(input_sample_rate: u32, output_sample_rate: u32) -> Self {
        Self {
            input_sample_rate,
            output_sample_rate,
            prev_samples: [0.0; 3],
            last_input_sample: None,
        }
    }


    /// Process a chunk of 16-bit PCM samples:
    /// 1. Convert to f32 (using dasp's `Sample` trait).
    /// 2. Optionally prepend the last sample from the previous chunk.
    /// 3. Apply a simple low-pass filter.
    /// 4. Resample from input_sample_rate to output_sample_rate using dasp's linear interpolator.
    /// 5. Convert back to i16.
    pub fn process_chunk(&mut self, input: &[i16]) -> Vec<i16> {
        // Convert input samples to f32 in the range [-1.0, 1.0].
        let mut float_samples: Vec<f32> = input.iter()
            .map(|&s| s.to_sample::<f32>())
            .collect();

        // Prepend the last sample from the previous chunk if available.
        if let Some(last) = self.last_input_sample {
            float_samples.insert(0, last);
        }

        // Apply a 3-point moving average filter.
        for i in 0..float_samples.len() {
            let current = float_samples[i];
            float_samples[i] = (self.prev_samples[0] + self.prev_samples[1] + self.prev_samples[2] + current) / 4.0;
            // Shift the previous samples.
            self.prev_samples[0] = self.prev_samples[1];
            self.prev_samples[1] = self.prev_samples[2];
            self.prev_samples[2] = current;
        }

        // Save the last sample for the next chunk.
        self.last_input_sample = float_samples.last().copied();

        // Resample using dasp's Linear interpolation.
        let ratio = self.output_sample_rate as f32 / self.input_sample_rate as f32;
        // Calculate the number of intervals (samples - 1).
        let intervals = float_samples.len().saturating_sub(1);
        let output_len = (intervals as f32 * ratio) as usize;
        let mut output_float = Vec::with_capacity(output_len);
        let mut pos: f32 = 0.0;

        for _ in 0..output_len {
            let idx = pos.floor() as usize;
            let frac = pos - idx as f32;
            if idx + 1 < float_samples.len() {
                let sample: f32 = (1.0 - frac) * float_samples[idx] + frac * float_samples[idx + 1];
                // Optional soft clipping to keep within [-0.95, 0.95].
                let clipped = sample.clamp(-0.95, 0.95);
                output_float.push(clipped);
            }
            pos += 1.0 / ratio;
        }

        // Convert the resampled f32 samples back to i16.
        output_float.into_iter()
            .map(|sample| sample.to_sample::<i16>())
            .collect()
    }
}

impl DefaultSpeakersManager {
    pub fn new(audio_rx: mpsc::UnboundedReceiver<Vec<i16>>) -> Self {
        Self {
            audio_rx: Some(audio_rx),
            inner: Arc::new(Mutex::new(Vec::new())),
            //processor: AudioProcessor::new(8000, 44100), // We'll update output rate later
            //processor: None,
        }
    }

    pub async fn build_output_stream(&mut self) -> (CpalStream, SampleRate) {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No output device available");

        let config = device.default_output_config()
            .expect("Failed to get default output config");

        let sample_rate = config.sample_rate();
        // Update processor with actual output sample rate
        //self.processor = AudioProcessor::new(8000, sample_rate.0);
        let mut processor = AudioProcessor::new(8000, sample_rate.0);

        let mut audio_rx = self.audio_rx.take().unwrap();
        let buffer = Arc::clone(&self.inner);
        let buffer_for_processing = Arc::clone(&self.inner);

        tokio::spawn(async move {
            while let Some(samples) = audio_rx.recv().await {
                let processed = processor.process_chunk(&samples);
                let mut buffer = buffer.lock().unwrap();
                buffer.extend(processed);
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



/// Decode a base64-encoded string of µ-law data into 16-bit PCM samples.
pub fn decode_base64_pcm(base64_data: &str) -> Result<Vec<i16>, AudioDecodeError> {
    let ulaw_bytes = BASE64_STANDARD
        .decode(base64_data)
        .map_err(|e| AudioDecodeError::Base64DecodeError(e.to_string()))?;

    // Log first few bytes for debugging.
    if !ulaw_bytes.is_empty() {
        let first_few = &ulaw_bytes[..ulaw_bytes.len().min(10)];
        //info!("First few µ-law bytes: {:?}", first_few);
    }

    // Decode each µ-law byte.
    let pcm_samples: Vec<i16> = ulaw_bytes
        .into_iter()
        .map(|b| {
            let sample = mu_law_decode(b);
            // Optional noise gate: adjust threshold (here, 500) as needed.
            if sample.abs() < 500 { 0 } else { sample }
        })
        .collect();

    if !pcm_samples.is_empty() {
        let min = pcm_samples.iter().min().unwrap();
        let max = pcm_samples.iter().max().unwrap();
        //info!("PCM sample range: min={}, max={}", min, max);
    }

    Ok(pcm_samples)
}


pub fn upmix_mono_to_stereo(input: &[i16]) -> Vec<i16> {
    input.iter()
        .flat_map(|&sample| std::iter::repeat(sample).take(2))
        .collect()
}


/// Standard µ-law decoder.
/// Extracts sign, exponent, and mantissa to reconstruct a 16-bit PCM sample.
pub fn mu_law_decode(u_val: u8) -> i16 {
    let u_val = !u_val;
    let sign = u_val & 0x80;
    let exponent = (u_val >> 4) & 0x07;
    let mantissa = u_val & 0x0F;
    let mut sample = ((mantissa as i16) << 4) + 0x08;
    sample <<= exponent;
    sample -= 0x84;
    if sign != 0 { -sample } else { sample }
}