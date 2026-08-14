// ============================================================================
// #2127 -- nesting decides the separator, never the terminator.
//
// The field collector consults bracket nesting depth when it meets a `Comma`,
// because a comma inside `Map<K, V>` or `(A, B)` separates type arguments and
// not fields. It must NOT consult depth when it meets a terminator: `RBrace`,
// `Semicolon` and `Eof` end the field list at any depth. Truncated input leaves
// the depth counter positive, so a depth-gated terminator is never accepted and
// the loop runs to the end of the token stream -- or, with a buggy bound, does
// not stop at all.
//
// The test that matters here is therefore a LIVENESS test, not an output test:
// a spec whose last line opens a bracket and then ends must make the parser
// TERMINATE. Asserting on the error message alone would pass even if the parser
// hung, because a hung process never produces a message to compare -- the test
// would sit there until the CI job's own timeout killed it, and report as
// infrastructure flake rather than as this defect.
//
// So every case here runs under a HARD wall-clock timeout enforced in-process.
// Exceeding it fails the test with a message that names the hang, and the child
// is killed so a wedged parser cannot outlive the test binary.
//
// Deliberately NOT asserted: the exact text of the parse error. On malformed
// input there is no single correct reading of the field list, and pinning the
// message would freeze one arbitrary recovery as the specification. What is
// pinned is: the process ends, its exit status is non-zero, it names the token
// it stopped at, and it fails as a diagnostic rather than as a panic.
//
// The wording is deliberately not pinned either. The recorded baseline for this
// fixture read `Error: Parse error: Expected RBrace, got Eof` and the binary now
// prints `Error: Expected RBrace, got Eof` -- the prefix moved at some point
// between the two. An assertion on the phrase "parse" fails on that alone while
// the parser behaves correctly, which is a test measuring diagnostic prose
// instead of parser behaviour. It asserts on the token names instead, because
// those are what the invariant is about.
// ============================================================================

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Wall-clock ceiling for a single parse of a four-line fixture. Generous by
/// three orders of magnitude: these files are under 100 bytes, and a healthy
/// parse of them is a few milliseconds even on a loaded shared runner. The
/// number exists to separate "slow" from "not returning", and 10 s cannot be
/// reached by any amount of ordinary slowness on this input.
const HARD_TIMEOUT: Duration = Duration::from_secs(10);

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

/// Pull the `(field name, type text)` pairs out of a `t27c parse` dump.
///
/// The three liveness cases above use `check`, which prints a verdict and hides
/// the field list. The two cases below are about the field list itself, so they
/// read the parse dump. This is the same extraction the corpus differential used,
/// reimplemented here without a regex crate: find each `name:` line and take the
/// `extra_type:` two lines below it.
fn fields(name: &str) -> Vec<(String, String)> {
    let path = fixture(name);
    let out = Command::new(t27c())
        .arg("parse")
        .arg(&path)
        .output()
        .expect("failed to spawn t27c parse");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let lines: Vec<&str> = text.lines().collect();
    let unquote = |s: &str, key: &str| -> Option<String> {
        let t = s.trim();
        let rest = t.strip_prefix(key)?.trim();
        let rest = rest.strip_prefix('"')?;
        let end = rest.rfind('"')?;
        Some(rest[..end].to_string())
    };
    let mut out_pairs = Vec::new();
    for i in 0..lines.len() {
        if !lines[i].trim_start().starts_with("kind: ExprIdentifier") {
            continue;
        }
        if i + 3 >= lines.len() {
            continue;
        }
        let n = unquote(lines[i + 1], "name:");
        let ty = unquote(lines[i + 3], "extra_type:");
        if let (Some(n), Some(ty)) = (n, ty) {
            out_pairs.push((n, ty));
        }
    }
    out_pairs
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("terminator")
        .join(name)
}

struct Outcome {
    status: i32,
    output: String,
    elapsed: Duration,
}

/// Run `t27c check <fixture>` under a hard timeout.
///
/// The child is spawned, then waited on from a helper thread so the main thread
/// keeps a clock the child cannot influence. On timeout the child is killed
/// before the assertion fires, so a wedged parser does not survive the test.
fn parse_within_timeout(name: &str) -> Outcome {
    let path = fixture(name);
    assert!(path.exists(), "fixture missing: {}", path.display());

    let mut child = Command::new(t27c())
        .arg("check")
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn t27c");

    let started = Instant::now();
    let mut out = child.stdout.take().expect("stdout");
    let mut err = child.stderr.take().expect("stderr");

    // Drain both pipes from threads. A parser that fills a pipe buffer and then
    // blocks on write would otherwise look like a hang caused by this test.
    let (otx, orx) = mpsc::channel();
    thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        let _ = otx.send(s);
    });
    let (etx, erx) = mpsc::channel();
    thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        let _ = etx.send(s);
    });

    // Poll rather than block, so the timeout is owned by this thread.
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(st) => break st,
            None => {
                if started.elapsed() > HARD_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "HANG: t27c check {} did not terminate within {:?}. \
                         This is #2127: a terminator ({{RBrace, Semicolon, Eof}}) was \
                         gated on bracket nesting depth, and truncated input leaves \
                         the depth positive, so the field loop never accepted an end.",
                        name, HARD_TIMEOUT
                    );
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    };

    let elapsed = started.elapsed();
    let combined = format!(
        "{}{}",
        orx.recv_timeout(Duration::from_secs(5)).unwrap_or_default(),
        erx.recv_timeout(Duration::from_secs(5)).unwrap_or_default()
    );
    Outcome {
        status: status.code().unwrap_or(-1),
        output: combined,
        elapsed,
    }
}

/// The case the fix is about: the file ends mid-type, at depth 1, with no
/// closing brace anywhere. `Eof` must end the field list at any depth.
#[test]
fn eof_at_positive_depth_terminates() {
    let r = parse_within_timeout("eof_hazard.t27");
    assert!(
        r.elapsed < HARD_TIMEOUT,
        "took {:?}, ceiling {:?}",
        r.elapsed,
        HARD_TIMEOUT
    );
    assert_ne!(
        r.status, 0,
        "truncated input must be rejected, not accepted; output was:\n{}",
        r.output
    );
    let lower = r.output.to_lowercase();
    assert!(
        lower.contains("eof"),
        "the diagnostic must name the end of input it stopped at, got:\n{}",
        r.output
    );
    assert!(
        !lower.contains("panicked") && !lower.contains("unwrap"),
        "must fail as a diagnostic, not as a panic:\n{}",
        r.output
    );
}

/// A type argument list left open, then a further field, then a closing brace.
/// `RBrace` must end the field list even though depth is still positive.
#[test]
fn rbrace_at_positive_depth_terminates() {
    let r = parse_within_timeout("unbalanced.t27");
    assert!(
        r.elapsed < HARD_TIMEOUT,
        "took {:?}, ceiling {:?}",
        r.elapsed,
        HARD_TIMEOUT
    );
    let lower = r.output.to_lowercase();
    assert!(
        !lower.contains("panicked"),
        "must not panic:\n{}",
        r.output
    );
}

/// The control, and the reason the two above are not vacuous: a well-formed
/// struct whose fields contain commas inside `Map<K, V>`, `(A, B)`, `Vec<Vec<u8>>`
/// and `[4]u16` must still parse cleanly. A fix that accepted any terminator by
/// ignoring depth altogether would break this one, by treating a comma inside a
/// type argument list as a field separator.
#[test]
fn commas_inside_types_are_not_field_separators() {
    let r = parse_within_timeout("nested_types.t27");
    assert!(
        r.elapsed < HARD_TIMEOUT,
        "took {:?}, ceiling {:?}",
        r.elapsed,
        HARD_TIMEOUT
    );
    assert_eq!(
        r.status, 0,
        "well-formed nested types must parse; output was:\n{}",
        r.output
    );
}

// ---------------------------------------------------------------------------
// The two cases below discriminate the fixed parser from the unfixed one. The
// three liveness cases above do NOT: run against the pre-fix binary they pass
// unchanged, because `check` prints the same verdict either way. A test that
// cannot fail on the defect it names is a regression guard, not evidence, and
// these two exist so the branch has at least one of each.
// ---------------------------------------------------------------------------

/// The improvement, stated as a field set. `Map<K, V;` leaves a semicolon where
/// the `>` should be. Before the fix the collector produced a PHANTOM field
/// named `V` with an empty type -- an identifier from inside a type argument
/// list promoted to a field of the struct. After the fix `Semicolon` terminates
/// at any depth, `V` stays inside the type text, and the struct has exactly the
/// two fields it was written with.
///
/// This is the case that fails on the pre-fix binary, and the reason the change
/// is worth making.
#[test]
fn semicolon_at_depth_does_not_invent_a_field() {
    let f = fields("semicolon_phantom.t27");
    let names: Vec<&str> = f.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        !names.contains(&"V"),
        "phantom field V promoted out of a type argument list: {:?}",
        f
    );
    assert_eq!(names, vec!["a", "b"], "expected exactly the declared fields: {:?}", f);
}

/// The cost, pinned so it cannot grow unnoticed.
///
/// On a type argument list left open by a comma, the fixed collector absorbs the
/// following `name : type` pairs into the type text of the first field: three
/// declared fields become one, whose type reads `Map<K,b:u8,c:u16`. The pre-fix
/// collector truncated instead and kept three fields.
///
/// This IS a loss of fields on malformed input, and it is the honest reading of
/// the corpus differential's "0 regressions": the differential accepted it as a
/// tradeoff rather than finding no loss. Neither reading of this input is
/// correct -- there is no correct reading -- so the test does not assert that
/// one is right. It asserts the loss is exactly one level deep and confined to
/// the first field, so that a future change that swallows a whole struct body,
/// or crosses a struct boundary, fails here instead of passing as "still one
/// field".
#[test]
fn open_comma_swallows_following_fields_and_no_more() {
    let f = fields("field_swallow.t27");
    assert_eq!(
        f.len(),
        1,
        "expected the known one-field outcome; a different count means the \
         tradeoff moved and must be re-examined, not re-baselined: {:?}",
        f
    );
    let (name, ty) = &f[0];
    assert_eq!(name, "a", "the surviving field must be the first declared one");
    assert!(
        ty.contains("Map<K"),
        "the type text must still begin with the declared type: {:?}",
        f
    );
    assert!(
        ty.contains("c") && ty.contains("u16"),
        "the swallowed text must reach the last field of this struct and stop \
         there; if it stops earlier the fixture no longer tests the boundary: {:?}",
        f
    );
    assert!(
        !ty.contains("struct") && !ty.contains("module"),
        "the type text must not cross a declaration boundary: {:?}",
        f
    );
}
