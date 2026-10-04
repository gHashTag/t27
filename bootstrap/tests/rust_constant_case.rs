//! Actual generation and strict Rust execution retain public value names.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-constant-case-{}-{}",
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/rust_constant_case.t27"),
    )
    .expect("read source fixture")
}
fn generate(scratch: &Scratch, source: &str) -> Vec<u8> {
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write source fixture");
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-rust")
        .arg(&spec)
        .output()
        .expect("run native generator");
    assert!(
        generated.status.success() && !generated.stdout.is_empty(),
        "generation failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    generated.stdout
}
fn compile(scratch: &Scratch, source: &[u8]) -> Output {
    let rust = scratch.0.join("fixture.rs");
    std::fs::write(&rust, source).expect("write actual Rust and independent driver");
    Command::new("rustc")
        .args(["--edition=2021", "-D", "warnings", "-A", "unused-parens"])
        .arg(&rust)
        .arg("-o")
        .arg(scratch.0.join("fixture"))
        .output()
        .expect("run rustc")
}
fn compile_and_run(source: &str, driver: &str) -> Output {
    let scratch = Scratch::new();
    let mut generated = generate(&scratch, source);
    generated.extend_from_slice(driver.as_bytes());
    let compiled = compile(&scratch, &generated);
    assert!(
        compiled.status.success(),
        "generated Rust must compile: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    Command::new(scratch.0.join("fixture"))
        .output()
        .expect("execute actual generated API")
}
const DRIVER: &str = r#"
fn main() {
    assert_eq!(type_documentation, 2);
    assert_eq!(mixedCount, 3);
    assert_eq!(UPPER_COUNT, 4);
    for value in (0u32..256).chain([65535, u32::MAX - 9]) {
        assert_eq!(document(value), value + 9);
        assert_eq!(record_state(value), value);
        assert_eq!(read_state(), value);
        let observed = unsafe { UPPER_STATE };
        assert_eq!(observed, value);
    }
}
"#;
#[test]
fn public_constant_names_values_and_mutable_state_compile_and_execute() {
    let output = compile_and_run(&source(), DRIVER);
    assert!(
        output.status.success(),
        "execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn a_type_correct_constant_value_mutation_fails_execution() {
    let original = source();
    let mutated = original.replacen(
        "type_documentation: u32 = 2;",
        "type_documentation: u32 = 3;",
        1,
    );
    assert_ne!(original, mutated, "mutation must reach the source");
    let output = compile_and_run(&mutated, DRIVER);
    assert!(
        !output.status.success() && String::from_utf8_lossy(&output.stderr).contains("assertion"),
        "changed exported value must fail a runtime assertion"
    );
}
#[test]
fn uppercase_source_control_compiles_and_executes() {
    let upper = source()
        .replace("type_documentation", "TYPE_DOCUMENTATION")
        .replace("mixedCount", "MIXED_COUNT");
    let driver = DRIVER
        .replace("type_documentation", "TYPE_DOCUMENTATION")
        .replace("mixedCount", "MIXED_COUNT");
    let output = compile_and_run(&upper, &driver);
    assert!(output.status.success(), "uppercase API control must pass");
}
#[test]
fn unrelated_unsigned_comparison_warning_remains_an_error() {
    let scratch = Scratch::new();
    let mut generated = generate(
        &scratch,
        r#"module ConstantWarningControl {
            const type_documentation: u32 = 2;
            fn always(value: u32) -> bool { return value >= 0 && type_documentation == 2; }
            test declared { assert(always(1), "control"); }
        }"#,
    );
    generated.extend_from_slice(b"fn main() { assert!(always(1)); }");
    let compiled = compile(&scratch, &generated);
    assert!(
        !compiled.status.success()
            && String::from_utf8_lossy(&compiled.stderr).contains("unused_comparisons"),
        "naming accommodation must not suppress arithmetic warnings: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
}

#[test]
fn lowercase_mutable_static_warning_remains_an_error() {
    let scratch = Scratch::new();
    let mut generated = generate(
        &scratch,
        r#"module MutableNamingControl {
            var lower_state: u32 = 0;
            fn record(value: u32) -> u32 { lower_state = value; return lower_state; }
            test declared { assert(record(1) == 1, "control"); }
        }"#,
    );
    assert!(
        !String::from_utf8_lossy(&generated).contains("#[allow(non_upper_case_globals)]"),
        "mutable statics retain their previous generated declaration"
    );
    generated.extend_from_slice(b"fn main() { assert_eq!(record(1), 1); }");
    let compiled = compile(&scratch, &generated);
    assert!(
        !compiled.status.success()
            && String::from_utf8_lossy(&compiled.stderr).contains("non_upper_case_globals"),
        "immutable naming accommodation must not change mutable-static lint policy: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
}
