//! Real code generation, compilation and execution for issue #5901.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-value-params-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("create fixture directory");
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/rust_value_parameters.t27"),
    )
    .expect("read source fixture")
}
fn compile_and_run(source: &str, driver: &str) -> Output {
    let scratch = Scratch::new();
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write source");
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-rust")
        .arg(&spec)
        .output()
        .expect("run compiler");
    assert!(
        generated.status.success(),
        "generation failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    assert!(!generated.stdout.is_empty(), "generation produced no Rust");
    let rust = scratch.0.join("fixture.rs");
    let mut contents = generated.stdout;
    contents.extend_from_slice(driver.as_bytes());
    std::fs::write(&rust, contents).expect("write actual generated Rust and driver");
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
        .expect("execute generated function")
}
const DRIVER: &str = r#"
fn main() {
    for seed in 0u32..256 {
        assert_eq!(increment(seed), seed + 1);
        assert_eq!(clamp_value(seed, 8, 64), seed.clamp(8, 64));
        assert_eq!(readonly(seed, [2, 3, 4, 5]), seed + 2);
        for counter in 0u32..16 {
            let original = [seed, 0, seed / 2, 255 - seed];
            assert_eq!(edit_array(original), original.iter().sum::<u32>() + original[3]);
            let mut expected = original;
            for (index, item) in expected.iter_mut().enumerate() {
                *item = (*item).max(counter + index as u32);
            }
            assert_eq!(nested_edits(original, counter), expected.iter().fold(0, |acc, item| acc ^ item));
            assert_eq!(original, [seed, 0, seed / 2, 255 - seed]);
        }
        let scalar = seed;
        let _changed = increment(scalar);
        assert_eq!(scalar, seed);
    }
    let values = [1, 2, 3, 4];
    assert_eq!(edit_array(values), 14);
    assert_eq!(values, [1, 2, 3, 4]);
    let mut slice = [1, 2];
    assert_eq!(write_slice(&mut slice, 7), 7);
    assert_eq!(slice, [7, 2]);
    assert_eq!(read_slice(vec![9, 10]), 9);
    let mut cell = Cell { value: 1 };
    assert_eq!(write_cell(&mut cell, 12), 12);
    assert_eq!(cell.value, 12);
}
"#;
#[test]
fn generated_value_and_reference_parameters_compile_and_execute() {
    let output = compile_and_run(&source(), DRIVER);
    assert!(
        output.status.success(),
        "execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn a_type_correct_source_mutation_is_rejected_by_execution() {
    let original = source();
    let mutated = original.replacen("value = value + 1;", "value = value + 2;", 1);
    assert_ne!(original, mutated, "mutation must reach the source");
    let output = compile_and_run(&mutated, DRIVER);
    assert!(
        !output.status.success(),
        "incorrect increment passed the independent driver"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("assertion"),
        "mutation must fail behavior"
    );
}

#[test]
fn local_shadows_do_not_change_outer_parameter_mutability() {
    let source = r#"module ParameterShadows {
        fn local_shadow(value: u32) -> u32 {
            let value: u32 = value + 1;
            value = value + 2;
            return value;
        }
        fn nested_shadow(value: u32) -> u32 {
            let result: u32 = value;
            if (value > 0) {
                let value: u32 = 7;
                value = value + 1;
                result = value;
            }
            return result;
        }
        fn before_shadow(value: u32) -> u32 {
            value = value + 1;
            let before: u32 = value;
            let value: u32 = 40;
            value = value + 2;
            return before + value;
        }
        fn after_branch(value: u32) -> u32 {
            let result: u32 = 0;
            if (value > 0) {
                let value: u32 = 99;
                value = value + 1;
                result = value;
            }
            value = value + 2;
            return value + result;
        }
        test expected { assert(local_shadow(5) == 8, "local shadow"); }
    }"#;
    let driver = r#"fn main() {
        for value in 0u32..256 {
            assert_eq!(local_shadow(value), value + 3);
            assert_eq!(nested_shadow(value), if value > 0 { 8 } else { 0 });
            assert_eq!(before_shadow(value), value + 43);
            assert_eq!(after_branch(value), value + 2 + if value > 0 { 100 } else { 0 });
        }
    }"#;
    let output = compile_and_run(source, driver);
    assert!(
        output.status.success(),
        "shadow execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn collection_loop_captures_do_not_rebind_function_parameters() {
    let scratch = Scratch::new();
    let spec = scratch.0.join("capture.t27");
    std::fs::write(
        &spec,
        r#"module CaptureShadow {
        fn sum(value: u32, values: [u32; 2]) -> u32 {
            let result: u32 = 0;
            for (values) |value| {
                value = value + 1;
                result = result + value;
            }
            return result + value;
        }
        test has_body { assert(sum(1, [2, 3]) == 8, "capture shadow"); }
    }"#,
    )
    .expect("capture source");
    let output = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-rust")
        .arg(spec)
        .output()
        .expect("run generator");
    assert!(output.status.success(), "generation must succeed");
    let generated = String::from_utf8(output.stdout).expect("UTF-8 Rust");
    // Check binding scope only: mutable loop-capture emission is a separate
    // existing limitation, so this fixture makes no executable-body claim.
    assert!(
        generated.contains("pub fn sum(value: u32, values: [u32; 2])"),
        "writes to the capture must not add unused mut to the outer parameter:\n{generated}"
    );
}
