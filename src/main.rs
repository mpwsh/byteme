use anyhow::Result;
use core::str;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use itertools::Itertools;
use std::io::{self, stdout, Read, Write};
use std::{env, fs::File};
use z85::*;

const CHUNK_SIZE: usize = 1024 * 100;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let tty = atty::is(atty::Stream::Stdin);
    if let Some(arg) = args.get(1) {
        match tty {
            //Decode + Decompress + Save to file
            false => from_raw(arg)?,
            //Read from path + Compress + Encode
            true => to_raw(arg)?,
        };
    } else {
        println!(
            "
        Provided file path will be compressed encoded to z85.
        Result string will be sent stdout.
        To convert back to original file read instructions below.
        ----------------
        Usage:
        Compress: byteme <path>
        Decompress: echo '<your-z85-string> | byteme <output-path>'
        ---------------
        Hint: If you're seeing this message something went wrong.
        "
        );
    };

    Ok(())
}
fn from_raw(arg: &str) -> Result<()> {
    let mut buffer = Vec::new();
    io::stdin().lock().read_to_end(&mut buffer)?;

    let raw = str::from_utf8(&buffer)?.replace('\n', "");
    let decompressed = decompress(decode(raw)?)?;

    let mut file = File::create(arg)?;
    Ok(file.write_all(&decompressed)?)
}

fn to_raw(arg: &str) -> Result<()> {
    let mut data = Vec::new();
    match File::open(arg) {
        Ok(mut f) => {
            f.read_to_end(&mut data)?;
            let compressed = compress(data)?;
            let encoded = encode(compressed);
            let mut lock = stdout().lock();
            write!(
                lock,
                "{}",
                str::from_utf8(
                    encoded
                        .chars()
                        .chunks(CHUNK_SIZE)
                        .into_iter()
                        .map(|x| x.collect())
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_bytes(),
                )?
            )?;
        }
        Err(e) => { println!("Error while reading file: {}", e); std::process::exit(1) }
    };
    Ok(())
}

fn compress(bytes: Vec<u8>) -> Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
    e.write_all(&bytes)?;
    let compressed_bytes = e.finish()?;
    Ok(compressed_bytes)
}
fn decompress(bytes: Vec<u8>) -> Result<Vec<u8>> {
    let mut d = ZlibDecoder::new(&*bytes);
    let mut b = Vec::new();
    d.read_to_end(&mut b)?;
    Ok(b)
}
