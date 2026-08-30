// =============================================================================
// An enum reached through `use` must be DECLARED in the Verilog that
// references it.
//
// `gen_verilog_expr` lowers `Enum.variant` to the flat identifier
// `Enum_variant` no matter which spec declared the enum, but the declaration
// -- `localparam Enum_variant = ...;` -- was emitted only for enums the spec
// declares ITSELF. So `specs/fpga/mac.t27`, which does `use base::ops;` and
// returns `Trit.neg`, generated Verilog naming `Trit_neg` with nothing
// anywhere declaring it. That is not valid Verilog under any standard:
//
//     gen/mac.v:100: error: Unable to bind wire/reg/memory `Trit_neg'
//         in `ZeroDSP_MAC.extract_trit.extract_trit_body'
//
// Yosys did not object, because these modules hold no `always` block and the
// emitted functions are dead code it never elaborates -- so the `fpga-lint`
// gate read 32/32 green while every one of the 32 files was failing iverilog.
// The gate that looked is the one that was right.
//
// These tests assert the property directly rather than a byte pattern: every
// `Enum_variant`-shaped identifier the output USES is one the output
// DECLARES. They shell out to the built `t27c` so the path threading
// (`gen-verilog <file>` -> `compile_verilog_at`) is exercised too; a unit test
// on a source string cannot see that half of it.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// =============================================================================

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn gen_verilog(spec: &Path) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let out = Command::new(bin)
        .args(["gen-verilog", spec.to_str().expect("spec path is utf8")])
        .output()
        .expect("t27c gen-verilog should run");
    assert!(
        out.status.success(),
        "t27c gen-verilog failed on {}:\nstderr: {}",
        spec.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Every name declared by a `localparam <name> = ...;` line.
fn declared_localparams(src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in src.lines() {
        let Some(rest) = line.trim_start().strip_prefix("localparam ") else {
            continue;
        };
        // `localparam [7:0] NAME = 0;` -- skip an optional range first.
        let rest = match rest.trim_start().strip_prefix('[') {
            Some(after) => match after.find(']') {
                Some(i) => &after[i + 1..],
                None => continue,
            },
            None => rest,
        };
        let name: String = rest
            .trim_start()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            out.insert(name);
        }
    }
    out
}

/// Every identifier in the code (comments stripped) that starts with
/// `<enum>_`, i.e. every flattened enum-variant reference.
fn variant_refs(src: &str, enum_name: &str) -> HashSet<String> {
    let prefix = format!("{}_", enum_name);
    let mut out = HashSet::new();
    for line in src.lines() {
        let code = match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        };
        let mut cur = String::new();
        for c in code.chars().chain(std::iter::once(' ')) {
            if c.is_alphanumeric() || c == '_' {
                cur.push(c);
                continue;
            }
            if cur.starts_with(&prefix) && cur.len() > prefix.len() {
                out.insert(cur.clone());
            }
            cur.clear();
        }
    }
    out
}

fn assert_every_variant_declared(src: &str, enum_name: &str, where_: &str) {
    let declared = declared_localparams(src);
    let used = variant_refs(src, enum_name);
    assert!(
        !used.is_empty(),
        "{}: expected the generated Verilog to reference {}_<variant>; \
         if the spec stopped using the enum this test needs a new subject",
        where_,
        enum_name
    );
    let undeclared: Vec<&String> = used.iter().filter(|u| !declared.contains(*u)).collect();
    assert!(
        undeclared.is_empty(),
        "{}: {} identifier(s) referenced with no declaration: {:?}. \
         iverilog reports these as `Unable to bind wire/reg/memory`.",
        where_,
        undeclared.len(),
        undeclared
    );
}

fn repo_root() -> PathBuf {
    let mut cur: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if cur.join("specs").join("fpga").join("mac.t27").is_file() {
            return cur;
        }
        assert!(cur.pop(), "could not locate a repo root holding specs/fpga/mac.t27");
    }
}

// -----------------------------------------------------------------------------
// 1. The spec the FPGA gate compiles: `use base::ops;`, returns `Trit.neg`.
// -----------------------------------------------------------------------------

#[test]
fn mac_spec_declares_the_trit_enum_it_imports() {
    let spec = repo_root().join("specs").join("fpga").join("mac.t27");
    let src = gen_verilog(&spec);
    assert_every_variant_declared(&src, "Trit", "specs/fpga/mac.t27");

    // The values come from the imported declaration, not from the ordinal:
    // `base/ops.t27` declares `Trit = enum(i8) { neg = -1, zero = 0, pos = 1 }`.
    for expected in [
        "localparam Trit_neg = -1;",
        "localparam Trit_zero = 0;",
        "localparam Trit_pos = 1;",
    ] {
        assert!(
            src.contains(expected),
            "specs/fpga/mac.t27: missing `{}` -- an imported enum must carry \
             its declared values, exactly as a same-spec enum does",
            expected
        );
    }
}

// -----------------------------------------------------------------------------
// 2. Importing a module that declares an enum is NOT enough.
//
// `specs/fpga/uart.t27` does `use base::types;` and `use base::ops;` -- both
// declare `Trit` -- and never names it. Its generated Verilog must be exactly
// what it was before. Without this the pass would rewrite the output of every
// spec that merely imports a module with an enum in it, and the 492 specs that
// carry a `use` line would all move at once.
// -----------------------------------------------------------------------------

#[test]
fn an_unreferenced_imported_enum_emits_nothing() {
    let spec = repo_root().join("specs").join("fpga").join("uart.t27");
    let src = gen_verilog(&spec);
    assert!(
        variant_refs(&src, "Trit").is_empty(),
        "specs/fpga/uart.t27 is the control case: it must not reference Trit_*"
    );
    assert!(
        !src.contains("localparam Trit"),
        "specs/fpga/uart.t27: an imported enum the module never names must not \
         be emitted.\n--- generated Verilog ---\n{}",
        src
    );
}

// -----------------------------------------------------------------------------
// 3. The same property on a spec tree this test owns, so the guarantee does
//    not depend on what the FPGA specs happen to import next month.
// -----------------------------------------------------------------------------

const PALETTE: &str = "module tb-palette;

pub const Hue = enum(i8) {
    low = -1,
    mid = 0,
    high = 1,
};
";

const IMPORTER: &str = "module ImportedEnumProbe;

use tb::palette;

fn pick(selector: u32) -> u32 {
    if (selector == 0) {
        return Hue.low;
    } else if (selector == 1) {
        return Hue.high;
    } else {
        return Hue.mid;
    }
}
";

#[test]
fn a_synthetic_import_resolves_through_the_specs_root() {
    // Keyed by a COUNTER, not by a property of the input. The old key was
    // `(pid, src.len())`, and every test in this binary shares the pid -- so two
    // tests whose sources happen to be the same length computed the SAME
    // directory, which each of them deletes on the way out. Under the default
    // parallel runner one test erases the spec another is mid-read of, `t27c`
    // prints nothing, and the assertion reports an empty result.
    //
    // Measured with a probe asserting the directory is fresh: it fired 8 runs
    // out of 8. The collision is not occasional -- it happens every run, and
    // only the timing of the delete decides whether a test dies.
    static SCRATCH_N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let scratch_n = SCRATCH_N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!("t27_imported_enum_{}_{}", std::process::id(), scratch_n));
    let dir = base.join("specs").join("tb");
    std::fs::create_dir_all(&dir).expect("create temp spec tree");
    std::fs::write(dir.join("palette.t27"), PALETTE).expect("write palette.t27");
    let importer = dir.join("importer.t27");
    std::fs::write(&importer, IMPORTER).expect("write importer.t27");

    let src = gen_verilog(&importer);
    let _ = std::fs::remove_dir_all(&base);

    assert_every_variant_declared(&src, "Hue", "synthetic tb::palette importer");
    assert!(
        src.contains("localparam Hue_low = -1;"),
        "synthetic importer: expected `localparam Hue_low = -1;`\n\
         --- generated Verilog ---\n{}",
        src
    );
}
