## Description

Compress/Decompress, Encode and Decode any file to a z85 string.
Useful to store binaries in text format, for NFC cards/stickers/keychains or anywhere you want.

## Build

```bash
cargo build --release
```

## Usage

Both commands read from `<file>` when given and from stdin otherwise, and write to stdout unless `-o` is set.

```bash
# Encode a file to z85 (stdout)
byteme encode raw.txt

# Encode and save to file
byteme encode raw.txt -o encoded.txt

# Pipe into encode
cat myimage.png | byteme encode -o encoded.txt

# Decode from file
byteme decode encoded.txt

# Decode and save to file
byteme decode encoded.txt -o raw.txt

# Pipe into decode
echo 'C?JLE:sHu(Qc%Y#!z.8[04z%)###00' | byteme decode

# Pipe into decode and save to file
echo 'C?JLE:sHu(Qc%Y#!z.8[04z%)###00' | byteme decode -o raw.txt
```

Whitespace in the encoded text is ignored when decoding, so Windows (CRLF) line endings, trailing spaces and
text that was hard-wrapped by an editor or a chat app all decode fine.

If decoding fails, the output file is left untouched.

## Options

```
-o, --output <path>   Write to file instead of stdout
-h, --help            Show help message
```

## Docker

The image's entrypoint is the binary, so arguments go straight after the image name. Use `-i` to pipe through stdin.

```bash
echo 'C?JLE:sHu(Qc%Y#!z.8[04z%)###00' | docker run --rm -i <dockerhub-user>/byteme decode
```

## Library

The crate also exposes the transformation as a library.

```rust
let text = byteme::encode(b"This is a test\n")?;
let bytes = byteme::decode(&text)?;

assert_eq!(bytes, b"This is a test\n");
```

Errors are a typed `byteme::Error` (`Compress`, `Z85`, `Decompress`), so callers can tell bad text apart from corrupted data.

## Encryption

Encrypt a file using `gpg` and process with `byteme`

```bash
# Interactive mode
gpg -c --no-symkey-cache raw.txt
# Non-interactive mode
gpg --batch --passphrase 'somepass' -c raw.txt
```

Now run the steps above but using `<filename>.gpg` instead.

## Decryption

```bash
gpg -d raw.txt.gpg
```

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
```

`imports_granularity` and `group_imports` in `.rustfmt.toml` are nightly-only. Stable `cargo fmt` ignores them with a
warning; run `cargo +nightly fmt` to apply them.
