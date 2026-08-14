//! W696: which specs could get a hardware boundary WITHOUT anyone guessing.
//!
//! T187 measured an exact equivalence over all 617 specs: **a spec produces a
//! module with a data port if and only if it declares `fn on_comb` or
//! `fn on_clock`.** Both directions, zero exceptions. 57 do; **387 generate
//! Verilog and cannot move a value across their boundary**, and the compiler
//! says so itself in every one of them:
//!
//! ```text
//! // NO DATA PORTS -- this module cannot move a value across its boundary.
//! ```
//!
//! The standing rule on repairing that is explicit and is the reason this file
//! is a census and not a rewriter: **the default must not be guessed.** Choosing
//! the wrong entry function does not fail loudly — it produces a module that
//! computes something nobody asked for, and every downstream check would pass.
//!
//! So this answers a narrower question with a yes-or-no answer per spec:
//! *is the choice FORCED?* A spec where exactly one function takes a parameter,
//! returns something, and has a body admits exactly one entry point, and
//! deriving it invents nothing. Everything else is reported and left alone.
//!
//! Two tiers are reported, because "forced" and "expressible in hardware" are
//! different questions:
//!
//!   FORCED         exactly one candidate function
//!   FORCED_SCALAR  ...and every parameter and the return is a primitive that
//!                  has a known bit width, so the port list is derivable too
//!
//! **FORCED_SCALAR is the actionable number.** A forced choice whose types
//! cannot cross a module boundary is still not something to act on.

use crate::compiler::{Compiler, Node, NodeKind};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Verdict {
    /// Already declares `on_comb` or `on_clock`.
    HasEntry,
    /// Exactly one candidate, and every type is a sized primitive.
    ForcedScalar,
    /// Exactly one candidate, but some type has no known width.
    ForcedWide,
    /// More than one function could be the entry point. NOT a decision to make
    /// mechanically.
    Ambiguous(usize),
    /// No function takes a parameter and returns a value -- there is nothing to
    /// forward to, and inventing one would be inventing behaviour.
    NoCandidate,
    /// Does not parse; another phase's business.
    NoParse,
}

impl Verdict {
    pub fn label(&self) -> String {
        match self {
            Verdict::HasEntry => "HAS_ENTRY".into(),
            Verdict::ForcedScalar => "FORCED_SCALAR".into(),
            Verdict::ForcedWide => "FORCED_WIDE".into(),
            Verdict::Ambiguous(n) => format!("AMBIGUOUS_{n}"),
            Verdict::NoCandidate => "NO_CANDIDATE".into(),
            Verdict::NoParse => "NOPARSE".into(),
        }
    }
}

/// A type whose width the Verilog backend knows. Anything else -- a slice, a
/// string, a struct, an unresolved alias -- cannot become a port without a
/// decision, and a decision is exactly what this module refuses to make.
fn is_sized_primitive(t: &str) -> bool {
    let t = t.trim();
    matches!(
        t,
        "bool"
            | "u1" | "u2" | "u4" | "u8" | "u16" | "u32" | "u64"
            | "i8" | "i16" | "i32" | "i64"
            | "usize" | "isize"
            | "trit" | "tri"
    )
}

fn is_void(t: &str) -> bool {
    let t = t.trim();
    t.is_empty() || t == "void" || t == "()"
}

pub fn classify(source: &str) -> Verdict {
    let ast = match Compiler::parse_ast(source) {
        Ok(a) => a,
        Err(_) => return Verdict::NoParse,
    };
    let fns: Vec<&Node> = ast
        .children
        .iter()
        .filter(|c| c.kind == NodeKind::FnDecl)
        .collect();

    if fns.iter().any(|f| f.name == "on_comb" || f.name == "on_clock") {
        return Verdict::HasEntry;
    }

    // A candidate must take something, give something back, and DO something.
    // The body test matters: 47% of this corpus declares functions with no
    // statements at all (T155), and forwarding to an empty function would
    // produce a port that carries a constant.
    let candidates: Vec<&&Node> = fns
        .iter()
        .filter(|f| {
            !f.params.is_empty() && !is_void(&f.extra_return_type) && !f.children.is_empty()
        })
        .collect();

    match candidates.len() {
        0 => Verdict::NoCandidate,
        1 => {
            let f = candidates[0];
            let all_sized = is_sized_primitive(&f.extra_return_type)
                && f.params.iter().all(|(_, ty)| is_sized_primitive(ty));
            if all_sized {
                Verdict::ForcedScalar
            } else {
                Verdict::ForcedWide
            }
        }
        n => Verdict::Ambiguous(n),
    }
}

/// The one candidate, when the choice is forced. `None` otherwise -- including
/// when it is forced but wide, because the caller should not be handed a
/// signature it cannot lower.
pub fn forced_signature(source: &str) -> Option<(String, Vec<(String, String)>, String)> {
    let ast = Compiler::parse_ast(source).ok()?;
    let fns: Vec<&Node> = ast
        .children
        .iter()
        .filter(|c| c.kind == NodeKind::FnDecl)
        .collect();
    if fns.iter().any(|f| f.name == "on_comb" || f.name == "on_clock") {
        return None;
    }
    let mut it = fns.iter().filter(|f| {
        !f.params.is_empty() && !is_void(&f.extra_return_type) && !f.children.is_empty()
    });
    let f = it.next()?;
    if it.next().is_some() {
        return None; // ambiguous
    }
    Some((f.name.clone(), f.params.clone(), f.extra_return_type.clone()))
}

fn spec_files(root: &Path, include_scratch: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
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

pub fn run(specs_root: &Path, verbose: bool) -> anyhow::Result<()> {
    let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
    let mut forced: Vec<(String, String)> = Vec::new();

    for f in spec_files(specs_root, false) {
        let Ok(src) = std::fs::read_to_string(&f) else { continue };
        let v = classify(&src);
        *counts.entry(v.label()).or_default() += 1;
        if v == Verdict::ForcedScalar {
            if let Some((name, params, ret)) = forced_signature(&src) {
                let sig = format!(
                    "fn on_comb({}) -> {ret} {{ return {name}({}); }}",
                    params
                        .iter()
                        .map(|(n, t)| format!("{n}: {t}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    params.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", ")
                );
                forced.push((f.to_string_lossy().to_string(), sig));
            }
        }
    }

    let total: usize = counts.values().sum();
    println!("--- entry points ---");
    println!("  T187: a spec has a data port IFF it declares `on_comb` or `on_clock`.");
    println!("  This asks, for the rest, whether the choice would be FORCED.");
    println!();
    for (k, n) in &counts {
        println!("  {k:<16} {n:>4}");
    }
    println!("  {:<16} {:>4}", "----", "----");
    println!("  {:<16} {:>4}", "total", total);
    println!();
    println!("  ACTIONABLE = FORCED_SCALAR: exactly one function takes a parameter,");
    println!("  returns a value, has a body, and every type has a known width.");
    println!("  Deriving an entry point there invents nothing.");
    println!();
    println!("  FORCED_WIDE is forced but not lowerable: a slice, string or struct");
    println!("  cannot become a port without a decision, and this command makes none.");
    println!("  AMBIGUOUS is left alone on purpose -- picking wrong does not fail");
    println!("  loudly, it produces a module that computes something nobody asked for.");

    if verbose {
        println!();
        println!("  --- the forced-scalar specs and the signature each would get ---");
        for (path, sig) in &forced {
            println!("  {path}");
            println!("      {sig}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spec_with_on_comb_already_has_an_entry() {
        let v = classify("module m\n\nfn on_comb(a: u8) -> u8 { return a; }\n");
        assert_eq!(v, Verdict::HasEntry);
    }

    #[test]
    fn one_candidate_with_sized_types_is_forced() {
        let v = classify("module m\n\nfn add(a: u8, b: u8) -> u8 { return a + b; }\n");
        assert_eq!(v, Verdict::ForcedScalar);
    }

    #[test]
    fn two_candidates_are_ambiguous_and_stay_that_way() {
        let src = "module m\n\nfn add(a: u8, b: u8) -> u8 { return a + b; }\n\
                   fn sub(a: u8, b: u8) -> u8 { return a - b; }\n";
        assert_eq!(classify(src), Verdict::Ambiguous(2));
        assert!(forced_signature(src).is_none(), "an ambiguous spec must yield no signature");
    }

    /// The body test is not decoration: forwarding to an empty function would
    /// give the module a port that carries a constant.
    #[test]
    fn an_empty_body_is_not_a_candidate() {
        assert_eq!(classify("module m\n\nfn add(a: u8, b: u8) -> u8 {\n}\n"), Verdict::NoCandidate);
    }

    #[test]
    fn a_void_return_is_not_a_candidate() {
        assert_eq!(
            classify("module m\n\nfn go(a: u8) -> void { let x = a; }\n"),
            Verdict::NoCandidate
        );
    }

    #[test]
    fn a_parameterless_function_is_not_a_candidate() {
        assert_eq!(classify("module m\n\nfn go() -> u8 { return 1; }\n"), Verdict::NoCandidate);
    }
}
