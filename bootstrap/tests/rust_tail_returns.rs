//! Compile and execute source-defined function tails in actual Rust.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-rust-tail-{}-{}",
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/rust_tail_returns.t27"),
    )
    .expect("read actual source fixture")
}
fn generate(scratch: &Scratch, source: &str, driver: &str) -> PathBuf {
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write source fixture");
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-rust")
        .arg(&spec)
        .output()
        .expect("run native Rust generator");
    assert!(
        generated.status.success() && !generated.stdout.is_empty(),
        "native generation failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let mut rust = generated.stdout;
    rust.extend_from_slice(driver.as_bytes());
    let file = scratch.0.join("fixture.rs");
    std::fs::write(&file, rust).expect("write untouched generated Rust and independent driver");
    file
}
fn compile(file: &Path, binary: &Path) -> Output {
    Command::new("rustc")
        .args(["--edition=2021", "-D", "warnings", "-A", "unused-parens"])
        .arg(file)
        .arg("-o")
        .arg(binary)
        .output()
        .expect("run actual Rust compiler")
}
fn execute(source: &str, driver: &str) -> Output {
    let scratch = Scratch::new();
    let file = generate(&scratch, source, driver);
    let binary = scratch.0.join("fixture");
    let compiled = compile(&file, &binary);
    assert!(
        compiled.status.success(),
        "generated Rust must compile before execution: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    Command::new(binary)
        .output()
        .expect("execute generated functions")
}
fn pass(output: Output) {
    assert!(
        output.status.success(),
        "native execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
const DRIVER: &str = r#"
fn main() {
    for value in (0u32..256).chain([u32::MAX]) {
        assert_eq!(top_tail(value), value ^ 165);
        assert_eq!(branch_tail(value), if value == 0 { 0 } else { value ^ 165 });
        assert_eq!(nested_tail(value), if value < 64 { value + 1 } else if value < 128 { value ^ 1 } else { value - 1 });
        assert_eq!(local_tail(value), if value == 0 { 9 } else { value ^ 12 });
        assert_eq!(narrow_tail(value), value as u8);
        assert_eq!(widen_tail(value as u8), u32::from(value as u8));
        assert_eq!(call_tail(value as u8), u32::from(value as u8));
        assert_eq!(wide_tail(value, 0), 0);
        assert_eq!(wide_tail(value, 3), (u64::from(value) << 33) / 3);
        assert_eq!(bool_tail(value), value < 128);
        assert_eq!(bool_to_int_tail(value), u32::from(value < 128));
        assert_eq!(global_tail(), 7);
        assert_eq!(explicit(value), if value == 0 { 17 } else { value ^ 3 });
        assert_eq!(mixed(value), if value == 0 { 17 } else { value ^ 3 });
        assert_eq!(ordinary_if(value), if value == 0 { 4 } else { value ^ 1 });
        assert_eq!(loop_tail(value), value);
        void_control(value & 7);
    }
}
"#;
#[test]
fn generated_function_and_nested_tails_execute_independent_results() {
    pass(execute(&source(), DRIVER));
}
#[test]
fn a_type_correct_source_tail_mutation_compiles_then_fails_at_runtime() {
    let original = source();
    let mutated = original.replacen("value ^ 165", "value ^ 164", 1);
    assert_ne!(original, mutated);
    let output = execute(&mutated, DRIVER);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr).contains("assertion `left == right` failed"),
        "SOURCE mutant must compile then fail executed assertion"
    );
}
#[test]
fn explicit_returns_non_tail_statements_and_void_controls_execute() {
    pass(execute(
        r#"module ExplicitControls {
        fn explicit(input: u32) -> u32 { if input == 0 { return 17; } return input ^ 3; }
        fn ordinary(input: u32) -> u32 { var result: u32 = 5; if input > 0 { result = input; } return result ^ 1; }
        fn unit(input: u32) -> void { var count: u32 = input; while count > 0 { count -= 1; } }
    }"#,
        r#"fn main() { for value in 0u32..256 { assert_eq!(explicit(value), if value == 0 { 17 } else { value ^ 3 }); assert_eq!(ordinary(value), if value == 0 { 4 } else { value ^ 1 }); unit(value); } }"#,
    ));
}
#[test]
fn branch_tail_coercions_and_float_values_execute() {
    pass(execute(
        r#"module TypedTails {
        fn widen(value: u8, choose: bool) -> u32 { if choose { value } else { value ^ 1 } }
        fn narrow(value: u32, choose: bool) -> u8 { if choose { value } else { value ^ 1 } }
        fn integer(value: u32, choose: bool) -> u32 { if choose { value < 128 } else { value > 128 } }
        fn half(value: f64, choose: bool) -> f64 { if choose { value / 2.0 } else { value + 0.5 } }
    }"#,
        r#"fn main() { for value in 0u32..256 { for choose in [false, true] { assert_eq!(widen(value as u8, choose), if choose { value } else { value ^ 1 }); assert_eq!(narrow(value, choose), if choose { value as u8 } else { (value ^ 1) as u8 }); assert_eq!(integer(value, choose), u32::from(if choose { value < 128 } else { value > 128 })); assert_eq!(half(f64::from(value), choose), if choose { f64::from(value) / 2.0 } else { f64::from(value) + 0.5 }); } } }"#,
    ));
}
#[test]
fn incompatible_tail_values_still_fail_actual_type_checking() {
    for value in ["[1, 2]", "\"string\""] {
        let scratch = Scratch::new();
        let file = generate(
            &scratch,
            &format!("module InvalidTail {{ fn bad() -> u32 {{ {value} }} }}"),
            "fn main() { let _ = bad(); }",
        );
        let compiled = compile(&file, &scratch.0.join("fixture"));
        assert!(
            !compiled.status.success()
                && String::from_utf8_lossy(&compiled.stderr).contains("E0308"),
            "incompatible values must remain actual Rust type errors"
        );
    }
}
#[test]
fn a_missing_else_does_not_fabricate_a_return_value() {
    let scratch = Scratch::new();
    let file = generate(
        &scratch,
        "module IncompleteTail { fn maybe(value: u32) -> u32 { if value > 0 { value } } }",
        "fn main() { let _ = maybe(0); }",
    );
    let compiled = compile(&file, &scratch.0.join("fixture"));
    assert!(
        !compiled.status.success() && String::from_utf8_lossy(&compiled.stderr).contains("E0308"),
        "uncovered return path must remain a type error"
    );
}
