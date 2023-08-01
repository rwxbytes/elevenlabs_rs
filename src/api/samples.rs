use crate::{api::ClientBuilder, error::Error, prelude::*};
use http_body_util::Empty;
use hyper::body::Bytes;

const GET: &str = "GET";
const DELTE: &str = "DELETE";

// Test a particular error case for get_audio_sample
//#[cfg(test)]
//mod tests {
//    const INVALID_LEN_VOICE_ID: &str = "123456789";
//    const INVALID_LEN_SAMPLE_ID: &str = "123456789012345678901";
//
//    #[tokio::test]
//    async fn get_audio_sample_is_erring_when_ids_length_are_not_20_chars() {
//        let voice_id = INVALID_LEN_VOICE_ID;
//        let sample_id = INVALID_LEN_SAMPLE_ID;
//        let result = super::get_audio_sample(voice_id, sample_id).await;
//        assert!(result.is_err());
//    }
//}

pub async fn get_audio_sample(voice_id: &str, sample_id: &str) -> Result<()> {
    let path = format!("/voices/{}/samples/{}/audio", voice_id, sample_id);
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(&path)?
        .method(GET)?
        .header("ACCEPT", "audio/*")?
        .build()?;
    let _path = c.send_request(Empty::<Bytes>::new()).await?;
    Ok(())
}

pub async fn delete_audio_sample(voice_id: &str, sample_id: &str) -> Result<()> {
    let path = format!("/voices/{}/samples/{}", voice_id, sample_id);
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(&path)?
        .method(DELTE)?
        .header("ACCEPT", "application/json")?
        .build()?;
    let _path = c.send_request(Empty::<Bytes>::new()).await?;
    Ok(())
}
