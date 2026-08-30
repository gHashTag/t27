//! The corpus table measured Zig with `zig build-obj`, which resolves
//! identifiers and never Sema-analyses a function nothing references.
//!
//! On 2026-08-31 a repair unblocked 32 specs under `zig test --test-no-exec`
//! and moved the `zig_build` column by exactly **0** — correctly, because
//! build-obj cannot see a defect inside a body. Without a second column that
//! repair had no number in the report it belongs to, and no future body defect
//! could redden a ratchet or be credited for being fixed.
//!
//! These tests are cheap on purpose: a small `--limit`, and they assert the
//! SHAPE and the INVARIANT rather than any particular count, because counts move
//! every time the emitter improves and a test pinned to one would be re-blessed
//! rather than read.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d
}

fn zig_present() -> bool {
    Command::new("zig")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn corpus_json(limit: &str) -> Option<serde_json::Value> {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["corpus", "--limit", limit, "--json"])
        .current_dir(repo_root())
        .output()
        .expect("run t27c corpus");
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().last()?.trim();
    serde_json::from_str(line).ok()
}

/// bodies ⊆ build ⊆ gen. A file the deeper ruler accepts must also pass the
/// shallower one, because analysing a declaration is strictly more work than
/// resolving its name. A violation means one of the two rulers is broken, not
/// that the compiler improved.
#[test]
fn the_deeper_ruler_is_a_subset_of_the_shallower_one() {
    if !zig_present() {
        eprintln!("zig not on PATH -- SKIPPED, and saying so rather than passing silently");
        return;
    }
    let v = corpus_json("60").expect("corpus --json emitted no parseable line");
    let gen = v["zig_gen"].as_u64().expect("zig_gen");
    let build = v["zig_build"].as_u64().expect("zig_build");
    let bodies = v["zig_bodies"].as_u64().expect("zig_bodies");
    assert!(
        bodies <= build && build <= gen,
        "expected bodies <= build <= gen, got {bodies} <= {build} <= {gen}"
    );
}

/// The column has to reach the JSON, which is what wave-to-wave comparison
/// reads. A row printed only in the human table is a number nothing can diff.
#[test]
fn the_column_is_in_the_json_not_only_the_table() {
    if !zig_present() {
        eprintln!("zig not on PATH -- SKIPPED");
        return;
    }
    let v = corpus_json("12").expect("corpus --json emitted no parseable line");
    assert!(
        v.get("zig_bodies").is_some(),
        "zig_bodies missing from the JSON: {v}"
    );
}

/// And it has to reach `--per-spec`, which is the file an actual before/after
/// diff is taken over. Zig now carries THREE digits; a reader keyed to two
/// would silently drop the new one.
#[test]
fn per_spec_carries_three_zig_digits_and_says_so_in_its_header() {
    if !zig_present() {
        eprintln!("zig not on PATH -- SKIPPED");
        return;
    }
    let dir = std::env::temp_dir().join(format!("t27-corpuscols-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("per-spec.txt");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["corpus", "--limit", "12", "--per-spec"])
        .arg(&path)
        .current_dir(repo_root())
        .output()
        .expect("run t27c corpus");
    assert!(out.status.success(), "corpus failed");
    let text = std::fs::read_to_string(&path).expect("per-spec file");
    let _ = std::fs::remove_dir_all(&dir);

    let header = text.lines().next().expect("header");
    assert!(
        header.contains("zig(gen,build,bodies)"),
        "the header must name the third digit: {header}"
    );
    let row = text
        .lines()
        .find(|l| !l.starts_with('#') && l.contains('\t'))
        .expect("at least one row");
    let zig_field = row.split('\t').nth(1).expect("zig field");
    assert_eq!(
        zig_field.len(),
        3,
        "expected three Zig digits, got {zig_field:?} in row {row:?}"
    );
    assert!(
        zig_field.chars().all(|c| c == '0' || c == '1'),
        "digits only: {zig_field:?}"
    );
}
