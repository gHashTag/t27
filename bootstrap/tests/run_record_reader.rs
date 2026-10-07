//! R2-4, the tool half (#7072): `t27c run-record` reads the receipts a spec's
//! hardware runs wrote and judges them by specs/verified/run_record.t27
//! (#7061). The fixtures are hand-written receipt and seal JSON in temp dirs
//! -- the reader does not need the writer, and #7044 (which writes receipts)
//! can land in either order.
//!
//! Exit codes: 0 citable, 1 not citable, 2 REFUSED -- the reader never
//! crashes on a malformed receipt dir; it answers.
//!
//! The codes pinned here are run_record.t27's constants:
//! RUN_MISSING_NONE=0, RUN_TOO_FEW_RECEIPTS=1, RUN_RECEIPT_INCOMPLETE=2,
//! RUN_WORDS_DISAGREE=3, RUN_PRODUCER_MISMATCH=4 (placements_needed()=3), with
//! receipt.t27's six-field rule underneath (an unknown verdict word counts as
//! absent) and verdict.t27's consumption point (an incomplete run is no run
//! reference: INVALID_NO_RUN before any chain is read).
//!
//! The producer fact needs #7076's `built_by` on the seal: `t27c-bootstrap@
//! <version>+<git>` written by `seal --save`, which the receipt's `toolchain`
//! must equal verbatim. A seal minted before #7076 carries no `built_by` and
//! therefore matches no producer -- that is the honest reading, pinned below.

use std::process::Command;

fn scratch(tag: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("t27c-run-record-{tag}-{}-{n}", std::process::id()))
}

fn t27c(cwd: &std::path::Path, spec: &str) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["run-record", spec])
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

/// A scratch tree holding one spec and its seal, ready for receipts. Returns
/// the tree root; receipts are written by each test into .trinity/receipts/.
fn tree(tag: &str, built_by: Option<&str>) -> std::path::PathBuf {
    let root = scratch(tag);
    std::fs::create_dir_all(root.join("specs").join("fpga")).expect("scratch dirs");
    std::fs::create_dir_all(root.join(".trinity/seals")).expect("seals dir");
    std::fs::create_dir_all(root.join(".trinity/receipts")).expect("receipts dir");
    std::fs::write(root.join("specs/fpga/link.t27"), "module Link;\n").expect("spec");
    let mut seal = format!(
        "{{\"module\":\"Link\",\"spec_path\":\"specs/fpga/link.t27\",\"gen_hash_verilog\":\"{SEAL_IMAGE}\""
    );
    match built_by {
        Some(b) => seal.push_str(&format!(",\"built_by\":\"{b}\"")),
        None => seal.push_str(",\"built_by\":null"),
    }
    seal.push_str("}\n");
    std::fs::write(root.join(".trinity/seals/fpga_Link.json"), seal).expect("seal");
    root
}

/// One receipt. `word` is the raw verdict_word field: 0=PASS, 1=FAIL, anything
/// else is a word the vocabulary does not define. `seal_hash`/`seeds`/
/// `toolchain` override the complete-receipt defaults.
fn receipt(
    root: &std::path::Path,
    name: &str,
    word: u64,
    seal_hash: Option<&str>,
    seeds: Option<&str>,
    toolchain: Option<&str>,
) {
    let txt = format!(
        "{{\"device_record\":\"QMTech Wukong V1\",\"full_idcode\":\"0x03636093\",\"verdict_word\":{word},\"seal_hash\":{},\"seeds\":{},\"toolchain\":{},\"spec\":\"specs/fpga/link.t27\",\"utc_unix\":1770000000}}\n",
        match seal_hash {
            Some(h) => format!("\"{h}\""),
            None => "null".to_string(),
        },
        seeds.unwrap_or("[1,7,42]"),
        match toolchain {
            Some(t) => format!("\"{t}\""),
            None => format!("\"{BUILT_BY}\""),
        },
    );
    std::fs::write(root.join(".trinity/receipts").join(name), txt).expect("receipt");
}

/// THE CONTROL: three complete receipts, agreeing words, every toolchain the
/// cited seal's built_by -- one verified run, citable as a verdict's run
/// reference, exit 0.
#[test]
fn three_complete_agreeing_receipts_are_one_verified_run() {
    let root = tree("control", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 0, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(0), "{text}");
    assert!(text.contains("First missing: NONE (0)"), "{text}");
    assert!(text.contains("Run complete: yes"), "{text}");
    assert!(text.contains("may cite this run"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Count is judged first: below three, agreement is vacuous -- it cannot be
/// judged on a set the rule already refuses (run_record.t27).
#[test]
fn two_receipts_are_too_few_no_matter_what_they_say() {
    let root = tree("toofew", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_TOO_FEW_RECEIPTS (1)"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// No receipts at all is still a count, not a usage error: the answer is
/// TOO_FEW over a population of zero.
#[test]
fn an_empty_receipts_dir_is_zero_receipts_not_a_refusal() {
    let root = tree("empty", Some(BUILT_BY));
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("(no receipts name this spec)"), "{text}");
    assert!(text.contains("RUN_TOO_FEW_RECEIPTS (1)"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A receipt the six-field rule calls incomplete fails the run before words
/// are read, and the line names WHICH receipt.
#[test]
fn a_receipt_without_seeds_fails_the_run_before_words() {
    let root = tree("noseeds", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), Some("[]"), None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 0, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_RECEIPT_INCOMPLETE (2)"), "{text}");
    assert!(text.contains("link-1770000000-1.json"), "the broken receipt must be named:\n{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// An unknown verdict word never reaches the agreement test: receipt.t27
/// counts it absent, its receipt goes incomplete, and the answer is (2), not
/// (3) -- "a word the vocabulary does not define is not a stricter verdict, it
/// is a typo with authority".
#[test]
fn an_unknown_verdict_word_is_an_absent_word_not_a_disagreement() {
    let root = tree("typoword", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 7, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 1, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_RECEIPT_INCOMPLETE (2)"), "{text}");
    assert!(!text.contains("RUN_WORDS_DISAGREE"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Which word the receipts agree on is not the run record's business: agreeing
/// FAILs are one complete run recording a failure (failure_loop owns the rest),
/// so the reference is citable.
#[test]
fn agreeing_failures_are_still_one_complete_run() {
    let root = tree("agreeingfail", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 1, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 1, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 1, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(0), "{text}");
    assert!(text.contains("Run complete: yes"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// PASS and FAIL in one set: the words disagree, code 3, judged only after
/// every receipt passed the six-field rule.
#[test]
fn pass_and_fail_in_one_set_disagree() {
    let root = tree("disagree", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 1, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_WORDS_DISAGREE (3)"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Order pin: run_record.t27 judges agreement (3) BEFORE producers (4). With
/// both broken, the answer must name the words, not the producer -- a mutant
/// that swaps the last two judgments passes every other test in this file.
#[test]
fn disagreement_is_judged_before_producers_when_both_break() {
    let root = tree("order34", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 1, Some(SEAL_IMAGE), None, Some("someone-else"));
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_WORDS_DISAGREE (3)"), "{text}");
    assert!(!text.contains("RUN_PRODUCER_MISMATCH (4)"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A receipt naming a producer other than the cited seal's built_by is a claim
/// about someone else's work (R2-2): code 4, after agreement.
#[test]
fn a_toolchain_that_is_not_the_cited_seals_producer_fails_last() {
    let root = tree("otherproducer", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(
        &root,
        "link-1770000002-3.json",
        0,
        Some(SEAL_IMAGE),
        None,
        Some("t27c-bootstrap@9.9.9+cafe123"),
    );
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_PRODUCER_MISMATCH (4)"), "{text}");
    assert!(
        text.contains("link-1770000002-3.json") && text.contains("not the cited seal's"),
        "the offending receipt and the reason must be named:\n{text}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// #7076 is load-bearing: a seal minted before it carries no built_by, and no
/// receipt can match a producer it does not name. That is a mismatch, not an
/// exception -- the reader does not soften R2-2 for old seals.
#[test]
fn a_seal_without_built_by_matches_no_producer() {
    let root = tree("oldseal", None);
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 0, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_PRODUCER_MISMATCH (4)"), "{text}");
    assert!(text.contains("carries no built_by"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A receipt citing a seal this spec does not hold (a stale image hash, or
/// another spec's seal) has no producer to match: mismatch, named.
#[test]
fn a_receipt_citing_a_seal_the_spec_does_not_hold() {
    let root = tree("strangeseal", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 0, Some("sha256:other"), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_PRODUCER_MISMATCH (4)"), "{text}");
    assert!(text.contains("cites a seal this spec does not hold"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A null seal_hash is an incomplete receipt (receipt.t27's fourth field), not
/// a producer question.
#[test]
fn a_null_seal_hash_is_an_incomplete_receipt() {
    let root = tree("nullhash", Some(BUILT_BY));
    receipt(&root, "link-1770000000-1.json", 0, None, None, None);
    receipt(&root, "link-1770000001-2.json", 0, Some(SEAL_IMAGE), None, None);
    receipt(&root, "link-1770000002-3.json", 0, Some(SEAL_IMAGE), None, None);
    let (code, text) = t27c(&root, "specs/fpga/link.t27");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("RUN_RECEIPT_INCOMPLETE (2)"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A spec that does not exist is REFUSED (exit 2): judging receipts against a
/// typo would answer TOO_FEW over files nobody meant.
#[test]
fn a_spec_that_does_not_exist_is_refused() {
    let root = tree("nospec", Some(BUILT_BY));
    let (code, text) = t27c(&root, "specs/fpga/nosuch.t27");
    assert_eq!(code, Some(2), "{text}");
    assert!(text.contains("REFUSED"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}
