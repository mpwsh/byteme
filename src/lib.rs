use core::str;
use std::{
    fs::File,
    io::{self, Read, Write},
};

use anyhow::{Context, Result};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use z85::{decode, encode};

pub const CHUNK_SIZE: usize = 1024 * 100;

pub fn from_raw(arg: &str) -> Result<()> {
    let mut buffer = Vec::new();
    io::stdin().lock().read_to_end(&mut buffer)?;

    let raw = str::from_utf8(&buffer)?.replace('\n', "");
    let decompressed = decompress(&decode(raw)?)?;

    let mut file = File::create(arg)?;
    Ok(file.write_all(&decompressed)?)
}

pub fn to_raw(arg: &str) -> Result<()> {
    let mut data = Vec::new();
    let mut f = File::open(arg).context("Error while reading file")?;
    f.read_to_end(&mut data)?;

    let compressed = compress(&data)?;
    let encoded = encode(compressed);
    let mut lock = io::stdout().lock();

    for chunk in encoded.as_bytes().chunks(CHUNK_SIZE) {
        lock.write_all(chunk)?;
    }

    Ok(())
}

pub fn compress(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
    e.write_all(bytes)?;
    let compressed_bytes = e.finish()?;
    Ok(compressed_bytes)
}

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut d = ZlibDecoder::new(bytes);
    let mut b = Vec::new();
    d.read_to_end(&mut b)?;
    Ok(b)
}
