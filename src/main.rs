use anyhow::Result;
use byteme::{from_raw, to_raw};

const HELP: &str = "\
byteme - Compress/encode files to z85 and back

USAGE:
  byteme encode <file> [-o <output>]
  byteme decode [-o <output>]

COMMANDS:
  encode    Compress a file and encode to z85 (stdout by default)
  decode    Read z85 from stdin, decompress and output (stdout by default)

OPTIONS:
  -o, --output <path>   Write to file instead of stdout
  -h, --help            Show this help message";

fn main() -> Result<()> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();
    let mut command: Option<String> = None;
    let mut file: Option<String> = None;
    let mut output: Option<String> = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Short('o') | Long("output") => {
                output = Some(parser.value()?.string()?);
            }
            Short('h') | Long("help") => {
                println!("{}", HELP);
                return Ok(());
            }
            Value(val) if command.is_none() => {
                command = Some(val.string()?);
            }
            Value(val) if file.is_none() => {
                file = Some(val.string()?);
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    match command.as_deref() {
        Some("encode") => {
            to_raw(file.as_deref(), output.as_deref())?;
        }
        Some("decode") => {
            from_raw(file.as_deref(), output.as_deref())?;
        }
        Some(other) => anyhow::bail!("unknown command '{}'\n\n{}", other, HELP),
        None => {
            println!("{}", HELP);
        }
    }

    Ok(())
}
