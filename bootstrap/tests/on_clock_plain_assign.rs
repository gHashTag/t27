// ============================================================================
// t27#2363: `on_clock` dropped an assignment whose right-hand side did not
// mention the assigned variable.
//
// `dead_store_elim` runs per function body and treats a simple-identifier LHS
// as a pure write. A module-level `var` is NOT function-local -- the Verilog
// backend emits it as an `output reg` port, so the write is read from outside
// the body. `acc = acc + x` survived only by accident, because its RHS mentions
// `acc` and so put `acc` in the `reads` set; the plain register load
// `acc = x` -- `reg <= input`, the commonest sequential idiom there is -- was
// deleted outright.
//
// What made it invisible: the input port and the reset arm are emitted from the
// DECLARATIONS, not from the body, so both survived the drop. The module kept
// its full interface and its `always @(posedge clk ...)` block and simply did
// nothing, with an EMPTY `en` arm. All three `on_clock` specs in the corpus are
// self-referencing accumulators, so no spec ever exercised the broken form.
//
// The first test is the guard. The second is its anti-vacuity control: the
// self-referencing form, which passes both before and after the fix -- so a
// failure of the first test means the drop, not a broken emitter.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

/// Write `src` to a scratch `.t27` and return the Verilog `gen-verilog` emits.
fn emit(label: &str, src: &str) -> String {
    let dir = env::temp_dir().join(format!("t27_2363_{}_{}", std::process::id(), label));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    let spec = dir.join(format!("{}.t27", label));
    fs::write(&spec, src).expect("write spec");
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(&spec)
        .output()
        .expect("invoke gen-verilog");
    let _ = fs::remove_dir_all(&dir);
    assert!(
        gen.status.success(),
        "gen-verilog failed for {}:\n{}",
        label,
        String::from_utf8_lossy(&gen.stderr)
    );
    String::from_utf8_lossy(&gen.stdout).into_owned()
}

const PLAIN: &str = "module PlainLatch;\n\nvar acc : u8 = 0\n\nfn on_clock(x: u8) {\n    acc = x\n}\n";

const SELF_REF: &str =
    "module SelfRefAcc;\n\nvar acc : u8 = 0\n\nfn on_clock(x: u8) {\n    acc = acc + x\n}\n";

/// THE GUARD. A non-self-referencing clocked assignment must reach the Verilog.
#[test]
fn on_clock_emits_assignment_whose_rhs_does_not_reference_the_target() {
    let v = emit("plain", PLAIN);

    // The port and the reset come from the declarations and survived the drop --
    // assert them first so a failure below cannot be blamed on a missing `var`.
    assert!(
        v.contains("input  wire [7:0] x"),
        "streaming input port missing -- the spec did not lower at all:\n{}",
        v
    );
    assert!(
        v.contains("output reg [7:0] acc"),
        "registered state not exposed as an output port:\n{}",
        v
    );
    assert!(
        v.contains("acc <= 0;"),
        "reset arm missing -- the spec did not lower at all:\n{}",
        v
    );

    // The body. This is what #2363 deleted.
    assert!(
        v.contains("acc <= x;"),
        "t27#2363: `acc = x` was dropped from the clocked body -- the module \
         keeps its ports and its reset and does nothing:\n{}",
        v
    );

    // And the `en` arm must not be empty, which is the shape the drop left behind.
    assert!(
        !v.contains("end else if (en) begin\n        end"),
        "t27#2363: clocked process has an EMPTY `en` arm:\n{}",
        v
    );
}

/// ANTI-VACUITY CONTROL. The self-referencing form was never broken: it passes
/// against the defective compiler as well as the fixed one. If this test ever
/// fails alongside the guard above, the emitter is broken generally and the
/// guard is not measuring the #2363 drop.
#[test]
fn on_clock_self_referencing_assignment_is_emitted_control() {
    let v = emit("selfref", SELF_REF);
    assert!(
        v.contains("input  wire [7:0] x"),
        "streaming input port missing:\n{}",
        v
    );
    assert!(
        v.contains("output reg [7:0] acc"),
        "registered state not exposed as an output port:\n{}",
        v
    );
    assert!(v.contains("acc <= 0;"), "reset arm missing:\n{}", v);
    assert!(
        v.contains("acc <= (acc + x);"),
        "self-referencing accumulate was not emitted -- the emitter is broken \
         generally, so the #2363 guard is not measuring the drop:\n{}",
        v
    );
}
