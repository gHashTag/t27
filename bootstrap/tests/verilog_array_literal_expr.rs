// =============================================================================
// Wave 31 -- R-CA-2 compliance test for the gen-verilog ExprArrayLiteral
// emitter in expression context.
//
// `gen_verilog_expr` for `NodeKind::ExprArrayLiteral` used to write a
// comment-only token of the form `/* array [...]{} */`. When such an
// array literal appeared as a function-call argument, the entire
// argument reduced to whitespace plus the `,` separator, and Yosys
// rejected the call with `syntax error, unexpected ','`. The bug was
// first observed in CI on PR #748 at `bridge.v:166`:
//
//     mac_dot_product(/* array [operand_a]{} */, /* array [operand_b]{} */, 1, unit_byte);
//
// Wave 31 patches the emitter to follow the precedent established by
// `ExprStructLit` and emit a synthesizable scalar `0` plus an
// explanatory TODO comment, so the surrounding expression remains
// parseable Verilog:
//
//     mac_dot_product(0 /* TODO: array literal [operand_a] not yet lowered to Verilog */,
//                     0 /* TODO: array literal [operand_b] not yet lowered to Verilog */,
//                     1, unit_byte);
//
// We assert:
//   (a) the emitted Verilog never contains a comment-only function-call
//       argument of the form `(/* ... */,` or `,/* ... */,` or
//       `,/* ... */)`;
//   (b) at least one `0 /* TODO: array literal ... */` placeholder is
//       present in the regression spec (i.e. the new emit path is
//       actually exercised, not just dead-code-removed).
//
// We also run a regression check against the real `specs/fpga/bridge.t27`
// spec, which is where the bug was first observed in CI.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// Closes #749
// =============================================================================

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Minimal spec exercising the ExprArrayLiteral path in expression
/// context (passing an array literal as a function argument).
const SPEC: &str = r#"module RCA2Probe {
    fn consume(values: [4]u32) {
        return
    }

    fn driver() {
        consume([1, 2, 3, 4])
        return
    }
}
"#;

fn compile_spec(spec_text: &str, file_stem: &str) -> Option<String> {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let tmp_dir = std::env::temp_dir();
    let spec_path = tmp_dir.join(format!("{}_{}.t27", file_stem, std::process::id()));
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

/// Strip Verilog block comments `/* ... */` (single-line only -- the
/// emitter never produces multi-line block comments). Used to detect
/// when, after comment removal, a function-call argument reduces to
/// empty whitespace.
fn strip_block_comments(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Skip until matching `*/` on the same line.
            let mut j = i + 2;
            while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                j += 1;
            }
            i = j.saturating_add(2);
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Detect the R-CA-2 violation pattern: a function-call argument that,
/// after stripping block comments, reduces to empty / whitespace. We
/// scan lines and look for `(<arg>,`, `,<arg>,`, or `,<arg>)` where
/// `<arg>` is empty after stripping comments.
fn has_comment_only_call_argument(src: &str) -> Option<String> {
    for (idx, line) in src.lines().enumerate() {
        let bare = strip_block_comments(line);
        // Look for `(` or `,` followed by only whitespace before the next `,` or `)`.
        // This is a deliberately conservative check that catches the
        // specific bug shape without trying to fully parse Verilog.
        let chars: Vec<char> = bare.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '(' || c == ',' {
                // Skip whitespace and look for next non-space.
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == ',' || chars[j] == ')') {
                    // Empty argument between `c` and `chars[j]`.
                    // Allow truly empty `()` (no-arg call): only flag when
                    // c == ',' (so there is at least one prior argument
                    // separator) OR when c == '(' and chars[j] == ',' (so
                    // there IS a next argument after the empty one).
                    if c == ',' || chars[j] == ',' {
                        return Some(format!(
                            "line {}: comment-only call argument: {}",
                            idx + 1,
                            line
                        ));
                    }
                }
            }
            i += 1;
        }
    }
    None
}

// -----------------------------------------------------------------------------
// Test 1: synthetic spec.
// -----------------------------------------------------------------------------

#[test]
fn r_ca_2_synthetic_no_comment_only_call_argument() {
    let Some(src) = compile_spec(SPEC, "wave31_r_ca_2_probe") else {
        panic!("t27c gen-verilog failed on synthetic R-CA-2 probe spec");
    };

    if let Some(viol) = has_comment_only_call_argument(&src) {
        panic!(
            "R-CA-2 regression: {}\n--- emitted Verilog ---\n{}",
            viol, src
        );
    }

    // Positive check: the placeholder TODO marker should appear at least
    // once in the synthetic spec emission. This guarantees the new emit
    // path is actually exercised.
    assert!(
        src.contains("TODO: array literal"),
        "Expected the new `0 /* TODO: array literal ... */` placeholder \
         in the synthetic spec emission, but none was found.\n\
         --- emitted Verilog ---\n{}",
        src
    );
}

// -----------------------------------------------------------------------------
// Test 2: regression against the real bridge.t27 spec (the one that broke
// CI on PR #748).
// -----------------------------------------------------------------------------

fn find_repo_root() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cur: &std::path::Path = &manifest;
    loop {
        let candidate = cur.join("specs").join("fpga").join("bridge.t27");
        if candidate.is_file() {
            return Some(cur.to_path_buf());
        }
        cur = cur.parent()?;
    }
}

#[test]
fn r_ca_2_real_bridge_spec_no_comment_only_call_argument() {
    let Some(repo) = find_repo_root() else {
        panic!("Could not locate repo root containing specs/fpga/bridge.t27");
    };
    let bridge_path = repo.join("specs").join("fpga").join("bridge.t27");
    let bin = env!("CARGO_BIN_EXE_t27c");
    let out = Command::new(bin)
        .args(["gen-verilog", bridge_path.to_str().expect("path utf8")])
        .output()
        .expect("t27c gen-verilog on bridge.t27 should run");
    assert!(
        out.status.success(),
        "t27c gen-verilog failed on real bridge.t27 spec:\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let src = String::from_utf8_lossy(&out.stdout).into_owned();

    if let Some(viol) = has_comment_only_call_argument(&src) {
        panic!("R-CA-2 regression on real bridge.t27: {}", viol);
    }
}
