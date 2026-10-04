//! Compile and execute source-defined array repeats in their C ABI contexts.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-c-repeat-wrapper-{}-{}",
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/c_array_repeat_wrapper.t27"),
    )
    .expect("read source fixture")
}
fn generate(scratch: &Scratch, source: &str) -> Vec<u8> {
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write source fixture");
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
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
fn compile(scratch: &Scratch, source: &[u8], native_tests: bool) -> Output {
    let c = scratch.0.join("fixture.c");
    std::fs::write(&c, source).expect("write actual generated C and independent driver");
    let mut command = Command::new("cc");
    command.args(["-std=gnu11", "-Werror"]);
    if native_tests {
        command.arg("-DT27_TEST_MAIN");
    }
    command.arg(&c).arg("-o").arg(scratch.0.join("fixture"));
    command.output().expect("run actual C compiler")
}
fn execute(source: &str, driver: &str, native_tests: bool) -> Output {
    let scratch = Scratch::new();
    let mut generated = generate(&scratch, source);
    generated.extend_from_slice(driver.as_bytes());
    let compiled = compile(&scratch, &generated, native_tests);
    assert!(
        compiled.status.success(),
        "generated C must compile: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    Command::new(scratch.0.join("fixture"))
        .output()
        .expect("execute actual generated C")
}
const DRIVER: &str = r#"
int main(void) {
    for (uint32_t value = 0; value < 256; ++value) {
        for (size_t index = 0; index < 4; ++index) {
            assert(wrapped_at(value, index) == value);
            assert(returned_at(value, index) == value);
            assert(plain_at(value, index) == value);
            assert(list_at(index) == 10 * (index + 1));
        }
    }
    for (size_t index = 0; index < 4; ++index) {
        assert(wrapped_at(UINT32_MAX, index) == UINT32_MAX);
        assert(returned_at(65535, index) == 65535);
    }
    return 0;
}
"#;
#[test]
fn wrapped_and_returned_repeats_execute_all_elements_independently() {
    let output = execute(&source(), DRIVER, false);
    assert!(
        output.status.success(),
        "execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn native_source_tests_execute() {
    let output = execute(&source(), "", true);
    assert!(
        output.status.success(),
        "native test execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn plain_arrays_and_list_literals_remain_valid_c() {
    let control = r#"module PlainRepeatControl {
        fn plain(index: usize) -> u32 { var values: [4]u32 = [7; 4]; return values[index]; }
        fn listed(index: usize) -> u32 { var values: [u32; 4] = [10, 20, 30, 40]; return values[index]; }
        test declared { assert(plain(3) == 7, "plain repeat"); assert(listed(3) == 40, "list"); }
    }"#;
    let output = execute(control, "int main(void) { for (size_t i = 0; i < 4; ++i) { assert(plain(i) == 7); assert(listed(i) == 10 * (i + 1)); } return 0; }", false);
    assert!(
        output.status.success(),
        "plain-array/list controls must execute"
    );
}
#[test]
fn a_source_repeat_extent_mutation_fails_on_the_last_element() {
    let original = source();
    let changed = original.replacen("[value; WIDTH]", "[value; WIDTH - 1]", 1);
    assert_ne!(original, changed, "extent mutation must reach the source");
    let output = execute(&changed, DRIVER, false);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .to_ascii_lowercase()
                .contains("assert"),
        "a missing final fill must compile then fail an independent assertion"
    );
}
#[test]
fn a_type_correct_source_fill_mutation_fails_at_runtime() {
    let original = source();
    let changed = original.replacen("[value; WIDTH]", "[value ^ 1; WIDTH]", 1);
    assert_ne!(original, changed, "source mutation must reach the repeat");
    let output = execute(&changed, DRIVER, false);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .to_ascii_lowercase()
                .contains("assert"),
        "changed fill must compile then fail a real assertion: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn an_out_of_bounds_repeat_designator_is_rejected_by_c() {
    let scratch = Scratch::new();
    let original = source();
    let changed = original.replacen("[value; WIDTH]", "[value; WIDTH + 1]", 1);
    assert_ne!(original, changed, "extent mutation must reach the source");
    let generated = generate(&scratch, &changed);
    let compiled = compile(&scratch, &generated, true);
    assert!(
        !compiled.status.success() && String::from_utf8_lossy(&compiled.stderr).contains("error:"),
        "actual C compiler must refuse an initializer exceeding the declared array"
    );
}
