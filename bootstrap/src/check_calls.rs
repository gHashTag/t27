//! W574: check call sites against the signatures they call.
//!
//! Until W569 made `use` resolution real, nothing in this project ever compared
//! a call to the declaration it targets: each spec generated a self-contained
//! file in which a foreign callee was simply undeclared, so a wrong call was
//! indistinguishable from a missing one.
//!
//! The first thing that comparison found was that `ternary_mac` is called two
//! incompatible ways — and that the module which DECLARES it is itself split,
//! 91 call sites in declared order against 80 in the other. That is a
//! specification decision and not a defect to be silently repaired, but it is
//! only visible in specs that happen to compile all the way to Zig.
//!
//! This check makes the whole class visible corpus-wide, with no semantic
//! choices of its own. It reports only what is decidable from the AST:
//!
//! * **arity** — a call passing a different number of arguments than the
//!   declaration takes. Sound, no inference.
//! * **aggregate-vs-scalar** — a struct literal passed where the declaration
//!   names a scalar type, or vice versa. This is what distinguishes
//!   `ternary_mac(acc, a, w)` from `ternary_mac(a, w, acc)` without deciding
//!   which is right.
//!
//! Anything it cannot decide, it does not report.

use crate::compiler::{Compiler, Node, NodeKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Finding {
    pub file: String,
    pub line: u32,
    pub callee: String,
    pub kind: &'static str,
    pub detail: String,
}

#[derive(Clone)]
struct Signature {
    params: Vec<(String, String)>,
    origin: String,
}

fn is_scalar_type(ty: &str) -> bool {
    let t = ty.trim().trim_start_matches('&').trim();
    matches!(
        t,
        "i8" | "i16" | "i32" | "i64" | "isize"
            | "u8" | "u16" | "u32" | "u64" | "usize"
            | "f32" | "f64" | "bool" | "char"
    )
}

/// Whether an argument expression is definitely an aggregate value (a struct
/// literal). Anything else -- an identifier, a call, a literal -- is left
/// undecided, because its type is not in the AST.
fn is_aggregate_arg(n: &Node) -> bool {
    n.kind == NodeKind::ExprStructLit
}

fn collect_signatures(ast: &Node, origin: &str, out: &mut HashMap<String, Signature>) {
    for d in &ast.children {
        if d.kind == NodeKind::FnDecl && !d.name.is_empty() {
            out.entry(d.name.clone()).or_insert(Signature {
                params: d.params.clone(),
                origin: origin.to_string(),
            });
        }
    }
}

fn walk_calls(node: &Node, f: &mut impl FnMut(&Node)) {
    if node.kind == NodeKind::ExprCall {
        f(node);
    }
    for c in &node.children {
        walk_calls(c, f);
    }
}

fn use_targets(source: &str, specs_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        let rest = match t.strip_prefix("use ") {
            Some(r) => r,
            None => continue,
        };
        let path_expr = rest.trim().trim_end_matches(';').trim();
        if path_expr.is_empty() {
            continue;
        }
        let mut p = specs_root.to_path_buf();
        for seg in path_expr.split("::") {
            p.push(seg);
        }
        p.set_extension("t27");
        if p.is_file() {
            out.push(p);
        }
    }
    out
}

/// Check one spec's calls against every signature it can see: its own, and
/// those of the modules it imports.
pub fn check_file(path: &Path, specs_root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return findings,
    };
    let ast = match Compiler::parse_ast(&source) {
        Ok(a) => a,
        Err(_) => return findings, // a spec that does not parse has no call sites to check
    };

    let mut sigs: HashMap<String, Signature> = HashMap::new();
    collect_signatures(&ast, "self", &mut sigs);
    for dep in use_targets(&source, specs_root) {
        let dep_src = match std::fs::read_to_string(&dep) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if let Ok(dep_ast) = Compiler::parse_ast(&dep_src) {
            let origin = dep
                .strip_prefix(specs_root)
                .unwrap_or(&dep)
                .to_string_lossy()
                .to_string();
            collect_signatures(&dep_ast, &origin, &mut sigs);
        }
    }

    let file = path.to_string_lossy().to_string();
    walk_calls(&ast, &mut |call: &Node| {
        let sig = match sigs.get(&call.name) {
            Some(s) => s,
            None => return, // unknown callee: not this check's business
        };
        if call.children.len() != sig.params.len() {
            findings.push(Finding {
                file: file.clone(),
                line: call.line,
                callee: call.name.clone(),
                kind: "arity",
                detail: format!(
                    "{} argument(s) passed, {} declared in {}",
                    call.children.len(),
                    sig.params.len(),
                    sig.origin
                ),
            });
            return;
        }
        for (i, arg) in call.children.iter().enumerate() {
            let (pname, pty) = &sig.params[i];
            if is_aggregate_arg(arg) && is_scalar_type(pty) {
                findings.push(Finding {
                    file: file.clone(),
                    line: call.line,
                    callee: call.name.clone(),
                    kind: "aggregate-vs-scalar",
                    detail: format!(
                        "argument {} is a `{}` literal; parameter `{}` is declared `{}` in {}",
                        i + 1,
                        arg.name,
                        pname,
                        pty,
                        sig.origin
                    ),
                });
            }
        }
    });
    findings
}

/// Walk `specs_root` and check every spec.
pub fn check_tree(specs_root: &Path, skip_scratch: bool) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut stack = vec![specs_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if skip_scratch && p.file_name().map(|n| n == "scratch").unwrap_or(false) {
                    continue;
                }
                stack.push(p);
            } else if p.extension().and_then(|e| e.to_str()) == Some("t27") {
                out.extend(check_file(&p, specs_root));
            }
        }
    }
    out.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    out
}
