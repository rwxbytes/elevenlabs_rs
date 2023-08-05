use crate::{api::ClientBuilder, prelude::*};
use http_body_util::Empty;
use hyper::body::Bytes;

const BASE_PATH: &str = "/voices";
const SAMPLES_PATH: &str = "/samples";
const AUDIO_PATH: &str = "/audio";

/// Get the audio sample for a specific voice
///
/// # Example
///
/// ```no_run
///
///use elevenlabs_rs::api::samples::get_audio_sample;
///use elevenlabs_rs::api::voice::*;
///use elevenlabs_rs::prelude::*;
///use elevenlabs_rs::utils::save;
///
///
///#[tokio::main]
///async fn main() -> Result<()> {
///    let v = get_voices().await?;
///    let cloned_voices = v.all_clones();
///    let voice = cloned_voices[0].clone();
///    let sample = get_audio_sample(
///        &voice.voice_id,
///        &voice.samples.unwrap()[0].clone().sample_id,
///    )
///    .await?;
///
///    let _saved_file = save("sample_test.mp3", sample)?;
///    Ok(())
///}
/// ```
pub async fn get_audio_sample(voice_id: &str, sample_id: &str) -> Result<Bytes> {
    let path = format!(
        "{}/{}{}/{}{}",
        BASE_PATH, voice_id, SAMPLES_PATH, sample_id, AUDIO_PATH
    );
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(&path)?
        .method(GET)?
        .header(ACCEPT, AUDIO_ALL)?
        .build()?;
    let data = c.send_request(Empty::<Bytes>::new()).await?;
    Ok(data)
}

pub async fn delete_audio_sample(voice_id: &str, sample_id: &str) -> Result<()> {
    let path = format!("{}/{}{}/{}", BASE_PATH, voice_id, SAMPLES_PATH, sample_id);
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(&path)?
        .method(DELETE)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let _data = c.send_request(Empty::<Bytes>::new()).await?;
    Ok(())
}
