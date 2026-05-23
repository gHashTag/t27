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

fn run_gen(extra: &[&str]) -> (bool, String, String) {
    let mut args = vec!["host-weight-gen"];
    args.extend(extra);
    run(&args)
}

#[test]
fn weight_gen_default_succeeds() {
    let (ok, stdout, _) = run_gen(&[]);
    assert!(ok, "should succeed with defaults");
    assert!(!stdout.trim().is_empty());
}

#[test]
fn weight_gen_all_n() {
    let (ok, stdout, _) = run_gen(&["--pattern=all-n"]);
    assert!(ok);
    for line in stdout.trim().lines() {
        assert_eq!(line, "0x0000000000000000", "all-n should be zero: {line}");
    }
}

#[test]
fn weight_gen_all_p() {
    let (ok, stdout, _) = run_gen(&["--pattern=all-p"]);
    assert!(ok);
    for line in stdout.trim().lines() {
        assert!(line.starts_with("0x"), "expected hex: {line}");
    }
}

#[test]
fn weight_gen_all_z() {
    let (ok, stdout, _) = run_gen(&["--pattern=all-z"]);
    assert!(ok);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn weight_gen_alternating() {
    let (ok, stdout, _) = run_gen(&["--pattern=alternating"]);
    assert!(ok);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn weight_gen_phi_sequence() {
    let (ok, stdout, _) = run_gen(&["--pattern=phi-sequence"]);
    assert!(ok);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn weight_gen_seeded_random() {
    let (ok, stdout, _) = run_gen(&["--pattern=seeded-random:42"]);
    assert!(ok);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn weight_gen_custom_neurons() {
    let (ok, stdout, _) = run_gen(&["--neurons=8"]);
    assert!(ok);
    let lines = stdout.trim().lines().count();
    assert_eq!(lines, 8 * 4, "8 neurons x 4 chunks = 32 words");
}

#[test]
fn weight_gen_custom_chunks() {
    let (ok, stdout, _) = run_gen(&["--chunks=2", "--neurons=3"]);
    assert!(ok);
    let lines = stdout.trim().lines().count();
    assert_eq!(lines, 6, "3 neurons x 2 chunks = 6 words");
}

#[test]
fn weight_gen_invalid_pattern_fails() {
    let (ok, _, stderr) = run_gen(&["--pattern=bogus"]);
    assert!(!ok);
    assert!(stderr.contains("invalid pattern"), "stderr = {stderr}");
}

#[test]
fn weight_gen_zero_neurons_fails() {
    let (ok, _, _stderr) = run_gen(&["--neurons=0"]);
    assert!(!ok);
}

#[test]
fn weight_gen_deterministic_default() {
    let (ok1, s1, _) = run_gen(&[]);
    let (ok2, s2, _) = run_gen(&[]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2);
}

#[test]
fn weight_gen_seeded_deterministic() {
    let (ok1, s1, _) = run_gen(&["--pattern=seeded-random:99"]);
    let (ok2, s2, _) = run_gen(&["--pattern=seeded-random:99"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2);
}

#[test]
fn weight_gen_seeded_different_seeds_differ() {
    let (ok1, s1, _) = run_gen(&["--pattern=seeded-random:1"]);
    let (ok2, s2, _) = run_gen(&["--pattern=seeded-random:2"]);
    assert!(ok1 && ok2);
    assert_ne!(s1, s2);
}

#[test]
fn weight_gen_output_is_hex_words() {
    let (ok, stdout, _) = run_gen(&[]);
    assert!(ok);
    for line in stdout.trim().lines() {
        assert!(line.starts_with("0x") && line.len() == 18, "expected 0x + 16 hex: {line}");
    }
}

#[test]
fn weight_gen_single_line_per_word() {
    let (ok, stdout, _) = run_gen(&["--neurons=1", "--chunks=1"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn weight_gen_no_stderr_on_success() {
    let (ok, _, stderr) = run_gen(&[]);
    assert!(ok);
    assert!(stderr.trim().is_empty(), "stderr = {stderr}");
}

#[test]
fn weight_gen_help_lists_flags() {
    let (ok, stdout, _) = run_gen(&["--help"]);
    assert!(ok);
    for flag in ["--neurons", "--chunks", "--pattern"] {
        assert!(stdout.contains(flag), "missing {flag}");
    }
}

#[test]
fn weight_gen_help_mentions_wave_45() {
    let (ok, stdout, _) = run_gen(&["--help"]);
    assert!(ok);
    assert!(stdout.contains("Wave 45") || stdout.contains("R-HT-2"), "expected Wave 45: {stdout}");
}
