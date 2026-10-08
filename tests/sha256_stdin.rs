use std::io::Write;
use std::process::{Command, Output, Stdio};

const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn verify(manifest: &str, data: &[u8], flags: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sz-sha256"))
        .args(["--check", manifest, "--threads", "1"])
        .args(flags)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(data).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn reads_a_checksum_manifest_from_stdin() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input");
    std::fs::write(&input, b"abc").unwrap();
    let manifest = format!("{ABC}  {}\n", input.display());
    let result = verify("-", manifest.as_bytes(), &[]);
    assert!(result.status.success(), "{:?}", result);
    assert_eq!(
        result.stdout,
        format!("{}: OK\n", input.display()).as_bytes()
    );
}

#[test]
fn verifies_stdin_data_from_a_named_manifest() {
    let directory = tempfile::tempdir().unwrap();
    let manifest = directory.path().join("checksums");
    std::fs::write(&manifest, format!("{ABC}  -\n{EMPTY}  -\n")).unwrap();
    let result = verify(manifest.to_str().unwrap(), b"abc", &[]);
    assert!(result.status.success(), "{:?}", result);
    assert_eq!(result.stdout, b"-: OK\n-: OK\n");
    std::fs::write(&manifest, format!("{ABC}  -\n{ABC}  -\n")).unwrap();
    let result = verify(manifest.to_str().unwrap(), b"abc", &[]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stdout, b"-: OK\n-: FAILED\n");
}

#[test]
fn a_stdin_manifest_cannot_also_verify_stdin_data() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input");
    std::fs::write(&input, b"abc").unwrap();
    let manifest = format!("{ABC}  {}\n{EMPTY}  -\n", input.display());
    let result = verify("-", manifest.as_bytes(), &[]);
    assert!(result.status.success(), "{:?}", result);
    assert!(String::from_utf8_lossy(&result.stderr).contains("not checksum lines"));
    let strict = verify("-", manifest.as_bytes(), &["--strict"]);
    assert_eq!(strict.status.code(), Some(1));
}
