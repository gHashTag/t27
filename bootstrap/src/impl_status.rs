//! W586: separate "this spec is unwritten" from "this spec is broken".
//!
//! For twenty-five waves those two facts have been one number. A spec whose
//! functions have no bodies fails exactly like a spec with a syntax error — the
//! Zig backend emits `@compileError("not yet implemented")` and the harness
//! records `COMPILE_FAIL`. After W585 removed the `default_input` mask, 101 of
//! the remaining failures are that, and they are not defects: they are specs
//! nobody has written yet.
//!
//! W586's Variant A proposed regenerating the bodies from the `.tri` sources
//! every one of those specs names in its header comment. The falsification
//! check killed it: **1 of 169** empty-body specs has a same-named `.tri`, and
//! that one is an accidental basename collision with an architecture diagram.
//! Across all 26 `.tri` files in the repository there are 94 function
//! declarations and 5 bodies. The sources do not exist.
//!
//! So the honest deliverable is this: count the two states separately and stop
//! reporting them as one.

use crate::compiler::{Compiler, Node, NodeKind};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Report {
    /// Specs that parse and whose functions all have bodies.
    pub implemented: usize,
    /// Specs that parse and have at least one empty function body.
    pub partial: usize,
    /// Specs where EVERY function is empty.
    pub unwritten: usize,
    /// Specs that do not parse -- genuinely broken, not merely unwritten.
    pub unparsable: usize,
    pub total_fns: usize,
    pub empty_fns: usize,
    /// (spec, empty, total) for the specs with at least one empty body.
    pub detail: Vec<(String, usize, usize)>,
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

/// A function declaration with no statements. This is exactly the condition the
/// Zig backend turns into `@compileError("not yet implemented")`.
fn is_empty_fn(n: &Node) -> bool {
    n.kind == NodeKind::FnDecl && n.children.is_empty()
}

/// Whether every function this source declares has an empty body. Shared with
/// the C gate (W587) so both commands agree on what "unwritten" means.
pub fn spec_is_unwritten(source: &str) -> bool {
    let ast = match Compiler::parse_ast(source) {
        Ok(a) => a,
        Err(_) => return false,
    };
    let fns: Vec<&Node> = ast
        .children
        .iter()
        .filter(|c| c.kind == NodeKind::FnDecl)
        .collect();
    !fns.is_empty() && fns.iter().all(|n| is_empty_fn(n))
}

/// (empty, total) function-declaration counts for one spec source.
///
/// W662: `spec_is_unwritten` answers only the all-or-nothing question, and the
/// corpus is not all-or-nothing -- T121 counted 159 entirely unwritten specs
/// against 667 bodiless functions spread over more. A spec with three written
/// functions and one stub generates a module that fails to compile for a reason
/// no compiler fix can repair, and counting it as a DEFECT inflates the backlog.
///
/// Returns (0, 0) when the source does not parse, so callers can distinguish
/// "no bodies missing" from "could not be asked".
pub fn spec_body_counts(source: &str) -> (usize, usize) {
    let ast = match Compiler::parse_ast(source) {
        Ok(a) => a,
        Err(_) => return (0, 0),
    };
    let fns: Vec<&Node> = ast
        .children
        .iter()
        .filter(|c| c.kind == NodeKind::FnDecl)
        .collect();
    (fns.iter().filter(|n| is_empty_fn(n)).count(), fns.len())
}

pub fn run(specs_root: &Path, include_scratch: bool) -> Report {
    let mut r = Report::default();
    for f in spec_files(specs_root, include_scratch) {
        let src = match std::fs::read_to_string(&f) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let ast = match Compiler::parse_ast(&src) {
            Ok(a) => a,
            Err(_) => {
                r.unparsable += 1;
                continue;
            }
        };
        let fns: Vec<&Node> = ast
            .children
            .iter()
            .filter(|c| c.kind == NodeKind::FnDecl)
            .collect();
        if fns.is_empty() {
            r.implemented += 1;
            continue;
        }
        let empty = fns.iter().filter(|n| is_empty_fn(n)).count();
        r.total_fns += fns.len();
        r.empty_fns += empty;
        if empty == 0 {
            r.implemented += 1;
        } else {
            if empty == fns.len() {
                r.unwritten += 1;
            } else {
                r.partial += 1;
            }
            r.detail
                .push((f.to_string_lossy().to_string(), empty, fns.len()));
        }
    }
    r.detail.sort_by(|a, b| b.1.cmp(&a.1));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_fn_is_one_with_no_statements() {
        let ast = Compiler::parse_ast("module m\n\nfn a() -> u32 {\n}\n").unwrap();
        let f = ast
            .children
            .iter()
            .find(|c| c.kind == NodeKind::FnDecl)
            .unwrap();
        assert!(is_empty_fn(f));
    }

    #[test]
    fn a_fn_with_a_body_is_not_empty() {
        let ast = Compiler::parse_ast("module m\n\nfn a() -> u32 { return 1; }\n").unwrap();
        let f = ast
            .children
            .iter()
            .find(|c| c.kind == NodeKind::FnDecl)
            .unwrap();
        assert!(!is_empty_fn(f));
    }
}
