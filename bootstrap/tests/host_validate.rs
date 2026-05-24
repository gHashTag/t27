use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to spawn t27c");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn validate_default_pattern_succeeds() {
    let (ok, stdout, _) = run(&["host-validate"]);
    assert!(ok);
    assert!(stdout.starts_with("OK "));
}

#[test]
fn validate_reports_word_count() {
    let (ok, stdout, _) = run(&["host-validate", "--neurons=8", "--chunks=2"]);
    assert!(ok);
    assert!(stdout.contains("words=16"));
}

#[test]
fn validate_json_output() {
    let (ok, stdout, _) = run(&["host-validate", "--json"]);
    assert!(ok);
    let v: serde_json::Value = stdout.trim().parse().unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["errors"], 0);
}

#[test]
fn validate_json_has_details() {
    let (ok, stdout, _) = run(&["host-validate", "--json"]);
    assert!(ok);
    let v: serde_json::Value = stdout.trim().parse().unwrap();
    assert!(v.get("total_words").is_some());
    assert!(v.get("error_details").is_some());
    assert!(v.get("warning_details").is_some());
}

#[test]
fn validate_hex_words_valid() {
    let (ok, stdout, _) = run(&["host-validate", "--words=0x0,0x0"]);
    assert!(ok);
    assert!(stdout.contains("words=2"));
}

#[test]
fn validate_hex_words_invalid() {
    let (ok, _, stderr) = run(&["host-validate", "--words=0xFFFFFFFFFFFFFFFF"]);
    assert!(!ok);
    assert!(stderr.contains("validation failed"));
}

#[test]
fn validate_file_input() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("words.txt");
    std::fs::write(&path, "0x0\n0x0\n").unwrap();
    let (ok, stdout, _) = run(&["host-validate", &format!("--words=@{}", path.display())]);
    assert!(ok);
    assert!(stdout.contains("words=2"));
}

#[test]
fn validate_file_with_bad_word() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.txt");
    std::fs::write(&path, "0xFFFFFFFFFFFFFFFF\n").unwrap();
    let (ok, _, stderr) = run(&["host-validate", &format!("--words=@{}", path.display())]);
    assert!(!ok);
    assert!(stderr.contains("validation failed"));
}

#[test]
fn validate_custom_pattern() {
    let (ok, stdout, _) = run(&["host-validate", "--pattern=all-n"]);
    assert!(ok);
    assert!(stdout.contains("OK"));
}

#[test]
fn validate_seeded_pattern() {
    let (ok, stdout, _) = run(&["host-validate", "--pattern=seeded-random:42"]);
    assert!(ok);
}

#[test]
fn validate_deterministic() {
    let (_, s1, _) = run(&["host-validate"]);
    let (_, s2, _) = run(&["host-validate"]);
    assert_eq!(s1, s2);
}

#[test]
fn validate_help_lists_flags() {
    let (ok, stdout, _) = run(&["host-validate", "--help"]);
    assert!(ok);
    for flag in ["--words", "--pattern", "--neurons", "--chunks", "--json"] {
        assert!(stdout.contains(flag), "missing {}", flag);
    }
}

#[test]
fn validate_help_mentions_wave_52() {
    let (ok, stdout, _) = run(&["host-validate", "--help"]);
    assert!(ok);
    assert!(stdout.contains("Wave 52") || stdout.contains("R-HS-7"), "stdout={}", stdout);
}

#[test]
fn validate_no_stderr_on_success() {
    let (ok, _, stderr) = run(&["host-validate"]);
    assert!(ok);
    assert!(stderr.trim().is_empty());
}

#[test]
fn validate_invalid_pattern_fails() {
    let (ok, _, stderr) = run(&["host-validate", "--pattern=bogus"]);
    assert!(!ok);
}

#[test]
fn validate_bad_hex_fails() {
    let (ok, _, stderr) = run(&["host-validate", "--words=zzzz"]);
    assert!(!ok);
}
