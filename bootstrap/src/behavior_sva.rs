//! Wave 34 -- R-SV-1 (Behavior-DSL to SystemVerilog Assertions emitter)
//!
//! Consumes a behavior description (name + given clause + when clause + then
//! clause, all free-form English-ish text) and emits a self-contained
//! IEEE 1800 SystemVerilog Assertions block:
//!
//! ```systemverilog
//! // Behavior: <name>
//! // Given:    <given text>
//! // When:     <when text>
//! // Then:     <then text>
//! property p_<name>;
//!     @(<timing>) disable iff (!rst_n)
//!     <antecedent> |-> <consequent>;
//! endproperty
//!
//! assert_<idx>_<name>: assert property (p_<name>)
//!     else $error("Assertion failed: <name>");
//!
//! cover_<idx>_<name>: cover property (p_<name>);
//! ```
//!
//! The keyword vocabulary (given/when/then -> antecedent/timing/consequent) is
//! ported verbatim from `gHashTag/vibee-lang`
//! `src/vibeec/verilog_codegen.zig` lines 2415-2531 (`generateSVAProperty`,
//! `parseGivenClause`, `parseWhenClause`, `parseThenClause`). Original
//! author: Dmitrii Vasilev. Zig syntax translated to Rust string-building.
//!
//! Pure string emission, zero dependencies on other bootstrap modules.
//!
//! Closes #756.

/// A single behavior description.
///
/// The four fields map 1:1 to the vibee-lang `.behavior` block syntax.
/// `name` becomes the SVA identifier suffix (must be a valid Verilog identifier:
/// `[A-Za-z_][A-Za-z0-9_]*`); the other three are free-form clauses parsed by
/// keyword matching.
#[derive(Debug, Clone)]
pub struct Behavior<'a> {
    /// Behavior identifier (used in `p_<name>`, `assert_<idx>_<name>`,
    /// `cover_<idx>_<name>`).
    pub name: &'a str,
    /// "Given" clause -- the SVA antecedent.
    pub given: &'a str,
    /// "When" clause -- the SVA timing (clock edge).
    pub when: &'a str,
    /// "Then" clause -- the SVA consequent.
    pub then: &'a str,
}

/// Case-insensitive substring containment check.
fn contains_ci(haystack: &str, needle: &str) -> bool {
    let h = haystack.to_ascii_lowercase();
    let n = needle.to_ascii_lowercase();
    h.contains(&n)
}

/// Parse the "given" clause into an SVA antecedent expression.
///
/// Recognized keywords (case-insensitive, in priority order):
///   running / active / valid / ready
///   reset (with "not" or "inactive" -> rst_n, else !rst_n)
///   idle / process -> state machine
///   counter / count (with max / zero / 0 modifiers)
///   fifo (with not + full/empty modifiers, or bare full/empty)
///   bare full / empty / not full / not empty
///
/// Default fallback: `1'b1` (always-true antecedent).
pub fn parse_given_clause(given: &str) -> &'static str {
    if contains_ci(given, "running") {
        return "running";
    }
    if contains_ci(given, "active") {
        return "active";
    }
    if contains_ci(given, "valid") {
        return "valid_in";
    }
    if contains_ci(given, "ready") {
        return "ready";
    }
    if contains_ci(given, "reset") {
        if contains_ci(given, "not") || contains_ci(given, "inactive") {
            return "rst_n";
        }
        return "!rst_n";
    }
    if contains_ci(given, "idle") {
        return "(state == IDLE)";
    }
    if contains_ci(given, "process") {
        return "(state == PROCESS)";
    }
    if contains_ci(given, "counter") || contains_ci(given, "count") {
        if contains_ci(given, "max") {
            return "(count == MAX_VALUE)";
        }
        if contains_ci(given, "zero") || contains_ci(given, "0") {
            return "(count == 0)";
        }
        return "(count > 0)";
    }
    if contains_ci(given, "fifo") {
        if contains_ci(given, "not") && contains_ci(given, "full") {
            return "!full";
        }
        if contains_ci(given, "not") && contains_ci(given, "empty") {
            return "!empty";
        }
        if contains_ci(given, "full") {
            return "full";
        }
        if contains_ci(given, "empty") {
            return "empty";
        }
        return "!empty";
    }
    // Direct signal names from types (full, empty, etc.)
    if contains_ci(given, "not") && contains_ci(given, "full") {
        return "!full";
    }
    if contains_ci(given, "not") && contains_ci(given, "empty") {
        return "!empty";
    }
    if contains_ci(given, "full") {
        return "full";
    }
    if contains_ci(given, "empty") {
        return "empty";
    }
    "1'b1"
}

/// Parse the "when" clause into an SVA clock-edge timing expression.
///
/// Recognized keywords: `falling` / `negedge` -> `negedge clk`. Default:
/// `posedge clk` (rising edge).
pub fn parse_when_clause(when: &str) -> &'static str {
    if contains_ci(when, "falling") || contains_ci(when, "negedge") {
        return "negedge clk";
    }
    "posedge clk"
}

/// Parse the "then" clause into an SVA consequent expression.
///
/// Recognized keywords (case-insensitive, in priority order):
///   increment / add (count -> $past(count)+1; else $past(data_out)+1)
///   decrement / subtract (same shape)
///   zero / clear / "set 0" (count -> 0; overflow -> !overflow; else data_out==0)
///   set flag (overflow/valid/done/full/empty mappings)
///   bare set + full/empty
///   valid + output -> valid_out
///   wrap -> (count == 0)
///
/// Default fallback: `1'b1` (vacuously true consequent).
pub fn parse_then_clause(then: &str) -> &'static str {
    // Increment operation
    if contains_ci(then, "increment") || contains_ci(then, "add") {
        if contains_ci(then, "count") {
            return "(count == $past(count) + 1)";
        }
        return "($past(data_out) + 1)";
    }
    // Decrement operation
    if contains_ci(then, "decrement") || contains_ci(then, "subtract") {
        if contains_ci(then, "count") {
            return "(count == $past(count) - 1)";
        }
        return "($past(data_out) - 1)";
    }
    // Set to zero / clear
    if contains_ci(then, "zero")
        || contains_ci(then, "clear")
        || (contains_ci(then, "set") && contains_ci(then, "0"))
    {
        if contains_ci(then, "count") {
            return "(count == 0)";
        }
        if contains_ci(then, "overflow") {
            return "(!overflow)";
        }
        return "(data_out == 0)";
    }
    // Set flag
    if contains_ci(then, "set") && contains_ci(then, "flag") {
        if contains_ci(then, "overflow") {
            return "overflow";
        }
        if contains_ci(then, "valid") {
            return "valid_out";
        }
        if contains_ci(then, "done") {
            return "done";
        }
        if contains_ci(then, "full") {
            return "full";
        }
        if contains_ci(then, "empty") {
            return "empty";
        }
        return "flag";
    }
    // Direct flag setting (from types)
    if contains_ci(then, "set") && contains_ci(then, "full") {
        return "full";
    }
    if contains_ci(then, "set") && contains_ci(then, "empty") {
        return "empty";
    }
    // Output valid
    if contains_ci(then, "valid") && contains_ci(then, "output") {
        return "valid_out";
    }
    // Wrap around
    if contains_ci(then, "wrap") {
        return "(count == 0)";
    }
    "1'b1"
}

/// Build one full SVA block (property + assert + cover) from a single behavior.
///
/// `index` is the integer suffix used in `assert_<index>_<name>` and
/// `cover_<index>_<name>`. Header comments quote the original given/when/then
/// clauses verbatim so the human-readable spec stays attached to the emitted
/// assertion.
pub fn build_behavior_sva_block(behavior: &Behavior<'_>, index: usize) -> String {
    let timing = parse_when_clause(behavior.when);
    let antecedent = parse_given_clause(behavior.given);
    let consequent = parse_then_clause(behavior.then);

    let mut out = String::with_capacity(512);
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("// Behavior: {}\n", behavior.name));
    out.push_str(&format!("// Given:    {}\n", behavior.given));
    out.push_str(&format!("// When:     {}\n", behavior.when));
    out.push_str(&format!("// Then:     {}\n", behavior.then));
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("property p_{};\n", behavior.name));
    out.push_str(&format!("    @({}) disable iff (!rst_n)\n", timing));
    out.push_str(&format!("    {} |-> {};\n", antecedent, consequent));
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

/// Build a complete SVA file containing one or more behavior blocks.
///
/// Wraps the blocks in the standard `` `timescale `` / `` `default_nettype none``
/// banding used by the rest of the t27 emitter family (matches Wave 32/33
/// `trit_stdlib` conventions).
pub fn build_behavior_sva_file(behaviors: &[Behavior<'_>]) -> String {
    let mut out = String::with_capacity(1024 + behaviors.len() * 512);
    out.push_str(HEADER);
    for (i, b) in behaviors.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&build_behavior_sva_block(b, i));
    }
    out.push_str(FOOTER);
    out
}

const HEADER: &str = "\
// ============================================================================
// Behavior-DSL SystemVerilog Assertions
// Generated by t27c gen-behavior-sva (Wave 34, R-SV-1, Closes #756)
//
// Vocabulary ported from gHashTag/vibee-lang src/vibeec/verilog_codegen.zig
// (lines 2415-2531). Original author: Dmitrii Vasilev.
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`timescale 1ns / 1ps
`default_nettype none

";

const FOOTER: &str = "\

`default_nettype wire
// ============================================================================
// End of behavior SVA block.
// ============================================================================
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_running_active_valid_ready() {
        assert_eq!(parse_given_clause("system is running"), "running");
        assert_eq!(parse_given_clause("unit ACTIVE now"), "active");
        assert_eq!(parse_given_clause("data is valid"), "valid_in");
        assert_eq!(parse_given_clause("consumer ready"), "ready");
    }

    #[test]
    fn given_reset_flip() {
        // bare reset -> !rst_n
        assert_eq!(parse_given_clause("reset asserted"), "!rst_n");
        // "not reset" or "reset inactive" -> rst_n
        assert_eq!(parse_given_clause("not in reset"), "rst_n");
        assert_eq!(parse_given_clause("reset inactive"), "rst_n");
    }

    #[test]
    fn given_counter_modifiers() {
        assert_eq!(parse_given_clause("counter at max"), "(count == MAX_VALUE)");
        assert_eq!(parse_given_clause("count is zero"), "(count == 0)");
        // bare counter mention -> nonzero
        assert_eq!(parse_given_clause("counter running"), "(count > 0)");
    }

    #[test]
    fn given_fifo_modifiers() {
        assert_eq!(parse_given_clause("fifo not full"), "!full");
        assert_eq!(parse_given_clause("fifo not empty"), "!empty");
        assert_eq!(parse_given_clause("fifo full"), "full");
        assert_eq!(parse_given_clause("fifo empty"), "empty");
    }

    #[test]
    fn given_default_is_always_true() {
        assert_eq!(parse_given_clause("some unrecognized thing"), "1'b1");
    }

    #[test]
    fn when_falling_vs_rising() {
        assert_eq!(parse_when_clause("on the falling edge"), "negedge clk");
        assert_eq!(parse_when_clause("negedge clock"), "negedge clk");
        // default
        assert_eq!(parse_when_clause("rising edge"), "posedge clk");
        assert_eq!(parse_when_clause(""), "posedge clk");
    }

    #[test]
    fn then_increment_decrement() {
        assert_eq!(
            parse_then_clause("increment count"),
            "(count == $past(count) + 1)"
        );
        assert_eq!(
            parse_then_clause("add to count"),
            "(count == $past(count) + 1)"
        );
        assert_eq!(parse_then_clause("increment"), "($past(data_out) + 1)");
        assert_eq!(
            parse_then_clause("decrement count"),
            "(count == $past(count) - 1)"
        );
        assert_eq!(
            parse_then_clause("subtract from count"),
            "(count == $past(count) - 1)"
        );
    }

    #[test]
    fn then_zero_clear_overflow() {
        assert_eq!(parse_then_clause("zero the count"), "(count == 0)");
        assert_eq!(parse_then_clause("clear count"), "(count == 0)");
        assert_eq!(parse_then_clause("clear overflow"), "(!overflow)");
        assert_eq!(parse_then_clause("clear all"), "(data_out == 0)");
    }

    #[test]
    fn then_set_flag_variants() {
        assert_eq!(parse_then_clause("set the overflow flag"), "overflow");
        assert_eq!(parse_then_clause("set the valid flag"), "valid_out");
        assert_eq!(parse_then_clause("set done flag"), "done");
        assert_eq!(parse_then_clause("set full flag"), "full");
        assert_eq!(parse_then_clause("set empty flag"), "empty");
    }

    #[test]
    fn then_wrap_and_default() {
        assert_eq!(parse_then_clause("wrap around"), "(count == 0)");
        assert_eq!(parse_then_clause("something else"), "1'b1");
    }

    #[test]
    fn block_emits_property_assert_and_cover() {
        let b = Behavior {
            name: "tick",
            given: "system is running",
            when: "rising edge",
            then: "increment count",
        };
        let block = build_behavior_sva_block(&b, 7);
        assert!(block.contains("property p_tick;"));
        assert!(block.contains("@(posedge clk) disable iff (!rst_n)"));
        assert!(block.contains("running |-> (count == $past(count) + 1);"));
        assert!(block.contains("endproperty"));
        assert!(block.contains(
            "assert_7_tick: assert property (p_tick)\n    else $error(\"Assertion failed: tick\");"
        ));
        assert!(block.contains("cover_7_tick: cover property (p_tick);"));
    }

    #[test]
    fn file_wraps_with_timescale_and_nettype_band() {
        let behaviors = [
            Behavior {
                name: "a",
                given: "running",
                when: "rising edge",
                then: "increment count",
            },
            Behavior {
                name: "b",
                given: "fifo not empty",
                when: "falling edge",
                then: "decrement count",
            },
        ];
        let v = build_behavior_sva_file(&behaviors);
        assert!(v.contains("`timescale 1ns / 1ps"));
        assert!(v.contains("`default_nettype none"));
        assert!(v.contains("`default_nettype wire"));
        assert!(v.contains("property p_a;"));
        assert!(v.contains("property p_b;"));
        assert!(v.contains("assert_0_a"));
        assert!(v.contains("assert_1_b"));
        assert!(v.contains("cover_0_a"));
        assert!(v.contains("cover_1_b"));
        // Falling-edge timing reached the second block.
        assert!(v.contains("@(negedge clk) disable iff (!rst_n)"));
    }
}
