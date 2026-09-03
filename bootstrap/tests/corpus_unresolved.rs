//! #3025: a run that produced no usable numbers must not be able to exit 0.
//!
//! The corpus reading that opened the issue died with `No space left on device`
//! and reported a corpus-wide collapse in acceptance. `run_timed`'s failure to
//! create a capture file met `== Some(0)` at the call site IDENTICALLY to a
//! compile error, so every column fell to the floor and the run exited green.
//!
//! The unit tests in `bootstrap/src/service.rs` pin each reason at the source.
//! This one pins the CONSEQUENCE end to end, which nothing in-process can: the
//! command must refuse, print no percentages, write no `--per-spec` table, and
//! exit non-zero.
//!
//! The trigger used here is an EMPTY `PATH`, so `rustc`, `cc`, `zig` and
//! `iverilog` cannot be spawned. It is deterministic on any machine, unlike
//! ENOSPC. What it does not cover: a full volume, which arrives through
//! `Unresolved::HostIo` rather than `NotSpawned`. Both are machine-wide and
//! share the refusal path, and the unit tests cover the HostIo branch itself.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d
}

/// Keyed by a counter as well as the pid: every test in this binary shares the
/// pid, and these directories are removed at the end of each test.
fn scratch(tag: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "t27-corpus-unres-{tag}-{}-{}",
        std::process::id(),
        n
    ))
}

/// An existing, EMPTY directory as `PATH`. Not `PATH=""`: an empty string is a
/// set-but-empty search list whose treatment differs between libcs, while a real
/// directory with nothing in it fails every lookup everywhere.
fn corpus_with_no_tools(extra: &[&str], empty_dir: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["corpus", "--limit", "3"])
        .args(extra)
        .env("PATH", empty_dir)
        .current_dir(repo_root())
        .output()
        .expect("run t27c corpus")
}

#[test]
fn a_run_whose_tools_cannot_be_spawned_is_refused_and_prints_no_percentages() {
    let empty = scratch("path");
    std::fs::create_dir_all(&empty).expect("empty PATH dir");
    let out = corpus_with_no_tools(&[], &empty);
    let _ = std::fs::remove_dir_all(&empty);
    let text = String::from_utf8_lossy(&out.stdout).to_string();

    assert_eq!(
        out.status.code(),
        Some(2),
        "a run with a machine-wide non-verdict must exit 2, not 0. stdout:\n{text}"
    );
    assert!(
        text.contains("REFUSED"),
        "the refusal must say so in the human report:\n{text}"
    );
    assert!(
        text.contains("tool ABSENT"),
        "and must name the reason:\n{text}"
    );
    // The whole point. A percentage on screen in the shape of a measurement is
    // read as one, whatever paragraph stands next to it.
    for headline in ["ALL FOUR accept", "generates Zig", "and cc accepts it"] {
        assert!(
            !text.contains(headline),
            "a refused run must print NO acceptance column, found {headline:?} in:\n{text}"
        );
    }
}

/// The refused JSON must not be a `{"specs":N,"zig_build":0,...}` object. A
/// ratchet reading `zig_build` has to find NOTHING there, not a zero.
#[test]
fn the_refused_json_carries_no_acceptance_column_for_a_ratchet_to_read() {
    let empty = scratch("json");
    std::fs::create_dir_all(&empty).expect("empty PATH dir");
    let out = corpus_with_no_tools(&["--json"], &empty);
    let _ = std::fs::remove_dir_all(&empty);
    let text = String::from_utf8_lossy(&out.stdout).to_string();

    assert_eq!(out.status.code(), Some(2), "must exit 2. stdout:\n{text}");
    let line = text.lines().last().expect("a JSON line").trim();
    let v: serde_json::Value = serde_json::from_str(line).expect("parseable JSON");

    assert_eq!(
        v["refused"], "unresolved",
        "the object must say it is a refusal: {v}"
    );
    for k in [
        "zig_build",
        "c_build",
        "rust_build",
        "verilog_build",
        "all_four_build",
    ] {
        assert!(
            v.get(k).is_none(),
            "a refused run must not publish {k}: {v}"
        );
    }
    assert!(
        v["unresolved_not_spawned"].as_u64().unwrap_or(0) > 0,
        "and must publish the reason it refused: {v}"
    );
}

/// A file whose only purpose is to be diffed must not exist for a run that is
/// not a reading. TAP's `Bail out!` drops the plan for the same reason.
#[test]
fn a_refused_run_writes_no_per_spec_table() {
    let dir = scratch("perspec");
    std::fs::create_dir_all(&dir).expect("scratch");
    let empty = dir.join("emptypath");
    std::fs::create_dir_all(&empty).expect("empty PATH dir");
    let table = dir.join("per-spec.txt");

    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["corpus", "--limit", "3", "--per-spec"])
        .arg(&table)
        .env("PATH", &empty)
        .current_dir(repo_root())
        .output()
        .expect("run t27c corpus");
    let existed = table.exists();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(out.status.code(), Some(2), "must exit 2");
    assert!(
        !existed,
        "a refused run wrote a --per-spec table; a later diff would read it as a reading"
    );
}

/// The counts must be printed on a CLEAN run too. #2945 printed its timeout
/// count only under `if to > 0`, so a green run said nothing at all and a reader
/// could not tell "zero unresolved" from "this binary cannot count them".
#[test]
fn a_clean_run_still_prints_the_unresolved_block_with_its_zeros() {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["corpus", "--limit", "3"])
        .current_dir(repo_root())
        .output()
        .expect("run t27c corpus");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if out.status.code() != Some(0) {
        eprintln!(
            "this machine is missing a tool, so the corpus refuses -- SKIPPED, \
             and saying so rather than passing silently:\n{text}"
        );
        return;
    }
    assert!(
        text.contains("unresolved (no verdict)"),
        "the block must be printed unconditionally:\n{text}"
    );
    for reason in [
        "... timed out",
        "... tool ABSENT (no spawn)",
        "... harness I/O (ENOSPC)",
        "... killed by a signal",
    ] {
        assert!(
            text.contains(reason),
            "every reason must be named even at zero, missing {reason:?}:\n{text}"
        );
    }
}

/// THE CHANNEL #3025 ACTUALLY DIED ON: a refused WRITE, not a refused spawn.
///
/// `run_corpus` writes each generated artefact to `$TMPDIR/t27-corpus/c.rs`
/// (`.c`, `.zig`, `.v`) before handing it to a compiler. That write is guarded
/// by `.is_ok()` and used to have no `else`, so a full volume left every
/// `*_build` column false with nothing recorded anywhere.
///
/// Reached here by giving the CHILD its own `TMPDIR` whose `t27-corpus`
/// directory already exists with mode 0500. `create_dir_all` then succeeds on
/// an existing directory and the write inside it is refused -- EACCES rather
/// than ENOSPC, the same `Err` on the same call. Only the child's environment
/// is touched; nothing process-global is set, and `t27-runtimed` under the same
/// TMPDIR is still created writable, so the generators really do run.
///
/// WHAT THIS DOES NOT COVER: a volume that fills up PART WAY through a run, so
/// that early specs are measured and later ones are not. The refusal is
/// all-or-nothing per run, which is the safe direction, but the test only
/// exercises the case where every spec fails the same way.
#[cfg(unix)]
#[test]
fn a_scratch_directory_that_refuses_writes_refuses_the_run() {
    use std::os::unix::fs::PermissionsExt;

    let home = scratch("tmpdir");
    let corpus_tmp = home.join("t27-corpus");
    std::fs::create_dir_all(&corpus_tmp).expect("scratch t27-corpus");
    std::fs::set_permissions(&corpus_tmp, std::fs::Permissions::from_mode(0o500))
        .expect("chmod 0500");

    // Probed, not assumed: root writes through the mode bits.
    let reachable = std::fs::File::create(corpus_tmp.join("probe")).is_err();
    if !reachable {
        let _ = std::fs::set_permissions(&corpus_tmp, std::fs::Permissions::from_mode(0o700));
        let _ = std::fs::remove_dir_all(&home);
        eprintln!(
            "a read-only directory still accepts files here (running as root?) \
             -- SKIPPED, and saying so rather than passing silently"
        );
        return;
    }

    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["corpus", "--limit", "3"])
        .env("TMPDIR", &home)
        .current_dir(repo_root())
        .output()
        .expect("run t27c corpus");
    let _ = std::fs::set_permissions(&corpus_tmp, std::fs::Permissions::from_mode(0o700));
    let _ = std::fs::remove_dir_all(&home);
    let text = String::from_utf8_lossy(&out.stdout).to_string();

    assert_eq!(
        out.status.code(),
        Some(2),
        "a run that could not write its own scratch files must refuse. stdout:\n{text}"
    );
    // NOT `contains("harness I/O")`: that label is printed at zero too, so the
    // assertion would be satisfied by a refusal for a completely different
    // reason. The `first ...` line is emitted only when the reason occurred.
    assert!(
        text.contains("first harness I/O"),
        "the refusal must be FOR the harness I/O reason, and name the write:\n{text}"
    );
    assert!(
        text.contains("write c."),
        "and must name which artefact write was refused:\n{text}"
    );
    assert!(
        !text.contains("ALL FOUR accept"),
        "and must print no acceptance column:\n{text}"
    );
}

/// FOUR CALL SITES, FOUR RULES. The test above cannot tell them apart: a full
/// disk fails all four artefact writes at once, so deleting any ONE guard
/// leaves the refusal, and the reason, and even the `write c.` text intact.
/// Mutation said so -- removing the Rust guard alone SURVIVED it.
///
/// So fail exactly one write at a time, by pre-creating that artefact as a
/// DIRECTORY under the child's own `TMPDIR/t27-corpus`. `std::fs::write` to a
/// directory is `Err` (EISDIR) while the other three succeed, which no
/// permission or quota trick can isolate as cleanly.
///
/// WHAT THIS DOES NOT COVER: EISDIR is not ENOSPC. It is the same `Err` on the
/// same call, and it is the only cause a test can aim at ONE of the four.
#[test]
fn each_backend_artefact_write_is_guarded_at_its_own_call_site() {
    for (artefact, tool) in [
        ("c.rs", "write c.rs"),
        ("c.c", "write c.c"),
        ("c.zig", "write c.zig"),
        ("c.v", "write c.v"),
    ] {
        let home = scratch("eisdir");
        let corpus_tmp = home.join("t27-corpus");
        // The artefact path itself, as a directory.
        std::fs::create_dir_all(corpus_tmp.join(artefact)).expect("artefact as a dir");

        let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
            .args(["corpus", "--limit", "3"])
            .env("TMPDIR", &home)
            .current_dir(repo_root())
            .output()
            .expect("run t27c corpus");
        let _ = std::fs::remove_dir_all(&home);
        let text = String::from_utf8_lossy(&out.stdout).to_string();

        assert_eq!(
            out.status.code(),
            Some(2),
            "an unwritable {artefact} must refuse the run on its own. stdout:\n{text}"
        );
        assert!(
            text.contains(tool),
            "the refusal must name {tool:?}:\n{text}"
        );
        // And ONLY that one, which is what makes this test see a single
        // deleted guard rather than the class.
        for (_, other) in [
            ("c.rs", "write c.rs"),
            ("c.c", "write c.c"),
            ("c.zig", "write c.zig"),
            ("c.v", "write c.v"),
        ] {
            if other == tool {
                continue;
            }
            assert!(
                !text.contains(other),
                "only {tool:?} was made to fail, but {other:?} is named too:\n{text}"
            );
        }
    }
}

/// THE OTHER WAY TO PRODUCE NO NUMBERS: a population that was never asked.
///
/// The module docstring above states the rule as "a run that produced no usable
/// numbers must not be able to exit 0". That sentence was FALSE for this input
/// while every test above it passed: a corpus over zero specs printed
/// `{"specs":0,"verilog_build":0,...}` and exited 0 -- the constant 0 in the
/// format of a measurement, which is the exact thing the machine-wide refusal
/// exists to prevent, arriving through a door the refusal does not watch.
///
/// A mistyped `--specs-dir` reaches it identically: the walk opens the
/// directory with `read_dir(..).else { continue }`, so a path that does not
/// exist is indistinguishable from a tree with no specs in it. Both are the
/// same defect and both are covered here.
///
/// WHAT THIS DOES NOT COVER: a tree that holds specs the walk SKIPS (a
/// `scratch` directory, a non-`.t27` extension). Those reduce the population
/// silently without emptying it, and no count on its own can see that.
#[test]
fn a_corpus_over_zero_specs_refuses_instead_of_printing_zeros() {
    for (tag, dir_exists) in [("empty", true), ("absent", false)] {
        let home = scratch(tag);
        let specs = home.join("specs");
        if dir_exists {
            std::fs::create_dir_all(&specs).expect("empty spec tree");
        }

        let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
            .args(["corpus", "--json", "--specs-dir"])
            .arg(&specs)
            .current_dir(repo_root())
            .output()
            .expect("run t27c corpus");
        let _ = std::fs::remove_dir_all(&home);
        let text = String::from_utf8_lossy(&out.stdout).to_string();

        assert_eq!(
            out.status.code(),
            Some(2),
            "zero specs ({tag}) is not zero acceptance and must not exit 0. stdout:\n{text}"
        );
        // The load-bearing half. `refused` being present is not enough: the
        // ratchets read `verilog_build`, and a refusal that still carries that
        // key would be read as a measurement of 0 by every one of them.
        for key in [
            "\"verilog_build\"",
            "\"rust_build\"",
            "\"c_build\"",
            "\"zig_build\"",
            "\"all_four_build\"",
        ] {
            assert!(
                !text.contains(key),
                "a refusal ({tag}) must carry no acceptance key, but {key} is in:\n{text}"
            );
        }
        assert!(
            text.contains("\"refused\":\"no_specs\""),
            "and must say why it refused ({tag}):\n{text}"
        );
    }
}
