// =============================================================================
// Wave 27 — R-SI-1 compliance test for the gen-verilog emitter.
//
// OpenLane / RTL synthesis rule R-SI-1 forbids the `*` operator in
// synthesizable Verilog. t27c must therefore emit a call to the helper
// function `__mul_noop(a, b)` whenever a multiplication appears in the
// source spec.
//
// This integration test shells out to the built `t27c` binary, feeds it a
// small synthetic spec containing two multiplications (the same shapes
// that `specs/fpga/mac.t27` uses: `index * 2` and `row * cols`), and
// asserts:
//
//   1. The emitted Verilog does not contain a bare `*` operator
//      (we allow `*` only inside `/* ... */` block comments and `// ...`
//      line comments such as the `phi^2 + 1/phi^2 = 3` header).
//   2. The emitted Verilog declares the `__mul_noop` helper function.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// Closes #692
// =============================================================================

use std::io::Write;
use std::process::Command;

const SPEC: &str = r#"module RSI1Probe {
    const WIDTH: u32 = 27;

    fn mul_bit_pos(index: u32) -> u32 {
        let bit_pos: u32 = index * 2;
        return bit_pos;
    }

    fn mul_mat_idx(row: u32, col: u32, cols: u32) -> u32 {
        let mat_idx: u32 = (row * cols) + col;
        return mat_idx;
    }
}
"#;

/// Strip `/* ... */` block comments and `// ...` line comments so we can
/// inspect operator usage in code-only positions.
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = i.saturating_add(2);
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Compile SPEC via the t27c binary. Returns stdout (the emitted Verilog).
/// If the binary fails for any reason, returns None — in that case the
/// front-end is broken in some other way and the R-SI-1 test is
/// inconclusive (we do not want to mask front-end regressions).
fn compile_spec() -> Option<String> {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let tmp_dir = std::env::temp_dir();
    let spec_path = tmp_dir.join(format!("wave27_r_si_1_probe_{}.t27", std::process::id()));
    {
        let mut f = std::fs::File::create(&spec_path).ok()?;
        f.write_all(SPEC.as_bytes()).ok()?;
    }
    let out = Command::new(bin)
        .args(["gen-verilog", spec_path.to_str()?])
        .output()
        .ok()?;
    if !out.status.success() {
        eprintln!(
            "t27c gen-verilog exited with status {:?}\nstderr: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn r_si_1_emitter_produces_no_bare_star_operator() {
    let Some(verilog) = compile_spec() else {
        panic!("t27c failed on probe spec -- front-end regression masked");
    };

    let code_only = strip_comments(&verilog);
    assert!(
        !code_only.contains('*'),
        "R-SI-1 violation: emitted Verilog contains bare `*` operator.\n\
         --- emitted (code-only) ---\n{code_only}\n\
         --- full output ---\n{verilog}"
    );
}

#[test]
fn r_si_1_emitter_injects_mul_noop_helper() {
    let Some(verilog) = compile_spec() else {
        panic!("t27c failed on probe spec -- front-end regression masked");
    };

    assert!(
        verilog.contains("function [31:0] __mul_noop;"),
        "R-SI-1 helper missing: `__mul_noop` function declaration not found.\n\
         --- emitted ---\n{verilog}"
    );
    assert!(
        verilog.contains("endfunction"),
        "R-SI-1 helper malformed: no `endfunction` keyword.\n\
         --- emitted ---\n{verilog}"
    );
}
