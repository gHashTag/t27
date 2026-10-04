//! End-to-end tests from t27 source text: the t27c front-end, lowering, then
//! every `test` and `invariant` block run in the reference interpreter and, on
//! arm64 macOS, in the JIT, which must agree with it.
//!
//! One test per construct family. Each checks both what runs (the outcome of
//! every block) and what is refused (the exact `unsupported construct`).

use std::path::Path;
use t27b::codegen::{self, TrapStyle};
use t27b::eval::{Interp, Stop};
use t27b::ir::*;
use t27b::jit::{Jit, JIT_SUPPORTED};
use t27b::{front, lower};

fn lower_src(src: &str) -> Result<Program, Vec<String>> {
    let parsed = front::parse(Path::new("/nonexistent/t.t27"), src).map_err(|e| vec![format!("parse: {}", e)])?;
    lower::lower_src(&parsed.ast, OverflowMode::Trap, Some(src)).map_err(|rs| rs.iter().map(|r| r.message()).collect())
}

/// Outcome of one block: `Ok(())`, or the trap kind and source line.
type Outcome = Result<(), (TrapKind, u32)>;

/// Run every test and invariant; returns (name, is_invariant, outcome).
fn run(src: &str) -> Vec<(String, bool, Outcome)> {
    let prog = lower_src(src).unwrap_or_else(|e| panic!("lowering failed:\n{}", e.join("\n")));
    let mut jit = if JIT_SUPPORTED {
        let code = codegen::compile(&prog, TrapStyle::Jit, true).expect("codegen");
        Some(Jit::load(&code, prog.funcs.len(), &prog.data).expect("jit load"))
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
fn rejected(src: &str) -> String {
    match lower_src(src) {
        Ok(_) => panic!("expected a rejection"),
        Err(e) => e[0].clone(),
    }
}

fn names_ok(r: &[(String, bool, Outcome)]) -> Vec<(&str, bool, bool)> {
    r.iter().map(|(n, i, o)| (n.as_str(), *i, o.is_ok())).collect()
}

// ------------------------------------------------------------ invariants

#[test]
fn invariants_run_like_tests() {
    let src = "module inv;

const N: u32 = 4;

fn sq(x: u32) -> u32 {
    return x * x;
}

invariant n_positive
    assert N > 0

invariant sq_four: sq(2) == N;

invariant broken
    assert sq(3) == N

invariant overflow: sq(70000) > 0;

invariant all_id: forall x: u32, sq(x) >= 0;

test t1 {
    assert(sq(2) == 4);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("n_positive", true, true),
            ("sq_four", true, true),
            ("broken", true, false),
            ("overflow", true, false),
            ("t1", false, true),
        ]
    );
    assert_eq!(r[2].2, Err((TrapKind::Assert, 15)));
    assert_eq!(r[3].2.unwrap_err().0, TrapKind::Overflow);
    // The quantified invariant has no body to run, and is not reported as run.
    let prog = lower_src(src).unwrap();
    assert_eq!(prog.unchecked, vec!["all_id".to_string()]);
}

#[test]
fn partially_parsed_invariant_is_rejected() {
    let src = "module inv;

const N: u32 = 4;

invariant q
    assert N > 0
    forall x in 0..N: x < N
";
    let m = rejected(src);
    assert!(m.starts_with("t27b: unsupported construct InvariantBlock at line 5"), "{}", m);
    assert!(m.contains("invariant `q` was only partially parsed"), "{}", m);
}

// --------------------------------------------------------------- structs

#[test]
fn structs_fields_pointers_and_results() {
    let src = "module st;

const Pt = struct {
    x: i32,
    y: i32,
    ok: bool,
};

const Box = struct { a: Pt, tag: u8, n: u64 = 7 };

const Node = struct { v: u32, next: *const Node };

const ORIGIN: Pt = Pt{ .x = -3, .y = 4, .ok = true };
const UNIT = Box{ .a = ORIGIN, .tag = 1 };
const OX: i32 = ORIGIN.x;

fn mk(x: i32, y: i32) Pt {
    return Pt{ .x = x, .y = y, .ok = false };
}

fn swap(p: Pt) Pt {
    return .{ .x = p.y, .y = p.x, .ok = p.ok };
}

fn sum(p: Pt) i32 {
    return p.x + p.y;
}

fn bump(p: *Pt) void {
    p.x += 1;
    p.*.y = 2;
}

fn incr(c: *u32) void {
    c.* += 1;
}

fn wrap(t: u8) Box {
    return Box{ .a = mk(5, 6), .tag = t };
}

test literals_and_fields {
    var p = mk(1, 2);
    p.x = 5;
    const q: Pt = .{ .x = 1, .y = 2, .ok = true };
    var b = Box{ .a = q, .tag = 3 };
    b.a.y = 9;
    bump(&p);
    assert(sum(p) == 8);
    assert(b.a.y == 9);
    assert(b.n == 7);
    assert(q.y == 2);
    assert(ORIGIN.x == -3);
    assert(OX == -3);
    assert(UNIT.a.y == 4 and UNIT.n == 7);
}

test results_and_copies {
    var p = mk(1, 2);
    p = swap(p);
    assert(p.x == 2 and p.y == 1);
    var w = wrap(9);
    assert(w.a.x == 5 and w.tag == 9);
    w.a = p;
    assert(w.a.y == 1);
    const c = w;
    w.tag = 0;
    assert(c.tag == 9);
    assert(sum(swap(mk(10, 20))) == 30);
    assert(mk(3, 4).y == 4);
    assert(sum(.{ .x = 1, .y = 1, .ok = true }) == 2);
}

test addresses {
    var n: u32 = 4;
    incr(&n);
    incr(&n);
    assert(n == 6);
    const tail = Node{ .v = 2, .next = undefined };
    const head = Node{ .v = 1, .next = &tail };
    assert(head.next.v == 2);
    var p = mk(0, 0);
    const pp = &p;
    pp.x = 7;
    pp.*.y += 3;
    assert(p.x == 7 and p.y == 3);
}

test overflow_through_a_field {
    var p = mk(2147483647, 0);
    p.x += 1;
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("literals_and_fields", false, true),
            ("results_and_copies", false, true),
            ("addresses", false, true),
            ("overflow_through_a_field", false, false),
        ]
    );
    assert_eq!(r[3].2.unwrap_err().0, TrapKind::Overflow);
    let prog = lower_src(src).unwrap();
    // mk, swap, sum, wrap take or return a struct; bump and incr take pointers.
    let internal: Vec<&str> = prog.internal_abi.iter().map(|&i| prog.funcs[i as usize].name.as_str()).collect();
    assert_eq!(internal, vec!["mk", "swap", "sum", "wrap"]);
}

#[test]
fn struct_rejections_are_precise() {
    let head = "module st;\n\nconst Pt = struct { x: u32, y: u32 };\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("test t { const p = Pt{ .x = 1 }; _ = p; }", "ExprStructLit", "missing field `y` of `Pt`"),
        ("test t { const p = Pt{ .x = 1, .y = 2, .z = 3 }; _ = p; }", "ExprStructLit", "`Pt` has no field `z`"),
        ("test t { const p = Pt{ .x = 1, .x = 2, .y = 3 }; _ = p; }", "ExprStructLit", "field `x` initialised twice"),
        ("test t { const p = .{ .x = 1, .y = 2 }; _ = p; }", "ExprStructLit", "anonymous `.{}` literal"),
        ("test t { const p = Pt{ .x = 1, .y = 2 }; p.x = 3; }", "StmtAssign", "assignment through a constant"),
        ("test t { const p = Pt{ .x = 1, .y = 2 }; assert(p == p); }", "type mismatch", "on a struct"),
        ("test t { const p = Pt{ .x = 1, .y = 2 }; assert(p.w == 1); }", "ExprFieldAccess", "`Pt` has no field `w`"),
        ("test t { assert(Color.red == 1); }", "ExprFieldAccess", "`Color.red`"),
        ("const L = struct { a: u32, l: L };\ntest t { const v: L = undefined; _ = v; }", "StructDecl", "`L` contains itself"),
        ("fn f(p: Pt) u32 { return p.x; }\ntest t { assert(f(3) == 3); }", "type mismatch", "expected Pt, found a scalar"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}
