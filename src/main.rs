use anyhow::Result;
use core::str;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use itertools::Itertools;
use std::io::{self, stdout, Read, Write};
use std::{env, fs::File, path::Path};
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

    write_file(decompressed, arg)
}

fn to_raw(arg: &str) -> Result<()> {
    let compressed = compress(read_file(arg)?)?;
    match Path::new(arg).exists() {
        true => {
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
        false => println!("File doesn't exist. Check path"),
    };
    Ok(())
}

pub fn write_file(data: Vec<u8>, path: &str) -> Result<()> {
    let mut file = File::create(path)?;
    Ok(file.write_all(&data)?)
}
fn read_file(path: &str) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut f = File::open(path)?;
    f.read_to_end(&mut data)?;
    Ok(data)
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
