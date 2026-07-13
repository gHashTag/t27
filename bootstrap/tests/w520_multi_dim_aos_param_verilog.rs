// =============================================================================
// Wave 520 -- multi-dimensional arrays of structs as function parameters.
//
// Previous lowering handled 1-D arrays of structs and scalar-struct parameters,
// but multi-dimensional arrays of structs failed at the module declaration,
// local initializer, parameter binding, or call-site packing stage. This
// integration test compiles a representative spec and asserts that the generated
// Verilog contains a packed-vector parameter for the 2-D array and a matching
// packed-vector argument at the call site, with no unsupported placeholders.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// Closes #1490
// =============================================================================

use std::io::Write;
use std::process::Command;

const SPEC: &str = r#"module W520MultiDimAOSProbe;

struct Pt {
    x : u16,
    y : u16,
}

fn sum_2d_pts(m : [2][3]Pt) -> u32 {
    var total : u32 = 0;
    for i in 0..2 {
        for j in 0..3 {
            total = total + (m[i][j].x as u32);
        }
    }
    return total;
}

test basic {
    var pts : [2][3]Pt = [2][3]Pt{
        Pt{.x=1,.y=0}, Pt{.x=2,.y=0}, Pt{.x=3,.y=0},
        Pt{.x=4,.y=0}, Pt{.x=5,.y=0}, Pt{.x=6,.y=0},
    };
    assert_eq(sum_2d_pts(pts), 21);
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

/// Extract the body of the named function from the generated Verilog.
fn function_body(src: &str, name: &str) -> Option<String> {
    let start_marker = format!("// function: {}", name);
    let start = src.find(&start_marker)? + start_marker.len();
    let rest = &src[start..];
    let end = rest
        .find("// function:")
        .or_else(|| rest.find("// -------------------------------------------------------"))
        .unwrap_or(rest.len());
    Some(src[start..start + end].to_string())
}

#[test]
fn w520_multi_dim_aos_param_has_packed_input() {
    let v = compile_spec(SPEC, "w520_multi_dim_aos_probe").expect("gen-verilog must succeed");

    // The callee must accept a packed vector whose width covers 6 structs
    // (2*3) of 32 bits each (2*u16).
    let body = function_body(&v,
        "sum_2d_pts")
        .expect("sum_2d_pts function must be present in generated Verilog");
    assert!(
        body.contains("input [191:0] m"),
        "expected packed 192-bit array parameter 'm', got:\n{}",
        body
    );

    // Field access on the parameter must slice the packed input, not reference
    // a non-existent module memory.
    assert!(
        body.contains("m["),
        "expected packed-vector slice for m[i][j].x"
    );
    assert!(
        !body.contains("pts_x"),
        "callee must not reference the caller's local per-field registers"
    );
}

#[test]
fn w520_multi_dim_aos_call_site_passes_packed_vector() {
    let v = compile_spec(SPEC, "w520_multi_dim_aos_probe_call").expect("gen-verilog must succeed");

    // The call site in the test block should pass a concatenation of the local
    // per-element per-field registers.
    assert!(
        v.contains("sum_2d_pts({"),
        "expected call site to pack the local array into a concatenation"
    );
}

#[test]
fn w520_multi_dim_aos_generates_icarus_acceptable_verilog() {
    let v = compile_spec(SPEC, "w520_multi_dim_aos_probe_icarus").expect("gen-verilog must succeed");
    assert!(
        !v.contains("UNSUPPORTED_ICARUS"),
        "generated Verilog contains UNSUPPORTED_ICARUS placeholder"
    );
    assert!(!v.contains("TODO:"), "generated Verilog contains TODO placeholder");
}
