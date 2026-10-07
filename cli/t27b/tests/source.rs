//! End-to-end tests from t27 source text: the t27c front-end, lowering, then
//! every `test` and `invariant` block run in the reference interpreter and, on
//! arm64 macOS and arm64 Linux, in the JIT, which must agree with it.
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

/// `@compileAssert` is `assert`: t27c's Zig backend lowers both through one
/// arm. In an invariant (a `comptime` block there) a false one fails the
/// reference's compile and fails the invariant here; in a test it is a
/// runtime check in both. An integer `as f64` is `@floatFromInt` with result
/// type f64, which is what the reference emits for it.
#[test]
fn compile_assert_is_assert() {
    let src = "module ca;

const E: u8 = 3;
const N: u16 = 10;

fn half(x: u32) -> f64 {
    return x as f64 / 2.0;
}

invariant widths {
    @compileAssert(E + 7 == N);
}

invariant exponent_bounds {
    @compileAssert((E as f64 - 0.5) * 2.618033988749895 <= N as f64 - 1.0);
    @compileAssert(N as f64 - 1.0 <= (E as f64 + 0.5) * 2.618033988749895);
}

invariant broken {
    @compileAssert(E as f64 > 3.5, \"exponent too small\");
}

test runtime_operand {
    @compileAssert(half(7) == 3.5);
}

test runtime_false {
    @compileAssert(half(7) == 3.0);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("widths", true, true),
            ("exponent_bounds", true, true),
            ("broken", true, false),
            ("runtime_operand", false, true),
            ("runtime_false", false, false),
        ]
    );
    assert_eq!(r[2].2, Err((TrapKind::Assert, 20)));
    assert_eq!(r[4].2, Err((TrapKind::Assert, 28)));

    // The message must be a string literal, as for `assert`.
    let m = rejected("module ca2;\nconst E: u8 = 3;\ninvariant i {\n    @compileAssert(E == 3, E);\n}\n");
    assert!(m.contains("unsupported construct ExprCall(assert with non-literal message)"), "{}", m);
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
    if (true) { p = swap(p); }
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
    const np = &n;
    np.* += 2;
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
    // mk, swap, sum, wrap take or return a struct; bump takes a pointer.
    let internal: Vec<&str> = prog.internal_abi.iter().map(|&i| prog.funcs[i as usize].name.as_str()).collect();
    assert_eq!(internal, vec!["mk", "swap", "sum", "wrap"]);
}

/// A scalar field initialised from a call inside a struct literal is a store
/// of the call's value. It once went down the in-place path meant for an
/// aggregate result, passing a result pointer to a fn that returns a
/// register, so the field was never written (gf16_dot4.t27, d6_test.t27).
#[test]
fn struct_field_initialised_from_a_scalar_call() {
    let src = "module sf;

const In = struct {
    a0: u16,
    b0: u16,
};

const Out = struct {
    result: u16,
};

fn mul(a: u16, b: u16) u16 {
    return a * b;
}

fn dot(inputs: In) Out {
    var r: Out = Out{ .result = mul(inputs.a0, inputs.b0) };
    return r;
}

fn direct(x: u16) Out {
    return Out{ .result = mul(x, x) };
}

test field_from_call {
    const o = dot(In{ .a0 = 3, .b0 = 5 });
    assert(o.result == 15);
    assert(direct(4).result == 16);
}

test overflow_in_field_call {
    const o = dot(In{ .a0 = 300, .b0 = 300 });
    assert(o.result == 0);
}
";
    let r = run(src);
    assert_eq!(names_ok(&r), vec![("field_from_call", false, true), ("overflow_in_field_call", false, false)]);
    assert_eq!(r[1].2.unwrap_err().0, TrapKind::Overflow);
}

#[test]
fn struct_rejections_are_precise() {
    let head = "module st;\n\nconst Pt = struct { x: u32, y: u32 };\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("test t { const p = Pt{ .x = 1 }; _ = p; }", "ExprStructLit", "missing field `y` of `Pt`"),
        ("test t { const p = Pt{ .x = 1, .y = 2, .z = 3 }; _ = p; }", "ExprStructLit", "`Pt` has no field `z`"),
        ("test t { const p = Pt{ .x = 1, .x = 2, .y = 3 }; _ = p; }", "ExprStructLit", "field `x` initialised twice"),
        ("test t { const p = .{ .x = 1, .y = 2 }; _ = p; }", "ExprStructLit", "anonymous `.{}` literal"),
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

// --------------------------------------------------------------- strings

#[test]
fn strings_literals_params_fields_and_equality() {
    let src = "module s;

pub const NAME: str = \"clk\";
const OTHER: &str = \"clk\";
const EMPTY = \"\";
const N = NAME.len;

const Pin = struct { name: str, num: u32 };

fn is_denied(answer: str) bool {
    return answer == \"denied\";
}

fn pick(b: bool) str {
    if (b) {
        return \"yes\";
    }
    return \"no\";
}

fn pin_name(p: Pin) []const u8 {
    return p.name;
}

fn length(s: string) u64 {
    return s.len;
}

test consts_fold {
    assert(NAME == \"clk\");
    assert(NAME == OTHER);
    assert(NAME != \"clK\");
    assert(EMPTY.len == 0 and N == 3);
    assert(\"a b \".len == 4);
}

test runtime_compare {
    assert(is_denied(\"denied\"));
    assert(!is_denied(\"denie\"));
    assert(!is_denied(\"Denied\"));
    assert(!is_denied(\"\"));
    assert(pick(true) == \"yes\" and pick(false) != \"yes\");
    assert(length(pick(false)) == 2);
}

test locals_and_fields {
    var s: str = \"abc\";
    const t = s;
    if (true) { s = \"tab\\tq\"; }
    assert(s.len == 5 and t.len == 3);
    assert(t == \"abc\" and s == \"tab\\tq\");
    var p = Pin{ .name = \"T9\", .num = 9 };
    assert(p.name == \"T9\");
    p.name = NAME;
    assert(pin_name(p) == NAME and pin_name(p).len == 3);
    var u: str = \"lit\";
    if (true) { u = \"other\"; }
    assert(u.len == 5);
}

test unequal_fails {
    assert(is_denied(\"granted\"));
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("consts_fold", false, true),
            ("runtime_compare", false, true),
            ("locals_and_fields", false, true),
            ("unequal_fails", false, false),
        ]
    );
    assert_eq!(r[3].2, Err((TrapKind::Assert, 62)));
    let prog = lower_src(src).unwrap();
    let internal: Vec<&str> = prog.internal_abi.iter().map(|&i| prog.funcs[i as usize].name.as_str()).collect();
    assert_eq!(internal, vec!["is_denied", "pick", "pin_name", "length", lower::STR_EQL]);
    // The helper sits right after the source fns, before the tests.
    assert_eq!(prog.funcs[4].name, lower::STR_EQL);
    assert!(prog.funcs[5].is_test);
}

/// A module constant of a struct type with `str` fields (Zig's comptime
/// struct value): a compile-time value written into a frame temporary where
/// memory is needed, like a constant array of strings.
#[test]
fn struct_constants_with_str_fields() {
    let src = "module sc;

const Dir = enum(u8) { in, out };
const Pos = struct { x: u8, y: u8 };
const Pin = struct {
    name: str,
    bank: u8,
    dir: Dir = .in,
    at: Pos,
    tag: str = \"io\",
};
const Board = struct { title: str, pins: [2]Pin, count: u32 };

const CLK: Pin = Pin{ .name = \"E3\", .bank = 35, .at = Pos{ .x = 1, .y = 2 } };
const TX = Pin{ .name = \"D10\", .bank = 16, .dir = .out, .at = .{ .x = 3, .y = 4 }, .tag = \"uart\" };
const ALIAS: Pin = CLK;
const PINS: [2]Pin = [CLK, TX];
const BOARD: Board = Board{ .title = \"arty\", .pins = [TX, CLK], .count = 2 };

fn bank_of(p: Pin) u8 {
    return p.bank;
}

fn name_len(p: *const Pin) u64 {
    return p.name.len;
}

test fields_fold_and_load {
    assert(CLK.name == \"E3\" and CLK.bank == 35 and CLK.dir == .in);
    assert(CLK.at.x == 1 and CLK.at.y == 2 and CLK.tag == \"io\");
    assert(TX.dir == .out and TX.tag == \"uart\" and TX.name.len == 3);
    assert(ALIAS.name == CLK.name);
}

test bound_passed_and_copied {
    const pin = TX;
    assert(pin.name == \"D10\" and bank_of(pin) == 16);
    assert(bank_of(CLK) == 35 and name_len(&CLK) == 2);
    var p = CLK;
    p.bank = 7;
    p.name = \"X\";
    assert(p.bank == 7 and p.name == \"X\" and CLK.bank == 35 and CLK.name == \"E3\");
}

test arrays_and_nesting {
    assert(PINS[1].name == \"D10\" and PINS.len == 2);
    var i: u32 = 0;
    assert(PINS[i].bank == 35);
    var n: u64 = 0;
    for (PINS) |q| {
        n += q.name.len;
    }
    assert(n == 5);
    assert(BOARD.title == \"arty\" and BOARD.pins[0].name == \"D10\" and BOARD.pins[1].at.y == 2);
    assert(BOARD.count == 2);
}

test wrong_name_fails {
    assert(CLK.name == \"E4\");
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("fields_fold_and_load", false, true),
            ("bound_passed_and_copied", false, true),
            ("arrays_and_nesting", false, true),
            ("wrong_name_fails", false, false),
        ]
    );
    assert_eq!(r[3].2, Err((TrapKind::Assert, 59)));
}

#[test]
fn string_rejections_are_precise() {
    let head = "module s;\n\nconst S: str = \"ab\";\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("test t { assert(S == 3); }", "type mismatch", "expected str, found a scalar"),
        ("test t { assert(S < \"b\"); }", "ExprBinary(<)", "on a string"),
        ("test t { assert(S.ptr == 0); }", "ExprFieldAccess(str)", "`.ptr` of a str"),
        ("test t { var s: str = \"x\"; s.len = 2; }", "StmtAssign", "assignment through a constant"),
        ("test t { assert(S); }", "condition", "expected bool, found a string"),
        ("const P = struct { s: str };\nvar Q: P = P{ .s = \"x\" };\ntest t { assert(Q.s.len == 1); }", "VarDecl(module, pointer/str/slice)", "module-level var `Q`"),
        ("fn g() str { return \"x\"; }\nconst P = struct { s: str };\nconst Q = P{ .s = g() };\ntest t { assert(Q.s.len == 1); }", "ConstDecl", "not a string literal"),
        ("const P = struct { s: str, n: u8 };\nfn h() u8 { return 1; }\nconst Q = P{ .s = \"x\", .n = h() };\ntest t { assert(Q.n == 1); }", "ConstDecl", "not a compile-time value"),
        ("fn f() u32 { return 1; }\nconst T: str = f();\ntest t { assert(T.len == 0); }", "ConstDecl", "not a string literal"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

// ---------------------------------------------------------------- arrays

#[test]
fn arrays_literals_indexing_len_and_for() {
    let src = "module a;

const N: u32 = 3;
const PRIMES: [4]u32 = [2, 3, 5, 7];
const FLAGS: [2]bool = [true, false];
pub const NAMES: [2]str = [\"in\", \"out\"];
const NONE: [0]str = [];

const Bus = struct { lanes: [N]u8, width: u16 };

fn total(a: [4]u32) u32 {
    var s: u32 = 0;
    for (a) |x| {
        s += x;
    }
    return s;
}

fn ramp(k: u8) [3]u8 {
    return [k, k + 1, k + 2];
}

fn at(a: [4]u32, i: u32) u32 {
    return a[i];
}

fn reset(a: [3]u8) [3]u8 {
    var r: [3]u8 = [0, 0, 0];
    r = a;
    r[0] = 1;
    return r;
}

fn fill(a: *[3]u8, v: u8) void {
    a[0] = v;
    a.*[2] = v;
}

test locals_and_indexing {
    var a: [4]i32 = [10, -20, 30, -40];
    a[1] = 5;
    a[3] += 1;
    var i: u32 = 2;
    assert(a[i] == 30);
    a[i] = a[0] + a[1];
    assert(a[2] == 15 and a[3] == -39);
    assert(a.len == 4 and PRIMES.len == 4);
    const b: [N]bool = [true, false, true];
    assert(b[0] and !b[1] and b[2]);
}

test consts_and_strings {
    assert(PRIMES[3] == 7 and at(PRIMES, 2) == 5);
    assert(FLAGS[0] and !FLAGS[1]);
    assert(NAMES[1] == \"out\" and NAMES.len == 2 and NONE.len == 0);
    var j: u32 = 0;
    assert(NAMES[j] == \"in\");
    var n: u64 = 0;
    for (NAMES) |s| {
        n += s.len;
    }
    assert(n == 5);
}

test copies_params_and_results {
    var a: [3]u8 = [1, 2, 3];
    const c = a;
    a[0] = 9;
    assert(c[0] == 1 and a[0] == 9);
    var r = ramp(4);
    assert(r[2] == 6 and ramp(1)[0] == 1);
    fill(&r, 0);
    assert(r[0] == 0 and r[1] == 5 and r[2] == 0);
    const d = reset(c);
    assert(d[0] == 1 and d[1] == 2 and c[0] == 1);
    assert(total(PRIMES) == 17 and total([1, 1, 1, 1]) == 4);
    var bus = Bus{ .lanes = [7, 8, 9], .width = 3 };
    bus.lanes[1] = 0;
    assert(bus.lanes[1] == 0 and bus.lanes.len == 3);
}

test for_with_break_and_continue {
    const a: [5]u32 = [1, 2, 3, 4, 5];
    var s: u32 = 0;
    for (a) |x| {
        if (x == 2) {
            continue;
        }
        if (x == 5) {
            break;
        }
        s += x;
    }
    assert(s == 8);
    var count: u32 = 0;
    for (a) |_| {
        count += 1;
    }
    assert(count == 5);
}

test index_out_of_bounds {
    const a: [3]u32 = [1, 2, 3];
    var i: u32 = 3;
    assert(a[i] == 0);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("locals_and_indexing", false, true),
            ("consts_and_strings", false, true),
            ("copies_params_and_results", false, true),
            ("for_with_break_and_continue", false, true),
            ("index_out_of_bounds", false, false),
        ]
    );
    assert_eq!(r[4].2, Err((TrapKind::Bounds, 105)));
}

#[test]
fn array_rejections_are_precise() {
    let head = "module a;\n\nconst A: [3]u32 = [1, 2, 3];\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("test t { const b: [3]u32 = [1, 2]; _ = b; }", "ExprArrayLiteral", "2 elements for `[3]u32`"),
        ("test t { const b = [1, 2]; _ = b; }", "ExprArrayLiteral", "array literal with no result type"),
        ("test t { assert(A[3] == 0); }", "ExprIndex", "index 3 out of bounds for `[3]u32`"),
        ("test t { var i: i32 = 0; assert(A[i] == 1); }", "type mismatch", "expected u64"),
        ("test t { assert(A == A); }", "type mismatch", "on an array"),
        ("test t { var b: [3]u32 = A; b.len = 2; }", "ExprFieldAccess(.len)", "not a place"),
        ("test t { assert(A.ptr == 0); }", "ExprFieldAccess", "`.ptr` of an array"),
        ("test t { const s: str = \"ab\"; assert(s.ptr == 0); }", "ExprFieldAccess(str)", "`.ptr` of a str"),
        ("test t { const b: [N]u32 = undefined; _ = b; }", "type [N]T", "not a compile-time integer"),
        ("test t { A[0] = 2; }", "StmtAssign", "assignment through a constant"),
        ("test t { assert(A); }", "condition", "expected bool, found an array"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

/// t27's own array spellings, matched to t27c's Zig backend: `[T; N]` is
/// `[N]T`, `[T]` is `[]T`, a repeat (`[v; n]`, `[_]T{ ... } ** n`) is
/// `.{ ... } ** n` with its elements evaluated once, an array literal passed
/// to a slice parameter is `@constCast(&[_]T{ ... })`, and an untyped local
/// so passed is a `[_]T` array passed by address. `t27c test-report` on this
/// source: 3 pass, `repeat_fails` FAIL.
#[test]
fn t27_array_spellings_repeats_and_slice_arguments() {
    let src = "module a;

const N: u32 = 4;
const ZEROS: [4]u32 = [_]u32{0} ** N;
const GRID: [9]f64 = [_]f64{0} ** 9;

var calls: u32 = 0;

const Rec = struct { id: u32, w: u32 };

fn tick() u32 {
    calls += 1;
    return calls;
}

fn total(xs: [u32]) u32 {
    var s: u32 = 0;
    for (xs) |x| {
        s += x;
    }
    return s;
}

fn first_plus(xs: [u32], k: u32) void {
    xs[0] = xs[0] + k;
}

fn weight(rs: [Rec]) u32 {
    var s: u32 = 0;
    for (rs) |r| {
        s += r.w;
    }
    return s;
}

fn rec(i: u32) Rec {
    return Rec{ .id = i, .w = i * 10 };
}

fn sum4(a: [u32; 4]) u32 {
    return a[0] + a[1] + a[2] + a[3];
}

fn spread(v: u32) [u32; 4] {
    return [v; 4];
}

test repeats {
    var a: [u32; 3] = [tick(); 3];
    a[1] = 9;
    assert(a[0] == 1 and a[1] == 9 and a[2] == 1 and calls == 1);
    const g: [5]u32 = [_]u32{7} ** 5;
    assert(g[4] == 7 and g.len == 5);
    const p: [4]u32 = [_]u32{ 1, 2 } ** 2;
    assert(p[2] == 1 and p[3] == 2);
    assert(sum4(spread(3)) == 12 and sum4(ZEROS) == 0 and GRID[8] == 0.0);
    const rs: [3]Rec = [rec(2); 3];
    assert(rs[2].w == 20 and rs[0].id == 2);
}

test slice_arguments {
    assert(total([1, 2, 3]) == 6 and total([]) == 0);
    assert(weight([rec(1), rec(2)]) == 30);
    const one: u32 = 5;
    assert(total([one]) == 5);
}

test slice_locals {
    var xs = [4, 5, 6];
    first_plus(xs, 10);
    assert(xs[0] == 14 and total(xs) == 25 and xs.len == 3);
}

test repeat_fails {
    const b: [2]u32 = [3; 2];
    assert(sum4([b[0], b[1], 0, 0]) == 7);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("repeats", false, true),
            ("slice_arguments", false, true),
            ("slice_locals", false, true),
            ("repeat_fails", false, false),
        ]
    );
    assert_eq!(r[3].2, Err((TrapKind::Assert, line_of(src, "sum4([b[0], b[1], 0, 0]) == 7"))));
}

#[test]
fn t27_array_spelling_rejections_are_precise() {
    let head = "module a;\n\nconst ONE: u32 = 1;\n\nfn total(xs: [u32]) u32 {\n    return 0;\n}\n\nfn nested(xs: [[2]u32]) u32 {\n    return 0;\n}\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("fn f(p: [*]u8) u32 { return 0; }", "type [*]T", "a many-item pointer"),
        ("fn f(m: [str:u32]) u32 { return 0; }", "type [K:V]", "a map"),
        ("test t { assert(total([1; 2]) == 2); }", "ExprArrayLiteral(repeat to slice)", "where a slice is declared"),
        (
            "test t { var v: u32 = 1; const a: [2]u32 = [v + 1; 2]; assert(a[0] == 2); }",
            "ExprArrayLiteral(text element)",
            "element `v+1` is not a literal",
        ),
        (
            "test t { const ONE: u32 = 2; const a: [2]u32 = [ONE; 2]; assert(a[0] == 2); }",
            "ExprArrayLiteral(text element)",
            "`ONE` shadows a module-level name",
        ),
        ("test t { assert(nested([[1, 2]]) == 0); }", "ExprArrayLiteral(to slice)", "a slice of arrays or slices"),
        ("test t { const a: [3]u32 = [_]u32{ 1, 2 } ** 2; assert(a[0] == 1); }", "ExprArrayLiteral", "4 elements for `[3]u32`"),
        ("test t { const a: [2]u32 = [7; 0]; assert(a.len == 2); }", "ExprArrayLiteral(repeat count)", "repeated zero times"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

// ---------------------------------------------------------------- slices

/// The 1-based line of the first line of `src` containing `needle`.
fn line_of(src: &str, needle: &str) -> u32 {
    src.lines().position(|l| l.contains(needle)).expect("needle") as u32 + 1
}

#[test]
fn slices_params_slicing_len_index_and_for() {
    let src = "module a;

const Pair = struct { xs: []const u32, tag: u8 };

fn sum(xs: []const u32) u32 {
    var t: u32 = 0;
    for (xs) |x| {
        t += x;
    }
    return t;
}

fn fill(xs: []u32, v: u32) void {
    var i: u64 = 0;
    while (i < xs.len) {
        xs[i] = v;
        i += 1;
    }
}

fn tail(xs: []const u32) []const u32 {
    return xs[1..];
}

fn first(s: []const u8) u8 {
    return s[0];
}

test slice_of_array {
    var a: [5]u32 = [1, 2, 3, 4, 5];
    assert(sum(&a) == 15);
    assert(sum(a[1..3]) == 5);
    assert(sum(a[2..]) == 12);
    assert(sum(a[5..]) == 0 and sum(a[2..2]) == 0);
    const t = a[1..4];
    assert(t.len == 3 and t[0] == 2 and t[2] == 4);
    const u = t[1..];
    assert(u.len == 2 and u[1] == 4);
    assert(sum(tail(&a)) == 14);
    var lo: u64 = 1;
    var hi: u64 = 4;
    assert(sum(a[lo..hi]) == 9);
}

test writes_through_a_slice {
    var a: [4]u32 = [1, 2, 3, 4];
    fill(a[1..3], 7);
    assert(a[0] == 1 and a[1] == 7 and a[2] == 7 and a[3] == 4);
    const s: []u32 = a[0..2];
    s[1] = 9;
    assert(a[1] == 9);
    fill(&a, 0);
    assert(sum(&a) == 0);
}

test strings_index_and_slice {
    const s: []const u8 = \"hello\";
    assert(s.len == 5 and s[1] == 101);
    var i: u64 = 4;
    assert(s[i] == 111);
    assert(\"abc\"[2] == 99);
    assert(first(s[3..]) == 108);
    assert(s[1..3] == \"el\");
    var n: u32 = 0;
    for (s[0..2]) |c| {
        n += c;
    }
    assert(n == 104 + 101);
    var buf: [3]u8 = [120, 121, 122];
    assert(first(&buf) == 120 and buf[0..2] == \"xy\");
}

test slice_literals_and_fields {
    assert(sum(&[_]u32{ 10, 20 }) == 30);
    assert(sum(&[_]u32{ 0 }) == 0);
    const p = Pair{ .xs = &[_]u32{ 3, 4 }, .tag = 1 };
    assert(p.xs.len == 2 and p.xs[1] == 4 and sum(p.xs) == 7);
}

test index_past_the_end {
    var a: [3]u32 = [1, 2, 3];
    const s = a[0..2];
    var i: u64 = 2;
    assert(s[i] == 3);
}

test slice_end_past_the_length {
    var a: [3]u32 = [1, 2, 3];
    var j: u64 = 4;
    assert(sum(a[0..j]) == 0);
}

test slice_start_past_the_end {
    const s: []const u8 = \"abc\";
    var i: u64 = 2;
    assert(s[i..1].len == 0);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("slice_of_array", false, true),
            ("writes_through_a_slice", false, true),
            ("strings_index_and_slice", false, true),
            ("slice_literals_and_fields", false, true),
            ("index_past_the_end", false, false),
            ("slice_end_past_the_length", false, false),
            ("slice_start_past_the_end", false, false),
        ]
    );
    assert_eq!(r[4].2, Err((TrapKind::Bounds, line_of(src, "assert(s[i] == 3)"))));
    assert_eq!(r[5].2, Err((TrapKind::Bounds, line_of(src, "a[0..j]"))));
    assert_eq!(r[6].2, Err((TrapKind::Bounds, line_of(src, "s[i..1]"))));
}

#[test]
fn slice_rejections_are_precise() {
    let head = "module a;\n\nconst A: [3]u32 = [1, 2, 3];\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("fn f(xs: []u32) u64 { return xs.len; }\ntest t { assert(f(&A) == 3); }", "type mismatch", "expected []u32, found a pointer"),
        ("fn f(xs: []const u32) u64 { return xs.len; }\ntest t { var b: [3]u32 = A; assert(f(b) == 3); }", "type mismatch", "expected []const u32, found [3]u32"),
        ("fn f(xs: []u8) u64 { return xs.len; }\ntest t { assert(f(\"ab\") == 2); }", "type mismatch", "expected []u8, found a string"),
        ("test t { assert(A[1..4].len == 3); }", "ExprIndex(slice)", "end 4 out of bounds for `[3]u32`"),
        ("test t { assert(A[2..1].len == 0); }", "ExprIndex(slice)", "start 2 is past end 1"),
        ("test t { assert(\"ab\"[2] == 0); }", "ExprIndex", "index 2 out of bounds for a string of length 2"),
        ("fn f(xs: []u32) void { xs.len = 0; }\ntest t { }", "StmtAssign", "assignment through a constant"),
        ("fn f(xs: []const u32) void { xs[0] = 1; }\ntest t { }", "StmtAssign", "assignment through a constant"),
        ("const S: []const u32 = A[0..];\ntest t { assert(S.len == 3); }", "ConstDecl(slice)", "module-level slice"),
        ("fn f(xs: []u32) u32 { return xs.ptr; }\ntest t { }", "ExprFieldAccess(slice)", "`.ptr` of a []u32"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

// ------------------------------------------------------------------ enums

#[test]
fn plain_enums_as_integer_tags() {
    let src = r#"module enums;

const Op = enum(u8) {
    add = 1,
    sub,
    mul = 7,
};

const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

enum Dir {
    north,
    east,
    south,
    west,
}

const One = enum { only };

const Cell = struct {
    op: Op,
    t: Trit,
    n: u32,
};

const START: Dir = Dir.east;

fn next(d: Dir) Dir {
    if (d == .north) {
        return .east;
    }
    if (d == Dir.east) {
        return Dir::south;
    }
    if (d != .south) {
        return .north;
    }
    return .west;
}

fn neg(t: Trit) Trit {
    return @enumFromInt(-@intFromEnum(t));
}

fn code(o: Op) u8 {
    return @intFromEnum(o);
}

fn dir_of(x: u32) Dir {
    return @enumFromInt(x);
}

fn trit_of(x: i32) Trit {
    return @enumFromInt(x);
}

fn wide(d: Dir) u32 {
    return @intFromEnum(d);
}

fn step(d: Dir) Dir {
    var cur: Dir = d;
    cur = next(cur);
    cur = next(cur);
    return cur;
}

fn cell(o: Op) Cell {
    return Cell{ .op = o, .t = .pos, .n = 3 };
}

test tags {
    assert(@intFromEnum(Op.add) == 1);
    assert(@intFromEnum(Op.sub) == 2);
    assert(code(Op.mul) == 7);
    assert(@intFromEnum(Trit.neg) == -1);
    assert(@intFromEnum(Dir.west) == 3);
    assert(wide(Dir.south) == 2);
    assert(@intFromEnum(One.only) == 0);
}

test compare {
    assert(next(Dir.north) == Dir.east);
    assert(next(.east) == .south);
    assert(next(Dir.south) == Dir.west);
    assert(step(START) == Dir.west);
    assert(Dir.north < Dir.west);
    assert(Op.mul > Op.sub);
    assert(next(.west) != Dir.west);
    assert_eq(next(Dir.north), Dir.east);
    assert_eq(neg(.pos), .neg);
}

test conversions {
    assert(neg(Trit.neg) == Trit.pos);
    assert(neg(.zero) == .zero);
    assert(dir_of(3) == Dir.west);
    assert(trit_of(-1) == Trit.neg);
    assert(@as(Trit, .pos) == Trit.pos);
    assert(@as(u8, @intFromEnum(Dir.east)) == 1);
}

test fields {
    const c = cell(Op.sub);
    assert(c.op == Op.sub);
    assert(c.t == .pos);
    assert(@intFromEnum(c.op) + c.n == 5);
}

test bad_dir {
    assert(dir_of(4) == Dir.west);
}

test bad_trit {
    assert(trit_of(2) == Trit.pos);
}
"#;
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("tags", false, true),
            ("compare", false, true),
            ("conversions", false, true),
            ("fields", false, true),
            ("bad_dir", false, false),
            ("bad_trit", false, false),
        ]
    );
    // `@enumFromInt` of a value that is no tag traps, as Zig's safety check
    // does ("invalid enum value").
    assert_eq!(r[4].2, Err((TrapKind::EnumTag, line_of(src, "fn dir_of") + 1)));
    assert_eq!(r[5].2, Err((TrapKind::EnumTag, line_of(src, "fn trit_of") + 1)));
}

#[test]
fn enum_rejections_are_precise() {
    let head = "module a;\n\nenum Dir { north, east, south, west, }\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("const E = enum { a, b, pub const x = 1; };\ntest t { assert(E.a == E.a); }", "EnumDecl(method)", "`E` declares something inside it"),
        ("const E = enum(u8) { a, b, _ };\ntest t { assert(E.a == E.a); }", "EnumDecl(non-exhaustive)", "`E` has a `_` variant"),
        ("const U = union(enum) { a: u8, b: u32 };\ntest t { assert(1 == 1); }", "EnumDecl(union)", "tagged union `U`"),
        ("const E = enum(f32) { a, b };\ntest t { assert(E.a == E.a); }", "EnumDecl(tag type)", "`E` has tag type `f32`"),
        ("const E = enum(u8) { a = 300, b };\ntest t { assert(E.a == E.a); }", "EnumDecl", "`E.a` = 300 does not fit the tag type u8"),
        ("const E = enum(u8) { a = 1, b = 1 };\ntest t { assert(E.a == E.b); }", "EnumDecl", "`E.a` and `E.b` have the same tag 1"),
        ("fn f(d: Dir) u8 {\n    return switch (d) {\n        .north => 1,\n        .east => 1,\n        .south => 1,\n        .west => 1,\n        else => 2,\n    };\n}\ntest t { assert(f(.north) == 1); }", "ExprSwitch(unreachable else)", "after every value is listed"),
        ("fn f(a: Dir, b: Dir) bool { return a < b; }\ntest t { assert(f(.north, .east)); }", "ExprBinary(<) on enum", "two `Dir` values, which Zig does not order"),
        ("fn f(d: Dir) bool { return d < .west; }\ntest t { assert(f(.north)); }", "ExprBinary(<) on enum", "which Zig does not order"),
        ("test t { assert(Dir.north + 1 == 1); }", "ExprBinary(+) on enum", "arithmetic on an enum"),
        ("test t { assert(Dir.north == 0); }", "type mismatch", "`==` on Dir and a scalar"),
        ("test t { const x = .north; assert(x == Dir.north); }", "ExprEnumValue", "enum literal `.north` with no result type"),
        ("test t { assert(Dir.up == Dir.north); }", "ExprFieldAccess(enum)", "`Dir` has no variant `up`"),
        ("const D: Dir = @enumFromInt(9);\ntest t { assert(D == Dir.north); }", "ExprCall(@enumFromInt)", "9 is no tag of `Dir`"),
        ("test t { assert(@enumFromInt(1) == Dir.east); }", "ExprCall(@enumFromInt)", "with no enum result type"),
        ("const E = enum(u8) { a = 1, b = 5 };\nfn f(x: u8) E { return @enumFromInt(x); }\ntest t { assert(f(1) == E.a); }", "ExprCall(@enumFromInt)", "whose tags are not contiguous"),
        ("fn f(d: Dir) u8 { const x = @intFromEnum(d); return x; }\ntest t { assert(f(.east) == 1); }", "ExprCall(@intFromEnum auto-tag)", "the tag type of `Dir` is u2"),
        ("test t { assert(@intFromEnum(.north) == 0); }", "ExprCall(@intFromEnum)", "of `.north`, which has no enum type here"),
        ("fn f(d: Dir) u32 { return d; }\ntest t { assert(f(.east) == 1); }", "type mismatch", "found Dir"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

// ------------------------------------------------------- module-level vars

#[test]
fn module_vars_are_fresh_per_test() {
    // t27c test-report runs every test in its own process, so each test sees
    // the initial values; the JIT and the interpreter reset them per entry.
    let src = "module mv;

const Mode = enum(u8) { idle, run, stop };
const Pt = struct { x: i32, y: i32 };

var counter: u32 = 14;
var flag: bool = false;
var small: u8 = 254;
var buf: [4]u32 = [1, 2, 3, 4];
var mode: Mode = .idle;
var pos: Pt = Pt { .x = 1, .y = -2 };

fn bump() {
    counter += 1;
    flag = !flag;
    mode = .run;
    pos.x = pos.x + 10;
}

fn poke(i: u32, v: u32) {
    buf[i] = v;
}

fn sum() u32 {
    var s: u32 = 0;
    var i: u32 = 0;
    while (i < 4) {
        s = s + buf[i];
        i = i + 1;
    }
    return s;
}

fn grow() u8 {
    small = small + 3;
    return small;
}

test first_bump {
    assert(counter == 14);
    bump();
    assert(counter == 15);
    assert(flag == true);
    assert(mode == .run);
    assert(pos.x == 11);
    assert(pos.y == -2);
}

test second_sees_fresh {
    assert(counter == 14);
    assert(flag == false);
    assert(mode == .idle);
    assert(pos.x == 1);
    var k: u32 = 0;
    while (k < 3) {
        counter = counter + 2;
        k = k + 1;
    }
    assert(counter == 20);
}

test array_state {
    assert(sum() == 10);
    poke(2, 100);
    assert(sum() == 107);
}

test array_fresh {
    assert(sum() == 10);
}

test overflow_traps {
    assert(grow() > 0);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("first_bump", false, true),
            ("second_sees_fresh", false, true),
            ("array_state", false, true),
            ("array_fresh", false, true),
            ("overflow_traps", false, false),
        ]
    );
    assert!(matches!(r[4].2, Err((TrapKind::Overflow, _))), "{:?}", r[4].2);
}

#[test]
fn module_var_rejections_are_precise() {
    let head = "module a;\n\nvar g: u32 = 0;\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("test t { var g: u32 = 1; assert(g == 1); }", "StmtLocal(shadows module var)", "`g` shadows"),
        ("fn f(g: u32) u32 { g = g + 1; return g; }\ntest t { assert(f(1) == 2); }", "StmtLocal(shadows module var)", "`g` shadows"),
        ("fn f(g: u32) u32 { return g; }\ntest t { assert(g == 0); }", "ExprIdentifier(renamed module var)", "`g_arg`"),
        ("invariant i { assert(g == 0); }", "ExprIdentifier(var at comptime)", "module-level var `g`"),
        ("var h = 3;\ntest t { assert(h == 3); }", "VarDecl(module, untyped)", "`h` has no type"),
        ("const B: u32 = 2;\nvar h: u32 = B * 2;\ntest t { assert(h == 4); }", "VarDecl(module)", "not a compile-time integer"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
    // A parameter the body only reads is renamed by the reference, so it
    // does not shadow; the following fn clears the rename.
    let ok = "module b;\n\nvar g: u32 = 7;\n\nfn f(g: u32) u32 { return g + 1; }\nfn h() u32 { return g; }\n\ntest t {\n    assert(f(1) == 2);\n    assert(h() == 7);\n}\n";
    assert_eq!(names_ok(&run(ok)), vec![("t", false, true)]);
    // Since #6295 a top-level write to a module var in a test is a write to
    // module state in the reference too (#6911): the fn sees it, and the
    // next test starts from the declared value again.
    let write = "module c;\n\nvar g: u32 = 0;\n\nfn read() u32 { return g; }\n\ntest w {\n    g = 5;\n    assert(read() == 5);\n    g += 1;\n    assert(g == 6);\n}\n\ntest fresh {\n    assert(g == 0);\n}\n";
    assert_eq!(names_ok(&run(write)), vec![("w", false, true), ("fresh", false, true)]);
}

#[test]
fn assert_with_message_checks_the_condition_only() {
    // t27c's Zig backend lowers `assert(cond, "msg")` to
    // `if (!(cond)) @panic("msg")`: the message never decides the verdict.
    let src = "module am;

fn twice(x: u32) u32 {
    return x * 2;
}

test holds {
    assert(twice(3) == 6, \"twice(3) is 6\");
}

test fails {
    assert(twice(3) == 7, \"twice(3) is not 7\");
}

invariant msg_invariant {
    assert(twice(1) == 2, \"invariant with a message\");
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![("holds", false, true), ("fails", false, false), ("msg_invariant", true, true)]
    );
    assert!(matches!(r[1].2, Err((TrapKind::Assert, 12))), "{:?}", r[1].2);
    // A message that is not a string literal is refused by name.
    let head = "module an;\n\nconst M: u32 = 1;\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("test t { assert(M == 1, M); }", "ExprCall(assert with non-literal message)", "not a string literal"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

#[test]
fn undefined_stub_only_where_zig_never_looks() {
    // `fn f() { undefined; }` is the stub a port leaves where plumbing was.
    // t27c's Zig backend emits it as is; Zig rejects it only in a fn it
    // analyzes. Nothing a test reaches names `plumbing` or `stub` here, and
    // `pub` or `main` is not a root, so the reference runs both tests.
    let src = "module st;

fn stub() u32 {
    undefined;
}

fn plumbing() {
    undefined;
}

fn early(x: u32) u32 {
    if (x > 0) {
        return x;
    }
    return undefined;
}

pub fn main() {
    plumbing();
    var x: u32 = stub();
}

fn inc(x: u32) u32 {
    return x + 1;
}

test inc_works {
    assert(inc(1) == 2);
}

test inc_fails {
    assert(inc(1) == 3);
}
";
    let r = run(src);
    assert_eq!(names_ok(&r), vec![("inc_works", false, true), ("inc_fails", false, false)]);
    // A stub something analyzed can reach (even through another fn, even on
    // a branch no test takes) makes the reference fail to compile: refused.
    let reached = [
        "fn stub() { undefined; }\nfn g() { stub(); }\ninvariant i { g(); assert(true); }\n",
        "test t { undefined; }\n",
    ];
    for body in reached {
        let m = rejected(&format!("module sr;\n\n{}", body));
        assert!(
            m.starts_with("t27b: unsupported construct ExprIdentifier(undefined) statement at line"),
            "{}: {}",
            body,
            m
        );
    }
    // In a non-void fn the stub is the tail, which the reference returns
    // since #6315 (`return undefined;`): it compiles, and a call hands the
    // caller an undefined value. `t27c test-report` passes the first source
    // here. t27b refuses the file rather than guess the value.
    let m = rejected("module sr;\n\nfn stub() u32 { undefined; }\nfn f(x: u32) u32 { if (x > 9) { return stub(); } return x; }\ntest t { assert(f(1) == 1); }\n");
    assert!(m.starts_with("t27b: unsupported construct ExprReturn(undefined) at line"), "{}", m);
    // An optional result is built in the caller's memory: a `return
    // undefined;` no test takes leaves it unwritten, as before #6315, and
    // the reference passes both tests (a port's health route does this).
    let opt = "module so;

fn maybe(b: bool) ?u32 {
    if (b) {
        return null;
    }
    return undefined;
}

test none_when_set {
    assert(maybe(true) == null);
}

test none_fails {
    assert(maybe(true) != null);
}
";
    assert_eq!(names_ok(&run(opt)), vec![("none_when_set", false, true), ("none_fails", false, false)]);
}

#[test]
fn ignored_value_only_where_zig_never_looks() {
    // A Rust-style tail expression (`fn f(v: u8) -> u32 { v }`) or a bare
    // comparison is emitted by t27c's Zig backend as `expr;`, with no
    // implicit return. Zig rejects that ("value of type 'bool' ignored") only
    // in a body it analyzes; nothing a test reaches names these fns, so the
    // reference compiles the file and runs both tests.
    let src = "module vi;

fn tail(v: u8) -> u32 {
    v
}

fn mixed(a: u32, b: u32) -> u32 {
    let c: u32 = a | b;
    c | 1
}

fn flag(x: u32) -> bool {
    if x > 3 {
        true
    } else {
        false
    }
}

fn inc(x: u32) u32 {
    return x + 1;
}

test inc_works {
    assert(inc(1) == 2);
}

test inc_fails {
    assert(inc(1) == 3);
}
";
    let r = run(src);
    assert_eq!(names_ok(&r), vec![("inc_works", false, true), ("inc_fails", false, false)]);
    // Where something analyzed reaches it -- a test, a fn a test calls --
    // the reference does not compile: refused under the expression's own
    // kind. A non-void fn's tail is returned instead and a brace invariant's
    // top-level predicate is asserted (#6315: tail.rs); what is left is a
    // value before the end of a body, a tail in a void fn or a test, and a
    // value in an `if` that is not the last statement. `t27c test-report`:
    // each source is BLOCKED.
    let reached = [
        ("fn f(v: u32) -> u32 { v; return v; }\ntest t { assert(f(1) == 1); }\n", "ExprIdentifier"),
        ("fn g(a: u32) { a + 1 }\nfn f(x: u32) -> u32 { g(x); return x; }\ntest t { assert(f(1) == 1); }\n", "ExprBinary"),
        ("fn f(v: u32) -> u32 { if v > 0 { v } return 0; }\ntest t { assert(f(0) == 0); }\n", "ExprIdentifier"),
        ("test t { 1; }\n", "ExprLiteral"),
    ];
    for (body, kind) in reached {
        let m = rejected(&format!("module vr;\n\n{}", body));
        assert!(
            m.starts_with(&format!("t27b: unsupported construct {}(value ignored) statement at line", kind)),
            "{}: {}",
            body,
            m
        );
    }
}

// ------------------------------------------------------ stack parameters

/// More than 8 parameters: the ninth and later of a class are passed on the
/// stack, one full word each (t27b's own convention, so such a fn is not
/// exported). Every position is weighted differently so a slot read from
/// the wrong place changes the result; narrow and negative values check
/// that the callee re-normalises what it loads; a struct result's hidden
/// pointer is itself the ninth word; a recursive fn passes its own
/// parameters on; and calls happen with temps live across them.
#[test]
fn stack_parameters_reach_the_callee_in_order() {
    let src = "module sp;

const Pair = struct { lo: i32, hi: i32 };

fn w9(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32, i: u32) u32 {
    return a + 2 * b + 3 * c + 4 * d + 5 * e + 6 * f + 7 * g + 8 * h + 100 * i;
}

fn mix(a: i8, b: u8, c: i16, d: u16, e: i32, f: u32, g: i64, h: u64, i: i8, j: u8, k: i16, l: bool, m: i64) i64 {
    var s: i64 = a;
    s = s * 3 + b;
    s = s * 3 + c;
    s = s * 3 + d;
    s = s * 3 + e;
    s = s * 3 + f;
    s = s * 3 + g;
    if (h > 5) {
        s = s + 1;
    }
    s = s * 3 + i;
    s = s * 3 + j;
    s = s * 3 + k;
    if (l) {
        s = s * 3 + 1;
    }
    return s * 3 + m;
}

fn pair8(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32) Pair {
    return Pair{ .lo = a + b + c + d, .hi = e + f + g + 10 * h };
}

fn down(n: u32, a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32, acc: u32) u32 {
    if (n == 0) {
        return acc + h;
    }
    return down(n - 1, b, c, d, e, f, g, h, a, acc + a * n);
}

fn outer(x: u32) u32 {
    return (x + 1) + w9(x, x + 1, x + 2, x + 3, x + 4, x + 5, x + 6, x + 7, x + 8) + (x + 2);
}

test ninth_word {
    assert(w9(1, 1, 1, 1, 1, 1, 1, 1, 1) == 136);
    assert(w9(0, 0, 0, 0, 0, 0, 0, 0, 7) == 700);
}

test narrow_and_negative_on_the_stack {
    assert(mix(-1, 255, -300, 65535, -70000, 4000000000, -5, 6, -128, 200, -32768, true, 9) == mix(-1, 255, -300, 65535, -70000, 4000000000, -5, 6, -128, 200, -32768, true, 9));
    assert(mix(0, 0, 0, 0, 0, 0, 0, 0, -128, 0, 0, false, 0) == -128 * 27);
    assert(mix(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, false, 0) == -3);
    assert(mix(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, true, 5) == 8);
}

test struct_result_pointer_on_the_stack {
    const p = pair8(1, 2, 3, 4, 5, 6, 7, 8);
    assert(p.lo == 10);
    assert(p.hi == 98);
}

test recursion_passes_its_stack_words_on {
    assert(down(0, 1, 2, 3, 4, 5, 6, 7, 8, 0) == 8);
    assert(down(3, 1, 2, 3, 4, 5, 6, 7, 8, 0) == 1 * 3 + 2 * 2 + 3 * 1 + 3);
}

test temps_live_across_the_call {
    assert(outer(10) == 11 + (10 + 22 + 36 + 52 + 70 + 90 + 112 + 136 + 1800) + 12);
}

test wrong_on_purpose {
    assert(w9(1, 2, 3, 4, 5, 6, 7, 8, 9) == 0);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("ninth_word", false, true),
            ("narrow_and_negative_on_the_stack", false, true),
            ("struct_result_pointer_on_the_stack", false, true),
            ("recursion_passes_its_stack_words_on", false, true),
            ("temps_live_across_the_call", false, true),
            ("wrong_on_purpose", false, false),
        ]
    );
    let prog = lower_src(src).unwrap();
    let internal: Vec<&str> = prog.internal_abi.iter().map(|&i| prog.funcs[i as usize].name.as_str()).collect();
    assert_eq!(internal, vec!["pair8", "w9", "mix", "down"]);
}

#[test]
fn fn_declaration_rejections_are_precise() {
    let head = "module fd;\n\n";
    let many: Vec<String> = (0..65).map(|i| format!("p{}: u8", i)).collect();
    let cases: Vec<(String, &str, &str)> = vec![
        ("fn f(x: struct {}) u32 { return 1; }\ntest t { assert(true); }".into(), "FnDecl(untyped param)", "parameter `x` of `f`"),
        ("fn f(x: u32) u32 { return x; }\nfn f(x: u32) u32 { return x; }\ntest t { assert(f(1) == 1); }".into(), "FnDecl(duplicate)", "duplicate function `f`"),
        (format!("fn f({}) u8 {{ return p0; }}\ntest t {{ assert(true); }}", many.join(", ")), "FnDecl(too many params)", "`f` has 65 parameters, at most 64"),
    ];
    for (body, construct, detail) in &cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

/// f64 parameters past d7 and integer ones past x7 in one signature: each
/// class fills its own registers, and the overflow of both shares one run of
/// stack words in parameter order. A leaf callee and one that calls.
#[test]
fn f64_and_integer_stack_parameters_interleave() {
    let src = "module spf;

fn wf(a: f64, b: u8, c: f64, d: f64, e: i16, f: f64, g: f64, h: f64, i: f64, j: f64, k: i32, l: f64, m: u8, n: f64, o: u8, p: u8, q: u8, r: u8, s: i8) f64 {
    var t: f64 = a;
    t = t * 2.0 + c;
    t = t * 2.0 + d;
    t = t * 2.0 + f;
    t = t * 2.0 + g;
    t = t * 2.0 + h;
    t = t * 2.0 + i;
    t = t * 2.0 + j;
    t = t * 2.0 + l;
    t = t * 2.0 + n;
    var u: i32 = b;
    u = u * 3 + e;
    u = u * 3 + k;
    u = u * 3 + m;
    u = u * 3 + o;
    u = u * 3 + p;
    u = u * 3 + q;
    u = u * 3 + r;
    u = u * 3 + s;
    return t * 100000.0 + @as(f64, @floatFromInt(u));
}

fn half(x: f64) f64 {
    return x / 2.0;
}

fn wg(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, i: f64, j: f64) f64 {
    return half(a) + b + c + d + e + f + g + h + 3.0 * i + 5.0 * j;
}

test leaf_callee {
    assert(wf(1.0, 0, 0.0, 0.0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 0, 0.0, 0, 0, 0, 0, 0) == 51200000.0);
    assert(wf(0.0, 0, 0.0, 0.0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 0, 1.0, 0, 0, 0, 0, 0) == 100000.0);
    assert(wf(0.0, 0, 0.0, 0.0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0.5, 0, 0.0, 0, 0, 0, 0, -1) == 99999.0);
    assert(wf(0.0, 1, 0.0, 0.0, -1, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 0, 0.0, 0, 0, 0, 0, 0) == 4374.0);
    assert(wf(0.0, 0, 0.0, 0.0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 0, 0.0, 0, 0, 0, 255, 0) == 765.0);
}

test calling_callee {
    assert(wg(2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0) == 1.0);
    assert(wg(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0) == 3.0);
    assert(wg(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0) == -5.0);
}
";
    let r = run(src);
    assert_eq!(names_ok(&r), vec![("leaf_callee", false, true), ("calling_callee", false, true)]);
}

// ------------------------------------------------------ if as a value

/// `if (c) a else b` used as a value (t27c emits it as written): the arms
/// take the result type (a return, a typed binding, a parameter) or, with
/// none, their peer type; only the taken arm runs, so a trap in the other
/// never fires; strings and structs select the arm's address; a condition
/// known at compile time picks its arm and the other is not lowered; and an
/// arm is evaluated with temps live around it.
#[test]
fn if_expression_runs_only_the_taken_arm() {
    let src = "module ie;

const PHI: f64 = 1.618;

struct P {
    x: u32,
    y: u32,
}

fn relu(x: i32) i32 {
    return if (x > 0) x else 0;
}

fn safe_div(n: u32, d: u32) u32 {
    return if (d != 0) n / d else 0;
}

fn bump(x: u8) u8 {
    return if (x < 100) x else x + 200;
}

fn sign(x: i32) i32 {
    return if (x > 0) 1 else if (x < 0) -1 else 0;
}

fn absd(x: f64) f64 {
    let d = if (x < PHI) PHI - x else x - PHI;
    return d;
}

fn name(code: []const u8) []const u8 {
    const n = if (code == \"A\") {
        \"alfa\"
    } else if (code == \"B\") {
        \"bravo\"
    } else {
        \"unknown\"
    };
    return n;
}

fn pick(c: bool, a: P, b: P) u32 {
    const p = if (c) a else b;
    return p.x * 10 + p.y;
}

fn wide(a: u8, b: u32, c: bool) u32 {
    return b + 2 * (if (c) a else b);
}

fn deep(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) u32 {
    return a + (b + (c + (d + (e + (f + (g + (if (h > 4) h * 3 else h + 1)))))));
}

fn fixed() u32 {
    const k = if (true) 7 else 9;
    return k;
}

test relu_works {
    assert(relu(5) == 5);
    assert(relu(-3) == 0);
}

test untaken_arm_does_not_trap {
    assert(safe_div(9, 0) == 0);
    assert(safe_div(9, 3) == 3);
    assert(bump(7) == 7);
}

test taken_arm_traps {
    assert(bump(150) == 94);
}

test chain_and_float {
    assert(sign(-4) == -1);
    assert(sign(4) == 1);
    assert(sign(0) == 0);
    assert(absd(1.0) > 0.617);
    assert(absd(1.0) < 0.619);
    assert(absd(2.0) > 0.381);
}

test strings_and_structs {
    assert(name(\"B\") == \"bravo\");
    assert(name(\"Z\") == \"unknown\");
    assert(name(\"A\").len == 4);
    assert(pick(true, P { x: 1, y: 2 }, P { x: 3, y: 4 }) == 12);
    assert(pick(false, P { x: 1, y: 2 }, P { x: 3, y: 4 }) == 34);
}

test peers_and_temps {
    assert(wide(5, 100, true) == 110);
    assert(wide(5, 100, false) == 300);
    assert(deep(1, 1, 1, 1, 1, 1, 1, 5) == 22);
    assert(deep(1, 1, 1, 1, 1, 1, 1, 2) == 10);
    assert(fixed() == 7);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("relu_works", false, true),
            ("untaken_arm_does_not_trap", false, true),
            ("taken_arm_traps", false, false),
            ("chain_and_float", false, true),
            ("strings_and_structs", false, true),
            ("peers_and_temps", false, true),
        ]
    );
    let t = r.iter().find(|x| x.0 == "taken_arm_traps").unwrap();
    assert_eq!(t.2, Err((TrapKind::Overflow, 19)));
    // Refused under names of their own: Zig cannot type the first two, and
    // the reference misprints the third.
    let cases = [
        ("fn f(c: bool, a: u32, b: u32) u32 { const k = if (c) 1 else 2; return k; }\n", "ExprIf(comptime arms)"),
        ("fn f(c: bool, a: u32, b: u32) u32 { return 1 + if (c) 1 else 2; }\n", "ExprIf(comptime arms)"),
        // Printed `if (c) a else b + 1`: the `+ 1` lands in the else arm.
        ("fn f(c: bool, a: u32, b: u32) u32 { return (if (c) a else b) + 1; }\n", "ExprIf(left operand)"),
    ];
    for (body, construct) in cases {
        let m = rejected(&format!("module ir;\n\n{}test t {{ assert(f(true, 1, 2) == 1); }}\n", body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
    }
}

#[test]
fn switch_runs_only_the_taken_prong() {
    // Verdicts match `t27c test-report` on the same source: five pass and
    // `taken_prong_traps` fails on the `x - 10` overflow.
    let src = "module sw;

pub const NEG: i32 = -3;

pub enum Trit {
    neg,
    zero,
    pos,
}

fn negate(a: Trit) Trit {
    return switch (a) {
        .neg => .pos,
        .zero => .zero,
        .pos => .neg,
    };
}

fn add(a: Trit, b: Trit) Trit {
    return switch (a) {
        .neg => switch (b) {
            .neg => .neg,
            .zero => .neg,
            .pos => .zero,
        },
        .zero => b,
        .pos => switch (b) {
            .neg => .zero,
            .zero => .pos,
            .pos => .pos,
        },
    };
}

fn name(a: Trit) []const u8 {
    return switch (a) {
        .neg => \"neg\",
        .zero => \"zero\",
        .pos => \"pos\",
    };
}

fn code(x: i32) u32 {
    return switch (x) {
        -1 => 10,
        0 => 20,
        7 => 30,
        else => 40,
    };
}

var calls: u32 = 0;

fn bump(x: u8) u8 {
    calls += 1;
    return x;
}

fn once(x: u8) u32 {
    return switch (bump(x)) {
        1 => 100,
        2 => 200,
        else => 300,
    };
}

fn sub(x: u32) u32 {
    return switch (x) {
        0 => 0,
        else => x - 10,
    };
}

fn peer(a: Trit, x: u8, w: u16) u16 {
    const v = switch (a) {
        .neg => x,
        .zero => w,
        else => 5,
    };
    return v;
}

test enum_switch {
    assert(negate(.neg) == .pos);
    assert(negate(.zero) == .zero);
    assert(add(.pos, .pos) == .pos);
    assert(add(.neg, .pos) == .zero);
    assert(add(.zero, .neg) == .neg);
    assert(name(.zero).len == 4);
}

test int_switch {
    assert(code(-1) == 10);
    assert(code(0) == 20);
    assert(code(7) == 30);
    assert(code(NEG) == 40);
    assert(NEG == -3);
}

test operand_runs_once {
    assert(once(2) == 200);
    assert(once(9) == 300);
    assert(calls == 2);
}

test peer_type {
    assert(peer(.neg, 7, 1000) == 7);
    assert(peer(.zero, 7, 1000) == 1000);
    assert(peer(.pos, 7, 1000) == 5);
}

test untaken_prong_does_not_trap {
    assert(sub(0) == 0);
    assert(sub(12) == 2);
}

test taken_prong_traps {
    assert(sub(3) == 0);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("enum_switch", false, true),
            ("int_switch", false, true),
            ("operand_runs_once", false, true),
            ("peer_type", false, true),
            ("untaken_prong_does_not_trap", false, true),
            ("taken_prong_traps", false, false),
        ]
    );
    // t27c gives expression nodes no line, so the trap lands on the
    // statement that holds the switch, as it does for `if`.
    assert_eq!(r[5].2, Err((TrapKind::Overflow, line_of(src, "fn sub") + 1)));
    // Every shape below is BLOCKED in the reference too (Zig refuses it, or
    // t27c misprints the prong), so each is refused under its own name.
    let cases = [
        ("a: Trit", "switch (a) { .neg => 1, .zero => 2, }", "ExprSwitch(not exhaustive)"),
        ("x: u8", "switch (x) { 0 => 1, 1 => 2, }", "ExprSwitch(not exhaustive)"),
        ("a: Trit", "switch (a) { .neg => 1, .zero => 2, .pos => 3, else => 4, }", "ExprSwitch(unreachable else)"),
        ("x: u8", "switch (x) { 'a' => 1, else => 2, }", "ExprSwitch(char prong)"),
        ("x: u8", "switch (x) { 1 => 1, 1 => 2, else => 3, }", "ExprSwitch(duplicate prong)"),
        ("a: Trit", "switch (a) { .neg => 1, .zero => 2, .up => 3, }", "ExprSwitch(prong)"),
        ("x: u8", "switch (x) { 300 => 1, else => 2, }", "ExprSwitch(prong)"),
        ("a: Trit", "switch (a) { 0 => 1, else => 2, }", "ExprSwitch(prong)"),
        ("x: u8", "switch (x) { zero => 1, else => 2, }", "ExprSwitch(prong)"),
    ];
    for (param, body, construct) in cases {
        let m = rejected(&format!(
            "module rj;\n\npub enum Trit {{\n    neg,\n    zero,\n    pos,\n}}\n\nfn f({}) u32 {{\n    return {};\n}}\n\ntest t {{ assert(f(0) == 1); }}\n",
            param, body
        ));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
    }
}

#[test]
fn discarded_call_result_is_dropped_by_name() {
    // A bare call to a module fn that returns a value: t27c prints
    // `_ = g();` (#6315), so the call runs and its value is dropped, in a
    // body Zig analyzes and in one nothing reaches (`never`, a bench, a fn
    // only the bench names). Verdicts match `t27c test-report`: the first
    // source passes; in the second, `counts` passes and `miscounts` fails.
    let src = "module vi;\n\nvar n: u32 = 0;\n\nfn g() u32 {\n    n += 1;\n    return n;\n}\n\nfn h() void {\n    n += 1;\n}\n\nfn never() void {\n    g();\n}\n\nfn bench_only() void {\n    g();\n}\n\ntest void_call {\n    h();\n    assert(n == 1);\n}\n\nbench b {\n    g();\n    bench_only();\n}\n";
    assert_eq!(names_ok(&run(src)), vec![("void_call", false, true)]);
    let src = "module vj;\n\nvar n: u32 = 0;\n\nfn g() u32 {\n    n += 1;\n    return n;\n}\n\ntest counts {\n    n = 0;\n    g();\n    g();\n    assert(n == 2);\n}\n\ntest miscounts {\n    n = 0;\n    g();\n    assert(n == 2);\n}\n";
    assert_eq!(names_ok(&run(src)), vec![("counts", false, true), ("miscounts", false, false)]);
}

#[test]
fn char_literals_are_comptime_ints() {
    let src = "module cl;

fn is_digit(c: u8) bool {
    return c >= '0' and c <= '9';
}

fn upper(c: u8) u8 {
    return c - 'a' + 'A';
}

fn quoted(c: u8) bool {
    return c == '\\'' or c == '\\\\' or c == '\"';
}

fn digit(c: u8) i32 {
    const d: i32 = c - '0';
    return d * 2;
}

fn bump(c: u8) u8 {
    return c + 'z';
}

test digits_and_case {
    assert(is_digit('5'));
    assert(!is_digit('x'));
    assert(upper('b') == 'B');
    assert(digit('7') == 14);
}

test escapes {
    assert('\\n' == 10);
    assert('\\r' == 13);
    assert('\\t' == 9);
    assert(quoted('\\''));
    assert(quoted('\\\\'));
    assert(quoted('\"'));
    assert(!quoted('a'));
    const k = 'z';
    assert(k == 122);
    assert('a' << 2 == 388);
    assert(-'a' == -97);
}

test overflow_traps {
    assert(bump(200) == 66);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![("digits_and_case", false, true), ("escapes", false, true), ("overflow_traps", false, false)]
    );
    let t = r.iter().find(|x| x.0 == "overflow_traps").unwrap();
    assert_eq!(t.2, Err((TrapKind::Overflow, 21)));
    // Zig has no `\\0` escape and no raw control byte in a char literal, and
    // t27c prints `'a' << @intCast(k)`, a comptime_int shifted at run time.
    let cases = [
        ("fn f(k: u32) u32 { return '\\0' + k; }\n", "ExprLiteral(char escape)"),
        ("fn f(k: u32) u32 { return '\t' + k; }\n", "ExprLiteral(char byte)"),
        ("fn f(k: u32) u32 { return 'a' << k; }\n", "ExprBinary(char literal << >>)"),
    ];
    for (body, construct) in cases {
        let m = rejected(&format!("module cr;\n\n{}test t {{ assert(f(1) == 1); }}\n", body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
    }
}

// ------------------------------------------------------------ for over a range

#[test]
fn for_over_a_range() {
    // `for (a..b) |i|`, `for i in a..b`, `for (i in a..=b)`, `for _ in`, and
    // `for x in xs` (an array, the loop variable copied into the node's
    // name). Verdicts as `t27c test-report` gives them: Zig computes the
    // length `b - a` before the first iteration and panics on overflow.
    let src = "module fr;

fn sum_paren(n: u32) -> usize {
    var s: usize = 0;
    for (0..n) |i| {
        s += i;
    }
    return s;
}

fn sum_in(lo: usize, hi: usize) -> usize {
    var s: usize = 0;
    for i in lo..hi {
        if (i == 5) {
            continue;
        }
        if (i == 8) {
            break;
        }
        s += i;
    }
    return s;
}

fn sum_inclusive(n: u8) -> usize {
    var s: usize = 0;
    for (i in 1..=n) {
        s += i;
    }
    return s;
}

fn count(n: usize) -> u32 {
    var c: u32 = 0;
    for _ in 0..n {
        c += 1;
    }
    return c;
}

fn total(xs: [4]u32) -> u32 {
    var s: u32 = 0;
    for x in xs {
        s += x;
    }
    return s;
}

test ranges {
    assert(sum_paren(4) == 6);
    assert(sum_paren(0) == 0);
    assert(sum_in(2, 10) == 22);
    assert(sum_in(3, 3) == 0);
    assert(sum_inclusive(4) == 10);
    assert(count(7) == 7);
    assert(total([1, 2, 3, 4]) == 10);
}

test nested_ranges {
    var n: usize = 0;
    for (0..3) |i| {
        for (i..3) |j| {
            n += j;
        }
    }
    assert(n == 8);
}

test reversed_range_traps {
    assert(sum_in(5, 3) == 0);
}
";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![("ranges", false, true), ("nested_ranges", false, true), ("reversed_range_traps", false, false)]
    );
    assert_eq!(r[2].2, Err((TrapKind::Overflow, 12)));
    // Each a compile error in the reference (`zig test` on t27c's output).
    // The multi-object loop is lowered now (tests/formulti.rs).
    let cases = [
        ("for (0..n) |i| { s += i; }", "n: i32", "type mismatch", "expected u64, found i32"),
        ("for (3..1) |i| { s += i; }", "n: usize", "StmtFor(range)", "range 3..1 runs backwards"),
        ("for i in -1..n { s += i; }", "n: usize", "literal out of range", "-1 does not fit in u64"),
    ];
    for (body, param, construct, detail) in cases {
        let m = rejected(&format!(
            "module rj;\n\nfn f({}) usize {{ var s: usize = 0; {} return s; }}\n\ntest t {{ assert(f(1) == 1); }}\n",
            param, body
        ));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

#[test]
fn try_statements_in_tests_match_the_reference() {
    // `try std.testing.<check>(...);` in a test: a failed check fails that
    // test alone and the next one still runs. `try` in a bench and in a fn
    // nothing reaches is never analyzed by the reference's Zig. Verdicts
    // match `t27c test-report` on both sources.
    let src = r#"module trystmt;

const PHI: f64 = 1.618033988749895;

const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

var calls: u32 = 0;

fn bump() u32 {
    calls += 1;
    return calls;
}

fn neg(t: Trit) Trit {
    return switch (t) {
        .neg => .pos,
        .zero => .zero,
        .pos => .neg,
    };
}

fn never() void {
    try std.testing.expect(false);
}

test expect_holds {
    try std.testing.expect(1 + 1 == 2);
    try std.testing.expectEqual(@as(u32, 1), bump());
    try std.testing.expectEqual(@as(Trit, .pos), neg(.neg));
    try std.testing.expectApproxEqAbs(PHI * PHI, PHI + 1.0, 0.000001);
    try std.testing.expectApproxEqAbs(2.0, PHI, 1);
}

test expect_fails {
    try std.testing.expect(bump() == 99);
    try std.testing.expect(false);
}

test expect_equal_fails {
    try std.testing.expectEqual(@as(u8, 3), @as(u8, 4));
}

test approx_fails {
    try std.testing.expectApproxEqAbs(PHI, 1.0, 0.5);
}

test after_failures_still_run {
    var i: u32 = 0;
    while (i < 3) {
        try std.testing.expect(i < 3);
        i += 1;
    }
    try std.testing.expectEqual(@as(u32, 3), i);
}

bench b {
    try std.testing.expect(false);
}
"#;
    assert_eq!(
        names_ok(&run(src)),
        vec![
            ("expect_holds", false, true),
            ("expect_fails", false, false),
            ("expect_equal_fails", false, false),
            ("approx_fails", false, false),
            ("after_failures_still_run", false, true),
        ]
    );
    // approxEqAbs: equal infinities pass, a NaN fails even with a huge
    // tolerance, each operand runs once, a negative tolerance panics.
    let src = r#"module edge;

var n: u32 = 0;

fn big() f64 {
    n += 1;
    const a: f64 = 1.0e308;
    return a * 10.0;
}

fn nan() f64 {
    const z: f64 = 0.0;
    return z / z;
}

fn tick() f64 {
    n += 1;
    return 1.0;
}

test inf_equal_passes {
    try std.testing.expectApproxEqAbs(big(), big(), 0.5);
}

test nan_fails {
    try std.testing.expectApproxEqAbs(nan(), nan(), 1.0e300);
}

test nan_equal_fails {
    try std.testing.expectEqual(nan(), nan());
}

test operands_once {
    const before: u32 = n;
    try std.testing.expectApproxEqAbs(tick(), tick(), tick());
    try std.testing.expectEqual(before + 3, n);
}

test zero_tol {
    try std.testing.expectApproxEqAbs(@as(f64, 1.0), 1.0, 0.0);
}

test near_miss {
    try std.testing.expectApproxEqAbs(@as(f64, 1.0), 1.5, 0.4);
}
"#;
    assert_eq!(
        names_ok(&run(src)),
        vec![
            ("inf_equal_passes", false, true),
            ("nan_fails", false, false),
            ("nan_equal_fails", false, false),
            ("operands_once", false, true),
            ("zero_tol", false, true),
            ("near_miss", false, false),
        ]
    );
    let neg = "module nt;\n\ntest t {\n    const tol: f64 = -1.0;\n    try std.testing.expectApproxEqAbs(tol, tol, tol);\n}\n\ntest u {\n    try std.testing.expect(true);\n}\n";
    assert_eq!(names_ok(&run(neg)), vec![("t", false, false), ("u", false, true)]);
}

#[test]
fn try_statements_the_reference_does_not_compile_are_refused() {
    // Each of these is BLOCKED under `t27c test-report`.
    let cases: [(&str, &str); 6] = [
        (
            "module a;\n\nfn reached() void {\n    try std.testing.expect(true);\n}\n\ntest t {\n    reached();\n}\n",
            "ExprUnary(try) in a fn at line 4",
        ),
        (
            "module b;\n\nconst PHI: f64 = 1.618033988749895;\n\ninvariant i {\n    try std.testing.expect(PHI > 1.0);\n}\n",
            "ExprUnary(try) in an invariant at line 6",
        ),
        (
            "module c;\n\nfn g() u32 {\n    return 1;\n}\n\ntest t {\n    try g();\n}\n",
            "ExprUnary(try) on a non-error call at line 8",
        ),
        (
            "module d;\n\ntest t {\n    try std.testing.expectApproxEqAbs(1.0, 1.5, 1.0);\n}\n",
            "ExprCall(std.testing.expectApproxEqAbs) on comptime floats at line 4",
        ),
        (
            "module e;\n\nfn g() u32 {\n    return 1;\n}\n\ntest t {\n    try std.testing.expectApproxEqAbs(g(), 1, 1);\n}\n",
            "ExprCall(std.testing.expectApproxEqAbs) on a non-float at line 8",
        ),
        (
            "module f;\n\ntest t {\n    try std.testing.expectError(1, 1);\n}\n",
            "ExprUnary(try) std.testing.expectError at line 4",
        ),
    ];
    for (src, want) in cases {
        let m = rejected(src);
        assert!(m.starts_with(&format!("t27b: unsupported construct {}", want)), "{}", m);
    }
}

// ------------------------------------------------------------ optionals

/// Verdicts confirmed under `t27c test-report`: every test passes but the
/// two that unwrap `null` (Zig: "attempt to use null value") and the one
/// that compares a `null` with a value.
#[test]
fn optionals_match_the_reference() {
    let src = r#"module opt2;

pub enum Color {
    red,
    green,
}

pub struct V {
    x: i32,
    y: i32,
}

pub struct Box2 {
    name: ?[]const u8,
    v: ?V,
    c: ?Color,
    ok: ?bool,
    f: ?f64,
}

fn pick(n: u32) -> ?V {
    if (n == 0) {
        return null;
    }
    return V{ .x = 1, .y = 2 };
}

fn name_of(n: u32) -> ?[]const u8 {
    if (n == 0) {
        return null;
    }
    return "abc";
}

fn count(x: ?i32) -> i32 {
    if (x == null) {
        return -1;
    }
    return x.?;
}

pub struct Ctr {
    n: i32,
}

fn next(c: *Ctr) ?i32 {
    c.n = c.n + 1;
    if (c.n > 2) {
        return null;
    }
    return c.n;
}

test "struct opt" {
    const a = pick(1);
    assert(a != null);
    assert(a.?.x == 1);
    assert(a.?.y == 2);
    const b = pick(0);
    assert(b == null);
}

test "str opt" {
    const s = name_of(1);
    assert(s.?.len == 3);
    assert(name_of(0) == null);
}

fn reassign() -> i32 {
    var w: ?i32 = null;
    if (count(w) != -1) {
        return 1;
    }
    w = 9;
    if (count(w) != 9) {
        return 2;
    }
    if (w != 9) {
        return 3;
    }
    if (w == 8) {
        return 4;
    }
    w.? = 4;
    if (w.? != 4) {
        return 5;
    }
    w = null;
    if (w != null) {
        return 6;
    }
    return 0;
}

test "param opt" {
    assert(count(null) == -1);
    assert(count(7) == 7);
    assert(reassign() == 0);
}

test "param opt old" {
    var w: ?i32 = null;
    assert(count(w) == -1);
}

test "compare call" {
    var i = Ctr{ .n = 0 };
    assert(next(&i) == 1);
    assert(next(&i) == 2);
    assert(next(&i) == null);
    assert(next(&i) != 5);
}

fn boxes() -> i32 {
    var bx = Box2{ .name = null, .v = null, .c = Color.green, .ok = true, .f = 2.5 };
    if (bx.name != null) {
        return 1;
    }
    if (bx.v != null) {
        return 2;
    }
    if (bx.c.? != Color.green) {
        return 3;
    }
    if (!bx.ok.?) {
        return 4;
    }
    if (bx.f.? != 2.5) {
        return 5;
    }
    bx.v = V{ .x = 3, .y = 4 };
    if (bx.v.?.y != 4) {
        return 6;
    }
    bx.name = "hi";
    if (bx.name.?.len != 2) {
        return 7;
    }
    const pv = &bx.v.?;
    pv.x = 10;
    if (bx.v.?.x != 10) {
        return 8;
    }
    return 0;
}

test "box fields" {
    assert(boxes() == 0);
}

test "unwrap null struct" {
    const b = pick(0);
    assert(b.?.x == 1);
}

test "null eq value" {
    const w: ?i32 = null;
    assert(w == 0);
}
"#;
    let r = run(src);
    let line_of = |needle: &str| src.lines().position(|l| l.contains(needle)).unwrap() as u32 + 1;
    let got: Vec<(&str, Outcome)> = r.iter().map(|(n, _, o)| (n.as_str(), *o)).collect();
    assert_eq!(
        got,
        vec![
            ("struct opt", Ok(())),
            ("str opt", Ok(())),
            ("param opt", Ok(())),
            ("param opt old", Ok(())),
            ("compare call", Ok(())),
            ("box fields", Ok(())),
            ("unwrap null struct", Err((TrapKind::Null, line_of("assert(b.?.x == 1);")))),
            ("null eq value", Err((TrapKind::Assert, line_of("assert(w == 0);")))),
        ]
    );
    let src = r#"module opt1;

pub struct P {
    a: u32,
    b: ?u32,
}

fn find(x: u32) -> ?u32 {
    if (x > 3) {
        return x * 2;
    }
    return null;
}

fn get(p: P) -> u32 {
    if (p.b != null) {
        return p.b.?;
    }
    return 0;
}

test "opt basic" {
    const a = find(5);
    const b = find(1);
    assert(a != null);
    assert(b == null);
    assert(a.? == 10);
    var p = P{ .a = 1, .b = null };
    assert(get(p) == 0);
    p.b = 4;
    assert(get(p) == 4);
}

test "unwrap null" {
    const b = find(1);
    assert(b.? == 1);
}
"#;
    let r = run(src);
    let got: Vec<(&str, bool)> = r.iter().map(|(n, _, o)| (n.as_str(), o.is_ok())).collect();
    assert_eq!(got, vec![("opt basic", true), ("unwrap null", false)]);
    assert!(matches!(r[1].2, Err((TrapKind::Null, _))));
}

/// Module-level constants holding an optional (#7116): alone, copied from
/// another, as struct fields (with a default, and beside a `str`), and a
/// present zero, which is not `null`.
#[test]
fn module_level_optionals() {
    let src = "module a;

const K: ?u32 = 7;
const N: ?u32 = null;
const C: ?u32 = K;

const P = struct {
    lo: ?i32,
    hi: ?i32 = null,
};

const Q: P = P{ .lo = -2 };

const R = struct {
    name: str,
    n: ?u8,
};

const RS: [2]R = [R{ .name = \"a\", .n = 0 }, R{ .name = \"b\", .n = null }];

fn or_zero(x: ?u32) -> u32 {
    if (x != null) {
        return x.?;
    }
    return 0;
}

test present {
    assert(K.? == 7 and C.? == 7 and N == null);
    assert(or_zero(C) + or_zero(N) == 7);
    assert(Q.lo.? == -2 and Q.hi == null);
    assert(RS[0].n.? == 0 and RS[1].n == null);
}

test null_is_not_present {
    assert(RS[1].n != null);
}
";
    let r = run(src);
    assert_eq!(names_ok(&r), vec![("present", false, true), ("null_is_not_present", false, false)]);
    assert_eq!(r[1].2, Err((TrapKind::Assert, line_of(src, "RS[1].n != null"))));
}

#[test]
fn optionals_the_reference_does_not_match_are_refused() {
    let cases: [(&str, &str); 3] = [
        // BLOCKED under `t27c test-report`: the Zig backend drops the capture.
        (
            "module a;\n\nfn f(x: ?u32) -> u32 {\n    if (x) |v| {\n        return v;\n    }\n    return 0;\n}\n\ntest t {\n    assert(f(3) == 3);\n}\n",
            "StmtIf(capture)",
        ),
        // These two pass under the reference; not lowered: two optionals
        // compared, and a module-level optional str.
        (
            "module b;\n\nfn f(x: ?u32) -> ?u32 {\n    return x;\n}\n\ntest t {\n    const a: ?u32 = 3;\n    assert(f(a) == f(a));\n}\n",
            "ExprBinary(?T)",
        ),
        (
            "module c;\n\nconst S: ?str = \"x\";\n\ntest t {\n    assert(S != null);\n}\n",
            "ConstDecl(?T)",
        ),
    ];
    for (src, want) in cases {
        let m = rejected(src);
        assert!(m.starts_with(&format!("t27b: unsupported construct {}", want)), "{}", m);
    }
}

/// Shapes t27c's Zig backend cannot compile (#6358): each source below is
/// BLOCKED under `t27c test-report`, so t27b must not pass it either.
#[test]
fn shapes_the_reference_cannot_compile_are_refused() {
    let cases: [(&str, &str); 6] = [
        // `var w = 1; w = 9;` at the top of a test: the reference emits the
        // assignment as `const w = 9;`, a redeclaration.
        (
            "module a;\n\ntest t {\n    var w: u32 = 1;\n    w = 9;\n    assert(w == 9);\n}\n",
            "StmtAssign(reference redeclares)",
        ),
        // A pointer param written only through `p.*`: the reference rebinds
        // it `var p = p_arg;`, which Zig rejects as never mutated.
        (
            "module b;\n\nfn set(p: *u32) {\n    p.* = 5;\n}\n\ntest t {\n    var x: u32 = 1;\n    set(&x);\n    assert(x == 5);\n}\n",
            "FnDecl(reference var param)",
        ),
        // `count + 1` twice: CSE hoists `_cse0 = count + 1` above `count`.
        (
            "module c;\n\nfn cnt(n: u32) -> u32 {\n    var count: u32 = n;\n    var x: u32 = count + 1;\n    var y: u32 = count + 1;\n    return x + y;\n}\n\ntest t {\n    assert(cnt(1) == 4);\n}\n",
            "FnDecl(reference CSE hoist)",
        ),
        // A Zig keyword as a field name, unescaped in the literal.
        (
            "module d;\n\nconst S = struct { align: u32, n: u32 };\n\nfn f() -> u32 {\n    const s: S = S{ .align = 4, .n = 1 };\n    return s.n;\n}\n\ntest t {\n    assert(f() == 1);\n}\n",
            "ExprStructLit(zig keyword field)",
        ),
        // `[_]u8{}`: the reference prints `.{ _ }`.
        (
            "module e;\n\ntest t {\n    var c: [0]u8 = [_]u8{};\n    assert(1 == 1);\n}\n",
            "ExprArrayLiteral(reference empty typed)",
        ),
        // A field of a type nothing declares, in a struct nothing uses.
        (
            "module g;\n\nconst S = struct { name: String, n: u32 };\n\ntest t {\n    assert(1 == 1);\n}\n",
            "StructDecl(reference undeclared field type)",
        ),
    ];
    for (src, want) in cases {
        let m = rejected(src);
        assert!(m.starts_with(&format!("t27b: unsupported construct {}", want)), "{}", m);
    }
    // The near misses still run: a param the body assigns directly (the
    // reference's `var n = n_arg;` is then mutated) and a mapped field type.
    let r = run("module h;\n\nconst S = struct { name: str, xs: [u32; 2] };\n\nfn inc(n: u32) -> u32 {\n    n = n + 1;\n    return n;\n}\n\ntest t {\n    assert(inc(1) == 2);\n}\n");
    assert_eq!(names_ok(&r), vec![("t", false, true)]);
}

/// A statement at top level is dropped, as t27c's Zig backend drops it
/// (`gen_decl` emits nothing for it): a dotted `module a.b;` / `use a.b;`
/// (parsed as `a` and a stray `.b`, #6102), a failing `assert` and a call all
/// leave the file compiling and its tests passing under `t27c test-report`.
/// A fn named only by such a statement is not analyzed by the reference
/// either, so a `try` in it is not refused.
#[test]
fn top_level_statements_are_dropped_like_the_reference() {
    let r = run("module sandbox.health;\n\nuse sandbox.session;\n\nconst N: u32 = 3;\n\nfn twice(x: u32) -> u32 {\n    return x * 2;\n}\n\ntest twice_works {\n    assert(twice(N) == 6);\n}\n");
    assert_eq!(names_ok(&r), vec![("twice_works", false, true)]);
    let r = run("module Specs.Lsp.Language;\n\ntest t {\n    assert(1 == 1);\n}\n");
    assert_eq!(names_ok(&r), vec![("t", false, true)]);
    let r = run("module top;\n\nassert(1 == 2);\n\nfn one() -> u32 {\n    return 1;\n}\n\ntest t {\n    assert(one() == 1);\n}\n");
    assert_eq!(names_ok(&r), vec![("t", false, true)]);
    let r = run("module top;\n\nfn sum(a: u32, b: u32) -> u32 {\n    return a + b;\n}\n\nfn only_from_top() -> u32 {\n    try std.testing.expect(true);\n    return 1;\n}\n\nonly_from_top();\n\ntest sum_works {\n    assert(sum(2, 3) == 5);\n}\n");
    assert_eq!(names_ok(&r), vec![("sum_works", false, true)]);
    // The same `try` in a fn a test reaches is still refused.
    let m = rejected("module top;\n\nfn reached() -> u32 {\n    try std.testing.expect(true);\n    return 1;\n}\n\ntest t {\n    assert(reached() == 1);\n}\n");
    assert!(m.contains("`try` in a fn a test, invariant or bench reaches"), "{}", m);
}

/// Array literals whose type comes from where they are used, as t27c's Zig
/// backend writes them: an argument where `[]const u8` is declared
/// (`@constCast(&[_]u8{ ... })`), an untyped local passed only where one
/// array type is declared (`.{ ... }`, which coerces at each call), an empty
/// literal for a slice field of a named struct literal, and an untyped local
/// of plain literals nothing reads (`_ = x; // dead after const-inlining`).
#[test]
fn array_literals_typed_by_their_use() {
    let src = "module a;\n\npub struct Cur {\n    pos: usize,\n    data: []i32,\n    name: []const u8,\n}\n\nfn mk(p: usize) -> Cur {\n    return Cur{ .pos = p, .data = [], .name = [] };\n}\n\nfn bsum(p: []const u8) -> u32 {\n    var s: u32 = 0;\n    for (p) |b| {\n        s += b;\n    }\n    return s;\n}\n\nfn tot(p: [2]i64) -> i64 {\n    return p[0] - p[1];\n}\n\nfn nine() -> i64 {\n    return 9;\n}\n\ntest str_arguments {\n    assert(bsum([1, 2, 250]) == 253);\n    const xs = [4, 5];\n    assert(bsum(xs) == 9);\n}\n\ntest tuple_locals {\n    const p = [2]i64{ 7, 2 };\n    assert(tot(p) == 5);\n    const a: i32 = -4;\n    const q = [a, nine()];\n    assert(tot(q) == -13 and tot(q) == -13);\n}\n\ntest empty_slice_fields {\n    const c = mk(3);\n    assert(c.pos == 3 and c.data.len == 0 and c.name.len == 0);\n}\n\ntest dead_literal_local {\n    const unused = [_]f32{ 1.0, -2.0 };\n    assert(bsum([]) == 0);\n}\n\ntest tuple_local_fails {\n    const r = [1, 2];\n    assert(tot(r) == 1);\n}\n";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("str_arguments", false, true),
            ("tuple_locals", false, true),
            ("empty_slice_fields", false, true),
            ("dead_literal_local", false, true),
            ("tuple_local_fails", false, false),
        ]
    );
    assert_eq!(r[4].2, Err((TrapKind::Assert, line_of(src, "assert(tot(r) == 1)"))));
}

/// The shapes next to those: the reference refuses the first three (`.{ ... }`
/// of the wrong length, `.{ ... }` for a slice field of an anonymous literal)
/// or points into a constant (a non-empty slice field); a local also read
/// other than as an argument stays unsupported.
#[test]
fn array_literals_typed_by_their_use_rejections() {
    let head = "module a;\n\npub struct Cur {\n    pos: usize,\n    data: []i32,\n}\n\nfn first(p: [2]u8) -> u8 {\n    return p[0];\n}\n\nfn take(c: Cur) -> usize {\n    return c.data.len;\n}\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("test t { const p = [1, 2, 3]; assert(first(p) == 1); }", "ExprArrayLiteral", "3 elements for `[2]u8`"),
        (
            "test t { const c = Cur{ .pos = 0, .data = [1, 2] }; assert(c.data.len == 2); }",
            "ExprArrayLiteral(to slice field)",
            "a non-empty array literal for a slice field",
        ),
        (
            "test t { assert(take(.{ .pos = 0, .data = [] }) == 0); }",
            "ExprArrayLiteral(to slice)",
            "anonymous struct literal",
        ),
        (
            "test t { const p = [1, 2]; assert(first(p) == 1); assert(p[1] == 2); }",
            "ExprArrayLiteral",
            "array literal with no result type",
        ),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

/// Tuples, as the reference lowers them: a tuple type in a fn return type
/// (positional or named), `return (a, b)`, `let`/`var` destructuring, the
/// top-of-test `(a, b) = f()` form, `.0` and comptime `[0]`, a struct inside
/// a tuple, forwarding a tuple return, and `.{ .. }` into an array.
#[test]
fn tuples_as_the_reference_lowers_them() {
    let src = "module a;\n\npub struct Pt {\n    x: u32,\n    y: u32,\n}\n\nfn mk(v: u32) -> Pt {\n    return Pt{ .x = v, .y = v + 1 };\n}\n\nfn pair(v: u32) -> (u32, bool) {\n    return (v * 2, v > 2);\n}\n\nfn with_pt(v: u32) -> (Pt, u64, u8) {\n    return (mk(v), 7, 2);\n}\n\nfn fwd(v: u32) -> (Pt, u64, u8) {\n    return with_pt(v + 1);\n}\n\nfn named(v: u32) -> (lo: u32, hi: u32) {\n    return (v, v ^ 1);\n}\n\nfn swap(v: u32) -> (u32, u32) {\n    var (a, b) = (v, v + 1);\n    const t = a + 0;\n    a = b;\n    b = t;\n    return (a, b);\n}\n\nfn sum_pair(v: u32) -> u32 {\n    let (d, big) = pair(v);\n    const t = pair(v + 1);\n    if (big) {\n        return d + t.0;\n    }\n    return d;\n}\n\nfn origin() -> [3]f64 {\n    return .{ 0.5, 1.5, 2.5 };\n}\n\ntest destructure {\n    const (p, n, k) = fwd(4);\n    assert(p.y == 6 and n == 7 and k == 2);\n    (d, big) = pair(5);\n    assert(d == 10 and big);\n}\n\ntest fields_and_indices {\n    const q = fwd(1);\n    const qp = q.0;\n    assert(qp.x == 2 and q[1] == 7 and q[2] == 2);\n    const m = named(6);\n    assert(m.lo == 6 and m.hi == 7);\n    const (lo, hi) = named(8);\n    assert(lo + hi == 17);\n}\n\ntest literals {\n    const (s0, s1) = swap(9);\n    assert(s0 == 10 and s1 == 9);\n    assert(sum_pair(3) == 14);\n    assert(origin()[2] == 2.5);\n}\n\ntest tuple_fails {\n    const t = pair(1);\n    assert(t[1]);\n}\n";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("destructure", false, true),
            ("fields_and_indices", false, true),
            ("literals", false, true),
            ("tuple_fails", false, false),
        ]
    );
    assert_eq!(r[3].2, Err((TrapKind::Assert, line_of(src, "assert(t[1])"))));
}

/// The shapes next to those, each refused by the reference build (a tuple
/// type outside a return type, `.{ a, b } = ..` in a fn body, a count
/// mismatch, a runtime or out-of-bounds index, `q.0.x`), and the untyped
/// tuple local, which stays unsupported.
#[test]
fn tuples_rejections() {
    let head = "module a;\n\npub struct Pt {\n    x: u32,\n}\n\nfn p() -> (u32, u32) {\n    return (1, 2);\n}\n\nfn q() -> (Pt, u32) {\n    return (Pt{ .x = 1 }, 2);\n}\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("fn f(t: (u32, u32)) -> u32 { return 1; }", "type (tuple)", "`(u32, u32)`"),
        ("test t { const t: (u32, u32) = p(); assert(t.0 == 1); }", "type (tuple)", "`(u32, u32)`"),
        (
            "fn f() -> u32 { var a: u32 = 0; var b: u32 = 0; (a, b) = p(); return a + b; }",
            "StmtAssign(tuple)",
            "outside the top of a test",
        ),
        ("fn f() -> (u32, u32) { return (1, 2, 3); }", "ExprTuple", "3 values for `(u32, u32)`"),
        ("test t { const (a, b, c) = p(); assert(a == 1); }", "StmtLocal(destructure)", "3 names for 2 elements"),
        ("fn g(i: usize) -> u32 { const t = p(); return t[i]; }", "ExprIndex(tuple)", "not known at compile time"),
        ("test t { const t = p(); assert(t[2] == 1); }", "ExprIndex(tuple)", "index 2 out of bounds"),
        ("test t { const t = q(); assert(t.0.x == 1); }", "ExprFieldAccess", "no field `0.x`"),
        ("fn g(v: u32) -> u32 { const t = (v, 2); return t[0]; }", "ExprTuple", ""),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

// An untyped local, as t27c's Zig backend prints it (#6967): `var i = 0;`
// takes u32 (u64 past u32::MAX), and `const c = undefined;` is dropped.
#[test]
fn untyped_locals_follow_the_reference() {
    let src = "module ul;\n\nfn narrow() u32 {\n    var z = 0;\n    z = z -% 1;\n    return z;\n}\n\nfn wide() u64 {\n    var v = 4294967296;\n    v = v -% 1;\n    v = v +% 2;\n    return v;\n}\n\nfn hex() u32 {\n    var m = 0xFF;\n    m = m * 16;\n    return m;\n}\n\nfn plumbing(n: u32) bool {\n    const _cfg = undefined;\n    return n > 2;\n}\n\ntest w {\n    assert(narrow() == 4294967295);\n    assert(wide() == 4294967297);\n    assert(hex() == 4080);\n    assert(plumbing(3));\n    assert(plumbing(1) == false);\n}\n";
    assert_eq!(names_ok(&run(src)), vec![("w", false, true)]);
    let head = "module ul;\nfn f() u32 {\n";
    for (body, construct, detail) in [
        ("    var x = -1;\n    x = x + 2;\n    return 0;\n}", "StmtLocal", "untyped integer"),
        ("    var x = 0.5;\n    x = x * 2.0;\n    return 0;\n}", "StmtLocal", "untyped float"),
        ("    var x = undefined;\n    return 0;\n}", "StmtLocal", "neither type nor value"),
        ("    const c = undefined;\n    return c;\n}", "ExprIdentifier", "`c`"),
    ] {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {}", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

// ------------------------------------------------------ parameter discard

/// `_ = p;` for a parameter does nothing (#7057): at the top of a fn body
/// the reference deletes it, in a nested block it is a Zig discard.
#[test]
fn parameter_discard_is_a_no_op() {
    let src = "module a;

const K: u32 = 3;

fn triple_first(x: i64, unused: i64) -> i64 {
    _ = unused;
    return x * 3;
}

fn twice(x: i64, K: u32) -> i64 {
    _ = x;
    _ = x;
    _ = K;
    return x * 2;
}

fn seven(a: u32, b: u32) -> u32 {
    if (a > 1) {
        _ = b;
    }
    return 7;
}

fn only_discards(x: u32, K: u32, y: u32) -> u32 {
    _ = x;
    if (y > 1) {
        _ = x;
        _ = x;
        _ = K;
    } else {
        _ = x;
    }
    return y;
}

test ok {
    assert(triple_first(5, 100) == 15);
    assert(seven(3, 4) == 7);
    assert(twice(-4, 1) == -8);
    assert(only_discards(1, 2, 5) == 5);
    assert(only_discards(1, 2, 0) == 0);
}

test fails {
    assert(triple_first(5, 100) == 500);
}
";
    let r = run(src);
    assert_eq!(names_ok(&r), vec![("ok", false, true), ("fails", false, false)]);
    assert_eq!(r[1].2, Err((TrapKind::Assert, line_of(src, "== 500"))));
}

/// The nested discards the reference cannot compile: Zig's AstGen refuses
/// a discard of a parameter the body also uses, in every fn. A parameter
/// that shares a module declaration's name is refused the same way, since
/// gen-zig renames it. A discard of a local stays unsupported.
#[test]
fn parameter_discard_rejections() {
    let head = "module a;\n\nconst K: u32 = 3;\n\nfn k2() -> u32 {\n    return 2;\n}\n\n";
    let cases: &[(&str, &str, &str)] = &[
        ("fn f(x: u32) -> u32 { if (x > 1) { _ = x; } return 1; }", "StmtAssign(discard)", "`_ = x;`"),
        ("fn f(x: u32, y: u32) -> u32 { if (y > 1) { _ = x; _ = x; } return x; }", "StmtAssign(discard)", "`_ = x;`"),
        ("fn f(x: u32, y: u32) -> u32 { _ = x; if (y > 1) { _ = x; } return y + x; }", "StmtAssign(discard)", "`_ = x;`"),
        ("fn f(K: u32, y: u32) -> u32 { if (y > 1) { _ = K; } return y + K; }", "StmtAssign(discard)", "`_ = K;`"),
        ("fn f(x: u32) -> u32 { var y: u32 = x; _ = y; return 1; }", "StmtAssign(undeclared)", "`_`"),
    ];
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with(&format!("t27b: unsupported construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}
