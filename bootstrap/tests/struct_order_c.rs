//! C requires a type before its use; the corpus writes structs in reading order.
//!
//!     typedef struct { ... Reduction reduction; } BinaryCEConfig;   // line 30
//!     typedef struct { ... } Reduction;                             // line 34
//!     error: unknown type name 'Reduction'
//!
//! Zig and Rust require no such ordering, so nothing upstream forces it and the
//! defect is C-only. Measured: 36 specs whose every unknown-type-name error is a
//! type declared LATER in the same file; reordering unblocks 5 outright and
//! removes the family from the rest, which still fail on other things.

use std::io::Write;
use std::process::Command;

fn gen_c(src: &str) -> String {
    // Keyed by a COUNTER, not by `src.len()`. Two tests whose sources happen to
    // be the same length shared one directory, and the `remove_dir_all` below
    // deletes the whole directory -- so under the default parallel runner one
    // test erased the spec another was mid-read of, `t27c` printed nothing, and
    // the assertion read `got []`. It passed the first time it ran: a race that
    // has not fired yet is not a test that passed.
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-structorder-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The order typedefs close in, which is the order C sees them defined.
fn typedef_order(c: &str) -> Vec<String> {
    // The bodies are TAGGED now -- `struct Name {` ... `};` -- so the name is on
    // the opening line, not the closing one. Reading the old `} Name;` shape
    // would return an empty list and every ordering assertion would pass
    // vacuously: a ruler that stops matching is a test that stops testing.
    c.lines()
        .filter_map(|l| {
            let t = l.trim_start();
            let r = t.strip_prefix("struct ")?.strip_suffix(" {")?;
            let n = r.trim();
            (!n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_'))
                .then(|| n.to_string())
        })
        .collect()
}

/// Every struct is forward-declared before any body.
fn forward_decls(c: &str) -> Vec<String> {
    c.lines()
        .filter_map(|l| {
            let t = l.trim_start().strip_prefix("typedef struct ")?;
            let (a, b) = t.trim_end_matches(';').split_once(' ')?;
            (a == b).then(|| a.to_string())
        })
        .collect()
}

const FORWARD: &str =
    "module m\n\nstruct Config {\n    r: Reduction,\n}\n\nstruct Reduction {\n    kind: u32,\n}\n";

#[test]
fn a_struct_used_before_it_is_declared_is_emitted_first() {
    let order = typedef_order(&gen_c(FORWARD));
    let r = order.iter().position(|n| n == "Reduction");
    let c = order.iter().position(|n| n == "Config");
    assert!(
        r.is_some() && c.is_some(),
        "both typedefs expected, got {order:?}"
    );
    assert!(r < c, "Reduction must precede Config, got {order:?}");
}

/// A file with no forward reference is emitted exactly as written.
///
/// The sort is stable, so it must be invisible where it is not needed --
/// otherwise every seal in the corpus moves for nothing.
#[test]
fn source_order_survives_when_nothing_is_used_early() {
    let src = "module m\n\nstruct A {\n    x: u32,\n}\n\nstruct B {\n    y: u32,\n}\n\nstruct C {\n    z: u32,\n}\n";
    let order = typedef_order(&gen_c(src));
    let idx = |n: &str| order.iter().position(|x| x == n);
    assert!(idx("A") < idx("B") && idx("B") < idx("C"), "got {order:?}");
}

/// A pointer cycle keeps every declaration.
///
/// Two structs pointing at each other have no topological order. The remainder
/// is emitted in source order rather than dropped: reordering is an improvement
/// to attempt, never a reason to lose a declaration. The output is still not
/// valid C here -- that needs a forward declaration, which is a separate change.
#[test]
fn a_cycle_loses_no_declaration() {
    let src = "module m\n\nstruct L {\n    r: *R,\n}\n\nstruct R {\n    l: *L,\n}\n";
    let order = typedef_order(&gen_c(src));
    assert!(order.contains(&"L".to_string()), "L missing from {order:?}");
    assert!(order.contains(&"R".to_string()), "R missing from {order:?}");
}

/// The dependency is the type NAME, whatever wraps it.
#[test]
fn wrappers_do_not_hide_the_dependency() {
    for field in ["r: []Reduction", "r: ?Reduction", "r: [4]Reduction"] {
        let src = format!("module m\n\nstruct Config {{\n    {field},\n}}\n\nstruct Reduction {{\n    kind: u32,\n}}\n");
        let order = typedef_order(&gen_c(&src));
        let r = order.iter().position(|n| n == "Reduction");
        let c = order.iter().position(|n| n == "Config");
        assert!(
            r < c,
            "field `{field}` must still order Reduction first, got {order:?}"
        );
    }
}

/// A constant of a declared type is emitted after that type.
///
/// The Constants section precedes Structs, and it cannot simply move: a
/// `[T; N]` value struct may size itself with a const name, so constants must
/// also come first. Both are true of DIFFERENT constants, which is why the
/// section splits rather than swaps.
#[test]
fn a_struct_typed_constant_is_emitted_after_its_struct() {
    let src = "module m\n\nconst SIZE: u32 = 4\n\nconst PIN: Pin = Pin { n: 1 }\n\nstruct Pin {\n    n: u32,\n}\n";
    let c = gen_c(src);
    let pos = |needle: &str| c.find(needle);
    let ty = pos("struct Pin {").expect("the struct body");
    let konst = pos("PIN").expect("the constant");
    assert!(ty < konst, "the struct must precede its constant:\n{c}");
}

/// A primitive constant stays where the array structs need it.
///
/// Moving every constant below the structs would break `[T; SIZE]`, whose
/// value struct is emitted from the const's numeric value.
#[test]
fn a_primitive_constant_stays_above_the_structs() {
    let src = "module m\n\nconst SIZE: u32 = 4\n\nstruct Pin {\n    n: u32,\n}\n";
    let c = gen_c(src);
    let konst = c.find("SIZE").expect("the constant");
    let ty = c.find("struct Pin {").expect("the struct body");
    assert!(konst < ty, "a primitive constant must stay first:\n{c}");
}

/// A struct that names itself compiles.
///
/// `BTreeNode** children` inside `BTreeNode` is a cycle of length one: no
/// ordering can put a declaration before itself, and the sort in this file is
/// powerless against it. The forward declaration gives C the NAME, which is all
/// a pointer member needs.
///
/// Measured: 24 specs write a self-referencing struct; four of them compiled
/// outright once the names existed, with zero regressions, and all four are the
/// tree structures this defect was found in.
#[test]
fn a_self_referencing_struct_is_forward_declared() {
    let src = "module m\n\nstruct Node {\n    next: *Node,\n    v: u32,\n}\n";
    let c = gen_c(src);
    assert!(
        forward_decls(&c).contains(&"Node".to_string()),
        "expected `typedef struct Node Node;`, got:\n{c}"
    );
    let fwd = c
        .find("typedef struct Node Node;")
        .expect("forward declaration");
    let body = c.find("struct Node {").expect("body");
    assert!(fwd < body, "the forward declaration must come first:\n{c}");
}

/// Forward declarations cover every struct, not only the recursive ones.
///
/// A rule that declared only self-referencing structs would leave a mutual
/// pointer cycle -- two structs naming each other -- with no name in scope for
/// either, and that cycle is exactly what the topological sort cannot order.
#[test]
fn every_struct_is_forward_declared_not_only_the_recursive_ones() {
    let src = "module m\n\nstruct L {\n    r: *R,\n}\n\nstruct R {\n    l: *L,\n}\n";
    let d = forward_decls(&gen_c(src));
    assert!(
        d.contains(&"L".to_string()) && d.contains(&"R".to_string()),
        "got {d:?}"
    );
}

/// The body carries the tag, or the forward declaration names nothing.
#[test]
fn bodies_are_tagged_rather_than_anonymous() {
    let c = gen_c(FORWARD);
    assert!(
        c.contains("struct Reduction {"),
        "expected a tagged body:\n{c}"
    );
    assert!(
        !c.contains("} Reduction;"),
        "an anonymous typedef cannot be forward-declared:\n{c}"
    );
}
