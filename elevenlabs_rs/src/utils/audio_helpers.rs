use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use cpal::SampleRate;
use dasp::{self, signal, Sample, Signal};
use dasp::interpolate::linear::Linear;
use dasp::slice::ToFrameSlice;

use crate::Result;

type AudioBuffer = Arc<Mutex<Vec<i16>>>;

pub trait UpmixMonoToStereo {
    fn upmix_mono_to_stereo(&self) -> Vec<i16>;
}

/// Decode a base64-encoded string into a `Vec<i16>` of PCM samples
pub fn decode_base64_pcm(base64_data: &str) -> Result<Vec<i16>> {
    let decoded_bytes = BASE64_STANDARD.decode(base64_data)?;
    let pcm_samples: &[i16] = bytemuck::cast_slice(&decoded_bytes);
    Ok(pcm_samples.to_vec())
}



//pub fn downmix_stereo_to_mono(data: &[f32], buf: AudioBuffer) {
//    println!("input data len: {}", data.len());
//    let mut buffer = buf.lock().unwrap();
//
//    for chunk in data.chunks(2) {
//        if chunk.len() == 2 {
//            let mono_signal = (chunk[0] + chunk[1]) / 2.0;
//            buffer.push(i16::from_sample(mono_signal));
//        }
//    }
//}

pub fn downmix_stereo_to_mono(data: &[f32], buf: AudioBuffer) {
    println!("input data len: {}", data.len());
    let mut buffer = buf.lock().unwrap();
    let frame: &[[f32; 2]] = data.to_frame_slice().expect("Failed to convert data to frame slice");

    for chunk in frame {
        let mono = (chunk[0] + chunk[1]) / 2.0;
        buffer.push(i16::from_sample(mono));
    }
    //for chunk in data.chunks(2) {
    //    if chunk.len() == 2 {
    //        let mono_signal = (chunk[0] + chunk[1]) / 2.0;
    //        buffer.push(i16::from_sample(mono_signal));
    //    }
    //}
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

//pub fn resample_to_48khz(data: &[i16], source_hz: SampleRate) -> Vec<i16> {
//    let mut source = signal::from_iter(data.iter().cloned());
//    let interp = Linear::new(source.next(), source.next());
//    let target_hz = 44100.0;
//    source
//        .from_hz_to_hz(interp, source_hz.0 as f64, target_hz)
//        .until_exhausted()
//        .collect::<Vec<i16>>()
//}

pub fn resample_and_encode_audio_to_b64(data: &[i16], source_hz: SampleRate) -> String {
    let resampled_audio = resample_to_16khz(data, source_hz);
    let resampled_bytes: &[u8] = bytemuck::cast_slice(&resampled_audio);
    let base64_chunk = BASE64_STANDARD.encode(&resampled_bytes);
    base64_chunk
}