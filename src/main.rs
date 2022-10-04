use anyhow::Result;
use core::str;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::io::{Read, Write};
use std::path::Path;
use std::process;
use std::{fmt::Write as fmtWrite, num::ParseIntError};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let tty = atty::is(atty::Stream::Stdin);
    //Usage
    if args.get(1).is_none() {
        println!(
            "
        Provided path will be compressed and a hex string will be sent stdout.
        To convert back to original file read instructions below.
        ----------------
        Usage:
        Compress: byteme <path>
        Decompress: echo '<your-hex-string> | byteme <path-to-save-output-file>'
        ---------------
        Hint: If you're seeing this message something went wrong.
        "
        );
        process::exit(0x0100);
    } else {
        match tty {
            false => {
                //Decode + Decompress + Save to file
                let mut buffer = String::new();
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                handle.read_line(&mut buffer)?;
                write_to_file(decompress(decode_hex(buffer.trim_end())?)?, &args[1])?;
            }
            true => {
                //Read from path + Compress + Encode
                match Path::new(&args[1]).exists() {
                    true => println!("{}", encode_hex(&compress(read_from_file(&args[1])?)?)),
                    false => println!("File doesn't exist. Check path"),
                };
                process::exit(0x0100);
            }
        }
    };

    Ok(())
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}
pub fn write_to_file(data: Vec<u8>, path: &str) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&data).unwrap();
    Ok(())
}
fn read_from_file(path: &str) -> Result<Vec<u8>> {
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
