use std::io::{Read, Write};
use std::fs::File;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use anyhow::Result;
use std::env;
use std::path::Path;
use std::io::{self, BufRead};
use std::process;
use core::str;
use std::{fmt::Write as fmtWrite, num::ParseIntError};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.get(1).is_none() && atty::is(atty::Stream::Stdin) {
        println!("File will be compressed and will output a hex string on stdout");
        println!("Usage:\nCompress: byteme <path>\nDecompress: echo '<your-hex-string> | byteme <path-to-save-decompressed-file>' ");
        process::exit(0x0100);
    };
    if args.get(1).is_some(){

    match Path::new(&args[1]).exists() {
    true => println!("{}", encode_hex(&compress(read_from_file(&args[1])?)?)),
    false => println!("File doesn't exist. Check path"),
    };
        process::exit(0x0100);
    }
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_line(&mut buffer)?;
    write_to_file(
        decompress(decode_hex(&buffer.trim_end())?)?
        , "decompressed.file")?;
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
    let mut file = File::create(&path)?;
    file.write_all(&data).unwrap();
    Ok(())
}
fn read_from_file(path: &str) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut f = File::open(path)?;
    f.read_to_end(&mut data)?;
    Ok(data)
}

fn compress(bytes: Vec<u8>) -> Result<Vec<u8>>{
    let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
    e.write_all(&bytes)?;
    let compressed_bytes = e.finish()?;
    Ok(compressed_bytes)
}
fn decompress(bytes: Vec<u8>) -> Result<Vec<u8>>{
    let mut d = ZlibDecoder::new(&*bytes);
    let mut b = Vec::new();
    d.read_to_end(&mut b)?;
    Ok(b)
}
