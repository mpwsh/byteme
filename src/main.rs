use std::{env, io::IsTerminal};

use anyhow::Result;
use byteme::{from_raw, to_raw};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.get(1) {
        if !std::io::stdin().is_terminal() {
            from_raw(arg)?;
        } else {
            to_raw(arg)?;
        }
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
    }

    Ok(())
}
