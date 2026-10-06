//! `pass_vacuous` (#6115): a test that passes having executed no runtime
//! assert. The count is the reference interpreter's, which `t27b test --check`
//! runs beside the JIT for every test; `corpus` labels a pass with a count of 0
//! `pass_vacuous`.

use std::path::Path;

use t27b::eval::Interp;
use t27b::ir::OverflowMode;
use t27b::{front, lower};

/// Runtime asserts executed by every test of `src`, summed.
fn runtime_asserts(name: &str, src: &str) -> u64 {
    let parsed = front::parse(Path::new(name), src).expect("fixture parses");
    let prog = lower::lower(&parsed.ast, OverflowMode::Trap).expect("fixture lowers");
    let mut total = 0;
    for (id, _) in prog.tests() {
        let mut it = Interp::new(&prog);
        it.call(id, &[]).expect("fixture test passes");
        total += it.asserts;
    }
    total
}

/// Every assert folds to a constant: nothing is checked at run time.
const CONSTANT_ONLY: &str = r#"module vacuous_fixture;
test folded {
    assert(1 + 1 == 2);
    assert_eq(3, 3);
    assert(true);
}
"#;

/// The negative control: the same shape, but each assert depends on a call.
const RUNTIME: &str = r#"module runtime_fixture;
fn inc(x: u32) -> u32 {
    return x + 1;
}
test checked {
    assert_eq(inc(1), 2);
    assert(inc(2) == 3);
}
"#;

/// An assert inside a loop counts once per execution, not once per site.
const LOOP: &str = r#"module loop_fixture;
fn id(x: u32) -> u32 {
    return x;
}
test loops {
    var i : u32 = 0;
    while i < 4 {
        assert(id(i) == i);
        i = i + 1;
    }
}
"#;

#[test]
fn constant_only_asserts_are_vacuous() {
    assert_eq!(runtime_asserts("vacuous_fixture.t27", CONSTANT_ONLY), 0);
}

#[test]
fn runtime_asserts_are_counted() {
    assert_eq!(runtime_asserts("runtime_fixture.t27", RUNTIME), 2);
}

#[test]
fn asserts_are_counted_per_execution() {
    assert_eq!(runtime_asserts("loop_fixture.t27", LOOP), 4);
}
