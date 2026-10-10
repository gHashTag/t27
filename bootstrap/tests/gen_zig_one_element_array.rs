//! Test for one-element array generation in Zig backend
//!
//! This test reproduces the issue where `t27c test-report` fails on specs with
//! one-element array constants like `[1]str = ["file"]` and `[1]u32 = [48]`.
//! The generated Zig code incorrectly produces `[48]` instead of `.{48}` which
//! causes a compilation error "expected type expression, found ';'.

mod common;

use std::process::Command;

fn gen_zig_test_report(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-zig-test-{}-{}",
        std::process::id(),
        tag
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("test.t27");
    std::fs::write(&p, spec).expect("write");
    
    // Run t27c test-report
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("test-report")
        .arg(&p)
        .output()
        .expect("t27c test-report");
    
    let output = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    
    // Check if the test succeeded
    if !out.status.success() {
        eprintln!("test-report failed: {}", stderr);
        eprintln!("stdout: {}", output);
        panic!("test-report failed for spec: {}", spec);
    }
    
    (output, d)
}

#[test]
fn one_element_array_indexes_should_not_be_blocked() {
    // Test case 1: [1]u32 array with index access
    let spec1 = r#"
module test_one_element_u32;

pub const TOKENS: [1]u32 = [48];

test test_index_access {
    assert TOKENS[0] == 48;
}
"#;
    
    let (output, _d) = gen_zig_test_report(spec1, "u32_array");
    
    // The test should not be BLOCKED due to compilation errors
    assert!(!output.contains("BLOCKED"), 
        "test should not be BLOCKED due to compilation errors:\n{}", output);
    assert!(output.contains("PASS"), 
        "test should pass, not be blocked:\n{}", output);
}

#[test]
fn one_element_string_array_should_work() {
    // Test case 2: [1]str array with index access  
    let spec2 = r#"
module test_one_element_str;

pub const FILES: [1]str = ["specs/queen/lotus.t27"];

test test_string_index {
    assert FILES[0] == "specs/queen/lotus.t27";
}
"#;
    
    let (output, _d) = gen_zig_test_report(spec2, "str_array");
    
    // The test should not be BLOCKED due to compilation errors
    assert!(!output.contains("BLOCKED"), 
        "test should not be BLOCKED due to compilation errors:\n{}", output);
    assert!(output.contains("PASS"), 
        "test should pass, not be blocked:\n{}", output);
}

#[test]
fn test_report_command_runs() {
    // Verify that t27c test-report command can run and produce output
    let simple_spec = r#"
module simple_test;

test basic_test {
    assert true;
}
"#;
    
    let (output, _d) = gen_zig_test_report(simple_spec, "simple");
    
    // Should contain test report output
    assert!(output.contains("test report"), 
        "output should contain test report header:\n{}", output);
    assert!(output.contains("basic_test"), 
        "output should contain test name:\n{}", output);
}