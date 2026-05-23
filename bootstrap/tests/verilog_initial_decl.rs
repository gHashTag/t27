// =============================================================================
// Wave 29 -- R-VD-1 compliance test for the gen-verilog bench emitter.
//
// `gen_verilog_module` used to emit
//
//     initial begin : NAME_bench
//         $display("[BENCH] NAME : starting");
//         integer _bench_cycles = 0;      // <-- R-VD-1 violation
//         ...
//     end
//
// Verilog-2005 forbids variable declarations inside procedural blocks
// (Yosys/iverilog reject this with `syntax error, unexpected TOK_INITIAL`).
// Wave 29 patches the bench-section emitter to hoist a uniquely-named
// `integer _bench_<name>_cycles = 0;` to module scope and only assign/use
// the counter inside the `initial begin ... end` block.
//
// This integration test shells out to the built `t27c` binary, feeds it a
// spec that contains a `bench` block, and asserts the emitted Verilog
// (a) contains no `integer ... ;` declaration inside any
// `initial begin ... end` block; and (b) contains a module-scope
// `integer _bench_<name>_cycles = 0;` line for each bench.
//
// We also run a regression check against the real `specs/fpga/uart.t27`
// spec, which is where the bug was first observed in CI on PR #744.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// Closes #745
// =============================================================================

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Minimal spec exercising the bench-block emitter path.
const SPEC: &str = r#"module RVD1Probe {
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

/// Return true if the source contains an `integer <name> ... ;` line
/// somewhere between an `initial begin` and its matching `end`. We use a
/// brace-free, line-oriented scan rather than a full parser because the
/// emitted Verilog is line-formatted and the only constructs we care about
/// are `initial begin`, `end`, and `integer`.
fn has_integer_decl_inside_initial(src: &str) -> bool {
    let mut depth: i32 = 0;
    for line in src.lines() {
        let trimmed = line.trim_start();
        // Track entry into an `initial begin` block.
        if trimmed.starts_with("initial begin") {
            depth += 1;
            continue;
        }
        // Track exit of any `end` line (we only count `end` when we are
        // inside an initial block; nested `begin`/`end` pairs are not
        // expected at the bench emitter level, but we conservatively only
        // close at the FIRST `end` after a depth increment).
        if depth > 0 && (trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end//") || trimmed.starts_with("end /*")) {
            depth -= 1;
            continue;
        }
        if depth > 0 && trimmed.starts_with("integer ") {
            eprintln!("BAD: integer decl inside initial block: {}", line);
            return true;
        }
    }
    false
}

/// Return the set of module-scope `integer ... = 0;` counter declarations
/// of the form `integer _bench_<name>_cycles = 0;`.
fn module_scope_bench_counters(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside_initial = false;
    for line in src.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("initial begin") {
            inside_initial = true;
            continue;
        }
        if inside_initial
            && (trimmed == "end"
                || trimmed.starts_with("end ")
                || trimmed.starts_with("end//")
                || trimmed.starts_with("end /*"))
        {
            inside_initial = false;
            continue;
        }
        if inside_initial {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("integer ") {
            if let Some(name_end) = rest.find(|c: char| c.is_whitespace() || c == '=' || c == ';')
            {
                let name = &rest[..name_end];
                if name.starts_with("_bench_") && name.ends_with("_cycles") {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

// -----------------------------------------------------------------------------
// Test 1: synthetic spec.
// -----------------------------------------------------------------------------

#[test]
fn r_vd_1_synthetic_no_integer_decl_inside_initial() {
    let Some(src) = compile_spec(SPEC, "wave29_r_vd_1_probe") else {
        panic!("t27c gen-verilog failed on synthetic R-VD-1 probe spec");
    };

    assert!(
        !has_integer_decl_inside_initial(&src),
        "R-VD-1 regression: emitter produced `integer ...;` inside an `initial begin ... end` block.\n\
         --- emitted Verilog ---\n{}",
        src
    );

    let counters = module_scope_bench_counters(&src);
    assert_eq!(
        counters.len(),
        2,
        "Expected exactly 2 module-scope `_bench_<name>_cycles` counters \
         (one per bench in synthetic spec), got {:?}.\n\
         --- emitted Verilog ---\n{}",
        counters,
        src
    );
    assert!(
        counters.iter().any(|c| c.contains("probe_latency_a")),
        "Missing module-scope counter for probe_latency_a in {:?}",
        counters
    );
    assert!(
        counters.iter().any(|c| c.contains("probe_latency_b")),
        "Missing module-scope counter for probe_latency_b in {:?}",
        counters
    );
}

// -----------------------------------------------------------------------------
// Test 2: regression against the real uart.t27 spec (the one that broke CI
// on PR #744).
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
fn r_vd_1_real_uart_spec_no_integer_decl_inside_initial() {
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

    assert!(
        !has_integer_decl_inside_initial(&src),
        "R-VD-1 regression on real uart.t27: emitter produced \
         `integer ...;` inside an `initial begin ... end` block."
    );

    let counters = module_scope_bench_counters(&src);
    assert!(
        counters.len() >= 3,
        "Expected at least 3 module-scope `_bench_<name>_cycles` counters \
         on real uart.t27 (3 benches), got {} ({:?}).",
        counters.len(),
        counters
    );
}
