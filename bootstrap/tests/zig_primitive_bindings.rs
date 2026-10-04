//! Compile and execute source-defined primitive value bindings in native Zig.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-zig-primitive-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("create isolated fixture directory");
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn source() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/zig_primitive_bindings.t27"),
    )
    .expect("read source fixture")
}
fn execute(source: &str, driver: &str) -> Output {
    let scratch = Scratch::new();
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write source fixture");
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen")
        .arg(&spec)
        .output()
        .expect("run native generator");
    assert!(
        generated.status.success() && !generated.stdout.is_empty(),
        "native generation failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let mut code = generated.stdout;
    code.extend_from_slice(driver.as_bytes());
    let zig = scratch.0.join("fixture.zig");
    std::fs::write(&zig, code).expect("write generated Zig and independent driver");
    Command::new("zig")
        .arg("test")
        .arg(&zig)
        .output()
        .expect("compile and execute actual Zig tests")
}
fn assert_pass(output: Output) {
    assert!(
        output.status.success(),
        "native Zig compile/execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
const DRIVER: &str = r#"
test "independent primitive binding results" {
    for (0..256) |raw| {
        const value: u32 = @intCast(raw);
        try std.testing.expectEqual(value, typed(value));
        try std.testing.expectEqual(value, untyped(value));
        try std.testing.expectEqual(value ^ 17, parameters(value, 17));
        try std.testing.expectEqual(value ^ 1, mutable_parameter(value));
        try std.testing.expectEqual(@as(u32, 7), ignored_parameter(value));
        try std.testing.expectEqual(@as(u32, 1), tuple_value(value));
        try std.testing.expectEqual(@as(u8, @intCast(raw)), narrow(@intCast(raw)));
        try std.testing.expectEqual(@as(f64, @floatFromInt(raw)), floating(value));
        try std.testing.expectEqual(@as(u8, @intCast(raw)), alias_value(@intCast(raw)));
    }
    try std.testing.expectEqual(std.math.maxInt(u32), typed(std.math.maxInt(u32)));
    try std.testing.expectEqual(std.math.maxInt(u32) ^ 1, mutable_parameter(std.math.maxInt(u32)));
    bench_primitive_values();
}
"#;
#[test]
fn declared_primitive_values_execute_independent_results() {
    assert_pass(execute(&source(), DRIVER));
}
#[test]
fn native_source_tests_and_bench_execute() {
    assert_pass(execute(
        &source(),
        "test \"execute generated bench\" { bench_primitive_values(); }",
    ));
}
#[test]
fn builtin_aliases_and_casts_remain_native_types() {
    let control = r#"module BuiltinPrimitiveControl {
        const BYTE = u8;
        fn narrow(value: u16) -> u8 { return value as u8; }
        fn floating(value: u32) -> f64 { return value as f64; }
        fn alias_value(value: u8) -> BYTE { return value; }
        test builtin { assert(narrow(511) == 255, "narrow"); assert(floating(19) == 19.0, "float"); assert(alias_value(19) == 19, "alias"); }
    }"#;
    assert_pass(execute(control, ""));
}
#[test]
fn primitive_value_scope_does_not_escape_an_item() {
    let control = r#"module PrimitiveScopeControl {
        fn value_binding(value: u32) -> u32 { let f16: u32 = value; return f16; }
        fn builtin_type() -> type { return f16; }
        test primitive { i0 = value_binding(7); assert(i0 == 7, "binding"); }
        fn builtin_integer_type() -> type { return i0; }
        bench primitive { f16 = value_binding(9); assert(f16 == 9, "bench"); }
        fn builtin_float_type() -> type { return f16; }
        fn nested_builtin_type(flag: bool) -> type {
            if flag { let f16: u32 = 7; assert(f16 == 7, "nested binding"); }
            return f16;
        }

    }"#;
    assert_pass(execute(control, "test \"fresh item scopes\" { try std.testing.expect(builtin_type() == f16); try std.testing.expect(builtin_integer_type() == i0); bench_primitive(); try std.testing.expect(builtin_float_type() == f16); try std.testing.expect(nested_builtin_type(true) == f16); try std.testing.expect(nested_builtin_type(false) == f16); }"));
}
#[test]
fn a_type_correct_source_value_mutation_fails_at_runtime() {
    let original = source();
    let changed = original.replacen("let f16: u32 = value;", "let f16: u32 = value ^ 1;", 1);
    assert_ne!(original, changed);
    let output = execute(&changed, DRIVER);
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success()
            && errors.contains("panic: assertion failed")
            && errors.contains("error: the following test command")
            && !errors.contains("error: name shadows")
            && !errors.contains("error: use of undeclared"),
        "source mutant must compile and fail a real assertion: {errors}"
    );
}
#[test]
fn a_source_bench_value_mutation_fails_when_called() {
    let original = source();
    let changed = original.replacen("i0 = typed(23);", "i0 = typed(24);", 1);
    assert_ne!(original, changed);
    let output = execute(&changed, DRIVER);
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success()
            && errors.contains("panic: assertion failed")
            && errors.contains("error: the following test command")
            && !errors.contains("error: name shadows"),
        "bench mutant must execute and fail: {errors}"
    );
}
