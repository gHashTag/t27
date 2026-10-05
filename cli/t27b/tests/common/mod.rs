//! Helpers shared by the source-text test files that came after source.rs:
//! the t27c front-end, lowering, then every `test` and `invariant` block run
//! in the reference interpreter and, where the JIT runs, in the JIT, which
//! must agree with it.
#![allow(dead_code)]

use std::path::Path;
use t27b::codegen::{self, TrapStyle};
use t27b::eval::{Interp, Stop};
use t27b::ir::*;
use t27b::jit::{Jit, JIT_SUPPORTED};
use t27b::{front, lower};

pub fn lower_src(src: &str) -> Result<Program, Vec<String>> {
    let parsed = front::parse(Path::new("/nonexistent/t.t27"), src).map_err(|e| vec![format!("parse: {}", e)])?;
    lower::lower_src(&parsed.ast, OverflowMode::Trap, Some(src)).map_err(|rs| rs.iter().map(|r| r.message()).collect())
}

/// Outcome of one block: `Ok(())`, or the trap kind and source line.
pub type Outcome = Result<(), (TrapKind, u32)>;

/// Run every test and invariant; returns (name, is_invariant, outcome).
pub fn run(src: &str) -> Vec<(String, bool, Outcome)> {
    let prog = lower_src(src).unwrap_or_else(|e| panic!("lowering failed:\n{}", e.join("\n")));
    let mut jit = if JIT_SUPPORTED {
        let code = codegen::compile(&prog, TrapStyle::Jit, true).expect("codegen");
        Some(Jit::load(&code, prog.funcs.len(), &prog.data, &prog.globals).expect("jit load"))
    } else {
        None
    };
    let mut out = Vec::new();
    for (id, f) in prog.tests() {
        let want = Interp::new(&prog).call(id, &[]);
        let o: Outcome = match &want {
            Ok(_) => Ok(()),
            Err(Stop::Trap { site, .. }) => {
                let s = &prog.sites[*site as usize];
                Err((s.kind, s.line))
            }
            Err(e) => panic!("{}: interpreter stopped: {:?}", f.name, e),
        };
        if let Some(j) = jit.as_mut() {
            let got = j.call(id as FuncId, &[]);
            match (&want, &got) {
                (Ok(_), Ok(_)) => {}
                (Err(Stop::Trap { site, .. }), Err(t)) => assert_eq!(*site, t.site, "{}: trap site", f.name),
                _ => panic!("{}: interpreter {:?}, jit {:?}", f.name, want, got),
            }
        }
        out.push((f.name.clone(), f.is_invariant, o));
    }
    out
}

/// The first rejection message, which must exist.
pub fn rejected(src: &str) -> String {
    match lower_src(src) {
        Ok(_) => panic!("expected a rejection"),
        Err(e) => e[0].clone(),
    }
}

pub fn names_ok(r: &[(String, bool, Outcome)]) -> Vec<(&str, bool, bool)> {
    r.iter().map(|(n, i, o)| (n.as_str(), *i, o.is_ok())).collect()
}
