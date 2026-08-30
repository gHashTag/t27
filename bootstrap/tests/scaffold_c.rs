//! The scaffold template, lowered for C.
//!
//! 571 test blocks are shaped
//!
//!     test f_basic_case
//!         given input = default_input()
//!         when result = f(input)
//!         then result != undefined
//!
//! and nothing in the tree defines either helper. Zig resolved this in W585,
//! Verilog in W660 -- whose comment names its sibling and stops there. The C
//! path was never grepped, and it emitted
//!
//!     __auto_type input = default_input();      // call to undeclared function
//!     __auto_type result = f(input);            // incomplete type 'void'
//!     assert((result != {0}));                  // expected expression
//!
//! Three cc messages from ONE construct, which is why a census that grouped by
//! message text reported this family as worth nothing. Measured: 174 -> 242
//! specs accepted by cc, the largest single lever in the project.

use std::io::Write;
use std::process::Command;

fn gen_c(src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("t27-scaffold-{}-{}", std::process::id(), src.len()));
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

fn body(c: &str, test_fn: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for l in c.lines() {
        if l.contains(test_fn) && l.contains("(void) {") {
            inside = true;
            continue;
        }
        if inside {
            if l.starts_with('}') {
                break;
            }
            out.push_str(l.trim());
            out.push('\n');
        }
    }
    out
}

const SCALAR: &str = "module m\n\nfn take(x: u32) -> u32 {\n    return x\n}\n\ntest take_basic\n    given input = default_input()\n    when result = take(input)\n    then result != undefined\n";

/// The binding takes its type from the consumer, not from the initialiser.
///
/// `default_input()` takes no arguments and returns whatever the next line
/// needs, so its own call site carries no type. Verilog could write a bare `0`
/// because its bindings are already declared `reg` of the right width; C has no
/// such declaration and `__auto_type input = 0` is an `int`. Measured: a plain
/// `0` traded `call to undeclared function` (86 -> 13) for `incompatible
/// integer to pointer conversion` (0 -> 68) and moved the accept count by zero.
#[test]
fn a_scaffold_binding_is_typed_by_its_consumer() {
    let b = body(&gen_c(SCALAR), "test_take_basic");
    assert!(b.contains("uint32_t input = 0;"), "expected a typed zero, got:\n{b}");
    assert!(!b.contains("default_input"), "the call must not survive:\n{b}");
}

/// `!= undefined` asserts nothing, and C must not pretend otherwise.
///
/// `{0}` is a brace initialiser, not an operand. Writing `0` instead would
/// compile and assert a claim the spec does not make; `(typeof(x)){0}` is still
/// illegal for a struct because `!=` does not apply to structs. The Zig
/// backend's own comment records that these tests "constrain the value not at
/// all".
#[test]
fn a_vacuous_assertion_is_not_invented() {
    let b = body(&gen_c(SCALAR), "test_take_basic");
    assert!(!b.contains("{0}"), "a brace initialiser is not an operand:\n{b}");
    assert!(!b.contains("assert("), "nothing is asserted here:\n{b}");
    assert!(b.contains("constrains nothing"), "and it must say so:\n{b}");
}

/// A void-returning consumer binds nothing.
///
/// Both spellings: an omitted return type and an explicit `-> void`. Checking
/// only for the omitted one left 80 specs failing on `variable has incomplete
/// type 'void'` -- all of them saying it out loud.
#[test]
fn a_void_consumer_binds_nothing_in_either_spelling() {
    for ret in ["", " -> void"] {
        let src = format!(
            "module m\n\nfn act(x: u32){ret} {{\n}}\n\ntest act_basic\n    given input = default_input()\n    when result = act(input)\n    then result != undefined\n"
        );
        let b = body(&gen_c(&src), "test_act_basic");
        assert!(b.contains("act(input);"), "the call is the clause's effect:\n{b}");
        assert!(
            !b.contains("result = act"),
            "`__auto_type` cannot deduce from void (ret={ret:?}):\n{b}"
        );
    }
}

/// A consumer that DOES return a value still binds.
///
/// The void arm must not swallow every call: `take` returns `u32`, so `result`
/// is a real binding and dropping it would lose the value the next clause reads.
#[test]
fn a_value_returning_consumer_still_binds() {
    let b = body(&gen_c(SCALAR), "test_take_basic");
    assert!(b.contains("result = take(input);"), "expected a binding, got:\n{b}");
}
