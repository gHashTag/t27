// ============================================================================
// #5904: struct and uN/iN entry ports are sized by one rule.
//
// An `on_comb` parameter becomes a module port only when its width can be
// derived exactly: a primitive, `uN`/`iN` for 1 <= N <= 128, a sized array
// of those, or a struct whose fields are all of those. The port must be as
// wide as the `input` the generated function declares for the same
// parameter (FR-001) -- otherwise the `assign result = on_comb(...)` line
// silently truncates or zero-extends. Anything without a derivable width (a
// slice, a float, a nested struct) is still refused, and the refusal is now
// also reported on stderr, not only in a comment inside the output file.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_5904_{}_{}", std::process::id(), label));
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn gen_verilog(label: &str, source: &str) -> Output {
    let dir = scratch_dir(label);
    let spec = dir.join(format!("{label}.t27"));
    fs::write(&spec, source).expect("write spec");
    let out = Command::new(t27c())
        .arg("gen-verilog")
        .arg(&spec)
        .output()
        .expect("run t27c gen-verilog");
    let _ = fs::remove_dir_all(&dir);
    out
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Width of `[H:0]` in `decl`, or 1 when the declaration carries no range.
fn range_width(decl: &str) -> u32 {
    match (decl.find('['), decl.find(":0]")) {
        (Some(open), Some(close)) if open < close => {
            decl[open + 1..close].trim().parse::<u32>().expect("numeric msb") + 1
        }
        _ => 1,
    }
}

/// Width of the module port named `name` (`input  wire [H:0] name,`).
fn port_width(verilog: &str, name: &str) -> Option<u32> {
    verilog
        .lines()
        .map(str::trim)
        .find(|l| {
            (l.starts_with("input  wire") || l.starts_with("output wire"))
                && l.trim_end_matches(',').split_whitespace().last() == Some(name)
        })
        .map(range_width)
}

/// Width of the `input` the function `func` declares for `name`.
fn function_input_width(verilog: &str, func: &str, name: &str) -> Option<u32> {
    let mut inside = false;
    for line in verilog.lines().map(str::trim) {
        if line.starts_with("function") {
            inside = line
                .split(';')
                .next()
                .and_then(|head| head.split_whitespace().last())
                == Some(func);
            continue;
        }
        if line.starts_with("endfunction") {
            inside = false;
            continue;
        }
        if inside
            && line.starts_with("input")
            && line.trim_end_matches(';').split_whitespace().last() == Some(name)
        {
            return Some(range_width(line));
        }
    }
    None
}

#[test]
fn a_one_field_struct_parameter_is_an_eight_bit_port() {
    let out = gen_verilog(
        "flag_port",
        "module flag_port;\n\npub const W = struct {\n    code : u8,\n};\n\n\
         fn on_comb(w: W) -> u8 {\n    return w.code;\n}\n",
    );
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    assert!(!v.contains("NO DATA PORTS"), "struct param was refused:\n{v}");
    assert!(
        v.lines().any(|l| l.trim() == "input  wire [7:0] w,"),
        "no `input wire [7:0] w` port:\n{v}"
    );
    assert_eq!(port_width(&v, "w"), function_input_width(&v, "on_comb", "w"));
    assert!(
        !stderr_of(&out).contains("ENTRY POINT REFUSED"),
        "an accepted entry point must not report a refusal"
    );
}

#[test]
fn a_u1_parameter_and_return_are_one_bit_ports() {
    let out = gen_verilog(
        "bit_port",
        "module bit_port;\n\nfn on_comb(x: u1) -> u1 {\n    return x;\n}\n",
    );
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    assert!(!v.contains("NO DATA PORTS"), "u1 param was refused:\n{v}");
    assert_eq!(port_width(&v, "x"), Some(1), "x is not 1 bit:\n{v}");
    assert_eq!(port_width(&v, "result"), Some(1), "result is not 1 bit:\n{v}");
    assert_eq!(function_input_width(&v, "on_comb", "x"), Some(1));
    assert!(
        !v.contains("function [31:0] on_comb"),
        "a u1 function must not be declared 32 bits wide:\n{v}"
    );
}

#[test]
fn a_parameterless_u1_on_comb_is_called_with_the_placeholder_argument() {
    // led_off_test.t27's shape. Refused while `-> u1` had no width; accepted
    // now, so its `assign result = on_comb(...)` line is reachable, and it must
    // pass the `_unused` input W530 declares rather than nothing.
    let source = "module led_on;\n\nfn on_comb() -> u1 {\n    return 1;\n}\n";
    let out = gen_verilog("led_on", source);
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    assert_eq!(port_width(&v, "result"), Some(1), "result is not 1 bit:\n{v}");
    assert!(
        v.contains("assign result = on_comb(1'b0);"),
        "zero-parameter on_comb not called with the W530 placeholder:\n{v}"
    );
    assert!(!v.contains("on_comb()"), "`on_comb()` does not elaborate:\n{v}");

    if Command::new("iverilog").arg("-V").output().map(|o| o.status.success()).unwrap_or(false) {
        let dir = scratch_dir("led_on_iverilog");
        let vp = dir.join("led_on.v");
        fs::write(&vp, &v).expect("write verilog");
        let iv = Command::new("iverilog")
            .args(["-g2012", "-o", "/dev/null"])
            .arg(&vp)
            .output()
            .expect("run iverilog");
        let _ = fs::remove_dir_all(&dir);
        assert!(
            iv.status.success(),
            "iverilog rejected the module:\n{}",
            String::from_utf8_lossy(&iv.stderr)
        );
    }
}

#[test]
fn a_slice_parameter_is_still_refused_and_says_so_on_stderr() {
    let out = gen_verilog(
        "slice_entry",
        "module slice_entry;\n\nfn on_comb(xs: []u8) -> u8 {\n    return 1;\n}\n",
    );
    // The exit status is unchanged by #5904: a refused entry point still
    // produces a (port-less) module, as before.
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    assert!(v.contains("NO DATA PORTS"), "slice param got a port:\n{v}");
    assert!(
        v.contains("// ENTRY POINT REFUSED"),
        "the in-file refusal comment must be kept:\n{v}"
    );
    assert_eq!(port_width(&v, "xs"), None, "slice param became a port:\n{v}");
    let err = stderr_of(&out);
    assert_eq!(
        err.matches("t27c gen-verilog: ENTRY POINT REFUSED -- xs: []u8").count(),
        1,
        "stderr must name the refused parameter exactly once:\n{err}"
    );
}

#[test]
fn a_two_field_struct_port_is_as_wide_as_the_function_input() {
    let out = gen_verilog(
        "pair_port",
        "module pair_port;\n\npub const Pair = struct {\n    lo : u8,\n    hi : u16,\n};\n\n\
         fn on_comb(p: Pair) -> u16 {\n    return p.hi;\n}\n",
    );
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    assert!(!v.contains("NO DATA PORTS"), "struct param was refused:\n{v}");
    // u8 + u16 = 24 bits, on both sides of `assign result = on_comb(p);`.
    assert_eq!(port_width(&v, "p"), Some(24), "port p is not 24 bits:\n{v}");
    assert_eq!(
        function_input_width(&v, "on_comb", "p"),
        Some(24),
        "on_comb's input p is not 24 bits:\n{v}"
    );
    assert_eq!(port_width(&v, "result"), Some(16));
}

#[test]
fn a_struct_holding_a_struct_is_still_refused() {
    let out = gen_verilog(
        "nested_port",
        "module nested_port;\n\npub const Inner = struct {\n    a : u8,\n};\n\n\
         pub const Outer = struct {\n    i : Inner,\n};\n\n\
         fn on_comb(o: Outer) -> u8 {\n    return 1;\n}\n",
    );
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    assert!(v.contains("NO DATA PORTS"), "nested struct param got a port:\n{v}");
    assert!(
        stderr_of(&out).contains("t27c gen-verilog: ENTRY POINT REFUSED -- o: Outer"),
        "stderr does not name the nested struct parameter: {}",
        stderr_of(&out)
    );
}

// ----------------------------------------------------------------------------
// #5963: the in-file refusal comment for `on_clock`.
//
// The comment used to be written only inside the `NO DATA PORTS` branch. An
// `on_clock` module exposes its mutable `var`s as output ports, so a refused
// `on_clock` parameter took the other branch and the `.v` file said nothing --
// only stderr did. The comment is now written whenever an entry point is
// refused, by the same helper `on_comb` uses.
// ----------------------------------------------------------------------------

const SLICE_CLOCK: &str = "module slice_clock;\n\nvar acc : u8 = 0\n\n\
     fn on_clock(xs: []u8) {\n    acc = acc + 1;\n}\n";

#[test]
fn a_refused_on_clock_parameter_is_named_in_the_file_too() {
    let out = gen_verilog("slice_clock", SLICE_CLOCK);
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    // The module still has a data port, so this is NOT the port-less case.
    assert!(!v.contains("NO DATA PORTS"), "acc should still be an output:\n{v}");
    assert_eq!(port_width(&v, "xs"), None, "slice param became a port:\n{v}");
    assert!(
        v.lines().any(|l| l.trim() == "output reg [7:0] acc"),
        "the `var acc` output port is gone:\n{v}"
    );
    assert_eq!(
        v.matches("// ENTRY POINT REFUSED -- a parameter or return has no derivable width:")
            .count(),
        1,
        "the refused on_clock entry point is not named exactly once in the file:\n{v}"
    );
    assert!(v.lines().any(|l| l == "//     xs: []u8"), "the comment does not name xs:\n{v}");
    assert_eq!(
        stderr_of(&out)
            .matches("t27c gen-verilog: ENTRY POINT REFUSED -- xs: []u8")
            .count(),
        1,
        "stderr must still name the refused parameter exactly once"
    );
}

#[test]
fn the_on_clock_and_on_comb_refusal_comments_are_the_same_text() {
    let clock = stdout_of(&gen_verilog("slice_clock_same", SLICE_CLOCK));
    let comb = stdout_of(&gen_verilog(
        "slice_comb_same",
        "module slice_comb;\n\nfn on_comb(xs: []u8) -> u8 {\n    return 1;\n}\n",
    ));
    let block = |v: &str| -> Vec<String> {
        let lines: Vec<&str> = v.lines().collect();
        let i = lines
            .iter()
            .position(|l| l.starts_with("// ENTRY POINT REFUSED"))
            .unwrap_or_else(|| panic!("no refusal comment in:\n{v}"));
        lines[i..i + 4].iter().map(|l| l.to_string()).collect()
    };
    assert_eq!(block(&clock), block(&comb));
    // And the port-less `on_comb` case still writes it exactly once.
    assert_eq!(comb.matches("// ENTRY POINT REFUSED").count(), 1, "{comb}");
}

#[test]
fn an_accepted_on_clock_writes_no_refusal_comment() {
    // Negative control: same shape, sized parameter.
    let out = gen_verilog(
        "sized_clock",
        "module sized_clock;\n\nvar acc : u8 = 0\n\n\
         fn on_clock(x: u8) {\n    acc = acc + x;\n}\n",
    );
    assert!(out.status.success(), "gen-verilog failed: {}", stderr_of(&out));
    let v = stdout_of(&out);
    assert_eq!(port_width(&v, "x"), Some(8), "x is not an 8-bit port:\n{v}");
    assert!(!v.contains("ENTRY POINT REFUSED"), "accepted entry point refused:\n{v}");
    assert!(!stderr_of(&out).contains("ENTRY POINT REFUSED"));
}
