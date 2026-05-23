//! Wave 34 -- R-SV-1 regression tests for the behavior-DSL SVA emitter.
//!
//! Strategy: shell out to the built t27c binary via `CARGO_BIN_EXE_t27c`,
//! run `gen-behavior-sva` with assorted given/when/then clauses, capture
//! stdout, and assert structural invariants on the emitted SVA text. No HDL
//! toolchain is required to run these tests; IEEE-1800 conformance of the
//! emitted blocks is downstream of the keyword parser, which has dedicated
//! unit tests in `bootstrap/src/behavior_sva.rs`.
//!
//! Closes #756.

use std::process::Command;

fn run_gen_behavior_sva(name: &str, given: &str, when: &str, then: &str, index: usize) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-behavior-sva")
        .arg("--name").arg(name)
        .arg("--given").arg(given)
        .arg("--when").arg(when)
        .arg("--then").arg(then)
        .arg("--index").arg(index.to_string())
        .output()
        .expect("failed to spawn t27c gen-behavior-sva");
    assert!(
        output.status.success(),
        "t27c gen-behavior-sva exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("t27c gen-behavior-sva produced non-UTF-8 output")
}

#[test]
fn emits_property_assert_and_cover_via_cli() {
    let v = run_gen_behavior_sva(
        "tick",
        "system is running",
        "rising edge of clk",
        "increment count",
        0,
    );
    // All three SVA building blocks must be present.
    assert!(v.contains("property p_tick;"), "missing property header");
    assert!(v.contains("endproperty"), "missing endproperty");
    assert!(
        v.contains("assert_0_tick: assert property (p_tick)"),
        "missing assert label/property reference"
    );
    assert!(
        v.contains("$error(\"Assertion failed: tick\")"),
        "assert must carry an $error message"
    );
    assert!(
        v.contains("cover_0_tick: cover property (p_tick);"),
        "missing cover statement (W34 adds auto-cover bonus)"
    );
}

#[test]
fn given_keyword_dispatch_via_cli() {
    // running -> antecedent "running"
    let v = run_gen_behavior_sva("b1", "module is running", "rising edge", "wrap around", 1);
    assert!(v.contains("running |->"), "expected 'running' antecedent");

    // fifo not empty -> "!empty"
    let v = run_gen_behavior_sva("b2", "fifo not empty", "rising edge", "wrap around", 2);
    assert!(v.contains("!empty |->"), "expected '!empty' antecedent");

    // counter at max -> "(count == MAX_VALUE)"
    let v = run_gen_behavior_sva("b3", "counter at max", "rising edge", "wrap around", 3);
    assert!(
        v.contains("(count == MAX_VALUE) |->"),
        "expected counter-max antecedent"
    );

    // unrecognized -> default "1'b1"
    let v = run_gen_behavior_sva("b4", "xyzzy", "rising edge", "wrap around", 4);
    assert!(v.contains("1'b1 |->"), "expected default '1'b1' antecedent");
}

#[test]
fn when_falling_edge_via_cli() {
    let v = run_gen_behavior_sva("late", "running", "falling edge", "wrap around", 0);
    assert!(
        v.contains("@(negedge clk) disable iff (!rst_n)"),
        "falling clause must produce 'negedge clk'"
    );
    // Sanity: rising remains default.
    let v = run_gen_behavior_sva("early", "running", "rising edge", "wrap around", 0);
    assert!(v.contains("@(posedge clk) disable iff (!rst_n)"));
}

#[test]
fn then_keyword_dispatch_via_cli() {
    // increment + count -> "(count == $past(count) + 1)"
    let v = run_gen_behavior_sva("inc", "running", "rising edge", "increment count", 0);
    assert!(v.contains("|-> (count == $past(count) + 1);"));

    // decrement + count -> "(count == $past(count) - 1)"
    let v = run_gen_behavior_sva("dec", "running", "rising edge", "decrement count", 0);
    assert!(v.contains("|-> (count == $past(count) - 1);"));

    // clear overflow -> "(!overflow)"
    let v = run_gen_behavior_sva("clr", "running", "rising edge", "clear overflow", 0);
    assert!(v.contains("|-> (!overflow);"));

    // set flag valid -> "valid_out"
    let v = run_gen_behavior_sva("sv", "running", "rising edge", "set the valid flag", 0);
    assert!(v.contains("|-> valid_out;"));
}

#[test]
fn custom_index_is_honoured_via_cli() {
    let v = run_gen_behavior_sva("anyhow", "running", "rising edge", "wrap around", 42);
    assert!(v.contains("assert_42_anyhow:"), "expected index 42 in assert label");
    assert!(v.contains("cover_42_anyhow:"), "expected index 42 in cover label");
    // The property identifier is always p_<name> (not indexed).
    assert!(v.contains("property p_anyhow;"));
}

#[test]
fn disable_iff_uses_rst_n_via_cli() {
    let v = run_gen_behavior_sva("dr", "running", "rising edge", "wrap around", 0);
    // The "disable iff (!rst_n)" guard is mandatory in the W34 emitter --
    // it is the convention vibee-lang uses and is required for any SVA
    // block that can fire while the design is in reset.
    assert!(
        v.contains("disable iff (!rst_n)"),
        "every property must carry 'disable iff (!rst_n)'"
    );
}

#[test]
fn header_comment_quotes_clauses_via_cli() {
    let v = run_gen_behavior_sva(
        "doc",
        "system is running and active",
        "on the rising edge",
        "increment the count",
        0,
    );
    // Header comments must mirror the original clauses verbatim so the
    // human spec stays attached to the emitted SVA.
    assert!(v.contains("// Behavior: doc"));
    assert!(v.contains("// Given:    system is running and active"));
    assert!(v.contains("// When:     on the rising edge"));
    assert!(v.contains("// Then:     increment the count"));
}

#[test]
fn output_is_self_contained_and_balanced() {
    let v = run_gen_behavior_sva("any", "running", "rising edge", "wrap around", 0);
    assert!(v.contains("`timescale 1ns / 1ps"));
    assert!(v.contains("`default_nettype none"));
    assert!(v.contains("`default_nettype wire"));
    // Exactly one property/endproperty pair (single-behavior CLI emit).
    assert_eq!(v.matches("\nproperty p_").count(), 1);
    assert_eq!(v.matches("\nendproperty").count(), 1);
    // Exactly one assert and one cover.
    let assert_count = v.matches("assert property (p_").count();
    let cover_count = v.matches("cover property (p_").count();
    assert_eq!(assert_count, 1, "expected exactly 1 'assert property'");
    assert_eq!(cover_count, 1, "expected exactly 1 'cover property'");
}
