## Description

Compress/Decompress, Encode and Decode any file to a z85 string.
Useful to store binaries in text format, for NFC cards/stickers/keychains or anywhere you want.

## Build

```bash
cargo build --release
```

## Usage

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

## Options

```
-o, --output <path>   Write to file instead of stdout
-h, --help            Show help message
```

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
