//! Execute source-defined branch returns and nested checks in actual Icarus.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch {
    path: PathBuf,
    stem: String,
}
impl Scratch {
    fn new() -> Self {
        // Native icarus-simulate also keys its temp files by the source stem.
        let stem = format!(
            "branch_context_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(&stem);
        std::fs::create_dir_all(&path).expect("create isolated fixture directory");
        Self { path, stem }
    }
    fn spec(&self, source: &str) -> PathBuf {
        let spec = self.path.join(format!("{}.t27", self.stem));
        std::fs::write(&spec, source).expect("write actual source fixture");
        spec
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
fn source() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/verilog_branch_context.t27"),
    )
    .expect("read source fixture")
}
fn native(source: &str) -> Output {
    let scratch = Scratch::new();
    Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("icarus-simulate")
        .arg(scratch.spec(source))
        .output()
        .expect("native generation, Icarus compilation and execution")
}
fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
fn assert_pass(output: Output, tests: &[&str]) {
    let text = text(&output);
    assert!(
        output.status.success() && !text.contains("FAILED") && !text.contains("NOT CHECKED"),
        "native simulation must pass real checks: {text}"
    );
    for name in tests {
        assert!(
            text.contains(&format!("[TEST] {name} : PASSED")),
            "named check did not pass: {text}"
        );
    }
}
const DRIVER: &str = r#"
module independent;
    reg [31:0] value;
    wire [31:0] result;
    reg [31:0] expected;
    reg [63:0] wide_expected;
    integer index;
    VerilogBranchContext dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .value(value), .result(result), .ready());
    initial begin
        for (index = 0; index < 256; index = index + 1) begin
            value = index;
            expected = index == 0 ? 0 : index ^ 165;
            #1;
            if (result !== expected || dut.branch_value(value) !== expected)
                $fatal(1, "independent branch return mismatch");
            if (dut.local_branch(value) !== (index == 0 ? 9 : index ^ 12)) $fatal(1, "branch local return mismatch");
            expected = index < 64 ? index + 1 : index < 128 ? index ^ 1 : index - 1;
            if (dut.nested_value(value) !== expected) $fatal(1, "nested return mismatch");
            if (dut.narrow_value(value) !== (index == 0 ? 7 : index)) $fatal(1, "narrow return mismatch");
            if (dut.wide_value(value, 0) !== 64'd0) $fatal(1, "zero divisor branch mismatch");
            wide_expected = ({32'd0, value} << 33) / 3;
            if (dut.wide_value(value, 3) !== wide_expected) $fatal(1, "wide return mismatch");
            expected = index == 0 ? 17 : index ^ 3;
            if (dut.explicit_value(value) !== expected) $fatal(1, "explicit return mismatch");
            expected = index == 0 ? 4 : index ^ 1;
            if (dut.ordinary_if(value) !== expected) $fatal(1, "non-tail statement mismatch");
        end
        value = 32'hffff_ffff; #1;
        if (result !== (value ^ 165) || dut.narrow_value(value) !== 8'hff) $fatal(1, "high value mismatch");
        wide_expected = ({32'd0, value} << 33) / 3;
        if (dut.wide_value(value, 3) !== wide_expected) $fatal(1, "high wide return mismatch");
        $display("INDEPENDENT RTL PASS");
        $finish;
    end
endmodule
"#;
fn independent(source: &str) -> Output {
    let scratch = Scratch::new();
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-verilog")
        .arg(scratch.spec(source))
        .output()
        .expect("run native RTL generator");
    assert!(
        generated.status.success() && !generated.stdout.is_empty(),
        "native generation failed: {}",
        text(&generated)
    );
    let mut rtl = generated.stdout;
    rtl.extend_from_slice(DRIVER.as_bytes());
    let file = scratch.path.join("fixture.v");
    std::fs::write(&file, rtl).expect("write generated RTL and independent driver");
    let executable = scratch.path.join("fixture.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-s", "independent", "-o"])
        .arg(&executable)
        .arg(&file)
        .output()
        .expect("run actual Icarus compiler");
    assert!(
        compile.status.success(),
        "RTL must compile before executing: {}",
        text(&compile)
    );
    Command::new("vvp")
        .arg(&executable)
        .output()
        .expect("execute actual RTL")
}
#[test]
fn branch_values_execute_independent_input_results() {
    let output = independent(&source());
    assert!(
        output.status.success() && text(&output).contains("INDEPENDENT RTL PASS"),
        "independent RTL execution failed: {}",
        text(&output)
    );
}
#[test]
fn native_source_tests_and_bench_execute_nested_checks() {
    let output = native(&source());
    let log = text(&output);
    assert_pass(output, &["nested_checks", "loop_checks"]);
    assert!(
        log.contains("[BENCH] nested_checks"),
        "generated bench must execute: {log}"
    );
}
#[test]
fn direct_assertions_and_explicit_returns_keep_their_native_behavior() {
    let control = r#"module DirectReturnControl {
        fn value(input: u32) -> u32 { if input == 0 { return 17; } return input ^ 3; }
        test direct { assert(value(0) == 17, "guard"); assert(value(19) == 16, "fallthrough"); }
    }"#;
    assert_pass(native(control), &["direct"]);
}
#[test]
fn skipped_branches_and_assertions_in_loops_execute_correctly() {
    let control = r#"module NestedCheckControl {
        test nested {
            if false { assert(false, "must be skipped"); } else { assert(7 == 7, "else check"); }
            var index: u32 = 0;
            while index < 4 { if index < 4 { assert_eq(index, index); } index += 1; }
            assert(index == 4, "loop ran");
        }
    }"#;
    assert_pass(native(control), &["nested"]);
}
#[test]
fn a_source_return_mutation_compiles_then_fails_an_independent_assertion() {
    let original = source();
    let changed = original.replacen("value ^ 165", "value ^ 164", 1);
    assert_ne!(original, changed);
    let output = independent(&changed);
    assert!(
        !output.status.success()
            && text(&output).contains("FATAL:")
            && text(&output).contains("independent branch return mismatch"),
        "source mutant must compile then fail actual RTL: {}",
        text(&output)
    );
}
#[test]
fn a_false_nested_source_assertion_produces_only_a_failed_verdict() {
    let control = r#"module NestedFailureControl {
        test nested { if true { assert(7 == 8, "must fail at runtime"); } assert(true, "direct"); }
    }"#;
    let output = native(control);
    let log = text(&output);
    assert!(
        log.contains("[TEST] nested : FAILED")
            && !log.contains("[TEST] nested : PASSED")
            && !log.contains("iverilog rejected"),
        "nested false SOURCE assertion must compile and fail actual simulation: {log}"
    );
}
