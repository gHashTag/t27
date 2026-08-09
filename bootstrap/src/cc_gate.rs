//! W585: compile the generated C headers, as a command rather than a shell loop.
//!
//! W583 compiled the C output for the first time in the project's life (36 of
//! 397 headers passed `cc -fsyntax-only`) and W584 raised it to 101 — both
//! measured by a loop assembled by hand at a prompt. Everything else this chain
//! trusts is a command: the parse census, the harness, `lex-conform`,
//! `parse-conform`, `check-calls`. A number nobody can reproduce with one
//! invocation is a number that quietly stops being re-measured.
//!
//! ## The metric this reports
//!
//! W584 fixed four real defects, drove every class it touched down, and left
//! the count of compiling headers at exactly 101 — because a header must clear
//! EVERY class to compile, and 296 failures were spread across eight. So this
//! gate reports the **class table** first and the header count second. The
//! header count becomes the interesting number when the classes are small; it
//! is not the interesting number today.

use crate::compiler::Compiler;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Report {
    pub total: usize,
    pub compiled: usize,
    pub failed: usize,
    pub gen_failed: usize,
    /// Normalised first-error text -> occurrences.
    pub classes: BTreeMap<String, usize>,
    /// The first error for each failing spec, in path order.
    pub failures: Vec<(String, String)>,
    pub cc: String,
}

/// Strip the parts of a compiler diagnostic that vary per site, so the same
/// defect counts once: quoted names, numbers, and the trailing `-W…` tag.
fn classify(err: &str) -> String {
    let mut out = String::with_capacity(err.len());
    let mut in_quote = false;
    for c in err.chars() {
        match c {
            '\'' => {
                if !in_quote {
                    out.push_str("'X'");
                }
                in_quote = !in_quote;
            }
            _ if in_quote => {}
            c if c.is_ascii_digit() => {
                if !out.ends_with('N') {
                    out.push('N');
                }
            }
            c => out.push(c),
        }
    }
    if let Some(i) = out.find(" [-W") {
        out.truncate(i);
    }
    out.trim().to_string()
}

fn spec_files(root: &Path, include_scratch: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if !include_scratch && p.file_name().map(|n| n == "scratch").unwrap_or(false) {
                    continue;
                }
                stack.push(p);
            } else if p.extension().and_then(|e| e.to_str()) == Some("t27") {
                files.push(p);
            }
        }
    }
    files.sort();
    files
}

/// Is a C compiler available? Returns its name, or None.
pub fn find_cc() -> Option<String> {
    for cc in ["cc", "clang", "gcc"] {
        if Command::new(cc)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(cc.to_string());
        }
    }
    None
}

pub fn run(specs_root: &Path, include_scratch: bool) -> Option<Report> {
    let cc = find_cc()?;
    let files = spec_files(specs_root, include_scratch);
    let dir = std::env::temp_dir().join("t27c-cc-gate");
    let _ = std::fs::create_dir_all(&dir);

    let mut report = Report {
        total: files.len(),
        compiled: 0,
        failed: 0,
        gen_failed: 0,
        classes: BTreeMap::new(),
        failures: Vec::new(),
        cc: cc.clone(),
    };

    for (i, f) in files.iter().enumerate() {
        let raw = match std::fs::read_to_string(f) {
            Ok(s) => s,
            Err(_) => continue,
        };
        // Same resolution the `gen-c` command performs, so the gate measures
        // what the command emits (W584).
        let resolved = crate::use_resolve::resolve(f, &raw);
        let code = match Compiler::compile_c(&resolved).or_else(|_| Compiler::compile_c(&raw)) {
            Ok(c) if !c.is_empty() => c,
            _ => {
                report.gen_failed += 1;
                continue;
            }
        };
        let header = dir.join(format!("h{}.h", i));
        if std::fs::write(&header, code).is_err() {
            report.gen_failed += 1;
            continue;
        }
        let out = Command::new(&cc)
            .args(["-fsyntax-only", "-x", "c"])
            .arg(&header)
            .output();
        let out = match out {
            Ok(o) => o,
            Err(_) => {
                report.gen_failed += 1;
                continue;
            }
        };
        if out.status.success() {
            report.compiled += 1;
            continue;
        }
        report.failed += 1;
        let stderr = String::from_utf8_lossy(&out.stderr);
        let first = stderr
            .lines()
            .find_map(|l| l.split(" error: ").nth(1))
            .unwrap_or("(no error line)")
            .to_string();
        *report.classes.entry(classify(&first)).or_insert(0) += 1;
        report
            .failures
            .push((f.to_string_lossy().to_string(), first));
    }
    let _ = std::fs::remove_dir_all(&dir);
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_collapses_names_and_numbers() {
        assert_eq!(
            classify("unknown type name 'f32'"),
            "unknown type name 'X'"
        );
        assert_eq!(
            classify("call to undeclared function 'foo'; ISO C99 and later [-Wimplicit]"),
            "call to undeclared function 'X'; ISO CN and later"
        );
    }

    #[test]
    fn classify_is_stable_across_sites() {
        assert_eq!(
            classify("unknown type name 'a'"),
            classify("unknown type name 'bbb'")
        );
    }
}
