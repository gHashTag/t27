// ============================================================================
// #1741 regression: gen-verilog must hoist function-local `reg` declarations to
// the top of the body block so a function with local vars -- especially inside
// a `while` loop -- elaborates under Icarus Verilog.
//
// Before the fix, gen-verilog emitted `reg` declarations at their point of
// declaration (after preceding statements, and inside loop blocks), which
// Verilog forbids: `iverilog` rejected it with "syntax error / Malformed
// statement". Skips gracefully when iverilog is not on PATH.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_decl_hoist_{}_{}", std::process::id(), label));
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    dir
}

fn iverilog_available() -> bool {
    Command::new("iverilog")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// A function that declares locals both at the top level (interleaved with
// statements) and inside a `while` loop -- the exact shape that used to fail.
const SPEC: &str = r#"module DeclHoist;
pub fn sum_to(n: u32) -> u32 {
    var acc : u32 = 0;
    var i : u32 = 0;
    while (i < n) {
        var step : u32 = i + 1;
        acc = acc + step;
        i = i + 1;
    }
    return acc;
}
endmodule
"#;

#[test]
fn while_loop_with_locals_lowers_and_elaborates() {
    // The spec must always parse + gen-verilog, even without a HDL toolchain.
    let dir = scratch_dir("hoist");
    fs::create_dir_all(&dir).expect("create scratch dir");
    let spec = dir.join("declhoist.t27");
    fs::write(&spec, SPEC).expect("write spec");

    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(&spec)
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let v = String::from_utf8_lossy(&gen.stdout).into_owned();

    // The three local decls must appear before the first assignment in the body
    // (hoisted), i.e. no `reg` declaration follows an `acc = ` / `i = ` line.
    let body_start = v.find("begin : sum_to_body").expect("body block present");
    let body = &v[body_start..];
    let first_assign = body.find("acc = ").expect("assignment present");
    let decls_region = &body[..first_assign];
    for name in ["acc", "i", "step"] {
        assert!(
            decls_region.contains(&format!("reg [31:0] {};", name))
                || decls_region.contains(&format!("{};", name)),
            "local `{}` not hoisted before first statement:\n{}",
            name,
            v
        );
    }

    if !iverilog_available() {
        eprintln!("SKIP(elaborate): iverilog not on PATH");
        let _ = fs::remove_dir_all(&dir);
        return;
    }

    let vfile = dir.join("declhoist.v");
    fs::write(&vfile, v.as_bytes()).expect("write verilog");
    let elab = Command::new("iverilog")
        .args(["-g2012", "-t", "null"])
        .arg(&vfile)
        .output()
        .expect("invoke iverilog");
    let stderr = String::from_utf8_lossy(&elab.stderr);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        elab.status.success(),
        "iverilog rejected the hoisted function (#1741 regression):\n{}",
        stderr
    );
}

// A function that declares a SECOND top-level local after a leading statement
// (`carry` then `sum`) AND a local inside a nested `if/else` branch (`result`)
// -- the exact shape of GF-T's `gft_mul_offset_full_p` that motivated #1741.
// Distinct from the while-loop case above: the offending decls are same-block
// interleaved and inside an `else begin ... end`, not a loop. On the unfixed
// emitter this produced `reg [31:0] sum;` after `carry = ...;`, which iverilog
// rejects ("syntax error / Malformed statement").
const SPEC_NESTED_IF: &str = r#"module DeclHoistIf;
pub fn mul_offset(offset_a: u32, offset_b: u32, bias: u32, offset_max: u32) -> u32 {
    let carry : u32 = offset_a & 1;
    let sum : u32 = (offset_a + offset_b) + carry;
    if (sum < bias) {
        return 0;
    } else {
        let result : u32 = sum - bias;
        if (result >= offset_max) {
            return offset_max;
        } else {
            return result;
        }
    }
}
endmodule
"#;

#[test]
fn nested_if_else_locals_lower_and_elaborate() {
    let dir = scratch_dir("hoist_if");
    fs::create_dir_all(&dir).expect("create scratch dir");
    let spec = dir.join("declhoistif.t27");
    fs::write(&spec, SPEC_NESTED_IF).expect("write spec");

    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(&spec)
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let v = String::from_utf8_lossy(&gen.stdout).into_owned();

    // All three locals -- `carry` and `sum` (top-level) and `result` (nested in
    // the else branch) -- must be declared before the first procedural
    // statement in the body block. The first statement is `carry = ...`; no
    // `reg` decl may follow it.
    let body_start = v.find("begin : mul_offset_body").expect("body block present");
    let body = &v[body_start..];
    let first_assign = body.find("carry = ").expect("assignment present");
    let decls_region = &body[..first_assign];
    for name in ["carry", "sum", "result"] {
        assert!(
            decls_region.contains(&format!("reg [31:0] {};", name)),
            "local `{}` not hoisted before first statement:\n{}",
            name,
            v
        );
    }
    // No local `reg` may be re-declared after the first statement -- that
    // interleaved form (`reg sum;` after `carry = ...;`, and `reg result;`
    // inside the else block) is exactly what iverilog rejects.
    let after_first_assign = &body[first_assign..];
    for name in ["sum", "result"] {
        assert!(
            !after_first_assign.contains(&format!("reg [31:0] {};", name)),
            "`{}` re-declared mid-block (interleaved decl regression):\n{}",
            name,
            v
        );
    }

    if !iverilog_available() {
        eprintln!("SKIP(elaborate): iverilog not on PATH");
        let _ = fs::remove_dir_all(&dir);
        return;
    }

    let vfile = dir.join("declhoistif.v");
    fs::write(&vfile, v.as_bytes()).expect("write verilog");
    let elab = Command::new("iverilog")
        .args(["-g2012", "-t", "null"])
        .arg(&vfile)
        .output()
        .expect("invoke iverilog");
    let stderr = String::from_utf8_lossy(&elab.stderr);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        elab.status.success(),
        "iverilog rejected the nested-if hoisted function (#1741 regression):\n{}",
        stderr
    );
}
