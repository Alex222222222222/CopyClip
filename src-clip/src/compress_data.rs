use std::io::{Read, Write};

use anyhow::Ok;
use log::debug;

pub fn compress_text(text: &str) -> Result<Vec<u8>, anyhow::Error> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(text.as_bytes())?;
    Ok(encoder.finish()?)
}

pub fn decompress_text(data: &[u8]) -> Result<String, anyhow::Error> {
    debug!("Decompressing text");
    let mut decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(data));
    let mut text = String::new();
    decoder.read_to_string(&mut text)?;

    debug!("Decompressed text: {}", text);

    Ok(text)
}
