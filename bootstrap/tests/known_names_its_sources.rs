//! #3073: `t27c known` printed three empty signals and told you to record the negative.
//!
//! The caption section read `tnf_paper.tex`. No file of that name exists in this
//! repository -- 0 on disk, 0 in the index, 0 ever added on any branch -- so
//! `find(|p| p.is_file())` yielded None, `unwrap_or_else` handed
//! `read_to_string` a path just proven absent, and `if let Ok` skipped the whole
//! loop. `chits` stayed 0 and printed `(none)`, byte-identical to "no caption
//! mentions this".
//!
//! And the directory itself was never checked. The project's own record invokes
//!
//!     $ t27c known --dir research/arxiv_tnf --about d_posit16
//!
//! against a directory that does not exist; every section fell through to its
//! `(none)` and the command signed off with "Nothing speaks to this. Measure --
//! and record the negative, it is a result."
//!
//! The rule was already written twenty lines above, for the gates directory:
//! *a silent "(none)" from looking in the wrong directory is exactly the false
//! all-clear this command exists to prevent.* It was paid for there with a
//! `gates read from` line and omitted for captions.
//!
//! Three states, and the third is the control: an absent directory REFUSES, a
//! directory with no paper says NOT READ, and a directory with a paper produces
//! a real count -- including an honest zero, which must remain distinguishable
//! from the other two.

use std::process::Command;

/// pid AND a counter: the pid separates concurrent runs, the counter separates
/// the tests inside one. Measured elsewhere in this repository that neither
/// alone is enough.
fn scratch(tag: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("t27c-known-{tag}-{}-{n}", std::process::id()))
}

const PAPER: &str = r#"\documentclass{article}
\begin{document}
\begin{table}[t]
\caption{Placer runtime across the widget corpus}
\label{tab:widget}
\begin{tabular}{ll} a & b \end{tabular}
\end{table}
\end{document}
"#;

fn known(root: &std::path::Path, dir: &str, about: &str) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["known", "--dir", dir, "--about", about])
        .current_dir(root)
        .output()
        .expect("run t27c known");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string()
            + &String::from_utf8_lossy(&out.stderr),
    )
}

#[test]
fn a_directory_that_is_not_there_refuses_instead_of_reporting_three_empty_signals() {
    let root = scratch("nodir");
    std::fs::create_dir_all(&root).expect("scratch");
    let (code, text) = known(&root, "research/arxiv_tnf", "placer");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(code, Some(2), "an absent --dir must refuse:\n{text}");
    assert!(text.contains("REFUSED"), "{text}");
    assert!(
        !text.contains("Nothing speaks to this"),
        "it must not conclude a negative it never took:\n{text}"
    );
}

#[test]
fn a_directory_with_no_paper_says_so_rather_than_printing_a_zero() {
    let root = scratch("nopaper");
    std::fs::create_dir_all(root.join("docs")).expect("scratch");
    let (code, text) = known(&root, "docs", "placer");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(code, Some(0), "two of three signals still work:\n{text}");
    assert!(text.contains("NO PAPER FOUND"), "{text}");
    assert!(
        text.contains("NOT READ"),
        "the summary must not print a caption count of 0:\n{text}"
    );
    assert!(
        !text.contains("caption    0"),
        "an absence was printed as a zero:\n{text}"
    );
}

/// THE CONTROL. Without it every assertion above is satisfied by a command that
/// refuses everything, and the caption loop could be dead without the tests
/// noticing -- which is exactly the state this test was written to end.
#[test]
fn a_directory_with_a_paper_reads_it_and_counts() {
    let root = scratch("paper");
    std::fs::create_dir_all(root.join("p")).expect("scratch");
    std::fs::write(root.join("p/tnf_paper.tex"), PAPER).expect("fixture");

    let (code, hit) = known(&root, "p", "widget");
    assert_eq!(code, Some(0), "{hit}");
    assert!(hit.contains("captions read from"), "the source must be named:\n{hit}");
    assert!(hit.contains("caption    1"), "one caption names the widget table:\n{hit}");

    // An honest zero, which must stay distinguishable from "not read".
    let (code, miss) = known(&root, "p", "zzzznotthere");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(code, Some(0), "{miss}");
    assert!(miss.contains("caption    0"), "a real zero is still a zero:\n{miss}");
    assert!(
        !miss.contains("NOT READ"),
        "a paper WAS read; this is a measured zero:\n{miss}"
    );
}
