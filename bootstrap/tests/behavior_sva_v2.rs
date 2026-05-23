//! Integration tests for `t27c gen-behavior-sva-v2` (Wave 37, R-BV-1).

use std::process::Command;

fn t27c_bin() -> String {
    std::env::var("CARGO_BIN_EXE_t27c").expect("CARGO_BIN_EXE_t27c not set")
}

fn run(args: &[&str]) -> (String, String, bool) {
    let out = Command::new(t27c_bin())
        .args(args)
        .output()
        .expect("failed to execute t27c");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.success(),
    )
}

fn args_for(name: &str, given: &str, when: &str, then: &str) -> Vec<String> {
    vec![
        "gen-behavior-sva-v2".to_string(),
        "--name".to_string(),
        name.to_string(),
        "--given".to_string(),
        given.to_string(),
        "--when".to_string(),
        when.to_string(),
        "--then".to_string(),
        then.to_string(),
    ]
}

fn run_owned(args: Vec<String>) -> (String, String, bool) {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&refs)
}

// ============================================================================
// Smoke
// ============================================================================

#[test]
fn v2_help_lists_subcommand() {
    let (stdout, _stderr, ok) = run(&["gen-behavior-sva-v2", "--help"]);
    assert!(ok);
    assert!(stdout.contains("--name"));
    assert!(stdout.contains("--given"));
    assert!(stdout.contains("--when"));
    assert!(stdout.contains("--then"));
    assert!(stdout.contains("--index"));
}

#[test]
fn v2_emits_file_wrapper() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    assert!(stdout.contains("`timescale 1ns / 1ps"));
    assert!(stdout.contains("`default_nettype none"));
    assert!(stdout.contains("`default_nettype wire"));
    assert!(stdout.contains("End of behavior SVA v2 block."));
}

// ============================================================================
// Property structure
// ============================================================================

#[test]
fn v2_property_and_assert_and_cover_present() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    assert!(stdout.contains("property p_b1;"));
    assert!(stdout.contains("assert_0_b1: assert property (p_b1)"));
    assert!(stdout.contains("cover_0_b1: cover property (p_b1);"));
    assert!(stdout.contains("@(posedge clk) disable iff (!rst_n)"));
}

#[test]
fn v2_index_propagates() {
    let (stdout, _stderr, ok) = run(&[
        "gen-behavior-sva-v2",
        "--name", "b1",
        "--given", "valid",
        "--when", "posedge clk",
        "--then", "set full",
        "--index", "42",
    ]);
    assert!(ok);
    assert!(stdout.contains("assert_42_b1:"));
    assert!(stdout.contains("cover_42_b1:"));
    assert!(!stdout.contains("assert_0_b1:"));
}

// ============================================================================
// Multi-clause antecedents
// ============================================================================

#[test]
fn v2_single_given_matches_v1_form() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    // Bare predicate, no parens, identical to v1.
    assert!(stdout.contains("valid_in |->"));
}

#[test]
fn v2_two_clause_and_joined() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid and ready",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    assert!(stdout.contains("(valid_in && ready) |->"));
}

#[test]
fn v2_three_clause_comma_joined() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "running, valid, ready",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    assert!(stdout.contains("(running && valid_in && ready) |->"));
}

#[test]
fn v2_double_amp_separator() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid && ready",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    assert!(stdout.contains("(valid_in && ready) |->"));
}

#[test]
fn v2_dedup_collapses_to_single_predicate() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid and valid",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    // Both clauses reduce to valid_in -> collapse to bare predicate.
    assert!(stdout.contains("valid_in |->"));
    assert!(!stdout.contains("(valid_in && valid_in)"));
}

// ============================================================================
// ##N delay-clock
// ============================================================================

#[test]
fn v2_delay_after_three_cycles() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid",
        "posedge clk",
        "after 3 cycles set full",
    ));
    assert!(ok);
    assert!(stdout.contains("|-> ##3 "));
}

#[test]
fn v2_delay_after_one_cycle_singular() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid",
        "posedge clk",
        "after 1 cycle set full",
    ));
    assert!(ok);
    assert!(stdout.contains("|-> ##1 "));
}

#[test]
fn v2_delay_direct_pound_pound() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid",
        "posedge clk",
        "##7 set full",
    ));
    assert!(ok);
    assert!(stdout.contains("|-> ##7 "));
}

#[test]
fn v2_no_delay_when_then_is_plain() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid",
        "posedge clk",
        "set full",
    ));
    assert!(ok);
    // No delay clause -> property body must not contain |-> ##
    assert!(!stdout.contains("|-> ##"));
}

// ============================================================================
// s_eventually
// ============================================================================

#[test]
fn v2_eventually_keyword_emits_s_eventually() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "running",
        "posedge clk",
        "eventually set full",
    ));
    assert!(ok);
    assert!(stdout.contains("|-> s_eventually "));
}

#[test]
fn v2_liveness_keyword_emits_s_eventually() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "running",
        "posedge clk",
        "liveness: set full",
    ));
    assert!(ok);
    assert!(stdout.contains("|-> s_eventually "));
}

#[test]
fn v2_eventually_wins_over_delay() {
    // Both "eventually" and a delay phrase present -> liveness wins.
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "running",
        "posedge clk",
        "eventually after 3 cycles set full",
    ));
    assert!(ok);
    assert!(stdout.contains("s_eventually"));
    // Eventually wins -> no |-> ##N in property body
    assert!(!stdout.contains("|-> ##"));
}

// ============================================================================
// Composition + determinism + ASCII
// ============================================================================

#[test]
fn v2_all_three_features_composed() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "combo",
        "valid and ready",
        "posedge clk",
        "eventually set full",
    ));
    assert!(ok);
    assert!(stdout.contains("(valid_in && ready) |-> s_eventually "));
    assert!(stdout.contains("assert_0_combo:"));
}

#[test]
fn v2_multi_clause_with_delay() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "mix",
        "valid, ready",
        "posedge clk",
        "after 5 cycles set full",
    ));
    assert!(ok);
    assert!(stdout.contains("(valid_in && ready) |-> ##5 "));
}

#[test]
fn v2_output_is_deterministic() {
    let a = run_owned(args_for("b1", "valid and ready", "posedge clk", "set full")).0;
    let b = run_owned(args_for("b1", "valid and ready", "posedge clk", "set full")).0;
    assert_eq!(a, b);
}

#[test]
fn v2_output_is_pure_ascii() {
    let (stdout, _stderr, ok) = run_owned(args_for(
        "b1",
        "valid and ready",
        "posedge clk",
        "after 3 cycles set full",
    ));
    assert!(ok);
    assert!(stdout.is_ascii());
}

#[test]
fn v2_output_to_file() {
    let path =
        std::env::temp_dir().join(format!("t27_bsv2_out_{}.sv", std::process::id()));
    let path_s = path.to_string_lossy().to_string();
    let (_stdout, stderr, ok) = run(&[
        "gen-behavior-sva-v2",
        "--name", "b1",
        "--given", "valid and ready",
        "--when", "posedge clk",
        "--then", "eventually set full",
        "--output", &path_s,
    ]);
    assert!(ok, "stderr: {}", stderr);
    let body = std::fs::read_to_string(&path).expect("output file missing");
    assert!(body.contains("(valid_in && ready) |-> s_eventually "));
    assert!(body.contains("End of behavior SVA v2 block."));
    let _ = std::fs::remove_file(&path);
}

// ============================================================================
// v1 backward-compat -- v2 must NOT break v1's CLI
// ============================================================================

#[test]
fn v1_still_works_after_v2_added() {
    let (stdout, _stderr, ok) = run(&[
        "gen-behavior-sva",
        "--name", "b1",
        "--given", "valid",
        "--when", "posedge clk",
        "--then", "set full",
    ]);
    assert!(ok);
    // v1 has no "v2" tag in header, no `##`, no `s_eventually`.
    assert!(stdout.contains("Generated by t27c gen-behavior-sva (Wave 34"));
    assert!(!stdout.contains("v2"));
}
