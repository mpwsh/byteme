use core::str;
use std::{
    fs::File,
    io::{self, Read, Write},
};

use anyhow::{Context, Result};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use z85::{decode, encode};

pub const CHUNK_SIZE: usize = 1024 * 100;

pub fn from_raw(input: Option<&str>, output: Option<&str>) -> Result<()> {
    let mut buffer = Vec::new();

    match input {
        Some(path) => {
            let mut f = File::open(path).with_context(|| format!("cannot open '{}'", path))?;
            f.read_to_end(&mut buffer)?;
        }
        None => {
            io::stdin().lock().read_to_end(&mut buffer)?;
        }
    }

    let raw = str::from_utf8(&buffer)
        .context("input is not valid z85 text (did you mean 'encode'?)")?
        .replace('\n', "");

    let decompressed = decompress(
        &decode(raw).context("failed to decode z85 (input may be corrupted or not z85 encoded)")?,
    )
    .context("failed to decompress (data may be corrupted)")?;

    match output {
        Some(path) => {
            let mut file =
                File::create(path).with_context(|| format!("cannot create '{}'", path))?;
            file.write_all(&decompressed)?;
        }
        None => {
            io::stdout().lock().write_all(&decompressed)?;
        }
    }

    Ok(())
}

pub fn to_raw(input: Option<&str>, output: Option<&str>) -> Result<()> {
    let mut data = Vec::new();

    match input {
        Some(path) => {
            let mut f = File::open(path).with_context(|| format!("cannot open '{}'", path))?;
            f.read_to_end(&mut data)?;
        }
        None => {
            io::stdin().lock().read_to_end(&mut data)?;
        }
    }

    let compressed = compress(&data).context("failed to compress input")?;
    let encoded = encode(compressed);

    match output {
        Some(out_path) => {
            let mut file =
                File::create(out_path).with_context(|| format!("cannot create '{}'", out_path))?;
            file.write_all(encoded.as_bytes())?;
        }
        None => {
            let mut lock = io::stdout().lock();
            encoded
                .as_bytes()
                .chunks(CHUNK_SIZE)
                .try_for_each(|chunk| lock.write_all(chunk))
                .context("failed to write to stdout")?;
        }
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
