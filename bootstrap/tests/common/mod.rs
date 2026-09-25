//! Helpers shared by the C-backend tests.
//!
//! Every `c_*.rs` test hands its generated header to the system `cc` and
//! asserts on what comes back. Fifteen of them carried a byte-identical copy of
//! the argument list, including `-ferror-limit=0` -- which is a CLANG flag.
//!
//! On a machine where `cc` is gcc, gcc rejects it:
//!
//!     cc: error: unrecognized command-line option '-ferror-limit=0'
//!
//! and compiles nothing. That message contains the word "error", so every
//! `assert!(!stderr.contains("error"))` in those files fails -- on every Ubuntu
//! run, since the flag was introduced. The ones that count error LINES rather
//! than matching the word fail the other way and are worse: gcc having refused
//! to run returns a count of zero, so those asserts pass while checking nothing
//! at all. Green there means the C backend is untested, not correct.
//!
//! The flag is not decoration. Clang stops after 20 errors by default, and a
//! test that asserts over the whole diagnostic set needs all of them. gcc
//! spells the same thing `-fmax-errors=0`. So ask the compiler which it takes,
//! once, instead of assuming, and let a compiler that takes neither run on its
//! own default rather than failing over a flag.
//!
//! Its own tests live in `cc_flags.rs`, not here: this module is included by
//! fifteen separate test crates, and a `#[cfg(test)]` block inside it would be
//! compiled and run fifteen times over.

// Each including crate uses some of these, not all. Without this, a crate that
// only wants `cc_syntax_args` warns on `cc_present` and a `-D warnings` build
// fails for a reason that has nothing to do with the test.
#![allow(dead_code)]

use std::process::Command;
use std::sync::OnceLock;

/// Is there a C compiler to hand the header to at all?
pub fn cc_present() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Does `cc` accept this flag? Compiles an empty translation unit read from
/// `/dev/null`, so the only thing that can fail is the flag itself.
pub fn cc_accepts(flag: &str) -> bool {
    Command::new("cc")
        .args([flag, "-fsyntax-only", "-x", "c", "-"])
        .stdin(std::process::Stdio::null())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// "Report every error, not just the first N", in whichever spelling this `cc`
/// has. `None` when it has neither, which is not a failure -- a missing flag
/// must never be reported as a broken header.
pub fn error_limit_flag() -> Option<&'static str> {
    static FLAG: OnceLock<Option<&'static str>> = OnceLock::new();
    *FLAG.get_or_init(|| {
        ["-ferror-limit=0", "-fmax-errors=0"]
            .into_iter()
            .find(|f| cc_accepts(f))
    })
}

/// The argument list every C-backend test used to spell out for itself:
/// syntax-check a C11 translation unit read as C, reporting every error.
pub fn cc_syntax_args() -> Vec<&'static str> {
    let mut a = vec!["-std=c11"];
    a.extend(error_limit_flag());
    a.extend(["-fsyntax-only", "-x", "c"]);
    a
}

/// The same, plus the warning set the tests that assert on warnings use.
/// `-Wno-unused-parameter` is there because a generated header declares
/// functions whose parameters the test body never reads; that warning is about
/// the test, not about the C backend.
pub fn cc_strict_args() -> Vec<&'static str> {
    let mut a = vec!["-std=c11", "-Wall", "-Wextra", "-Wno-unused-parameter"];
    a.extend(error_limit_flag());
    a.extend(["-fsyntax-only", "-x", "c"]);
    a
}
