//! Test for global array repeat generation issue #6050
//!
//! This test verifies that global arrays initialized with repeat operations
//! generate the correct C code that initializes the array with the repeated values,
//! not just zeros.

use t27c_bootstrap::test_support::{assert_c_output, TestSpec};

#[test]
fn test_global_array_repeat_literal() {
    // Test literal repeat: [_]u8{5} ** 4
    let spec = TestSpec {
        name: "global_array_repeat_literal",
        code: r#"
module m;
const N: i64 = 4;
var a: [4]u8 = [_]u8{5} ** 4;
"#,
        expected_c: Some(r#"
static uint8_t a[4] = /* repeat: { 5 } ** 4 */ { [0 ... 3] = 5 };
"#),
    };
    
    assert_c_output(&spec);
}

#[test]
fn test_global_array_repeat_const() {
    // Test const repeat: [_]u8{7} ** N
    let spec = TestSpec {
        name: "global_array_repeat_const",
        code: r#"
module m;
const N: i64 = 4;
var r: [N]u8 = [_]u8{7} ** N;
"#,
        expected_c: Some(r#"
static uint8_t r[N] = /* repeat: { 7 } ** N */ { [0 ... 3] = 7 };
"#),
    };
    
    assert_c_output(&spec);
}

#[test]
fn test_global_array_repeat_both() {
    // Test both forms in one module
    let spec = TestSpec {
        name: "global_array_repeat_both",
        code: r#"
module m;
const N: i64 = 4;
var r: [N]u8 = [_]u8{7} ** N;
var a: [4]u8 = [_]u8{5} ** 4;
"#,
        expected_c: Some(r#"
static uint8_t r[N] = /* repeat: { 7 } ** N */ { [0 ... 3] = 7 };
static uint8_t a[4] = /* repeat: { 5 } ** 4 */ { [0 ... 3] = 5 };
"#),
    };
    
    assert_c_output(&spec);
}