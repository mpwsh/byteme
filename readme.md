## Description

Compress/Decompress, Encode and Decode any file to a z85 string.  
Useful to store binaries in text format, for NFC cards/stickers/keychains or anywhere you want.

## Build

```bash
#Build
❯ cargo build --release
```

## Usage

Compress a file and encode bytes to z85

```bash
#Create a test file
echo "This is a test" >> raw.txt
#Process
❯ ./target/release/byteme raw.txt
#Output
78da0bc9c82c5600a2448592d4e2122e0029730500
```

Decode z85 and decompress back to original file

```bash
#Decode
❯ echo "78da0bc9c82c5600a2448592d4e2122e0029730500" | ./target/release/byteme out.txt
#Check your new file
❯ cat out.txt
This is a test
```

## Encryption

Encrypt a file using `gpg` and process with `byteme`

```bash
#Interactive mode
❯ gpg -c --no-symkey-cache raw.txt
#No interactive mode
❯ gpg --batch --passphrase 'somepass' -c raw.txt
```

Now run the steps above but using `<filename>.gpg` instead.

## Decryption

```bash
❯ gpg -d raw.txt.gpg
```
