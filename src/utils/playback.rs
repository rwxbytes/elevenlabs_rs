use crate::client::Result;
use bytes::Bytes;
use bytes::{BufMut, BytesMut};
use futures_util::{pin_mut, Stream, StreamExt};
use rodio::{Decoder, OutputStream, Sink};

/// Play audio
pub fn play(data: Bytes) -> Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let source = Decoder::new(std::io::Cursor::new(data))?;
    let sink = Sink::try_new(&stream_handle)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

// TODO: Improve
pub async fn stream_audio(data: impl Stream<Item = Result<Bytes>>) -> Result<()> {
    pin_mut!(data);
    let mut buf = BytesMut::with_capacity(16384);
    let audio_output = rodio::OutputStream::try_default()?;
    let audio_sink = rodio::Sink::try_new(&audio_output.1)?;

    while let Some(resulting_bytes) = data.next().await {
        let bytes = resulting_bytes?;
        buf.put(bytes);

        while buf.len() >= 16384 {
            let audio_data = buf.split_to(16384).freeze();
            let cursor = std::io::Cursor::new(audio_data);
            let source = rodio::Decoder::new(cursor)?;
            audio_sink.append(source);

            // Sleep for a short duration to allow the audio sink to play the data
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    // Play the remaining bytes in the buffer
    if !buf.is_empty() {
        let audio_data = buf.freeze();
        let cursor = std::io::Cursor::new(audio_data);
        let source = rodio::Decoder::new(cursor)?;
        audio_sink.append(source);
    }

    // Wait for the audio sink to finish playing
    audio_sink.sleep_until_end();

    Ok(())
}
