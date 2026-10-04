//! Compile and execute actual Rust generated from integral repetition counts.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-repeat-counts-{}-{}",
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/rust_repeat_counts.t27"),
    )
    .expect("read source fixture")
}
fn compile_and_run(source: &str, driver: &str) -> Output {
    let scratch = Scratch::new();
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write source fixture");
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-rust")
        .arg(&spec)
        .output()
        .expect("run native generator");
    assert!(
        generated.status.success(),
        "generation failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    assert!(!generated.stdout.is_empty(), "generator must emit Rust");
    let rust = scratch.0.join("fixture.rs");
    let mut contents = generated.stdout;
    contents.extend_from_slice(driver.as_bytes());
    std::fs::write(&rust, contents).expect("write generated Rust and independent driver");
    let binary = scratch.0.join("fixture");
    let compiled = Command::new("rustc")
        .args(["--edition=2021", "-D", "warnings", "-A", "unused-parens"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("run rustc");
    assert!(
        compiled.status.success(),
        "generated Rust must compile: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    Command::new(&binary)
        .output()
        .expect("execute generated functions")
}
const DRIVER: &str = r#"
fn main() {
    for value in (0u32..256).chain([65535, 65536, 1_000_000, u32::MAX]) {
        assert_eq!(repeat8(value), [value; 4]);
        assert_eq!(repeat16(value), [value; 4]);
        assert_eq!(repeat32(value), [value; 4]);
        assert_eq!(repeat64(value), [value; 4]);
        assert_eq!(grouped(value), [value; 7]);
        assert_eq!(literal(value), [value; 4]);
        assert_eq!(list(value), [value, 2, 3, 4]);
    }
    assert_eq!(empty(), []);
}
"#;
#[test]
fn generated_named_grouped_and_zero_counts_compile_and_execute() {
    let output = compile_and_run(&source(), DRIVER);
    assert!(
        output.status.success(),
        "execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn a_type_correct_repeated_value_mutation_fails_execution() {
    let original = source();
    let mutated = original.replacen("[value; COUNT32]", "[value ^ 1; COUNT32]", 1);
    assert_ne!(mutated, original, "mutation must reach the source");
    let output = compile_and_run(&mutated, DRIVER);
    assert!(
        !output.status.success() && String::from_utf8_lossy(&output.stderr).contains("assertion"),
        "incorrect repeated value must fail a runtime assertion"
    );
}
#[test]
fn literal_repeat_and_list_controls_compile_and_execute() {
    let output = compile_and_run(
        r#"module LiteralRepeatControl {
            fn literal(value: u32) -> [u32; 4] { return [value; 4]; }
            fn list(value: u32) -> [u32; 4] { return [value, 2, 3, 4]; }
            test retained_list { let a: [u32; 4] = list(9); assert(a[3] == 4, "list"); }
        }"#,
        r#"fn main() {
            for value in 0u32..256 {
                assert_eq!(literal(value), [value; 4]);
                assert_eq!(list(value), [value, 2, 3, 4]);
            }
        }"#,
    );
    assert!(output.status.success(), "literal/list control must pass");
}

#[test]
fn invalid_bool_and_float_counts_still_fail_actual_rust_compilation() {
    let scratch = Scratch::new();
    for (index, (declaration, count)) in [
        ("const COUNT: bool = true;", "COUNT"),
        ("const COUNT: f64 = 1.5;", "COUNT"),
        ("", "true"),
        ("", "1.5"),
        ("const COUNT: u32 = 4;", "COUNT < 2"),
    ]
    .into_iter()
    .enumerate()
    {
        let spec = scratch.0.join(format!("invalid-{index}.t27"));
        std::fs::write(
            &spec,
            format!(
                "module InvalidCount {{ {declaration} fn make() -> [u32; 1] {{ return [0; {count}]; }} test declaration {{ assert(1 == 1, \"control\"); }} }}"
            ),
        )
        .expect("write invalid count fixture");
        let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
            .arg("gen-rust")
            .arg(&spec)
            .output()
            .expect("generate invalid count control");
        assert!(
            generated.status.success(),
            "existing generator accepts syntax"
        );
        let rust = scratch.0.join(format!("invalid-{index}.rs"));
        std::fs::write(&rust, generated.stdout).expect("write actual output");
        let compiled = Command::new("rustc")
            .args([
                "--edition=2021",
                "-D",
                "warnings",
                "-A",
                "unused-parens",
                "--crate-type",
                "lib",
                "--crate-name",
                "invalid_count",
            ])
            .arg(&rust)
            .arg("-o")
            .arg(scratch.0.join(format!("invalid-{index}.rlib")))
            .output()
            .expect("compile invalid count control");
        assert!(
            !compiled.status.success()
                && String::from_utf8_lossy(&compiled.stderr).contains("E0308"),
            "non-integral repetition must remain a type error: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
    }
}

#[test]
fn existing_usize_counts_compile_and_execute() {
    let output = compile_and_run(
        r#"module UsizeRepeatControl {
            const COUNT: usize = 4;
            fn repeated(value: u32) -> [u32; 4] { return [value; COUNT]; }
            test retained_count { let a: [u32; 4] = repeated(9); assert(a[3] == 9, "count"); }
        }"#,
        r#"fn main() {
            for value in 0u32..256 { assert_eq!(repeated(value), [value; 4]); }
        }"#,
    );
    assert!(output.status.success(), "existing usize count must pass");
}
