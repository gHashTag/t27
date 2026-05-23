//! Wave 37 -- R-BV-1 (behavior-DSL v2 to SystemVerilog Assertions emitter)
//!
//! Extends the Wave-34 `behavior_sva` emitter with three temporal-logic
//! features that are pre-requisites for any spec-emit work (W38+):
//!
//!   1. **Multi-clause antecedents** -- the "given" clause may contain
//!      multiple predicates joined by `and`, `,`, or `&&`. v2 emits a
//!      parenthesised conjunction `(a && b && c)`.
//!   2. **`##N` delay-clock** -- the "then" clause may include a phrase
//!      like `after 3 cycles ...` or `##3 ...`; v2 emits a delayed
//!      implication `A |-> ##N B`.
//!   3. **`s_eventually`** -- when the "then" clause contains
//!      `eventually` or `liveness`, v2 emits a strong-fairness
//!      consequent `A |-> s_eventually B`.
//!
//! Pure string emission, zero dependencies on other bootstrap modules
//! besides `behavior_sva` (re-uses `parse_when_clause` for the clock-edge
//! timing).
//!
//! The Wave-34 baseline emitter (`behavior_sva.rs`) is intentionally not
//! touched: backward compatibility for its existing 8 integration tests
//! and the `gen-behavior-sva` CLI is preserved by routing v2 through a
//! separate module and a separate `gen-behavior-sva-v2` CLI subcommand.
//!
//! Vocabulary inherited from `gHashTag/vibee-lang`
//! `src/vibeec/verilog_codegen.zig` lines 2415-2531
//! (`generateSVAProperty`, `parseGivenClause`, `parseWhenClause`,
//! `parseThenClause`). The `##N` and `s_eventually` extensions are
//! standard IEEE 1800 SVA operators not present in the upstream emitter.
//! Original author for the parent vocabulary: Dmitrii Vasilev.
//!
//! Closes #775.

use crate::behavior_sva::{parse_given_clause, parse_then_clause, parse_when_clause};

/// A single behavior description for the v2 emitter.
///
/// Field layout is identical to `behavior_sva::Behavior` so callers can
/// trivially port v1 fixtures to v2.
#[derive(Debug, Clone)]
pub struct BehaviorV2<'a> {
    /// Behavior identifier (used in `p_<name>`, `assert_<idx>_<name>`,
    /// `cover_<idx>_<name>`).
    pub name: &'a str,
    /// "Given" clause -- the SVA antecedent. May contain multiple
    /// predicates joined by `and` / `,` / `&&`.
    pub given: &'a str,
    /// "When" clause -- the SVA clock-edge timing.
    pub when: &'a str,
    /// "Then" clause -- the SVA consequent, possibly delayed by `##N`
    /// or wrapped in `s_eventually`.
    pub then: &'a str,
}

/// Parsed shape of the consequent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsequentV2 {
    /// Plain consequent: emits `A |-> expr`.
    Plain(String),
    /// Delayed: emits `A |-> ##N expr`.
    Delayed { cycles: u32, expr: String },
    /// Liveness: emits `A |-> s_eventually expr`.
    Eventually(String),
}

/// Case-insensitive substring containment check.
fn contains_ci(haystack: &str, needle: &str) -> bool {
    let h = haystack.to_ascii_lowercase();
    let n = needle.to_ascii_lowercase();
    h.contains(&n)
}

/// Split a "given" clause into individual sub-clauses.
///
/// Recognised separators (case-insensitive for the textual ones):
///   * `,` (comma)
///   * `&&`
///   * ` and ` (with leading/trailing spaces to avoid clipping words like
///     `command`)
///
/// Each sub-clause is trimmed of whitespace; empty sub-clauses are
/// dropped. If no separator is found, the result is a single-element
/// vector containing the original clause.
pub fn split_given_clauses(given: &str) -> Vec<String> {
    // First normalise textual separators into a single canonical marker
    // `\x01` (a control char that cannot legally appear in user input),
    // then split.
    let lower = given.to_ascii_lowercase();
    let mut buf = String::with_capacity(given.len());
    let bytes = given.as_bytes();
    let lower_bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // " and " match against lower-case view
        if i + 5 <= lower_bytes.len() && &lower_bytes[i..i + 5] == b" and " {
            buf.push('\x01');
            i += 5;
            continue;
        }
        if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"&&" {
            buf.push('\x01');
            i += 2;
            continue;
        }
        if bytes[i] == b',' {
            buf.push('\x01');
            i += 1;
            continue;
        }
        buf.push(bytes[i] as char);
        i += 1;
    }
    buf.split('\x01')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse a (possibly multi-clause) "given" clause into an SVA antecedent.
///
/// Each sub-clause is fed through the W34 `parse_given_clause`
/// vocabulary. A single sub-clause emits exactly what v1 would emit; two
/// or more sub-clauses are joined as `(a && b && c)`.
pub fn parse_given_clause_v2(given: &str) -> String {
    let parts = split_given_clauses(given);
    if parts.len() <= 1 {
        // Either zero or one clause: defer to v1 behaviour. For zero we
        // pass the original (empty) string so v1's default `1'b1` kicks
        // in.
        return parse_given_clause(given).to_string();
    }
    let mut mapped: Vec<String> = parts
        .iter()
        .map(|p| parse_given_clause(p).to_string())
        .collect();
    // Drop duplicate predicates (case-sensitive on emitted form) but keep
    // first-occurrence order so output is deterministic.
    let mut seen = std::collections::BTreeSet::new();
    mapped.retain(|p| seen.insert(p.clone()));
    if mapped.len() == 1 {
        return mapped.remove(0);
    }
    format!("({})", mapped.join(" && "))
}

/// Try to extract an `##N` delay from a "then" clause.
///
/// Recognised forms (case-insensitive):
///   * `##N ...`            (direct SVA syntax)
///   * `after N cycles ...` (English-ish)
///   * `after N cycle ...`
///
/// Returns `Some(N)` with the integer value or `None` if no delay is
/// recognised. `N` must fit in a `u32`.
pub fn extract_delay_cycles(then: &str) -> Option<u32> {
    let lower = then.to_ascii_lowercase();
    // Direct ##N
    if let Some(rest) = lower.split("##").nth(1) {
        if !rest.is_empty() {
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                if let Ok(n) = digits.parse::<u32>() {
                    return Some(n);
                }
            }
        }
    }
    // "after N cycle(s)"
    if let Some(rest) = lower.split("after ").nth(1) {
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            let tail = &rest[digits.len()..];
            if tail.contains("cycle") {
                if let Ok(n) = digits.parse::<u32>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Detect whether the "then" clause requests liveness (`s_eventually`).
pub fn is_eventually(then: &str) -> bool {
    contains_ci(then, "eventually") || contains_ci(then, "liveness")
}

/// Parse the "then" clause into a typed `ConsequentV2`.
pub fn parse_then_clause_v2(then: &str) -> ConsequentV2 {
    let expr = parse_then_clause(then).to_string();
    if is_eventually(then) {
        return ConsequentV2::Eventually(expr);
    }
    if let Some(n) = extract_delay_cycles(then) {
        return ConsequentV2::Delayed { cycles: n, expr };
    }
    ConsequentV2::Plain(expr)
}

/// Build one full v2 SVA block (property + assert + cover) from a single
/// behavior.
///
/// `index` is the integer suffix used in `assert_<index>_<name>` and
/// `cover_<index>_<name>`.
pub fn build_behavior_sva_v2_block(behavior: &BehaviorV2<'_>, index: usize) -> String {
    let timing = parse_when_clause(behavior.when);
    let antecedent = parse_given_clause_v2(behavior.given);
    let consequent = parse_then_clause_v2(behavior.then);

    let impl_rhs = match &consequent {
        ConsequentV2::Plain(e) => e.clone(),
        ConsequentV2::Delayed { cycles, expr } => format!("##{} {}", cycles, expr),
        ConsequentV2::Eventually(e) => format!("s_eventually {}", e),
    };

    let mut out = String::with_capacity(512);
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("// Behavior (v2): {}\n", behavior.name));
    out.push_str(&format!("// Given:         {}\n", behavior.given));
    out.push_str(&format!("// When:          {}\n", behavior.when));
    out.push_str(&format!("// Then:          {}\n", behavior.then));
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("property p_{};\n", behavior.name));
    out.push_str(&format!("    @({}) disable iff (!rst_n)\n", timing));
    out.push_str(&format!("    {} |-> {};\n", antecedent, impl_rhs));
    out.push_str("endproperty\n\n");
    out.push_str(&format!(
        "assert_{}_{}: assert property (p_{})\n",
        index, behavior.name, behavior.name
    ));
    out.push_str(&format!(
        "    else $error(\"Assertion failed: {}\");\n\n",
        behavior.name
    ));
    out.push_str(&format!(
        "cover_{}_{}: cover property (p_{});\n",
        index, behavior.name, behavior.name
    ));
    out
}

/// Build a complete v2 SVA file containing one or more behavior blocks.
///
/// Wraps the blocks in the same `` `timescale `` / `` `default_nettype none``
/// banding used by v1 so the two emitters can be cleanly composed.
pub fn build_behavior_sva_v2_file(behaviors: &[BehaviorV2<'_>]) -> String {
    let mut out = String::with_capacity(1024 + behaviors.len() * 512);
    out.push_str(HEADER);
    for (i, b) in behaviors.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&build_behavior_sva_v2_block(b, i));
    }
    out.push_str(FOOTER);
    out
}

const HEADER: &str = "\
// ============================================================================
// Behavior-DSL v2 SystemVerilog Assertions
// Generated by t27c gen-behavior-sva-v2 (Wave 37, R-BV-1, Closes #775)
//
// v2 extensions over the Wave-34 baseline:
//   * Multi-clause antecedents (and / , / &&)
//   * ##N delayed implication
//   * s_eventually liveness consequent
//
// Vocabulary inherited from gHashTag/vibee-lang
// src/vibeec/verilog_codegen.zig (lines 2415-2531). Original author:
// Dmitrii Vasilev.
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`timescale 1ns / 1ps
`default_nettype none

";

const FOOTER: &str = "\

`default_nettype wire
// ============================================================================
// End of behavior SVA v2 block.
// ============================================================================
";

#[cfg(test)]
mod tests {
    use super::*;

    // ------- split_given_clauses --------

    #[test]
    fn split_single_clause_no_separator() {
        assert_eq!(split_given_clauses("valid"), vec!["valid"]);
    }

    #[test]
    fn split_on_and() {
        assert_eq!(
            split_given_clauses("valid and ready"),
            vec!["valid", "ready"]
        );
    }

    #[test]
    fn split_on_comma() {
        assert_eq!(
            split_given_clauses("valid, ready, running"),
            vec!["valid", "ready", "running"]
        );
    }

    #[test]
    fn split_on_double_amp() {
        assert_eq!(
            split_given_clauses("valid && ready"),
            vec!["valid", "ready"]
        );
    }

    #[test]
    fn split_does_not_clip_command_word() {
        // The substring "and" appears inside "command" but the splitter
        // requires surrounding spaces, so it should NOT split here.
        assert_eq!(split_given_clauses("command active"), vec!["command active"]);
    }

    #[test]
    fn split_is_case_insensitive_on_and() {
        assert_eq!(
            split_given_clauses("valid AND ready"),
            vec!["valid", "ready"]
        );
    }

    #[test]
    fn split_empty_input() {
        assert!(split_given_clauses("").is_empty());
    }

    // ------- parse_given_clause_v2 --------

    #[test]
    fn given_v2_single_matches_v1() {
        assert_eq!(parse_given_clause_v2("valid"), "valid_in");
    }

    #[test]
    fn given_v2_two_clauses_joined() {
        // valid -> valid_in, ready -> ready
        assert_eq!(parse_given_clause_v2("valid and ready"), "(valid_in && ready)");
    }

    #[test]
    fn given_v2_three_clauses_joined() {
        // running, valid, ready
        assert_eq!(
            parse_given_clause_v2("running, valid, ready"),
            "(running && valid_in && ready)"
        );
    }

    #[test]
    fn given_v2_dedup_repeated_predicates() {
        // Two clauses that both reduce to the same predicate should
        // collapse to a single conjunct (degenerate to bare predicate).
        assert_eq!(parse_given_clause_v2("valid and valid"), "valid_in");
    }

    // ------- extract_delay_cycles --------

    #[test]
    fn delay_direct_pound_pound() {
        assert_eq!(extract_delay_cycles("##3 done"), Some(3));
    }

    #[test]
    fn delay_after_cycles_english() {
        assert_eq!(extract_delay_cycles("after 5 cycles done"), Some(5));
    }

    #[test]
    fn delay_after_cycle_singular() {
        assert_eq!(extract_delay_cycles("after 1 cycle done"), Some(1));
    }

    #[test]
    fn delay_no_match() {
        assert_eq!(extract_delay_cycles("done"), None);
        assert_eq!(extract_delay_cycles("eventually done"), None);
    }

    // ------- is_eventually --------

    #[test]
    fn eventually_keyword_detected() {
        assert!(is_eventually("eventually done"));
        assert!(is_eventually("liveness: data settles"));
    }

    #[test]
    fn eventually_absent() {
        assert!(!is_eventually("done"));
        assert!(!is_eventually("after 3 cycles done"));
    }

    // ------- parse_then_clause_v2 --------

    #[test]
    fn then_plain() {
        match parse_then_clause_v2("set full") {
            ConsequentV2::Plain(_) => (),
            other => panic!("expected Plain, got {:?}", other),
        }
    }

    #[test]
    fn then_delayed_three() {
        match parse_then_clause_v2("after 3 cycles set full") {
            ConsequentV2::Delayed { cycles, .. } => assert_eq!(cycles, 3),
            other => panic!("expected Delayed, got {:?}", other),
        }
    }

    #[test]
    fn then_eventually_wins_over_delay() {
        // If both `eventually` and a delay are present, liveness wins.
        match parse_then_clause_v2("eventually after 3 cycles done") {
            ConsequentV2::Eventually(_) => (),
            other => panic!("expected Eventually, got {:?}", other),
        }
    }

    // ------- block emission --------

    #[test]
    fn block_plain_has_no_delay_or_eventually() {
        let b = BehaviorV2 {
            name: "b1",
            given: "valid",
            when: "posedge clk",
            then: "set full",
        };
        let s = build_behavior_sva_v2_block(&b, 0);
        assert!(s.contains("valid_in |->"));
        assert!(!s.contains("##"));
        assert!(!s.contains("s_eventually"));
    }

    #[test]
    fn block_delayed_emits_pound_pound_n() {
        let b = BehaviorV2 {
            name: "b1",
            given: "valid",
            when: "posedge clk",
            then: "after 3 cycles set full",
        };
        let s = build_behavior_sva_v2_block(&b, 0);
        assert!(s.contains("|-> ##3 "));
    }

    #[test]
    fn block_eventually_emits_s_eventually() {
        let b = BehaviorV2 {
            name: "b1",
            given: "valid",
            when: "posedge clk",
            then: "eventually set full",
        };
        let s = build_behavior_sva_v2_block(&b, 0);
        assert!(s.contains("|-> s_eventually "));
    }

    #[test]
    fn block_multi_clause_antecedent() {
        let b = BehaviorV2 {
            name: "b1",
            given: "valid and ready",
            when: "posedge clk",
            then: "set full",
        };
        let s = build_behavior_sva_v2_block(&b, 0);
        assert!(s.contains("(valid_in && ready) |->"));
    }

    #[test]
    fn block_compose_all_three_features() {
        let b = BehaviorV2 {
            name: "combo",
            given: "valid and ready",
            when: "posedge clk",
            then: "eventually set full",
        };
        let s = build_behavior_sva_v2_block(&b, 7);
        assert!(s.contains("(valid_in && ready) |-> s_eventually "));
        assert!(s.contains("assert_7_combo:"));
        assert!(s.contains("cover_7_combo:"));
    }

    // ------- file emission --------

    #[test]
    fn file_wraps_blocks_with_timescale_band() {
        let bs = [BehaviorV2 {
            name: "b1",
            given: "valid",
            when: "posedge clk",
            then: "set full",
        }];
        let s = build_behavior_sva_v2_file(&bs);
        assert!(s.contains("`timescale 1ns / 1ps"));
        assert!(s.contains("`default_nettype none"));
        assert!(s.contains("`default_nettype wire"));
        assert!(s.contains("End of behavior SVA v2 block."));
    }

    #[test]
    fn file_is_pure_ascii() {
        let bs = [BehaviorV2 {
            name: "b1",
            given: "valid and ready",
            when: "posedge clk",
            then: "after 3 cycles set full",
        }];
        let s = build_behavior_sva_v2_file(&bs);
        assert!(s.is_ascii());
    }
}
