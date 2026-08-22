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

fn compile_spec(spec_text: &str, file_stem: &str, sub: &str) -> Option<String> {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let tmp_dir = std::env::temp_dir();
    let spec_path = tmp_dir.join(format!("{}.t27", file_stem));
    {
        let mut f = std::fs::File::create(&spec_path).ok()?;
        f.write_all(spec_text.as_bytes()).ok()?;
    }
    let out = Command::new(bin)
        .args([sub, spec_path.to_str()?])
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
/// Count standalone `` `ifndef SIMULATION `` lines — the guard band that
/// replaced `// synthesis translate_off` in W458.
///
/// The marker changed but the hazard did not: yosys treats `translate_off` as
/// a line-range skip, so a marker on the same line as `initial begin :NAME`
/// swallows the matching `end`. A `` `ifndef `` on its own line cannot do that,
/// which is why the emitter moved to it — and why "standalone" is still the
/// property worth asserting.
fn count_standalone_ifndef_sim(src: &str) -> usize {
    src.lines()
        .filter(|l| l.trim() == "`ifndef SIMULATION")
        .count()
}

fn count_standalone_endif(src: &str) -> usize {
    src.lines().filter(|l| l.trim() == "`endif").count()
}

/// Every **named** `initial begin : NAME` must sit inside an open
/// `` `ifndef SIMULATION `` band, and no guard may share its line.
///
/// Two things this deliberately does NOT require, each learned by running it:
///
/// * **Anonymous `initial begin` is exempt.** The emitter uses it for register
///   power-on initialisation, which is synthesizable and must not be guarded —
///   wrapping `uart_state`'s initialiser would drop the reset state from the
///   bitstream. Only named blocks are simulation constructs.
/// * **The guard is a band, not a per-block prefix.** One `` `ifndef `` can wrap
///   the `$dumpfile` block and several test blocks together, so checking the
///   immediately preceding line rejects correct output. Depth is tracked instead.
///
/// The hazard R-TR-1 names is unchanged: yosys treats `translate_off` as a
/// line-range skip, so a marker sharing the `initial begin :NAME` line swallows
/// the matching `end`. A standalone `` `ifndef `` cannot, which is why W458 moved
/// to it — and why "standalone" is still the property asserted.
fn initial_begin_without_standalone_guard(src: &str) -> Option<String> {
    let mut depth: i32 = 0;
    for (n, l) in src.lines().enumerate() {
        let t = l.trim();
        if t == "`ifndef SIMULATION" {
            depth += 1;
            continue;
        }
        if t == "`endif" {
            depth -= 1;
            continue;
        }
        if !l.contains("initial begin :") {
            continue;
        }
        if l.contains("`ifndef") || l.contains("translate_off") {
            return Some(format!("line {}: guard is inline on `{}`", n + 1, t));
        }
        if depth <= 0 {
            return Some(format!(
                "line {}: `{}` is outside any `ifndef SIMULATION band",
                n + 1,
                t
            ));
        }
    }
    None
}

// -----------------------------------------------------------------------------
// Test 1: synthetic spec.
// -----------------------------------------------------------------------------

#[test]
fn r_tr_1_synthetic_no_inline_translate_marker() {
    // Bench blocks are no longer lowered by `gen-verilog` — it emits
    // synthesizable RTL and a bench is a simulation construct (#2391). The
    // guards live in the simulation backend, so that is where the bands are
    // counted; the inline-marker rule is checked on both.
    let Some(rtl) = compile_spec(SPEC, "wave30_r_tr_1_probe", "gen-verilog") else {
        panic!("t27c gen-verilog failed on synthetic R-TR-1 probe spec");
    };
    let Some(sim) = compile_spec(SPEC, "wave30_r_tr_1_probe_sim", "gen-verilog-for-simulation")
    else {
        panic!("t27c gen-verilog-for-simulation failed on synthetic R-TR-1 probe spec");
    };

    for (which, src) in [("gen-verilog", &rtl), ("gen-verilog-for-simulation", &sim)] {
        if let Some(viol) = has_inline_translate_marker(src) {
            panic!("R-TR-1 regression in {which}: {viol}\n--- emitted Verilog ---\n{src}");
        }
        if let Some(viol) = initial_begin_without_standalone_guard(src) {
            panic!("R-TR-1 regression in {which}: {viol}\n--- emitted Verilog ---\n{src}");
        }
    }

    // One band around the module-scope counter declarations plus one per bench.
    let off = count_standalone_ifndef_sim(&sim);
    let on = count_standalone_endif(&sim);
    assert!(
        off >= 3,
        "Expected >= 3 standalone `ifndef SIMULATION lines from \
         gen-verilog-for-simulation (1 counter band + 1 per bench), got {off}.\n\
         --- emitted Verilog ---\n{sim}"
    );
    assert!(
        on >= 3,
        "Expected >= 3 standalone `endif lines, got {on}.\n--- emitted Verilog ---\n{sim}"
    );
    assert_eq!(
        off, on,
        "Unbalanced guard bands: {off} `ifndef vs {on} `endif"
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

/// Run one emitter subcommand over a spec file and return its stdout.
fn emit_file(sub: &str, path: &std::path::Path) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let out = Command::new(bin)
        .args([sub, path.to_str().expect("path utf8")])
        .output()
        .unwrap_or_else(|e| panic!("t27c {sub} should run: {e}"));
    assert!(
        out.status.success(),
        "t27c {sub} failed on {}:\nstderr: {}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn r_tr_1_real_uart_spec_no_inline_translate_marker() {
    let Some(repo) = find_repo_root() else {
        panic!("Could not locate repo root containing specs/fpga/uart.t27");
    };
    let uart_path = repo.join("specs").join("fpga").join("uart.t27");

    let rtl = emit_file("gen-verilog", &uart_path);
    let sim = emit_file("gen-verilog-for-simulation", &uart_path);

    for (which, src) in [("gen-verilog", &rtl), ("gen-verilog-for-simulation", &sim)] {
        if let Some(viol) = has_inline_translate_marker(src) {
            panic!("R-TR-1 regression on real uart.t27 via {which}: {viol}");
        }
        if let Some(viol) = initial_begin_without_standalone_guard(src) {
            panic!("R-TR-1 regression on real uart.t27 via {which}: {viol}");
        }
    }

    // uart.t27 has 3 benches; with the counter band that is >= 4 guard pairs.
    let off = count_standalone_ifndef_sim(&sim);
    let on = count_standalone_endif(&sim);
    assert!(
        off >= 4,
        "Expected >= 4 standalone `ifndef SIMULATION on real uart.t27 from \
         gen-verilog-for-simulation, got {off}"
    );
    assert!(on >= 4, "Expected >= 4 standalone `endif on real uart.t27, got {on}");
    assert_eq!(off, on, "Unbalanced guard bands on uart.t27: {off} vs {on}");
}
