use byteme::{CHUNK_SIZE, compress, decompress};
use z85::{decode, encode};

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
        let input = vec![1, 2, 3, 4, 5];
        let result = compress(&input).unwrap();
        assert!(!result.is_empty(), "Compressed output should not be empty");
    }

    #[test]
    fn should_handle_empty_input() {
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
    fn should_return_error_for_invalid_zlib_data() {
        let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC];
        let result = decompress(&garbage);
        assert!(
            result.is_err(),
            "Decompressing invalid data should return an error"
        );
    }

    #[test]
    fn should_roundtrip_empty_input() {
        let compressed = compress(&[]).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }
}

mod encode_decode {
    use super::*;

    #[test]
    fn should_roundtrip_compressed_data_through_z85() {
        let input = b"This is a test";
        let compressed = compress(input).unwrap();
        let encoded = encode(&compressed);
        let decoded = decode(encoded).unwrap();
        let decompressed = decompress(&decoded).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn should_produce_ascii_safe_encoded_string() {
        let input = b"binary \x00\x01\x02\xFF data";
        let compressed = compress(input).unwrap();
        let encoded = encode(&compressed);
        assert!(
            encoded.is_ascii(),
            "z85 encoded output should be ASCII-safe"
        );
    }
}

mod full_pipeline {
    use super::*;

    #[test]
    fn should_roundtrip_small_text() {
        let original = b"This is a test\n";
        let compressed = compress(original).unwrap();
        let encoded = encode(&compressed);

        let raw = encoded.replace('\n', "");
        let decoded = decode(raw).unwrap();
        let decompressed = decompress(&decoded).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn should_roundtrip_binary_data() {
        let original: Vec<u8> = (0..=255).collect();
        let compressed = compress(&original).unwrap();
        let encoded = encode(&compressed);

        let raw = encoded.replace('\n', "");
        let decoded = decode(raw).unwrap();
        let decompressed = decompress(&decoded).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn should_roundtrip_large_data_with_chunking() {
        let original = vec![42u8; CHUNK_SIZE * 3 + 500];
        let compressed = compress(&original).unwrap();
        let encoded = encode(&compressed);

        // Simulate chunked output reassembly
        let chunked: String = encoded
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<&str>>()
            .join("\n");

        let raw = chunked.replace('\n', "");
        let decoded = decode(raw).unwrap();
        let decompressed = decompress(&decoded).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn should_roundtrip_readme_example() {
        let input = b"This is a test\n";
        let compressed = compress(input).unwrap();
        let encoded = encode(&compressed);
        let decoded = decode(encoded).unwrap();
        let decompressed = decompress(&decoded).unwrap();
        assert_eq!(decompressed, input);
    }
}
