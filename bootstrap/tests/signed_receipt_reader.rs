//! R3-1, the tool half (#7332): `t27c receipt-key` and `t27c run-record
//! --challenge/--require-level`, judged by specs/verified/signed_receipt.t27.
//!
//! The receipts here are signed by THIS test with ed25519-dalek over a message
//! the test builds from the spec's own words (domain line, then nine
//! `name=<JSON text>` lines) -- a second, independent writer of the format, so
//! a reader that drifted from the spec would fail here even if the tool's own
//! writer drifted the same way.
//!
//! Exit codes: 0 citable at the required level, 1 not, 2 REFUSED.

use std::process::Command;

fn scratch(tag: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("t27c-signed-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

fn t27c(cwd: &std::path::Path, key: &std::path::Path, args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(args)
        .env("T27_RECEIPT_KEY", key)
        .current_dir(cwd)
        .output()
        .expect("run t27c");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr),
    )
}

const SEAL_IMAGE: &str = "sha256:face";
const BUILT_BY: &str = "t27c-bootstrap@9.9.9+deadbee";
const CHALLENGE: &str = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";
const OTHER: &str = "33333333333333333333333333333333";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// A repository (with `.git`) holding one spec and its seal, plus a key made
/// by `t27c receipt-key init` OUTSIDE it. Returns (repo, key path, init output).
fn tree(tag: &str) -> (std::path::PathBuf, std::path::PathBuf, String) {
    let base = scratch(tag);
    let root = base.join("repo");
    for d in [".git", "specs/fpga", ".trinity/seals", ".trinity/receipts"] {
        std::fs::create_dir_all(root.join(d)).expect("scratch dirs");
    }
    std::fs::write(root.join("specs/fpga/link.t27"), "module Link;\n").expect("spec");
    std::fs::write(
        root.join(".trinity/seals/fpga_Link.json"),
        format!("{{\"module\":\"Link\",\"spec_path\":\"specs/fpga/link.t27\",\"gen_hash_verilog\":\"{SEAL_IMAGE}\",\"built_by\":\"{BUILT_BY}\"}}\n"),
    )
    .expect("seal");
    let key = base.join("home/receipt-ed25519.key");
    let (code, text) = t27c(&root, &key, &["receipt-key", "init"]);
    assert_eq!(code, Some(0), "{text}");
    (root, key, text)
}

/// One complete receipt for the scratch spec, signed (when `key` is given)
/// over the spec's message with `nonce` as the signed nonce.
fn receipt(root: &std::path::Path, name: &str, nonce: Option<&str>, key: Option<&std::path::Path>) {
    use ed25519_dalek::Signer;
    let vals: [(&str, String); 9] = [
        ("device_record", "\"--busdev-num 1:4\"".into()),
        ("full_idcode", "\"idcode 0x03636093\"".into()),
        ("verdict_word", "0".into()),
        ("seal_hash", format!("\"{SEAL_IMAGE}\"")),
        ("seeds", "[7]".into()),
        ("toolchain", format!("\"{BUILT_BY}\"")),
        ("spec", "\"specs/fpga/link.t27\"".into()),
        ("utc_unix", "1770000000".into()),
        ("nonce", nonce.map(|n| format!("\"{n}\"")).unwrap_or_else(|| "null".into())),
    ];
    let mut body: Vec<String> = vals.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
    if let Some(key) = key {
        let seed_hex = std::fs::read_to_string(key).expect("key");
        let seed: Vec<u8> = (0..64)
            .step_by(2)
            .map(|i| u8::from_str_radix(&seed_hex.trim()[i..i + 2], 16).unwrap())
            .collect();
        let sk = ed25519_dalek::SigningKey::from_bytes(&seed.try_into().unwrap());
        let mut msg = String::from("t27-receipt-v1\n");
        for (k, v) in &vals {
            msg.push_str(&format!("{k}={v}\n"));
        }
        let public = sk.verifying_key().to_bytes();
        use sha2::Digest;
        let id = hex(&sha2::Sha256::digest(public))[..16].to_string();
        body.push(format!("\"key_id\":\"{id}\""));
        body.push(format!("\"signature\":\"{}\"", hex(&sk.sign(msg.as_bytes()).to_bytes())));
    }
    std::fs::write(root.join(".trinity/receipts").join(name), format!("{{{}}}\n", body.join(",")))
        .expect("receipt");
}

const SPEC: &str = "specs/fpga/link.t27";

/// init writes the private key outside the tree, the public half under
/// .trinity/keys, never prints the private half, and never overwrites.
#[test]
fn init_registers_the_key_and_never_prints_or_overwrites_it() {
    let (root, key, text) = tree("init");
    let seed = std::fs::read_to_string(&key).expect("key written");
    assert!(!text.contains(seed.trim()), "the private key must never be printed: {text}");
    let pubs: Vec<_> = std::fs::read_dir(root.join(".trinity/keys")).unwrap().collect();
    assert_eq!(pubs.len(), 1, "one public key registered");
    let (code, again) = t27c(&root, &key, &["receipt-key", "init"]);
    assert_eq!(code, Some(2), "{again}");
    assert!(again.contains("never overwritten"), "{again}");
    assert_eq!(std::fs::read_to_string(&key).unwrap(), seed);
    let (code, show) = t27c(&root, &key, &["receipt-key", "show"]);
    assert_eq!(code, Some(0), "{show}");
    assert!(show.contains("registered in .trinity/keys: yes"), "{show}");
    let inside = root.join(".trinity/receipt.key");
    let (code, refused) = t27c(&root, &inside, &["receipt-key", "init"]);
    assert_eq!(code, Some(2), "{refused}");
    assert!(!inside.exists(), "a refused init writes nothing");
    let _ = std::fs::remove_dir_all(root.parent().unwrap());
}

/// THE CONTROL: three receipts signed for this challenge -- the run is FRESH
/// and citable when the citation requires FRESH.
#[test]
fn three_receipts_signed_for_this_challenge_are_a_fresh_run() {
    let (root, key, _) = tree("fresh");
    for i in 1..=3 {
        receipt(&root, &format!("link-177000000{i}-{i}.json"), Some(CHALLENGE), Some(&key));
    }
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--challenge", CHALLENGE, "--require-level", "fresh"]);
    assert_eq!(code, Some(0), "{text}");
    assert!(text.contains("Authentication: FRESH"), "{text}");
    assert!(text.contains("may cite this run"), "{text}");
    let _ = std::fs::remove_dir_all(root.parent().unwrap());
}

/// THE REPLAY: the same genuine receipts offered to a verifier with another
/// challenge reach AUTHOR, not FRESH -- complete, but not citable at FRESH.
#[test]
fn receipts_for_another_challenge_are_authored_not_fresh() {
    let (root, key, _) = tree("replay");
    for i in 1..=3 {
        receipt(&root, &format!("link-177000000{i}-{i}.json"), Some(OTHER), Some(&key));
    }
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--challenge", CHALLENGE, "--require-level", "fresh"]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("Run complete: yes"), "{text}");
    assert!(text.contains("AUTH_NONCE_NOT_CHALLENGE 5"), "{text}");
    assert!(text.contains("NOT CITABLE at FRESH -- the run reaches only AUTHOR"), "{text}");
    // the same run is citable at AUTHOR
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--challenge", CHALLENGE, "--require-level", "author"]);
    assert_eq!(code, Some(0), "{text}");
    let _ = std::fs::remove_dir_all(root.parent().unwrap());
}

/// One unsigned receipt makes the run unsigned (a run is its weakest
/// receipt), and an edited signed receipt is no better than an unsigned one.
#[test]
fn one_unsigned_or_edited_receipt_lowers_the_whole_run() {
    let (root, key, _) = tree("weakest");
    receipt(&root, "link-1770000001-1.json", Some(CHALLENGE), Some(&key));
    receipt(&root, "link-1770000002-2.json", Some(CHALLENGE), Some(&key));
    receipt(&root, "link-1770000003-3.json", Some(CHALLENGE), None);
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--require-level", "author"]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("Authentication: NONE"), "{text}");
    assert!(text.contains("AUTH_UNSIGNED 1"), "{text}");
    // the default level is NONE: the same run stays citable, as before R3-1
    let (code, text) = t27c(&root, &key, &["run-record", SPEC]);
    assert_eq!(code, Some(0), "{text}");
    // re-sign the third, then edit one byte of a signed field
    receipt(&root, "link-1770000003-3.json", Some(CHALLENGE), Some(&key));
    let p = root.join(".trinity/receipts/link-1770000003-3.json");
    let t = std::fs::read_to_string(&p).unwrap().replacen("\"seeds\":[7]", "\"seeds\":[8]", 1);
    std::fs::write(&p, t).unwrap();
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--require-level", "author"]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("AUTH_BAD_SIGNATURE 3"), "{text}");
    let _ = std::fs::remove_dir_all(root.parent().unwrap());
}

/// FRESH without a challenge, a short challenge and an unknown level are
/// usage errors, not answers.
#[test]
fn a_fresh_requirement_needs_a_long_enough_challenge() {
    let (root, key, _) = tree("refuse");
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--require-level", "fresh"]);
    assert_eq!(code, Some(2), "{text}");
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--challenge", "abcd"]);
    assert_eq!(code, Some(2), "{text}");
    let (code, text) = t27c(&root, &key, &["run-record", SPEC, "--require-level", "device"]);
    assert_eq!(code, Some(2), "{text}");
    let _ = std::fs::remove_dir_all(root.parent().unwrap());
}
