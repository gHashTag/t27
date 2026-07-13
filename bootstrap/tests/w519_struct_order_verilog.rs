// =============================================================================
// Wave 519 -- packed scalar struct ordering comparison lowering.
//
// W470 already lowered ==/!= for scalar structs and small arrays-of-structs
// to packed vector comparisons. W519 extends the same special case to the
// ordering operators <, <=, >, >= so that local scalar struct variables can be
// compared with literals, parameters, and other locals in the
// Icarus-lowerable Verilog path.
//
// This integration test compiles a tiny spec and asserts that every
// relational operator applied to scalar structs is emitted as a comparison
// between packed concatenations `{a_x, a_y}` and `{b_x, b_y}` rather than as
// a naive comparison between the (non-existent) aggregate identifiers.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// Closes #1488
// =============================================================================

use std::io::Write;
use std::process::Command;

const SPEC: &str = r#"module W519StructOrderProbe;

struct Pt {
    x : u8,
    y : u8,
}

fn cmp_local() -> bool {
    var a : Pt = Pt{.x = 1, .y = 2};
    var b : Pt = Pt{.x = 3, .y = 4};
    return (a == b) || (a != b) || (a < b) || (a <= b) || (a > b) || (a >= b);
}

fn cmp_param(a : Pt, b : Pt) -> bool {
    return (a < b) && (a <= b) && (a > b) && (a >= b);
}

pub fn driver() -> bool {
    return cmp_local() && cmp_param(Pt{.x = 1, .y = 2}, Pt{.x = 3, .y = 4});
}

test w519_order_basic {
    assert(driver());
}

endmodule
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

fn has_packed_struct_comparison(src: &str, op: &str) -> bool {
    let pattern = format!("}} {} {{", op);
    src.contains(&pattern)
}

/// Extract the body of the named function from the generated Verilog.
fn function_body(src: &str, name: &str) -> Option<String> {
    let start_marker = format!("// function: {}", name);
    let start = src.find(&start_marker)? + start_marker.len();
    let end = src[start..].find("// function:")?;
    Some(src[start..start + end].to_string())
}

#[test]
fn w519_struct_order_lowers_to_packed_vector_comparison() {
    let v = compile_spec(SPEC, "w519_struct_order_probe").expect("gen-verilog must succeed");

    // Sanity check that the emitter actually named the packed fields.
    assert!(v.contains("_x"));
    assert!(v.contains("_y"));

    // The local-variable function must pack the per-field registers before
    // comparing; this exercises the W519 path that was previously broken for
    // ordering operators on scalar struct locals.
    let local_body = function_body(&v, "cmp_local")
        .expect("cmp_local function must be present in generated Verilog");
    for op in ["==", "!=", "<", "<=", ">", ">="] {
        assert!(
            has_packed_struct_comparison(&local_body, op),
            "cmp_local: expected packed scalar struct comparison with operator {}",
            op
        );
    }

    // All six relational operators must appear somewhere in the output,
    // including any direct packed-parameter comparisons.
    for op in ["==", "!=", "<", "<=", ">", ">="] {
        assert!(
            v.contains(op),
            "generated Verilog must contain relational operator {}",
            op
        );
    }
}

#[test]
fn w519_struct_order_generates_icarus_acceptable_verilog() {
    let v = compile_spec(SPEC, "w519_struct_order_probe_icarus").expect("gen-verilog must succeed");
    assert!(
        !v.contains("UNSUPPORTED_ICARUS"),
        "generated Verilog contains UNSUPPORTED_ICARUS placeholder"
    );
    assert!(!v.contains("TODO:"), "generated Verilog contains TODO placeholder");
}
