use crate::prelude::*;
use bytes::Bytes;
use rodio::{Decoder, OutputStream, Sink};
use std::{fs::File, io::prelude::*};

/// Save audio to a file
pub fn save(filename: &str, data: Bytes) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(&data)?;
    Ok(())
}

/// Play audio
pub fn play(data: Bytes) -> Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let source = Decoder::new(std::io::Cursor::new(data))?;
    let sink = Sink::try_new(&stream_handle)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
