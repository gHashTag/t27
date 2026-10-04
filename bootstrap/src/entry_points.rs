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
//!   FORCED_SCALAR  ...and every parameter and the return has a width the
//!                  backend's `entry_port_width` derives (a sized primitive,
//!                  `[N]T`, or a flat struct), so the port list is derivable too
//!
//! **FORCED_SCALAR is the actionable number.** A forced choice whose types
//! cannot cross a module boundary is still not something to act on.

use crate::compiler::{Compiler, Node, NodeKind, VerilogCodegen};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Verdict {
    /// Already declares `on_comb` or `on_clock`.
    HasEntry,
    /// Exactly one candidate, and every type has a derivable port width.
    ForcedScalar,
    /// W697: several candidates, but exactly ONE is not called by any other
    /// function in the spec -- the root of the call graph -- and its types are
    /// all sized. The choice is forced by the call structure rather than by
    /// count, and forwarding to it still invents nothing.
    ForcedRoot,
    /// The call graph has a unique root, but one of its types has no width.
    ForcedRootWide,
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
            Verdict::ForcedRoot => "FORCED_ROOT".into(),
            Verdict::ForcedRootWide => "FORCED_ROOT_WIDE".into(),
            Verdict::ForcedWide => "FORCED_WIDE".into(),
            Verdict::Ambiguous(n) => format!("AMBIGUOUS_{n}"),
            Verdict::NoCandidate => "NO_CANDIDATE".into(),
            Verdict::NoParse => "NOPARSE".into(),
        }
    }
}

/// The struct declarations a spec's types are sized against: `struct_decls_of`,
/// the builder `gen_verilog` itself uses, plus the structs reached through `use`
/// when the spec's path is known.
type StructMap = HashMap<String, Vec<(String, String)>>;

fn struct_map(ast: &Node, source: &str, path: Option<&Path>) -> StructMap {
    let imported = path
        .map(|p| crate::use_resolve::imported_structs(p, source))
        .unwrap_or_default();
    VerilogCodegen::struct_decls_of(ast, &imported)
}

/// W698: a type whose width is DERIVABLE: `[8]u64` is 512 bits and nothing
/// about that is a decision.
///
/// #5904: THIS FILE NO LONGER KEEPS A LIST OF ITS OWN. It kept one --
/// `is_sized_primitive` -- and it disagreed with the backend it describes: it
/// accepted `u1 | u2 | u4`, which `entry_port_width` refused, so a spec could be
/// counted FORCED_SCALAR here and still have its entry point refused by
/// `gen-verilog`. The answer now comes from `VerilogCodegen::entry_port_width`,
/// the function that sizes the port, so the two cannot drift: T190b's "the
/// accepting side must be the stricter" holds because it is the SAME side.
///
/// What that function refuses, and why, is documented there. In short:
///
///   `[]T`      a slice has no length in the type. Choosing one is a decision.
///   `f64`      64 bits, but whether a float port carries raw IEEE bits or a
///              fixed-point encoding is a DESIGN choice.
///   nested     a struct field that is itself a struct, or an array of structs:
///              T145 recorded nested packing shipping a silent wrong width.
///
/// A flat struct -- every field a sized primitive or `[N]` of one -- IS
/// accepted, at the sum of its fields, which is also the width of the function
/// `input` it feeds.
///
/// History of `[N]T`: accepted in W698 while `gen_verilog` still sized entry
/// ports with `type_to_width`'s `_ => 32`, so `[8]u64` became `input wire
/// [31:0]` -- a silent 16x narrowing nothing noticed. Retracted the same wave;
/// re-enabled in W699 once `entry_port_width` refused instead of defaulting.
fn has_derivable_width(t: &str, structs: &StructMap) -> bool {
    VerilogCodegen::entry_port_width(t, structs).is_some()
}

fn is_void(t: &str) -> bool {
    let t = t.trim();
    t.is_empty() || t == "void" || t == "()"
}

pub fn classify(source: &str) -> Verdict {
    classify_at(source, None)
}

/// `classify`, told where the spec lives, so a struct imported through `use`
/// is sized the way `gen-verilog <path>` sizes it.
pub fn classify_at(source: &str, path: Option<&Path>) -> Verdict {
    let ast = match Compiler::parse_ast(source) {
        Ok(a) => a,
        Err(_) => return Verdict::NoParse,
    };
    let structs = struct_map(&ast, source, path);
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
            if sized(f, &structs) { Verdict::ForcedScalar } else { Verdict::ForcedWide }
        }
        n => {
            // W697: count is not the only thing that can force the choice.
            //
            // If exactly one candidate is called by NO OTHER FUNCTION, it is the
            // root of the spec's call graph and everything else is a helper it
            // reaches. Forwarding to the root still invents nothing.
            //
            // THE CALL GRAPH IS BUILT FROM FUNCTION BODIES ONLY. A first attempt
            // scanned the whole source and found ZERO uncalled functions in every
            // spec sampled -- because every function is called by its own `test`
            // block. Including tests makes the rule vacuous: it would report no
            // roots, always.
            let roots: Vec<&&Node> = candidates
                .iter()
                .filter(|c| !called_by_other_fn(&fns, &c.name))
                .copied()
                .collect();
            match roots.len() {
                1 => {
                    if sized(roots[0], &structs) { Verdict::ForcedRoot } else { Verdict::ForcedRootWide }
                }
                _ => Verdict::Ambiguous(n),
            }
        }
    }
}

fn sized(f: &Node, structs: &StructMap) -> bool {
    has_derivable_width(&f.extra_return_type, structs)
        && f.params.iter().all(|(_, ty)| has_derivable_width(ty, structs))
}

/// Does any function OTHER than `name` call `name`, anywhere in its body?
///
/// Only `FnDecl` bodies are walked. `test`, `invariant` and `bench` blocks are
/// deliberately excluded: they exercise the functions, so counting them as
/// callers makes every function look reachable and the root rule vacuous.
fn called_by_other_fn(fns: &[&Node], name: &str) -> bool {
    fn mentions_call(n: &Node, name: &str) -> bool {
        if n.kind == NodeKind::ExprCall && n.name == name {
            return true;
        }
        // Some call sites carry the callee in `value` rather than `name`.
        if n.kind == NodeKind::ExprCall && n.value == name {
            return true;
        }
        n.children.iter().any(|c| mentions_call(c, name))
    }
    fns.iter()
        .filter(|f| f.name != name)
        .any(|f| f.children.iter().any(|c| mentions_call(c, name)))
}

/// Like `forced_signature` but WITHOUT the sizedness filter -- used only to
/// report which type blocks a forced-but-wide spec. Never used to emit code.
pub fn signature_of_forced_any(source: &str) -> Option<(String, Vec<(String, String)>, String)> {
    let ast = Compiler::parse_ast(source).ok()?;
    let fns: Vec<&Node> = ast
        .children
        .iter()
        .filter(|c| c.kind == NodeKind::FnDecl)
        .collect();
    if fns.iter().any(|f| f.name == "on_comb" || f.name == "on_clock") {
        return None;
    }
    let candidates: Vec<&&Node> = fns
        .iter()
        .filter(|f| {
            !f.params.is_empty() && !is_void(&f.extra_return_type) && !f.children.is_empty()
        })
        .collect();
    let f = match candidates.len() {
        0 => return None,
        1 => candidates[0],
        _ => {
            let roots: Vec<&&&Node> = candidates
                .iter()
                .filter(|c| !called_by_other_fn(&fns, &c.name))
                .collect();
            if roots.len() != 1 { return None; }
            roots[0]
        }
    };
    Some((f.name.clone(), f.params.clone(), f.extra_return_type.clone()))
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
    let candidates: Vec<&&Node> = fns
        .iter()
        .filter(|f| {
            !f.params.is_empty() && !is_void(&f.extra_return_type) && !f.children.is_empty()
        })
        .collect();
    let f = match candidates.len() {
        0 => return None,
        1 => candidates[0],
        _ => {
            // W697: the unique call-graph root, if there is exactly one.
            let roots: Vec<&&&Node> = candidates
                .iter()
                .filter(|c| !called_by_other_fn(&fns, &c.name))
                .collect();
            if roots.len() != 1 {
                return None;
            }
            roots[0]
        }
    };
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

pub fn run(specs_root: &Path, verbose: bool, suggest: bool) -> anyhow::Result<()> {
    let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
    let mut forced: Vec<(String, String)> = Vec::new();
    // W709: the SUGGESTIONS the demotion created and did not print.
    //
    // W708 removed `FORCED_ROOT` from the applicable list and said it would
    // print "as a suggestion a human accepts" -- and then printed nothing. An
    // attempt to reconstruct the list outside the compiler, with a regex over
    // the source, returned ZERO where the AST returns five. That is lesson 404
    // for the fifth time: a predicate reimplemented outside the AST gives a
    // different answer. The list belongs here, next to the classifier that
    // produces it.
    let mut suggestions: Vec<(String, String, String)> = Vec::new();

    for f in spec_files(specs_root, false) {
        let Ok(src) = std::fs::read_to_string(&f) else { continue };
        let v = classify_at(&src, Some(&f));
        *counts.entry(v.label()).or_default() += 1;
        // W708: ONLY the count rule is actionable. `ForcedRoot` is a SUGGESTION.
        //
        // The root rule picks the one candidate no other function calls. W704
        // defended it against two known bad picks (`e8_lie_algebra -> abs`,
        // `gf4 -> decode`) on the grounds that the width filter excluded both.
        // `queen/lotus.t27` is a third, and the width filter does NOT help:
        // `lotus_spawn(agent_type: u8, count: u8) -> bool` is entirely scalar.
        //
        // The module is a six-phase orchestration cycle. Its subject is
        // `lotus_orchestrate()`, which takes NO parameters and so cannot be a
        // candidate at all -- leaving the rule to pick a primitive the cycle
        // calls and present it as the hardware boundary.
        //
        // A further syntactic filter was measured and rejected: "block when the
        // spec has a parameterless function with a return" would exclude 5 of
        // the 20 applied boundaries to catch this 1, and one of the five is
        // `fpga/uart.t27`'s `uart_tx_ready()` -- a status getter, not a driver.
        // The distinction between a driver and a getter is SEMANTIC, and no
        // predicate available here draws it.
        //
        // So the root rule stays, as a suggestion a human accepts. It is not
        // emitted for automatic application.
        if suggest && (v == Verdict::ForcedRoot || v == Verdict::ForcedRootWide) {
            if let Some((name, params, ret)) = signature_of_forced_any(&src) {
                let module = Compiler::parse_ast(&src)
                    .ok()
                    .map(|a| a.name.clone())
                    .unwrap_or_default();
                let sig = format!(
                    "fn on_comb({}) -> {ret} {{ return {name}({}); }}",
                    params.iter().map(|(n, t)| format!("{n}: {t}")).collect::<Vec<_>>().join(", "),
                    params.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", ")
                );
                suggestions.push((f.to_string_lossy().to_string(), module, sig));
            }
        }
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

    // W698: which TYPE blocks each forced-but-wide spec. Naming the blocker is
    // the measurement that decides whether widening the predicate is possible
    // without a decision -- a sized array has a width, a slice does not.
    let mut wide_blockers: Vec<(String, Vec<String>)> = Vec::new();
    for f in spec_files(specs_root, false) {
        let Ok(src) = std::fs::read_to_string(&f) else { continue };
        let v = classify_at(&src, Some(&f));
        if v != Verdict::ForcedWide && v != Verdict::ForcedRootWide {
            continue;
        }
        let Ok(ast) = Compiler::parse_ast(&src) else { continue };
        let structs = struct_map(&ast, &src, Some(&f));
        if let Some((_, params, ret)) = signature_of_forced_any(&src) {
            let mut bad: Vec<String> = params
                .iter()
                .map(|(_, t)| t.clone())
                .filter(|t| !has_derivable_width(t, &structs))
                .collect();
            if !has_derivable_width(&ret, &structs) {
                bad.push(format!("-> {ret}"));
            }
            wide_blockers.push((f.to_string_lossy().to_string(), bad));
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
    println!("  ACTIONABLE = FORCED_SCALAR ONLY: exactly one function takes a parameter,");
    println!("  returns a value, has a body, and every type has a known width.");
    println!("  Deriving an entry point there invents nothing.");
    println!();
    println!("  FORCED_WIDE is forced but not lowerable: a slice, string, float or nested");
    println!("  struct cannot become a port without a decision, and this command makes none.");
    println!("  AMBIGUOUS is left alone on purpose -- picking wrong does not fail");
    println!("  loudly, it produces a module that computes something nobody asked for.");
    println!();
    println!("  W708: FORCED_ROOT is a SUGGESTION, not an action. The rule picks the one");
    println!("  candidate nothing else calls -- and in queen/lotus.t27 that is a spawn");
    println!("  primitive, while the module's subject is a six-phase cycle whose driver");
    println!("  takes no parameters and so cannot be a candidate. The width filter does");
    println!("  not catch it: (u8, u8) -> bool is entirely scalar. Driver-versus-getter");
    println!("  is a SEMANTIC distinction and no predicate here draws it.");

    if suggest {
        println!();
        println!("  --- SUGGESTIONS (FORCED_ROOT): review each against the module's SUBJECT ---");
        println!("  The rule picks the one candidate nothing else calls. In queen/lotus.t27");
        println!("  that was a spawn primitive while the module is a six-phase cycle whose");
        println!("  driver takes no parameters (T202). Compare the module name to the chosen");
        println!("  function: if the function is the module's subject, accept; if it is a");
        println!("  thing the subject USES, reject. No predicate here draws that line.");
        println!();
        for (path, module, sig) in &suggestions {
            println!("  {path}");
            println!("      module {module}");
            println!("      {sig}");
        }
        if suggestions.is_empty() {
            println!("  (none)");
        }
        println!();
    }

    if !wide_blockers.is_empty() {
        println!();
        println!("  --- FORCED but WIDE: the choice is forced, the TYPE is the blocker ---");
        let mut kinds: std::collections::BTreeMap<String, usize> = Default::default();
        for (path, bad) in &wide_blockers {
            println!("  {path}");
            println!("      {}", bad.join(", "));
            for b in bad {
                let k = if b.trim_start_matches("-> ").starts_with("[]") {
                    "slice (no width)"
                } else if b.trim_start_matches("-> ").starts_with('[') {
                    "sized array (HAS a width)"
                } else if b.contains("str") || b.contains("String") {
                    "string (no width)"
                } else {
                    "named type (struct or alias)"
                };
                *kinds.entry(k.to_string()).or_default() += 1;
            }
        }
        println!();
        for (k, n) in &kinds {
            println!("  {k:<28} {n:>3}");
        }
    }

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

    /// W697: the root rule is worthless unless call detection actually fires.
    /// If `called_by_other_fn` never returned true, every candidate would be a
    /// "root", the root count would equal the candidate count, and the rule
    /// would silently decline to resolve anything -- conservative, vacuous, and
    /// indistinguishable from working.
    #[test]
    fn a_helper_called_from_another_function_is_not_a_root() {
        let src = "module m\n\n                   fn helper(a: u8) -> u8 { return a + 1; }\n                   fn top(a: u8) -> u8 { return helper(a); }\n";
        assert_eq!(classify(src), Verdict::ForcedRoot);
        let (name, _, _) = forced_signature(src).expect("a unique root must yield a signature");
        assert_eq!(name, "top", "the root is the caller, not the helper");
    }

    /// Two independent functions have two roots -- that is a library, not a
    /// module with an entry point, and it must stay ambiguous.
    #[test]
    fn two_independent_functions_have_two_roots_and_stay_ambiguous() {
        let src = "module m\n\n                   fn a(x: u8) -> u8 { return x + 1; }\n                   fn b(x: u8) -> u8 { return x - 1; }\n";
        assert_eq!(classify(src), Verdict::Ambiguous(2));
        assert!(forced_signature(src).is_none());
    }

    /// A test block calling every function must NOT make them all look reachable.
    #[test]
    fn a_test_block_is_not_a_caller() {
        let src = "module m\n\n                   fn helper(a: u8) -> u8 { return a + 1; }\n                   fn top(a: u8) -> u8 { return helper(a); }\n                   test t1 { assert_eq(top(1), 2); }\n                   test t2 { assert_eq(helper(1), 2); }\n";
        assert_eq!(
            classify(src),
            Verdict::ForcedRoot,
            "a test calling `top` must not turn `top` into a non-root"
        );
    }

    /// W698: `[8]u64` is 512 bits and that is arithmetic, not a decision.
    #[test]
    fn a_sized_array_of_primitives_has_a_derivable_width() {
        // W699: re-enabled once `entry_port_width` could size it. The emitter
        // writes [511:0] for [8]u64 and refuses a slice loudly.
        let none = StructMap::new();
        assert!(has_derivable_width("[8]u64", &none));
        assert!(has_derivable_width("[2][4]u8", &none), "nesting is still arithmetic");
        assert!(!has_derivable_width("[]u8", &none), "a slice has no length in the type");
        assert!(!has_derivable_width("[N]u8", &none), "a symbolic count is not a number");
        assert!(!has_derivable_width("f64", &none), "the Verilog backend has no float");
        assert!(!has_derivable_width("BrainState", &none), "a struct needs its declaration");
    }

    /// #5904: the census asks the backend, so a type the census calls sized is
    /// one `gen-verilog` gives a port. `u1` was the counterexample: the old list
    /// here accepted it and the backend refused it.
    #[test]
    fn the_census_and_the_backend_agree_on_odd_widths_and_flat_structs() {
        let mut structs = StructMap::new();
        structs.insert(
            "Pair".into(),
            vec![("lo".into(), "u8".into()), ("hi".into(), "u16".into())],
        );
        structs.insert("Outer".into(), vec![("p".into(), "Pair".into())]);
        assert!(has_derivable_width("u1", &structs));
        assert!(has_derivable_width("i2", &structs));
        assert!(has_derivable_width("Pair", &structs), "a flat struct is the sum of its fields");
        assert!(!has_derivable_width("Outer", &structs), "a nested struct stays refused");
        assert!(!has_derivable_width("[2]Pair", &structs), "an array of structs stays refused");
        assert!(!has_derivable_width("[4]usize", &structs), "port 128 bits, function input 32");
    }

    #[test]
    fn a_spec_taking_a_flat_struct_is_forced_scalar() {
        let src = "module m\n\npub const W = struct {\n    code : u8,\n};\n\n\
                   fn decode(w: W) -> i8 { return 1; }\n";
        assert_eq!(classify(src), Verdict::ForcedScalar);
    }

    /// W699: accepted again, now that the emitter sizes it. The regression this
    /// guards against is the W698 one: predicate ahead of backend.
    #[test]
    fn a_spec_taking_a_sized_array_is_forced_scalar() {
        let src = "module m\n\nfn dot(a: [8]u64, b: [8]u64) -> u8 { return 1; }\n";
        assert_eq!(classify(src), Verdict::ForcedScalar);
    }

    #[test]
    fn a_spec_taking_a_slice_stays_wide() {
        let src = "module m\n\nfn dot(a: []u64) -> u8 { return 1; }\n";
        assert_eq!(classify(src), Verdict::ForcedWide);
    }

    #[test]
    fn a_parameterless_function_is_not_a_candidate() {
        assert_eq!(classify("module m\n\nfn go() -> u8 { return 1; }\n"), Verdict::NoCandidate);
    }
}
