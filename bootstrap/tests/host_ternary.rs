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

fn run_pack(trits: &str) -> (bool, String, String) {
    run(&["host-pack", &format!("--trits={}", trits)])
}

// -- host-pack --

#[test]
fn pack_single_n() {
    let (ok, stdout, _) = run_pack("-1");
    assert!(ok);
    assert!(stdout.contains("0x0000000000000000"), "stdout = {stdout}");
}

#[test]
fn pack_single_z() {
    let (ok, stdout, _) = run_pack("0");
    assert!(ok);
    assert!(stdout.contains("0x0000000000000001"), "stdout = {stdout}");
}

#[test]
fn pack_single_p() {
    let (ok, stdout, _) = run_pack("1");
    assert!(ok);
    assert!(stdout.contains("0x0000000000000002"), "stdout = {stdout}");
}

#[test]
fn pack_pn_pair() {
    let (ok, stdout, _) = run_pack("1,-1");
    assert!(ok);
    assert!(stdout.contains("0x0000000000000002"), "stdout = {stdout}");
}

#[test]
fn pack_27_trits_one_word() {
    let trits: Vec<String> = (0..27).map(|i| format!("{}", (i % 3) as i8 - 1)).collect();
    let (ok, stdout, _) = run_pack(&trits.join(","));
    assert!(ok);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 1, "27 trits should produce 1 word");
}

#[test]
fn pack_54_trits_two_words() {
    let trits: Vec<String> = (0..54).map(|i| format!("{}", (i % 3) as i8 - 1)).collect();
    let (ok, stdout, _) = run_pack(&trits.join(","));
    assert!(ok);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2, "54 trits should produce 2 words");
}

#[test]
fn pack_invalid_trit_fails() {
    let (ok, _, stderr) = run_pack("1,2,3");
    assert!(!ok);
    assert!(stderr.contains("invalid trit"));
}

#[test]
fn pack_empty_fails() {
    let (ok, _, _stderr) = run_pack("");
    assert!(!ok);
}

#[test]
fn pack_output_is_hex_words() {
    let (ok, stdout, _) = run_pack("1,0,-1");
    assert!(ok);
    for line in stdout.trim().lines() {
        assert!(line.starts_with("0x"), "expected hex word: {line}");
    }
}

fn run_unpack(words: &str) -> (bool, String, String) {
    run(&["host-unpack", &format!("--words={}", words)])
}

// -- host-unpack --

#[test]
fn unpack_single_n() {
    let (ok, stdout, _) = run_unpack("0x0");
    assert!(ok);
    assert!(stdout.trim().starts_with("-1"), "stdout = {stdout}");
}

#[test]
fn unpack_single_z() {
    let (ok, stdout, _) = run_unpack("0x1");
    assert!(ok);
    let trits: Vec<&str> = stdout.trim().split(',').collect();
    assert_eq!(trits[0], "0");
}

#[test]
fn unpack_single_p() {
    let (ok, stdout, _) = run_unpack("0x2");
    assert!(ok);
    let trits: Vec<&str> = stdout.trim().split(',').collect();
    assert_eq!(trits[0], "1");
}

#[test]
fn unpack_produces_27_trits_per_word() {
    let (ok, stdout, _) = run_unpack("0x0");
    assert!(ok);
    let count = stdout.trim().split(',').count();
    assert_eq!(count, 27);
}

#[test]
fn unpack_two_words_produces_54_trits() {
    let (ok, stdout, _) = run_unpack("0x0,0x0");
    assert!(ok);
    let count = stdout.trim().split(',').count();
    assert_eq!(count, 54);
}

#[test]
fn unpack_invalid_hex_fails() {
    let (ok, _, stderr) = run_unpack("zzzz");
    assert!(!ok);
    assert!(stderr.contains("invalid hex"));
}

#[test]
fn unpack_empty_fails() {
    let (ok, _, _stderr) = run_unpack("");
    assert!(!ok);
}

// -- round-trip: pack then unpack --

#[test]
fn roundtrip_pack_unpack() {
    let trits = "-1,0,1,0,-1,1,1,-1,0";
    let (ok1, packed, _) = run_pack(trits);
    assert!(ok1);
    let hex_words = packed.trim().lines().collect::<Vec<_>>().join(",");
    let (ok2, unpacked, _) = run_unpack(&hex_words);
    assert!(ok2);
    let original: Vec<&str> = trits.split(',').collect();
    let result: Vec<&str> = unpacked.trim().split(',').collect();
    for (i, o) in original.iter().zip(result.iter()) {
        assert_eq!(i, o, "mismatch: original={i} result={o}");
    }
}

#[test]
fn roundtrip_full_27_trits() {
    let trits: Vec<String> = (0..27).map(|i| format!("{}", (i % 3) as i8 - 1)).collect();
    let trits_str = trits.join(",");
    let (ok1, packed, _) = run_pack(&trits_str);
    assert!(ok1);
    let hex = packed.trim();
    let (ok2, unpacked, _) = run_unpack(hex);
    assert!(ok2);
    let result: Vec<&str> = unpacked.trim().split(',').collect();
    assert_eq!(result.len(), 27);
    for (i, (o, r)) in trits.iter().zip(result.iter()).enumerate() {
        assert_eq!(o.as_str(), *r, "trit {i} mismatch");
    }
}

// -- determinism --

#[test]
fn pack_deterministic() {
    let (ok1, s1, _) = run_pack("1,0,-1,1,0");
    let (ok2, s2, _) = run_pack("1,0,-1,1,0");
    assert!(ok1 && ok2);
    assert_eq!(s1, s2);
}
