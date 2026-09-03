//! #3077: two counts over a population that could silently shrink.
//!
//! `t27c deadcode --repo` walks `specs/` and `compiler/` and prints
//! `Total functions`, `Potentially dead` and `Dead ratio`. Three drops sat
//! between the walk and the count -- a directory that will not open, a file that
//! will not read, and a spec whose `parse_ast` returns Err -- and each
//! contributed 0 to BOTH accumulators. An unparseable spec therefore shrank the
//! denominator, and a shrinking denominator makes the ratio go UP.
//!
//! Measured after the fix: **666 walked, 76 did not parse, 590 counted**. The
//! census had been reporting 4593 functions and 13.5% over 590 files while
//! naming 666 nowhere.
//!
//! `t27c backlog` (which is `service::run_depth`) walks with the same loop as
//! `t27c corpus` and had no empty-population guard, five hundred lines from the
//! one `corpus` was given in #3025 -- whose comment states the rule: the guard is
//! on the COUNT, not on the path, because an absent directory and an empty one
//! reach `read_dir(..).else { continue }` identically.

use std::process::Command;

fn scratch(tag: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("t27c-denominator-{tag}-{}-{n}", std::process::id()))
}

fn t27c(cwd: &std::path::Path, args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run t27c");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr),
    )
}

fn repo_root() -> std::path::PathBuf {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d
}

/// The census must say how many specs it never opened, so the denominator can be
/// audited rather than trusted.
#[test]
fn the_dead_code_census_names_what_it_skipped() {
    let (code, text) = t27c(&repo_root(), &["deadcode", "--repo"]);
    assert_eq!(code, Some(0), "{}", &text[text.len().saturating_sub(400)..]);
    for line in ["Specs walked:", "did not parse:", "Specs counted:"] {
        assert!(text.contains(line), "missing {line:?} in the summary");
    }
    // The ratio must carry its denominator on the same line, so quoting the
    // percentage without the population takes deliberate effort.
    assert!(
        text.contains("Dead ratio:") && text.contains("of") && text.contains("specs"),
        "the ratio must name the population it is over"
    );
    // And the skipped count must be REAL here: this repository has specs that do
    // not parse, and a zero would mean the counter is not wired.
    let skipped: u64 = text
        .lines()
        .find(|l| l.contains("did not parse:"))
        .and_then(|l| l.split_whitespace().last())
        .and_then(|n| n.parse().ok())
        .expect("a did-not-parse count");
    assert!(
        skipped > 0,
        "this tree is known to hold specs that do not parse; a 0 means the counter never fires"
    );
}

/// `backlog` walks the same loop as `corpus` and must refuse the same way.
#[test]
fn backlog_refuses_a_population_of_zero() {
    let root = scratch("backlog");
    std::fs::create_dir_all(root.join("specs")).expect("scratch");

    let (code, text) = t27c(&root, &["backlog", "--specs-dir", "specs"]);
    assert_eq!(code, Some(2), "an empty spec tree must refuse:\n{text}");
    assert!(text.contains("REFUSED"), "{text}");
    // Named for the subcommand the user typed. `t27c depth` is a different
    // command taking a file, and sending the reader there is the wrong subject.
    assert!(
        text.contains("backlog:") && !text.contains("depth: REFUSED"),
        "the refusal must name the subcommand that was run:\n{text}"
    );

    let (code, _) = t27c(&root, &["backlog", "--specs-dir", "nosuchdir"]);
    assert_eq!(code, Some(2), "an absent --specs-dir reaches the same guard");

    let _ = std::fs::remove_dir_all(&root);
}

/// THE CONTROL. Without it a command that refuses everything passes both tests
/// above -- and clap's own usage error is also exit 2, which is how the first
/// run of this suite read three passes off a subcommand that does not exist.
#[test]
fn backlog_still_produces_a_table_on_a_real_tree() {
    let (code, text) = t27c(&repo_root(), &["backlog", "--specs-dir", "specs", "--limit", "3"]);
    assert_eq!(code, Some(0), "{}", &text[text.len().saturating_sub(400)..]);
    assert!(!text.contains("REFUSED"), "a real tree must not be refused:\n{text}");
    assert!(
        text.contains("Depth is a PROXY"),
        "the table's own footer must be there, so this asserts the command RAN"
    );
}
