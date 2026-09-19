//! End-to-end tests that run the real `byteme` binary, covering argument parsing and the
//! file/stdin/stdout handling that lives in `src/main.rs`.

use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const README_ENCODED: &str = "C?JLE:sHu(Qc%Y#!z.8[04z%)###00";
const README_DECODED: &[u8] = b"This is a test\n";

/// Runs the binary. `stdin` is only piped when given, so commands that never read it cannot
/// race against a closed pipe.
fn run<I, S>(args: I, stdin: Option<&[u8]>) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new(env!("CARGO_BIN_EXE_byteme"))
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(bytes) = stdin {
        child.stdin.take().unwrap().write_all(bytes).unwrap();
    }

    child.wait_with_output().unwrap()
}

/// A scratch directory private to one test, so tests can run in parallel.
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test_name);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

mod decode {
    use super::*;

    #[test]
    fn should_decode_readme_example_from_stdin() {
        let output = run(["decode"], Some(format!("{README_ENCODED}\n").as_bytes()));

        assert_eq!(
            output.stdout,
            README_DECODED,
            "stderr: {}",
            stderr_of(&output)
        );
    }

    #[test]
    fn should_decode_stdin_with_crlf_line_ending() {
        let output = run(["decode"], Some(format!("{README_ENCODED}\r\n").as_bytes()));

        assert_eq!(
            output.stdout,
            README_DECODED,
            "stderr: {}",
            stderr_of(&output)
        );
    }

    #[test]
    fn should_decode_from_file_argument() {
        let input = scratch_dir("decode_from_file_argument").join("encoded.txt");
        fs::write(&input, README_ENCODED).unwrap();

        let output = run([OsStr::new("decode"), input.as_os_str()], None);

        assert_eq!(
            output.stdout,
            README_DECODED,
            "stderr: {}",
            stderr_of(&output)
        );
    }

    #[test]
    fn should_write_decoded_bytes_to_output_file() {
        let decoded = scratch_dir("decode_to_output_file").join("raw.txt");

        run(
            [OsStr::new("decode"), OsStr::new("-o"), decoded.as_os_str()],
            Some(README_ENCODED.as_bytes()),
        );

        assert_eq!(fs::read(&decoded).unwrap(), README_DECODED);
    }

    #[test]
    fn should_exit_with_failure_when_input_is_not_z85() {
        let output = run(["decode"], Some(b"\x00\xFF\x10\x80\x7F"));

        assert!(
            !output.status.success(),
            "Expected failure, found {:?}",
            output.status
        );
    }

    #[test]
    fn should_hint_at_encode_when_input_is_not_z85() {
        let output = run(["decode"], Some(b"\x00\xFF\x10\x80\x7F"));

        let stderr = stderr_of(&output);
        assert!(
            stderr.contains("did you mean 'encode'?"),
            "stderr: {stderr}"
        );
    }

    #[test]
    fn should_not_touch_existing_output_file_when_decoding_fails() {
        let existing = scratch_dir("decode_failure_keeps_output").join("precious.txt");
        fs::write(&existing, "do not clobber").unwrap();

        run(
            [OsStr::new("decode"), OsStr::new("-o"), existing.as_os_str()],
            Some(b"not z85"),
        );

        assert_eq!(fs::read_to_string(&existing).unwrap(), "do not clobber");
    }
}

mod encode {
    use super::*;

    #[test]
    fn should_encode_stdin_to_text_that_decodes_back() {
        let encoded = run(["encode"], Some(README_DECODED)).stdout;

        let decoded = run(["decode"], Some(&encoded)).stdout;

        assert_eq!(decoded, README_DECODED);
    }

    #[test]
    fn should_roundtrip_binary_file_through_output_files() {
        let dir = scratch_dir("encode_roundtrip_binary_file");
        let (raw, encoded, decoded) = (
            dir.join("raw.bin"),
            dir.join("enc.txt"),
            dir.join("out.bin"),
        );
        let original: Vec<u8> = (0..=255).cycle().take(100_000).collect();
        fs::write(&raw, &original).unwrap();

        run(
            [
                OsStr::new("encode"),
                raw.as_os_str(),
                OsStr::new("-o"),
                encoded.as_os_str(),
            ],
            None,
        );
        run(
            [
                OsStr::new("decode"),
                encoded.as_os_str(),
                OsStr::new("-o"),
                decoded.as_os_str(),
            ],
            None,
        );

        assert_eq!(fs::read(&decoded).unwrap(), original);
    }

    #[test]
    fn should_accept_long_output_flag() {
        let encoded = scratch_dir("encode_long_output_flag").join("enc.txt");

        run(
            [
                OsStr::new("encode"),
                OsStr::new("--output"),
                encoded.as_os_str(),
            ],
            Some(README_DECODED),
        );

        assert!(
            encoded.exists(),
            "Expected {} to be created",
            encoded.display()
        );
    }

    // Only Linux filesystems accept file names that are not valid UTF-8.
    #[cfg(target_os = "linux")]
    #[test]
    fn should_accept_non_utf8_file_names() {
        use std::os::unix::ffi::OsStrExt;

        let input =
            scratch_dir("encode_non_utf8_file_name").join(OsStr::from_bytes(b"bad\xFFname"));
        fs::write(&input, README_DECODED).unwrap();

        let output = run([OsStr::new("encode"), input.as_os_str()], None);

        assert!(output.status.success(), "stderr: {}", stderr_of(&output));
    }
}

mod arguments {
    use super::*;

    #[test]
    fn should_name_the_missing_input_file_in_the_error() {
        let output = run(["encode", "definitely-not-here.bin"], None);

        let stderr = stderr_of(&output);
        assert!(
            stderr.contains("cannot read 'definitely-not-here.bin'"),
            "stderr: {stderr}"
        );
    }

    #[test]
    fn should_name_the_input_path_when_it_is_a_directory() {
        let dir = scratch_dir("arguments_input_is_directory");

        let output = run([OsStr::new("encode"), dir.as_os_str()], None);

        let stderr = stderr_of(&output);
        assert!(
            stderr.contains(&dir.display().to_string()),
            "stderr: {stderr}"
        );
    }

    #[test]
    fn should_exit_with_failure_for_unknown_command() {
        let output = run(["frobnicate"], None);

        assert!(
            !output.status.success(),
            "Expected failure, found {:?}",
            output.status
        );
    }

    #[test]
    fn should_reject_a_third_positional_argument() {
        let output = run(["encode", "a.txt", "b.txt"], None);

        assert!(
            !output.status.success(),
            "Expected failure, found {:?}",
            output.status
        );
    }

    #[test]
    fn should_print_usage_for_help_flag() {
        let output = run(["--help"], None);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("USAGE:"), "stdout: {stdout}");
    }

    #[test]
    fn should_document_that_decode_accepts_a_file() {
        let output = run(["--help"], None);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("byteme decode [<file>]"),
            "stdout: {stdout}"
        );
    }
}
