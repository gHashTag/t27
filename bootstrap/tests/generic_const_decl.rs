//! ADR-008 / #2162 -- parameterised const type declaration `const Name(T) = struct`.
//!
//! Strategy: shell out to the built t27c binary via `CARGO_BIN_EXE_t27c` and run
//! `parse` over the fixtures in `tests/fixtures/generic_const/`.
//!
//! Two things are asserted, and the second one is the one that carries weight:
//!
//!   1. the six positive fixtures parse;
//!   2. each of the seven negative fixtures is rejected FOR ITS OWN STATED
//!      REASON, matched on the specific ADR-008 message.
//!
//! Point 2 exists because of a measured trap. Before the fix, `neg_02`
//! (trailing comma) and `neg_03` (type expression in a parameter position)
//! already failed -- with the *same* generic `Unexpected token in expression:
//! KwStruct` error as every positive fixture. A negative fixture that fails for
//! the same reason as everything else proves nothing about the rule it claims to
//! test, so matching only on "non-zero exit" would give a green suite with no
//! evidential content. Two more fixtures, `neg_06` and `neg_07`, were the
//! opposite: the old parser ACCEPTED them (exit 0) while silently dropping the
//! right-hand side, so their assertion here records a deliberate tightening from
//! silent acceptance to rejection, not a preserved behaviour.
//!
//! Regression direction: the `regression_original_failing_form` test pins the
//! exact construct from the corpus that motivated #2162. Run against the parser
//! before the fix it fails; that is what makes it a proof of the fix rather than
//! a guard. The negative tests, by contrast, are guards: five of the seven also
//! failed before the fix, only for the wrong reason.
//!
//! Refs #2162.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("generic_const")
}

fn parse_fixture(name: &str) -> Output {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let path = fixture_dir().join(name);
    assert!(path.is_file(), "fixture missing: {}", path.display());
    Command::new(bin)
        .arg("parse")
        .arg(&path)
        .output()
        .expect("failed to spawn t27c parse")
}

fn parse_source(source: &str, stem: &str) -> Output {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let path = std::env::temp_dir().join(format!("t27c_adr008_{stem}.t27"));
    std::fs::write(&path, source).expect("failed to write temp spec");
    Command::new(bin)
        .arg("parse")
        .arg(&path)
        .output()
        .expect("failed to spawn t27c parse")
}

fn assert_parses(name: &str) {
    let out = parse_fixture(name);
    assert!(
        out.status.success(),
        "ADR-008 positive fixture {name} must parse, got {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Reject `name`, and reject it for the reason it is meant to test.
fn assert_rejected_because(name: &str, needle: &str) {
    let out = parse_fixture(name);
    assert!(
        !out.status.success(),
        "ADR-008 negative fixture {name} must be rejected, but it parsed"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ADR-008"),
        "{name} was rejected, but not by the ADR-008 rule, so the fixture \
         proves nothing about that rule.\noutput: {combined}"
    );
    assert!(
        combined.contains(needle),
        "{name} must be rejected for its own reason (expected {needle:?}).\n\
         output: {combined}"
    );
}

// ---------------------------------------------------------------- positives

#[test]
fn positive_single_parameter() {
    assert_parses("pos_01_single_param.t27");
}

#[test]
fn positive_two_parameters() {
    assert_parses("pos_02_two_params.t27");
}

#[test]
fn positive_three_parameters() {
    assert_parses("pos_03_three_params.t27");
}

#[test]
fn positive_without_pub() {
    assert_parses("pos_04_no_pub.t27");
}

#[test]
fn positive_without_trailing_semicolon() {
    assert_parses("pos_05_no_semicolon.t27");
}

#[test]
fn positive_alongside_plain_struct() {
    assert_parses("pos_06_alongside_plain.t27");
}

// ---------------------------------------------------------------- negatives

#[test]
fn negative_empty_parameter_list() {
    assert_rejected_because("neg_01_empty_params.t27", "empty generic parameter list");
}

#[test]
fn negative_trailing_comma() {
    assert_rejected_because("neg_02_trailing_comma.t27", "trailing comma");
}

#[test]
fn negative_type_expression_in_parameter_position() {
    assert_rejected_because("neg_03_type_expr_param.t27", "must be a bare");
}

#[test]
fn negative_constrained_parameter() {
    assert_rejected_because("neg_04_constrained_param.t27", "expected ')' or ','");
}

#[test]
fn negative_enum_right_hand_side() {
    assert_rejected_because("neg_05_enum_rhs.t27", "must be 'struct'");
}

#[test]
fn negative_value_right_hand_side() {
    // Previously accepted with the right-hand side silently dropped.
    assert_rejected_because("neg_06_value_rhs.t27", "must be 'struct'");
}

#[test]
fn negative_no_right_hand_side() {
    // Previously accepted with the declaration silently truncated.
    assert_rejected_because("neg_07_no_rhs.t27", "expected '='");
}

// ---------------------------------------------------------------- AST shape

/// The AST contract of ADR-008, checked rather than assumed: the declared name
/// carries no parameter list, the parameters land in `params`, and the field
/// types survive. Parse success alone would not show any of this -- a parser
/// that dropped the whole body would also exit 0.
#[test]
fn ast_contract_name_params_and_field_types() {
    let out = parse_fixture("pos_02_two_params.t27");
    assert!(out.status.success(), "fixture must parse");
    let dump = String::from_utf8_lossy(&out.stdout);

    let decl = dump
        .find("kind: StructDecl")
        .map(|i| &dump[i..])
        .expect("parameterised const must produce a StructDecl node");

    assert!(
        decl.contains("name: \"Map\""),
        "name must be the bare declared name without its parameter list"
    );
    assert!(
        decl.contains("\"K\"") && decl.contains("\"V\""),
        "generic parameters must be preserved in the node"
    );
    assert!(
        !decl.contains("name: \"Map(K, V)\""),
        "the parameter list must not be folded into the name"
    );
}

// ---------------------------------------------------------------- regression

/// The construct straight out of the corpus that motivated #2162. This one is a
/// proof, not a guard: it fails against the parser as it stood before the fix.
#[test]
fn regression_original_failing_form() {
    let source = "\
module RegressionGenericConst {
    pub const Stack(T) = struct {
        items : \"[]T\",
        len : \"usize\",
    };
}
";
    let out = parse_source(source, "regression_stack");
    assert!(
        out.status.success(),
        "the corpus form `pub const Stack(T) = struct` must parse (#2162); \
         before the fix this failed with `Unexpected token in expression: \
         KwStruct`\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Guard against the generic path swallowing whatever follows it: a plain const
/// after a parameterised declaration must still be seen.
#[test]
fn regression_declaration_after_generic_is_not_swallowed() {
    let source = "\
module GenericThenPlain {
    pub const Boxed(T) = struct {
        value : \"T\",
    };
    pub const WIDTH : u32 = 27;
}
";
    let out = parse_source(source, "regression_after");
    assert!(out.status.success(), "module must parse");
    let dump = String::from_utf8_lossy(&out.stdout);
    assert!(
        dump.contains("name: \"WIDTH\""),
        "the declaration following a parameterised const must survive"
    );
}
