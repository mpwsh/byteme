//! Library-level tests for the public `byteme` API.

use byteme::{Error, compress, decode, decompress, encode};

/// The example string documented in the readme, and the bytes it decodes to.
const README_ENCODED: &str = "C?JLE:sHu(Qc%Y#!z.8[04z%)###00";
const README_DECODED: &[u8] = b"This is a test\n";

mod compress_tests {
    use super::*;

    #[test]
    fn should_return_smaller_output_for_repetitive_data() {
        let input = vec![b'A'; 10_000];

        let result = compress(&input).unwrap();

        assert!(
            result.len() < input.len(),
            "Compressed size {} should be smaller than input size {}",
            result.len(),
            input.len()
        );
    }

    #[test]
    fn should_produce_non_empty_output_for_non_empty_input() {
        let result = compress(&[1, 2, 3, 4, 5]).unwrap();

        assert!(!result.is_empty(), "Compressed output should not be empty");
    }

    #[test]
    fn should_produce_zlib_header_for_empty_input() {
        let result = compress(&[]).unwrap();

        assert!(
            !result.is_empty(),
            "Even empty input produces zlib header bytes"
        );
    }
}

mod decompress_tests {
    use super::*;

    #[test]
    fn should_recover_original_bytes_after_compress() {
        let input = b"Hello, byteme!";
        let compressed = compress(input).unwrap();

        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(decompressed, input);
    }

    #[test]
    fn should_roundtrip_empty_input() {
        let compressed = compress(&[]).unwrap();

        let decompressed = decompress(&compressed).unwrap();

        assert!(
            decompressed.is_empty(),
            "Expected no bytes, found {decompressed:?}"
        );
    }

    #[test]
    fn should_return_decompress_error_for_invalid_zlib_data() {
        let garbage = [0xFF, 0xFE, 0xFD, 0xFC];

        let error = decompress(&garbage).unwrap_err();

        assert!(
            matches!(error, Error::Decompress(_)),
            "Expected `Decompress`, found {error:?}"
        );
    }
}

mod encode_tests {
    use super::*;

    #[test]
    fn should_produce_ascii_safe_string() {
        let encoded = encode(b"binary \x00\x01\x02\xFF data").unwrap();

        assert!(
            encoded.is_ascii(),
            "z85 output should be ASCII, found {encoded:?}"
        );
    }

    #[test]
    fn should_not_emit_whitespace() {
        let encoded = encode(&vec![42u8; 300_000]).unwrap();

        assert!(
            !encoded.contains(char::is_whitespace),
            "z85 output should be a single unbroken line"
        );
    }
}

mod decode_tests {
    use super::*;

    #[test]
    fn should_decode_readme_example() {
        let decoded = decode(README_ENCODED).unwrap();

        assert_eq!(decoded, README_DECODED);
    }

    #[test]
    fn should_ignore_trailing_lf() {
        let decoded = decode(format!("{README_ENCODED}\n")).unwrap();

        assert_eq!(decoded, README_DECODED);
    }

    #[test]
    fn should_ignore_trailing_crlf() {
        let decoded = decode(format!("{README_ENCODED}\r\n")).unwrap();

        assert_eq!(decoded, README_DECODED);
    }

    #[test]
    fn should_ignore_surrounding_spaces_and_tabs() {
        let decoded = decode(format!(" \t{README_ENCODED} \t")).unwrap();

        assert_eq!(decoded, README_DECODED);
    }

    #[test]
    fn should_ignore_hard_wrapped_lines() {
        let (first, second) = README_ENCODED.split_at(10);

        let decoded = decode(format!("{first}\r\n{second}\r\n")).unwrap();

        assert_eq!(decoded, README_DECODED);
    }

    #[test]
    fn should_accept_raw_bytes_as_input() {
        let decoded = decode(README_ENCODED.as_bytes()).unwrap();

        assert_eq!(decoded, README_DECODED);
    }

    #[test]
    fn should_return_z85_error_for_characters_outside_the_alphabet() {
        let error = decode("_____").unwrap_err();

        assert!(
            matches!(error, Error::Z85(_)),
            "Expected `Z85`, found {error:?}"
        );
    }

    #[test]
    fn should_return_z85_error_for_wrong_length() {
        let error = decode("abc").unwrap_err();

        assert!(
            matches!(error, Error::Z85(_)),
            "Expected `Z85`, found {error:?}"
        );
    }

    #[test]
    fn should_return_z85_error_for_binary_input() {
        let error = decode([0x00, 0xFF, 0x10, 0x80, 0x7F]).unwrap_err();

        assert!(
            matches!(error, Error::Z85(_)),
            "Expected `Z85`, found {error:?}"
        );
    }

    #[test]
    fn should_return_decompress_error_for_z85_that_is_not_zlib() {
        let not_zlib = z85::encode([0xFF, 0xFE, 0xFD, 0xFC]);

        let error = decode(not_zlib).unwrap_err();

        assert!(
            matches!(error, Error::Decompress(_)),
            "Expected `Decompress`, found {error:?}"
        );
    }

    #[test]
    fn should_hint_at_encode_in_z85_error_message() {
        let error = decode("abc").unwrap_err();

        assert_eq!(
            error.to_string(),
            "failed to decode z85 (input may be corrupted or not z85 text - did you mean 'encode'?)"
        );
    }
}

mod roundtrip {
    use super::*;

    #[test]
    fn should_roundtrip_small_text() {
        let original = b"This is a test\n";

        let decoded = decode(encode(original).unwrap()).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn should_roundtrip_every_byte_value() {
        let original: Vec<u8> = (0..=255).collect();

        let decoded = decode(encode(&original).unwrap()).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn should_roundtrip_large_data() {
        let original = vec![42u8; 300_000];

        let decoded = decode(encode(&original).unwrap()).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn should_roundtrip_empty_input() {
        let decoded = decode(encode(&[]).unwrap()).unwrap();

        assert!(decoded.is_empty(), "Expected no bytes, found {decoded:?}");
    }
}
