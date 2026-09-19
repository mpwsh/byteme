//! Command line front end for the `byteme` library: reads a file or stdin, encodes or decodes
//! it, and writes the result to a file or stdout.

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

const HELP: &str = "\
byteme - Compress/encode files to z85 and back

USAGE:
  byteme encode [<file>] [-o <path>]
  byteme decode [<file>] [-o <path>]

COMMANDS:
  encode    Compress the input and encode it to z85
  decode    Decode z85 input and decompress it (whitespace in the input is ignored)

ARGS:
  <file>    Input file. Reads from stdin when omitted

OPTIONS:
  -o, --output <path>   Write to file instead of stdout
  -h, --help            Show this help message";

fn main() -> Result<()> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();
    let mut command: Option<String> = None;
    let mut file: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Short('o') | Long("output") => {
                output = Some(PathBuf::from(parser.value()?));
            }
            Short('h') | Long("help") => {
                println!("{HELP}");
                return Ok(());
            }
            Value(val) if command.is_none() => {
                command = Some(val.string()?);
            }
            Value(val) if file.is_none() => {
                file = Some(PathBuf::from(val));
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    // The output is only opened once the transformation has succeeded, so a failed run never
    // truncates an existing file.
    let result = match command.as_deref() {
        Some("encode") => byteme::encode(&read_input(file.as_deref())?)?.into_bytes(),
        Some("decode") => byteme::decode(read_input(file.as_deref())?)?,
        Some(other) => anyhow::bail!("unknown command '{other}'\n\n{HELP}"),
        None => {
            println!("{HELP}");
            return Ok(());
        }
    };

    write_output(output.as_deref(), &result)
}

fn read_input(path: Option<&Path>) -> Result<Vec<u8>> {
    if let Some(path) = path {
        return fs::read(path).with_context(|| format!("cannot read '{}'", path.display()));
    }

    let mut buffer = Vec::new();
    io::stdin()
        .lock()
        .read_to_end(&mut buffer)
        .context("cannot read stdin")?;
    Ok(buffer)
}

fn write_output(path: Option<&Path>, bytes: &[u8]) -> Result<()> {
    match path {
        Some(path) => {
            fs::write(path, bytes).with_context(|| format!("cannot write '{}'", path.display()))
        }
        None => io::stdout()
            .lock()
            .write_all(bytes)
            .context("failed to write to stdout"),
    }
}
