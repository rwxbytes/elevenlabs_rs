use crate::prelude::*;
use bytes::Bytes;
use std::fs::File;
use std::io::prelude::*;

/// Save audio to a file
pub fn save(filename: &str, data: Bytes) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(&data)?;
    Ok(())
}
