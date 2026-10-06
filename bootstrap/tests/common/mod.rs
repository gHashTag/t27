//! What the C-backend tests ask `cc`, decided in one place.
//!
//! #4832 / #5907: fifteen `c_*.rs` files each carried a hand-copied
//! `-ferror-limit=0`. That is a clang flag; CI's `cc` is gcc, which refuses
//! the whole command line with `cc: error: unrecognized command-line option`.
//! The refusal broke the two assertion shapes in opposite directions:
//!
//! - `!stderr.contains("error")` went red: the refusal says "error".
//! - counting `file:line:col: error:` lines went GREEN, because the refusal
//!   has no line:col. Nothing had been compiled. Green meant untested.
//!
//! So this module does two things. `get_cc_args` asks `cc` which spelling of
//! "no error limit" it accepts instead of assuming one. `cc_check` runs the
//! check and refuses to hand back the stderr of a `cc` that exited non-zero
//! without naming a single line of the file: that is `cc` refusing its own
//! command line, and reading it as "0 errors" is the defect above.
//!
//! Test files use it with `mod common;`.

// Each test target compiles its own copy of this module and uses part of it.
#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

static CC_ARGS: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Does `cc` take `flag` without a word? Probed on an empty translation unit,
/// syntax only, so the probe writes no object file into the working directory.
///
/// Exit status alone is not enough: Apple clang ACCEPTS gcc's `-fmax-errors=0`
/// with "argument unused during compilation" and exit 0. A flag `cc` warns
/// about is a flag it ignores, so any stderr at all counts as a refusal.
fn cc_accepts(flag: &str) -> bool {
    Command::new("cc")
        .args([flag, "-fsyntax-only", "-x", "c", "-"])
        .stdin(Stdio::null())
        .output()
        .map(|o| o.status.success() && o.stderr.is_empty())
        .unwrap_or(false)
}

/// The flags for a syntax-only check of one C file, ending in `-x c` so the
/// path comes next. Probed once per test binary:
///
/// - `-ferror-limit=0` if `cc` takes it (clang, Apple clang);
/// - else `-fmax-errors=0` (gcc's spelling of the same thing);
/// - else neither. An absent flag only means `cc` may stop after its own
///   default number of errors; it must never be reported as a broken header.
pub fn get_cc_args() -> &'static [&'static str] {
    CC_ARGS
        .get_or_init(|| {
            let limit = ["-ferror-limit=0", "-fmax-errors=0"]
                .into_iter()
                .find(|flag| cc_accepts(flag));
            let mut args = vec!["-std=c11"];
            args.extend(limit);
            args.extend(["-fsyntax-only", "-x", "c"]);
            args
        })
        .as_slice()
}

/// `file:line:col: error: ` and `file:line:col: fatal error: ` lines in `cc`'s
/// stderr. A fatal error (a missing `#include`) stops the compile, so it is an
/// error of the file too; counting only `error: ` read it as zero.
pub fn error_count(stderr: &str) -> usize {
    stderr
        .lines()
        .filter(|l| {
            let mut it = l.splitn(4, ':');
            it.next().is_some()
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| {
                    let s = s.trim_start();
                    s.starts_with("error: ") || s.starts_with("fatal error: ")
                })
        })
        .count()
}

/// Syntax-check `path` as C with `extra` flags (warnings) and return `cc`'s
/// stderr.
///
/// Panics if `cc` exited non-zero and no `file:line:col:` error names the
/// file. Then `cc` never read the file -- it refused its command line, which
/// is how fifteen files came to check nothing on gcc -- and its stderr says
/// nothing about the header either way.
pub fn cc_check(path: &Path, extra: &[&str]) -> String {
    let out = Command::new("cc")
        .args(extra)
        .args(get_cc_args())
        .arg(path)
        .output()
        .expect("run cc");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success() || error_count(&stderr) > 0,
        "cc exited with {} and named no line of {}: it refused its command line \
         rather than compile the file, so this is not a reading of the header.\n\
         args: {:?} {:?}\n{stderr}",
        out.status,
        path.display(),
        extra,
        get_cc_args(),
    );
    stderr
}
