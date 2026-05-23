// =============================================================================
// Wave 30 -- R-TR-1 compliance test for the gen-verilog bench emitter.
//
// `gen_verilog` used to emit `initial begin :NAME // synthesis translate_off`
// and `end // synthesis translate_on` with the translate_off / translate_on
// markers INLINE on the same line as the `initial begin` and `end`
// keywords. Yosys treats `translate_off` as a line-range skip directive:
// when the skip starts on the same line as `initial begin :NAME`, the
// matching `end` keyword is consumed inside the skipped region. The
// parser is left mid-`initial begin`, hits the next `initial begin`,
// and emits:
//
//     ERROR: syntax error, unexpected TOK_INITIAL
//
// Wave 30 patches the bench-section emitter to write
// `// synthesis translate_off` and `// synthesis translate_on` as
// STANDALONE comment lines wrapping the full `initial begin ... end`
// block. This guarantees the entire block is uniformly inside the
// skip region OR uniformly outside, never split.
//
// This integration test shells out to the built `t27c` binary, feeds it a
// spec that contains a `bench` block, and asserts the emitted Verilog
// (a) never places `translate_off` or `translate_on` on the same line as
// `initial begin` or `end`; (b) emits at least one standalone
// `// synthesis translate_off` line and one standalone
// `// synthesis translate_on` line per bench.
//
// We also run a regression check against the real `specs/fpga/uart.t27`
// spec, which is where the bug was first observed in CI on PR #746.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// Closes #747
// =============================================================================

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Minimal spec exercising the bench-block emitter path.
const SPEC: &str = r#"module RTR1Probe {
    bench probe_latency_a
        measure: nanoseconds to probe_a()
        target: < 10ns

    bench probe_latency_b
        measure: nanoseconds to probe_b()
        target: < 20ns
}
"#;

fn compile_spec(spec_text: &str, file_stem: &str) -> Option<String> {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let tmp_dir = std::env::temp_dir();
    let spec_path = tmp_dir.join(format!("{}.t27", file_stem));
    {
        let mut f = std::fs::File::create(&spec_path).ok()?;
        f.write_all(spec_text.as_bytes()).ok()?;
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

/// Return true if any line that begins with `initial begin` or `end`
/// (i.e. starts a Verilog initial-block boundary) contains a
/// `translate_off` or `translate_on` marker as an inline trailing
/// comment. Both are R-TR-1 violations: they cause Yosys to split the
/// initial-block tokens across a skip boundary.
fn has_inline_translate_marker(src: &str) -> Option<String> {
    for (idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        let starts_initial = trimmed.starts_with("initial begin");
        // Look for `end` as a statement boundary: an `end` keyword followed
        // by whitespace, end-of-line, or a comment. We deliberately do NOT
        // match `endmodule`, `endfunction`, `endtask`, `endcase`, etc.
        let starts_end = trimmed == "end"
            || trimmed.starts_with("end ")
            || trimmed.starts_with("end//")
            || trimmed.starts_with("end /*");
        if !(starts_initial || starts_end) {
            continue;
        }
        if line.contains("translate_off") || line.contains("translate_on") {
            return Some(format!(
                "line {}: inline translate marker on initial/end boundary: {}",
                idx + 1,
                line
            ));
        }
    }
    None
}

/// Count standalone `// synthesis translate_off` lines (i.e. the line,
/// after trimming, equals exactly `// synthesis translate_off`).
fn count_standalone_translate_off(src: &str) -> usize {
    src.lines()
        .filter(|l| l.trim() == "// synthesis translate_off")
        .count()
}

fn count_standalone_translate_on(src: &str) -> usize {
    src.lines()
        .filter(|l| l.trim() == "// synthesis translate_on")
        .count()
}

// -----------------------------------------------------------------------------
// Test 1: synthetic spec.
// -----------------------------------------------------------------------------

#[test]
fn r_tr_1_synthetic_no_inline_translate_marker() {
    let Some(src) = compile_spec(SPEC, "wave30_r_tr_1_probe") else {
        panic!("t27c gen-verilog failed on synthetic R-TR-1 probe spec");
    };

    if let Some(viol) = has_inline_translate_marker(&src) {
        panic!(
            "R-TR-1 regression: {}\n--- emitted Verilog ---\n{}",
            viol, src
        );
    }

    // Each of the 2 benches in the synthetic spec must be wrapped in its
    // own standalone translate_off / translate_on pair. We also have one
    // pair around the module-scope `integer` counter declarations from
    // Wave 29, so we expect AT LEAST 3 of each marker (1 counter band +
    // 1 per bench).
    let off_count = count_standalone_translate_off(&src);
    let on_count = count_standalone_translate_on(&src);
    assert!(
        off_count >= 3,
        "Expected >= 3 standalone `// synthesis translate_off` lines, got {}.\n\
         --- emitted Verilog ---\n{}",
        off_count,
        src
    );
    assert!(
        on_count >= 3,
        "Expected >= 3 standalone `// synthesis translate_on` lines, got {}.\n\
         --- emitted Verilog ---\n{}",
        on_count,
        src
    );
}

// -----------------------------------------------------------------------------
// Test 2: regression against the real uart.t27 spec.
// -----------------------------------------------------------------------------

fn find_repo_root() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cur: &std::path::Path = &manifest;
    loop {
        let candidate = cur.join("specs").join("fpga").join("uart.t27");
        if candidate.is_file() {
            return Some(cur.to_path_buf());
        }
        cur = cur.parent()?;
    }
}

#[test]
fn r_tr_1_real_uart_spec_no_inline_translate_marker() {
    let Some(repo) = find_repo_root() else {
        panic!("Could not locate repo root containing specs/fpga/uart.t27");
    };
    let uart_path = repo.join("specs").join("fpga").join("uart.t27");
    let bin = env!("CARGO_BIN_EXE_t27c");
    let out = Command::new(bin)
        .args(["gen-verilog", uart_path.to_str().expect("path utf8")])
        .output()
        .expect("t27c gen-verilog on uart.t27 should run");
    assert!(
        out.status.success(),
        "t27c gen-verilog failed on real uart.t27 spec:\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let src = String::from_utf8_lossy(&out.stdout).into_owned();

    if let Some(viol) = has_inline_translate_marker(&src) {
        panic!("R-TR-1 regression on real uart.t27: {}", viol);
    }

    // uart.t27 has 3 benches; with the Wave 29 counter band we expect
    // >= 4 standalone translate_off and >= 4 standalone translate_on.
    let off_count = count_standalone_translate_off(&src);
    let on_count = count_standalone_translate_on(&src);
    assert!(
        off_count >= 4,
        "Expected >= 4 standalone translate_off on real uart.t27, got {}",
        off_count
    );
    assert!(
        on_count >= 4,
        "Expected >= 4 standalone translate_on on real uart.t27, got {}",
        on_count
    );
}
