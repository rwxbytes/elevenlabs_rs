use crate::client::Result;
use async_stream::stream;
use bytes::Bytes;
use futures_util::{pin_mut, Stream, StreamExt};
use std::sync::mpsc;
use std::{fs::File, io::prelude::*};

#[cfg(feature = "playback")]
mod playback;

#[cfg(feature = "playback")]
pub use playback::{play, stream_audio};

/// Save audio to a file
pub fn save(filename: &str, data: Bytes) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(&data)?;
    Ok(())
}

pub fn text_chunker<S>(text_stream: S) -> impl Stream<Item = String>
where
    S: Stream<Item = String> + Send + 'static,
{
    let splitters = [
        '.', ',', '?', '!', ';', ':', '—', '-', '(', ')', '[', ']', '{', '}', ' ',
    ];
    let mut buf = String::new();
    let (tx, rx) = mpsc::channel::<String>();

    tokio::spawn(async move {
        pin_mut!(text_stream);
        while let Some(text) = text_stream.next().await {
            if buf.ends_with(splitters) {
                tx.send(format!("{} ", buf.as_str())).unwrap();
                buf = text
            } else if text.starts_with(splitters) {
                tx.send(format!(
                    "{}{} ",
                    buf.as_str(),
                    text.char_indices().next().unwrap().1
                ))
                .unwrap();
                buf = text[1..].to_string();
            } else {
                buf.push_str(&text)
            }
        }
        if !buf.is_empty() {
            tx.send(buf).unwrap();
        }
    });

    stream! {
        while let Ok(buf) = rx.recv() {
            yield buf
        }
    }
}
