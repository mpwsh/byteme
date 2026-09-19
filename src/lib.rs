//! Compress any bytes with zlib and encode them as a [Z85] string, and back.
//!
//! Useful for storing binaries as text: NFC tags, stickers, config files, or anywhere
//! that only accepts printable characters.
//!
//! The crate is pure in-memory transformation. File and stdin/stdout handling lives in the
//! `byteme` binary.
//!
//! # Examples
//!
//! ```
//! let text = byteme::encode(b"This is a test\n")?;
//! let bytes = byteme::decode(&text)?;
//!
//! assert_eq!(bytes, b"This is a test\n");
//! # Ok::<(), byteme::Error>(())
//! ```
//!
//! [Z85]: https://rfc.zeromq.org/spec/32/

use std::io::{self, Read, Write};

use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};

/// Everything that can go wrong while encoding or decoding.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// zlib compression failed.
    #[error("failed to compress input")]
    Compress(#[source] io::Error),

    /// The input is not Z85 text: wrong length or characters outside the Z85 alphabet.
    #[error(
        "failed to decode z85 (input may be corrupted or not z85 text - did you mean 'encode'?)"
    )]
    Z85(#[from] z85::DecodeError),

    /// The input is valid Z85 but does not contain a zlib stream.
    #[error("failed to decompress (data may be corrupted)")]
    Decompress(#[source] io::Error),
}

/// Compresses `bytes` and encodes the result as a Z85 string.
///
/// The output is plain ASCII with no whitespace and no trailing newline.
///
/// # Errors
///
/// Returns [`Error::Compress`] if zlib compression fails.
///
/// # Examples
///
/// ```
/// let text = byteme::encode(&[0x00, 0xFF, 0x10, 0x80])?;
///
/// assert!(text.is_ascii());
/// # Ok::<(), byteme::Error>(())
/// ```
pub fn encode(bytes: &[u8]) -> Result<String, Error> {
    Ok(z85::encode(compress(bytes)?))
}

/// Decodes Z85 `text` and decompresses it back into the original bytes.
///
/// ASCII whitespace anywhere in `text` is ignored, so CRLF line endings, trailing spaces and
/// hard-wrapped input all decode correctly.
///
/// # Errors
///
/// - Returns [`Error::Z85`] if `text` is not valid Z85.
/// - Returns [`Error::Decompress`] if the decoded bytes are not a zlib stream.
///
/// # Examples
///
/// ```
/// let bytes = byteme::decode("C?JLE:sHu(Qc%Y#!z.8[04z%)###00\r\n")?;
///
/// assert_eq!(bytes, b"This is a test\n");
/// # Ok::<(), byteme::Error>(())
/// ```
pub fn decode(text: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    // The Z85 alphabet contains no whitespace, so dropping it can never change the payload.
    let z85_text: Vec<u8> = text
        .as_ref()
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();

    decompress(&z85::decode(z85_text)?)
}

/// Compresses `bytes` into a zlib stream at the highest compression level.
///
/// # Errors
///
/// Returns [`Error::Compress`] if the encoder fails.
pub fn compress(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes).map_err(Error::Compress)?;
    encoder.finish().map_err(Error::Compress)
}

/// Decompresses a zlib stream produced by [`compress`].
///
/// # Errors
///
/// Returns [`Error::Decompress`] if `bytes` is not a complete, valid zlib stream.
pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decompressed = Vec::new();
    ZlibDecoder::new(bytes)
        .read_to_end(&mut decompressed)
        .map_err(Error::Decompress)?;
    Ok(decompressed)
}
