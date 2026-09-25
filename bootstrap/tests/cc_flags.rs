//! The C-backend tests' own flag selection, tested once.
//!
//! The defect this guards is not a compiler bug -- it is a test harness that
//! hands `cc` a flag `cc` does not have, and then reads the refusal as a
//! failure of the generated C. Fifteen files had a hard-coded `-ferror-limit=0`
//! in them; on gcc that is not a stricter check, it is no check.

mod common;

use common::{cc_accepts, cc_present, cc_syntax_args, error_limit_flag};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn the_argument_list_keeps_its_shape() {
    let a = cc_syntax_args();
    assert_eq!(a[0], "-std=c11");
    assert_eq!(&a[a.len() - 3..], &["-fsyntax-only", "-x", "c"]);
    // Either a limit flag was found and sits between them, or none was.
    assert!(a.len() == 4 || a.len() == 5, "unexpected args: {a:?}");
}

#[test]
fn whatever_flag_is_chosen_is_one_this_cc_accepts() {
    // The whole point: never hand `cc` a flag it will refuse. A refusal prints
    // the word "error" and is indistinguishable, to the asserts downstream,
    // from a header that does not compile.
    if !cc_present() {
        return;
    }
    if let Some(f) = error_limit_flag() {
        assert!(cc_accepts(f), "chose {f}, which this cc rejects");
    }
}

#[test]
fn a_flag_this_cc_does_not_have_is_actually_detected() {
    // Guards the probe rather than the choice. If `cc_accepts` returned true
    // unconditionally -- a silent stdin hang, a swallowed status -- the
    // selection above would pick the first candidate every time and we would
    // be back to a hard-coded clang flag with extra steps.
    if !cc_present() {
        return;
    }
    assert!(
        !cc_accepts("-fthis-flag-does-not-exist-anywhere"),
        "the probe accepts flags no compiler has; it is not probing"
    );
}

#[test]
fn the_chosen_arguments_compile_a_valid_translation_unit() {
    // End to end, with the list the other fifteen files now use: valid C in,
    // empty stderr out. This is the assert that was inverted by the bad flag.
    if !cc_present() {
        return;
    }
    // A counter AND the pid, which is `tri harness scratch`'s rule and not
    // belt-and-braces. This test deletes the WHOLE directory on the way out:
    // the counter keeps it away from the other tests in this binary, which
    // cargo runs on parallel threads, and the pid keeps it away from a second
    // concurrent RUN sharing $TMPDIR -- two worktrees, or a manual `cargo test`
    // beside this one. The pid alone is shared by every test in the process.
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "t27-cc-flags-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let src = dir.join("ok.c");
    std::fs::write(&src, "int main(void) { return 0; }\n").expect("write");

    let out = std::process::Command::new("cc")
        .args(cc_syntax_args())
        .arg(&src)
        .output()
        .expect("run cc");
    let err = String::from_utf8_lossy(&out.stderr).to_string();

    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && !err.contains("error"),
        "valid C rejected with the shared arguments {:?}:\n{err}",
        cc_syntax_args()
    );
}
