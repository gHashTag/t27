//! Wave 37 -- R-BV-1 (Extended behavior-DSL to SystemVerilog Assertions emitter)
//!
//! Extends the Wave 34 (R-SV-1) `behavior_sva.rs` with three temporal-logic
//! features required for spec-emit work (W38+):
//!
//! 1. **Multi-clause antecedents** -- conjunction of two or more predicates
//!    joined by `and` / `,` / `&&`, e.g. `given = "valid and ready"` emits
//!    `(valid_in && ready)`.
//! 2. **`##N` delay-clock** -- delayed implication `A |-> ##N B` so the
//!    consequent is required `N` cycles after the antecedent fires. Parsed
//!    from "then" clauses like `then = "after 3 cycles done"`.
//! 3. **`s_eventually`** -- strong-fairness operator (`A |-> s_eventually B`)
//!    for liveness properties, triggered when the "then" clause contains
//!    `eventually`.
//!
//! The v1 emitter (`behavior_sva.rs`) is frozen for backward compatibility.
//! This module is a separate file with a separate CLI subcommand
//! `gen-behavior-sva-v2`.
//!
//! Closes #775.

use crate::behavior_sva::{parse_when_clause, Behavior};

/// Extended consequent for the v2 emitter.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsequentV2 {
    Plain(String),
    Delayed { cycles: usize, expr: String },
    Eventually(String),
}

/// Single-token identifiers recognised by the v2 given-clause parser.
const GIVEN_TOKENS: &[(&str, &str)] = &[
    ("running", "running"),
    ("active", "active"),
    ("valid", "valid_in"),
    ("ready", "ready"),
    ("reset", "!rst_n"),
    ("idle", "(state == IDLE)"),
    ("process", "(state == PROCESS)"),
    ("full", "full"),
    ("empty", "empty"),
    ("busy", "busy"),
    ("done", "done"),
    ("start", "start"),
    ("error", "error"),
];

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
}

/// Map a single given-atom (one word or short phrase) to an SVA expression.
///
/// Falls back to passthrough: if the atom contains no recognised keyword it is
/// emitted verbatim (allowing arbitrary signal names like `irq_status[2]`).
fn map_given_atom(atom: &str) -> String {
    let a = atom.trim();
    if a.is_empty() {
        return "1'b1".to_string();
    }
    if contains_ci(a, "not") {
        if contains_ci(a, "reset") {
            return "rst_n".to_string();
        }
        if contains_ci(a, "full") {
            return "!full".to_string();
        }
        if contains_ci(a, "empty") {
            return "!empty".to_string();
        }
        if contains_ci(a, "busy") {
            return "!busy".to_string();
        }
    }
    if contains_ci(a, "reset") {
        if contains_ci(a, "not") || contains_ci(a, "inactive") {
            return "rst_n".to_string();
        }
        return "!rst_n".to_string();
    }
    if contains_ci(a, "counter") || contains_ci(a, "count") {
        if contains_ci(a, "max") {
            return "(count == MAX_VALUE)".to_string();
        }
        if contains_ci(a, "zero") || contains_ci(a, "0") {
            return "(count == 0)".to_string();
        }
        return "(count > 0)".to_string();
    }
    if contains_ci(a, "fifo") {
        if contains_ci(a, "not") && contains_ci(a, "full") {
            return "!full".to_string();
        }
        if contains_ci(a, "not") && contains_ci(a, "empty") {
            return "!empty".to_string();
        }
        if contains_ci(a, "full") {
            return "full".to_string();
        }
        if contains_ci(a, "empty") {
            return "empty".to_string();
        }
        return "!empty".to_string();
    }
    for &(keyword, sva_expr) in GIVEN_TOKENS {
        if contains_ci(a, keyword) {
            return sva_expr.to_string();
        }
    }
    a.to_string()
}

/// Split a given clause on conjunction separators (`and`, `,`, `&&`) and map
/// each atom individually.
///
/// Returns the conjunction as a parenthesised expression:
/// - single atom -> the atom itself (no parens needed)
/// - multiple atoms -> `(a && b && c)`
pub fn parse_given_clause_v2(given: &str) -> String {
    let atoms = split_conjunctions(given);
    let mapped: Vec<String> = atoms.iter().map(|a| map_given_atom(a)).collect();
    match mapped.len() {
        0 => "1'b1".to_string(),
        1 => mapped.into_iter().next().unwrap(),
        _ => format!("({})", mapped.join(" && ")),
    }
}

/// Split a clause on conjunction separators: `and`, `,`, `&&`.
fn split_conjunctions(clause: &str) -> Vec<&str> {
    let mut atoms = Vec::new();
    let mut rest = clause;
    loop {
        if let Some(pos) = find_conjunction_sep(rest) {
            let (head, tail) = rest.split_at(pos);
            atoms.push(head.trim());
            rest = skip_sep(tail).trim_start();
        } else {
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                atoms.push(trimmed);
            }
            break;
        }
    }
    if atoms.is_empty() {
        atoms.push(clause.trim());
    }
    atoms
}

/// Find the earliest conjunction separator (` and `, `,`, `&&`) in the string.
fn find_conjunction_sep(s: &str) -> Option<usize> {
    let lower = s.to_ascii_lowercase();
    if let Some(pos) = lower.find(" and ") {
        return Some(pos);
    }
    if let Some(pos) = s.find("&&") {
        return Some(pos);
    }
    if let Some(pos) = s.find(',') {
        return Some(pos);
    }
    None
}

/// Skip past a conjunction separator at the start of `s`.
fn skip_sep(s: &str) -> &str {
    let lower_start = s.to_ascii_lowercase();
    if lower_start.starts_with(" and ") {
        return &s[5..];
    }
    if s.starts_with("&&") {
        return &s[2..];
    }
    if s.starts_with(',') {
        return &s[1..];
    }
    s
}

/// Map a single then-atom to a plain consequent string (reuses v1 vocabulary).
fn map_then_atom(atom: &str) -> String {
    let a = atom.trim();
    if a.is_empty() {
        return "1'b1".to_string();
    }
    if contains_ci(a, "increment") || contains_ci(a, "add") {
        if contains_ci(a, "count") {
            return "(count == $past(count) + 1)".to_string();
        }
        return "($past(data_out) + 1)".to_string();
    }
    if contains_ci(a, "decrement") || contains_ci(a, "subtract") {
        if contains_ci(a, "count") {
            return "(count == $past(count) - 1)".to_string();
        }
        return "($past(data_out) - 1)".to_string();
    }
    if contains_ci(a, "zero") || contains_ci(a, "clear") {
        if contains_ci(a, "count") {
            return "(count == 0)".to_string();
        }
        if contains_ci(a, "overflow") {
            return "(!overflow)".to_string();
        }
        return "(data_out == 0)".to_string();
    }
    if contains_ci(a, "set") && contains_ci(a, "flag") {
        if contains_ci(a, "overflow") {
            return "overflow".to_string();
        }
        if contains_ci(a, "valid") {
            return "valid_out".to_string();
        }
        if contains_ci(a, "done") {
            return "done".to_string();
        }
        if contains_ci(a, "full") {
            return "full".to_string();
        }
        if contains_ci(a, "empty") {
            return "empty".to_string();
        }
        return "flag".to_string();
    }
    if contains_ci(a, "set") && contains_ci(a, "full") {
        return "full".to_string();
    }
    if contains_ci(a, "set") && contains_ci(a, "empty") {
        return "empty".to_string();
    }
    if contains_ci(a, "valid") && contains_ci(a, "output") {
        return "valid_out".to_string();
    }
    if contains_ci(a, "wrap") {
        return "(count == 0)".to_string();
    }
    for &(keyword, sva_expr) in &[("done", "done"), ("busy", "busy"), ("error", "error"), ("start", "start")] {
        if contains_ci(a, keyword) {
            return sva_expr.to_string();
        }
    }
    a.to_string()
}

/// Parse the "then" clause into an extended consequent.
///
/// Three branches:
/// - `eventually` / `liveness` -> `ConsequentV2::Eventually(expr)`
/// - `after N cycles` / `##N` -> `ConsequentV2::Delayed { cycles, expr }`
/// - otherwise -> `ConsequentV2::Plain(expr)`
pub fn parse_then_clause_v2(then: &str) -> ConsequentV2 {
    if contains_ci(then, "eventually") || contains_ci(then, "liveness") {
        let cleaned = then
            .replace("eventually-set", "")
            .replace("eventually", "")
            .replace("liveness", "");
        let expr = map_then_atom(&cleaned);
        let expr = if expr.is_empty() || expr.trim().is_empty() {
            "1'b1".to_string()
        } else {
            expr
        };
        return ConsequentV2::Eventually(expr);
    }

    if let Some(cycles) = extract_delay_cycles(then) {
        let cleaned = remove_delay_phrase(then);
        let expr = map_then_atom(&cleaned);
        return ConsequentV2::Delayed { cycles, expr };
    }

    ConsequentV2::Plain(map_then_atom(then))
}

/// Extract the cycle count from `after N cycles` or `##N` in a then-clause.
fn extract_delay_cycles(then: &str) -> Option<usize> {
    let lower = then.to_ascii_lowercase();
    if let Some(pos) = lower.find("after") {
        let rest = &then[pos + 5..].trim();
        if let Some(n) = rest.split_whitespace().next() {
            if let Ok(cycles) = n.parse::<usize>() {
                return Some(cycles);
            }
        }
    }
    if let Some(pos) = lower.find("##") {
        let rest = &then[pos + 2..].trim();
        if let Some(n) = rest.split_whitespace().next() {
            if let Ok(cycles) = n.parse::<usize>() {
                return Some(cycles);
            }
        }
    }
    None
}

/// Remove the delay phrase (`after N cycles` or `##N`) from a then-clause.
fn remove_delay_phrase(then: &str) -> String {
    let lower = then.to_ascii_lowercase();
    if let Some(pos) = lower.find("after") {
        let rest = &lower[pos + 6..];
        if let Some(n) = rest.split_whitespace().next() {
            if n.parse::<usize>().is_ok() {
                let phrase = &then[pos..pos + 6 + n.len()];
                let cleaned = then.replace(phrase.trim(), "");
                return cleaned.trim().to_string();
            }
        }
    }
    if let Some(pos) = lower.find("##") {
        let rest = &lower[pos + 2..].trim();
        if let Some(n) = rest.split_whitespace().next() {
            if n.parse::<usize>().is_ok() {
                let phrase = format!("##{}", n);
                return then.replace(&phrase, "").trim().to_string();
            }
        }
    }
    then.to_string()
}

/// Build one full SVA block (property + assert + cover) from a single behavior
/// using the v2 extended consequent.
pub fn build_behavior_sva_v2_block(behavior: &Behavior<'_>, index: usize) -> String {
    let timing = parse_when_clause(behavior.when);
    let antecedent = parse_given_clause_v2(behavior.given);
    let consequent = parse_then_clause_v2(behavior.then);

    let consequent_sva = match &consequent {
        ConsequentV2::Plain(expr) => expr.clone(),
        ConsequentV2::Delayed { cycles, expr } => {
            if *cycles == 0 {
                expr.clone()
            } else {
                format!("##{} {}", cycles, expr)
            }
        }
        ConsequentV2::Eventually(expr) => format!("s_eventually {}", expr),
    };

    let mut out = String::with_capacity(640);
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("// Behavior: {}\n", behavior.name));
    out.push_str(&format!("// Given:    {}\n", behavior.given));
    out.push_str(&format!("// When:     {}\n", behavior.when));
    out.push_str(&format!("// Then:     {}\n", behavior.then));
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("property p_{};\n", behavior.name));
    out.push_str(&format!("    @({}) disable iff (!rst_n)\n", timing));
    out.push_str(&format!("    {} |-> {};\n", antecedent, consequent_sva));
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

/// Build a complete SVA file containing one or more behavior blocks (v2).
/// SystemVerilog tokens that appear in an emitted property block but are not
/// design signals.
const SVA_RESERVED: &[&str] = &[
    "property",
    "endproperty",
    "posedge",
    "negedge",
    "disable",
    "iff",
    "assert",
    "cover",
    "else",
    "s_eventually",
    "module",
    "endmodule",
    "input",
    "wire",
    "logic",
    "timescale",
    "default_nettype",
    "none",
    "ns",
    "ps",
    // Procedural keywords: the Yosys-subset emitter wraps assertions in
    // `always @(...) begin if (...) ... end`, so these appear in the body.
    "always",
    "begin",
    "end",
    "if",
    "wire",
];

/// Collect the design signals a generated SVA body references, so they can be
/// declared as module ports.
///
/// Scanning the emitted text rather than re-deriving names from the behavior
/// DSL keeps one source of truth: as the DSL vocabulary grows, the port list
/// follows the emitter automatically instead of drifting from it.
pub fn collect_sva_signals(body: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for raw_line in body.lines() {
        // Drop comments and string literals before tokenising -- `$error("...")`
        // messages contain behavior names that are not signals.
        let line = match raw_line.find("//") {
            Some(i) => &raw_line[..i],
            None => raw_line,
        };
        let mut cleaned = String::with_capacity(line.len());
        let mut in_string = false;
        for c in line.chars() {
            match c {
                '"' => in_string = !in_string,
                _ if !in_string => cleaned.push(c),
                _ => {}
            }
        }

        let mut cur = String::new();
        // `$error(...)` and friends tokenise to a bare `error` once `$` is
        // treated as a separator, which would declare a system task as a port.
        let mut dollar_prefixed = false;
        let mut prev = ' ';
        for c in cleaned.chars().chain(std::iter::once(' ')) {
            if c.is_ascii_alphanumeric() || c == '_' {
                if cur.is_empty() {
                    dollar_prefixed = prev == '$';
                }
                cur.push(c);
                prev = c;
                continue;
            }
            prev = c;
            if cur.is_empty() {
                continue;
            }
            let tok = std::mem::take(&mut cur);
            if dollar_prefixed {
                dollar_prefixed = false;
                continue;
            }
            // Assertion labels: `p_<name>`, `assert_<i>_<name>`,
            // `cover_<i>_<name>`, and the Yosys-subset form `a_<i>_<name>`.
            // `a_` alone is too broad to blanket-exclude (it could be a real
            // signal), so require a digit after it.
            let is_label = tok.starts_with("p_")
                || tok.starts_with("assert_")
                || tok.starts_with("cover_")
                || (tok.strip_prefix("a_").is_some_and(|r| {
                    r.chars().next().is_some_and(|c| c.is_ascii_digit())
                }));
            let is_number = tok.chars().next().is_some_and(|c| c.is_ascii_digit());
            if is_label
                || is_number
                || SVA_RESERVED.contains(&tok.as_str())
                || found.contains(&tok)
            {
                continue;
            }
            found.push(tok);
        }
    }

    // clk and rst_n lead the port list; the rest keep first-appearance order so
    // the emitted module is byte-stable across runs.
    let mut ports: Vec<String> = Vec::with_capacity(found.len());
    for lead in ["clk", "rst_n"] {
        if found.iter().any(|s| s == lead) {
            ports.push(lead.to_string());
        }
    }
    for s in found {
        if s != "clk" && s != "rst_n" {
            ports.push(s);
        }
    }
    ports
}

pub fn build_behavior_sva_v2_file(behaviors: &[Behavior<'_>]) -> String {
    let mut body = String::with_capacity(behaviors.len() * 640);
    for (i, b) in behaviors.iter().enumerate() {
        if i > 0 {
            body.push('\n');
        }
        body.push_str(&build_behavior_sva_v2_block(b, i));
    }

    // SystemVerilog forbids `property` at file scope: it must live in a module,
    // interface, or checker. Emitting it bare produced a file that no formal
    // tool could read -- yosys rejects it with
    // "syntax error, unexpected TOK_PROPERTY", and since SymbiYosys uses yosys
    // as its frontend, this bundle could never have been formally checked. The
    // properties now sit in a module whose ports are the signals they
    // reference, which is `bind`-able onto the DUT.
    let ports = collect_sva_signals(&body);

    let mut out = String::with_capacity(1024 + body.len() + ports.len() * 24);
    out.push_str(HEADER_V2);
    out.push_str("module behavior_sva_v2 (\n");
    if ports.is_empty() {
        out.push_str("    // no signals referenced\n");
    } else {
        for (i, p) in ports.iter().enumerate() {
            let comma = if i + 1 == ports.len() { "" } else { "," };
            out.push_str(&format!("    input wire {}{}\n", p, comma));
        }
    }
    out.push_str(");\n\n");
    out.push_str(&body);
    out.push_str("\nendmodule\n");
    out.push_str(FOOTER_V2);
    out
}

const HEADER_V2: &str = "\
// ============================================================================
// Behavior-DSL SystemVerilog Assertions (v2)
// Generated by t27c gen-behavior-sva-v2 (Wave 37, R-BV-1, Closes #775)
//
// Extensions: multi-clause antecedents, ##N delay, s_eventually
// Vocabulary ported from gHashTag/vibee-lang src/vibeec/verilog_codegen.zig
// (lines 2415-2531). Original author: Dmitrii Vasilev.
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`timescale 1ns / 1ps
`default_nettype none

";

const FOOTER_V2: &str = "\
\n`default_nettype wire
// ============================================================================
// End of behavior SVA v2 block.
// ============================================================================
";

/// Build a companion SVA module that binds to a DUT module via `bind`.
///
/// Emits:
/// ```systemverilog
/// module <dut>_sva (
///     input wire clk,
///     input wire rst_n
/// );
///     // ... SVA properties, asserts, covers ...
/// endmodule
///
/// bind <dut> <dut>_sva sva_inst (.*);
/// ```
///
/// This is the canonical way to attach SVA assertions to a module without
/// modifying the module itself. The `bind` statement makes the DUT signals
/// visible inside the SVA module.
pub fn build_behavior_sva_bind_block(
    dut_module_name: &str,
    behaviors: &[Behavior<'_>],
) -> String {
    if behaviors.is_empty() {
        return String::new();
    }
    let sva_module_name = format!("{}_sva", dut_module_name);
    let mut out = String::with_capacity(1024 + behaviors.len() * 640);
    out.push_str(&format!(
        "\n// ============================================================================\n\
         // Companion SVA module for {} (bind-based, Wave 38, R-BV-2)\n\
         // Generated by t27c gen-verilog --with-sva\n\
         // ============================================================================\n\n",
        dut_module_name
    ));
    out.push_str("`timescale 1ns / 1ps\n");
    out.push_str("`default_nettype none\n\n");
    out.push_str(&format!(
        "module {} (\n    input wire clk,\n    input wire rst_n\n);\n",
        sva_module_name
    ));
    for (i, b) in behaviors.iter().enumerate() {
        let timing = crate::behavior_sva::parse_when_clause(b.when);
        let antecedent = parse_given_clause_v2(b.given);
        let consequent = parse_then_clause_v2(b.then);
        let consequent_sva = match &consequent {
            ConsequentV2::Plain(expr) => expr.clone(),
            ConsequentV2::Delayed { cycles, expr } => {
                if *cycles == 0 {
                    expr.clone()
                } else {
                    format!("##{} {}", cycles, expr)
                }
            }
            ConsequentV2::Eventually(expr) => format!("s_eventually {}", expr),
        };
        out.push_str(&format!(
            "\n    // Behavior: {} (Given: {}, When: {}, Then: {})\n",
            b.name, b.given, b.when, b.then
        ));
        out.push_str(&format!("    property p_{};\n", b.name));
        out.push_str(&format!(
            "        @({}) disable iff (!rst_n)\n",
            timing
        ));
        out.push_str(&format!("        {} |-> {};\n", antecedent, consequent_sva));
        out.push_str("    endproperty\n");
        out.push_str(&format!(
            "    assert_{}_{}: assert property (p_{})\n",
            i, b.name, b.name
        ));
        out.push_str(&format!(
            "        else $error(\"Assertion failed: {}\");\n",
            b.name
        ));
        out.push_str(&format!(
            "    cover_{}_{}: cover property (p_{});\n",
            i, b.name, b.name
        ));
    }
    out.push_str(&format!("\nendmodule\n\nbind {} {} sva_inst (.*);\n", dut_module_name, sva_module_name));
    out.push_str("\n`default_nettype wire\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_single_keyword_passthrough() {
        assert_eq!(parse_given_clause_v2("running"), "running");
        assert_eq!(parse_given_clause_v2("valid"), "valid_in");
        assert_eq!(parse_given_clause_v2("ready"), "ready");
    }

    #[test]
    fn given_two_clause_and() {
        assert_eq!(
            parse_given_clause_v2("valid and ready"),
            "(valid_in && ready)"
        );
    }

    #[test]
    fn given_three_clause_and() {
        assert_eq!(
            parse_given_clause_v2("valid and ready and busy"),
            "(valid_in && ready && busy)"
        );
    }

    #[test]
    fn given_comma_separator() {
        assert_eq!(
            parse_given_clause_v2("running, active"),
            "(running && active)"
        );
    }

    #[test]
    fn given_double_amp_separator() {
        assert_eq!(
            parse_given_clause_v2("valid && ready"),
            "(valid_in && ready)"
        );
    }

    #[test]
    fn given_default_empty() {
        assert_eq!(parse_given_clause_v2(""), "1'b1");
    }

    #[test]
    fn given_reset_variants() {
        assert_eq!(parse_given_clause_v2("not reset"), "rst_n");
        assert_eq!(parse_given_clause_v2("reset"), "!rst_n");
    }

    #[test]
    fn given_fifo_not_full() {
        assert_eq!(parse_given_clause_v2("fifo not full"), "!full");
    }

    #[test]
    fn given_passthrough_unknown_signal() {
        assert_eq!(parse_given_clause_v2("irq_status[2]"), "irq_status[2]");
    }

    #[test]
    fn given_mixed_known_and_unknown() {
        assert_eq!(
            parse_given_clause_v2("valid and custom_signal"),
            "(valid_in && custom_signal)"
        );
    }

    #[test]
    fn then_plain_keyword() {
        assert_eq!(
            parse_then_clause_v2("increment count"),
            ConsequentV2::Plain("(count == $past(count) + 1)".to_string())
        );
    }

    #[test]
    fn then_delay_after_cycles() {
        assert_eq!(
            parse_then_clause_v2("after 3 cycles done"),
            ConsequentV2::Delayed {
                cycles: 3,
                expr: "done".to_string()
            }
        );
    }

    #[test]
    fn then_delay_hash_hash() {
        assert_eq!(
            parse_then_clause_v2("##5 valid_out"),
            ConsequentV2::Delayed {
                cycles: 5,
                expr: "valid_out".to_string()
            }
        );
    }

    #[test]
    fn then_eventually_keyword() {
        assert_eq!(
            parse_then_clause_v2("eventually done"),
            ConsequentV2::Eventually("done".to_string())
        );
    }

    #[test]
    fn then_eventually_liveness() {
        assert_eq!(
            parse_then_clause_v2("liveness check for done"),
            ConsequentV2::Eventually("done".to_string())
        );
    }

    #[test]
    fn then_default_fallback() {
        assert_eq!(
            parse_then_clause_v2("something"),
            ConsequentV2::Plain("something".to_string())
        );
    }

    #[test]
    fn then_delay_with_increment() {
        assert_eq!(
            parse_then_clause_v2("after 2 cycles increment count"),
            ConsequentV2::Delayed {
                cycles: 2,
                expr: "(count == $past(count) + 1)".to_string()
            }
        );
    }

    #[test]
    fn block_plain_consequent() {
        let b = Behavior {
            name: "plain_test",
            given: "valid and ready",
            when: "rising edge",
            then: "increment count",
        };
        let block = build_behavior_sva_v2_block(&b, 0);
        assert!(block.contains("property p_plain_test;"));
        assert!(block.contains("(valid_in && ready) |-> (count == $past(count) + 1);"));
        assert!(block.contains("endproperty"));
        assert!(block.contains("assert_0_plain_test:"));
        assert!(block.contains("cover_0_plain_test:"));
    }

    #[test]
    fn block_delay_consequent() {
        let b = Behavior {
            name: "delayed",
            given: "start",
            when: "posedge clk",
            then: "after 3 cycles done",
        };
        let block = build_behavior_sva_v2_block(&b, 1);
        assert!(block.contains("start |-> ##3 done;"));
    }

    #[test]
    fn block_eventually_consequent() {
        let b = Behavior {
            name: "live",
            given: "start",
            when: "posedge clk",
            then: "eventually done",
        };
        let block = build_behavior_sva_v2_block(&b, 2);
        assert!(block.contains("start |-> s_eventually done;"));
    }

    #[test]
    fn file_header_footer_v2() {
        let behaviors = [Behavior {
            name: "x",
            given: "running",
            when: "rising",
            then: "done",
        }];
        let file = build_behavior_sva_v2_file(&behaviors);
        assert!(file.contains("`timescale 1ns / 1ps"));
        assert!(file.contains("`default_nettype none"));
        assert!(file.contains("`default_nettype wire"));
        assert!(file.contains("gen-behavior-sva-v2"));
        assert!(file.contains("Wave 37"));
    }

    #[test]
    fn file_multi_behavior() {
        let behaviors = [
            Behavior {
                name: "a",
                given: "valid",
                when: "rising",
                then: "done",
            },
            Behavior {
                name: "b",
                given: "ready",
                when: "falling",
                then: "after 2 cycles busy",
            },
        ];
        let file = build_behavior_sva_v2_file(&behaviors);
        assert!(file.contains("property p_a;"));
        assert!(file.contains("property p_b;"));
        assert!(file.contains("assert_0_a"));
        assert!(file.contains("assert_1_b"));
        assert!(file.contains("##2"));
    }

    #[test]
    fn split_conjunctions_basic() {
        let atoms = split_conjunctions("valid and ready");
        assert_eq!(atoms, vec!["valid", "ready"]);
    }

    #[test]
    fn split_conjunctions_comma() {
        let atoms = split_conjunctions("running, active");
        assert_eq!(atoms, vec!["running", "active"]);
    }

    #[test]
    fn split_conjunctions_double_amp() {
        let atoms = split_conjunctions("valid && ready");
        assert_eq!(atoms, vec!["valid", "ready"]);
    }

    #[test]
    fn extract_delay_after_n_cycles() {
        assert_eq!(extract_delay_cycles("after 3 cycles done"), Some(3));
        assert_eq!(extract_delay_cycles("after 10 cycles busy"), Some(10));
    }

    #[test]
    fn extract_delay_hash_hash() {
        assert_eq!(extract_delay_cycles("##5 valid_out"), Some(5));
    }

    #[test]
    fn extract_delay_none() {
        assert_eq!(extract_delay_cycles("increment count"), None);
    }

    #[test]
    fn bind_block_empty_behaviors_returns_empty() {
        let result = build_behavior_sva_bind_block("my_module", &[]);
        assert!(result.is_empty(), "empty behaviors should produce no output");
    }

    #[test]
    fn bind_block_single_plain() {
        let b = Behavior {
            name: "check",
            given: "valid and ready",
            when: "posedge clk",
            then: "done",
        };
        let result = build_behavior_sva_bind_block("my_module", &[b]);
        assert!(result.contains("module my_module_sva"));
        assert!(result.contains("input wire clk"));
        assert!(result.contains("input wire rst_n"));
        assert!(result.contains("endmodule"));
        assert!(result.contains("bind my_module my_module_sva sva_inst (.*);"));
        assert!(result.contains("property p_check"));
        assert!(result.contains("(valid_in && ready) |-> done"));
        assert!(result.contains("assert_0_check"));
        assert!(result.contains("cover_0_check"));
    }

    #[test]
    fn bind_block_delay_consequent() {
        let b = Behavior {
            name: "delayed",
            given: "start",
            when: "rising",
            then: "after 3 cycles done",
        };
        let result = build_behavior_sva_bind_block("top", &[b]);
        assert!(result.contains("top_sva"));
        assert!(result.contains("start |-> ##3 done"));
        assert!(result.contains("bind top top_sva sva_inst (.*);"));
    }

    #[test]
    fn bind_block_eventually_consequent() {
        let b = Behavior {
            name: "live",
            given: "start",
            when: "rising",
            then: "eventually done",
        };
        let result = build_behavior_sva_bind_block("top", &[b]);
        assert!(result.contains("start |-> s_eventually done"));
    }

    #[test]
    fn bind_block_multi_behavior() {
        let behaviors = [
            Behavior {
                name: "a",
                given: "running",
                when: "rising",
                then: "done",
            },
            Behavior {
                name: "b",
                given: "valid and ready",
                when: "rising",
                then: "after 2 cycles busy",
            },
        ];
        let result = build_behavior_sva_bind_block("dut", &behaviors);
        assert!(result.contains("property p_a"));
        assert!(result.contains("property p_b"));
        assert!(result.contains("assert_0_a"));
        assert!(result.contains("assert_1_b"));
        assert!(result.contains("##2"));
    }

    #[test]
    fn bind_block_module_name_used_in_bind() {
        let b = Behavior {
            name: "x",
            given: "running",
            when: "rising",
            then: "done",
        };
        let result = build_behavior_sva_bind_block("my_fancy_module", &[b]);
        assert!(result.contains("module my_fancy_module_sva"));
        assert!(result.contains("bind my_fancy_module my_fancy_module_sva sva_inst"));
    }
}

#[cfg(test)]
mod sva_module_wrapper_tests {
    use super::*;

    // SystemVerilog forbids `property` at file scope. The emitter produced it
    // bare for months; yosys rejects that with "unexpected TOK_PROPERTY", and
    // since SymbiYosys uses yosys as its frontend, the bundle could never have
    // been formally checked by the open-source flow.
    #[test]
    fn properties_are_wrapped_in_a_module() {
        let b = Behavior {
            name: "engine_busy_safety",
            given: "running",
            when: "posedge clk",
            then: "set full",
        };
        let file = build_behavior_sva_v2_file(&[b]);
        let m = file.find("module behavior_sva_v2").expect("module header");
        let p = file.find("property p_").expect("property decl");
        let e = file.find("endmodule").expect("endmodule");
        assert!(m < p, "property must come after the module header");
        assert!(p < e, "property must come before endmodule");
    }

    #[test]
    fn referenced_signals_become_ports() {
        let body = "property p_x;\n    @(posedge clk) disable iff (!rst_n)\n    running |-> full;\nendproperty\n";
        let ports = collect_sva_signals(body);
        for want in ["clk", "rst_n", "running", "full"] {
            assert!(ports.iter().any(|p| p == want), "missing port {want}");
        }
    }

    // `$error("...")` tokenises to a bare `error` once `$` is a separator, which
    // would declare a system task as a module port.
    #[test]
    fn system_tasks_are_not_ports() {
        let body = "assert_0_x: assert property (p_x)\n    else $error(\"Assertion failed: x\");\n";
        let ports = collect_sva_signals(body);
        assert!(!ports.iter().any(|p| p == "error"), "$error leaked: {ports:?}");
    }

    // The message text inside $error(...) names the behavior, not a signal.
    #[test]
    fn string_literal_contents_are_not_ports() {
        let body = "else $error(\"Assertion failed: engine_busy_safety\");\n";
        let ports = collect_sva_signals(body);
        assert!(ports.is_empty(), "string contents leaked: {ports:?}");
    }

    #[test]
    fn comments_are_not_ports() {
        let body = "// Behavior: irq_clear_on_reset\n// Given: reset inactive\nrst_n |-> full;\n";
        let ports = collect_sva_signals(body);
        assert!(!ports.iter().any(|p| p == "Behavior" || p == "Given"));
        assert!(ports.iter().any(|p| p == "rst_n"));
    }

    #[test]
    fn sva_keywords_and_labels_are_not_ports() {
        let body = "property p_a;\n@(posedge clk) disable iff (!rst_n)\na |-> s_eventually b;\nendproperty\ncover_0_a: cover property (p_a);\n";
        let ports = collect_sva_signals(body);
        for bad in ["property", "endproperty", "posedge", "disable", "iff", "s_eventually", "cover"] {
            assert!(!ports.iter().any(|p| p == bad), "keyword {bad} leaked");
        }
        assert!(!ports.iter().any(|p| p.starts_with("p_") || p.starts_with("cover_")));
    }

    #[test]
    fn clk_and_rst_n_lead_the_port_list() {
        let body = "zzz_signal |-> @(posedge clk) disable iff (!rst_n) aaa_signal;\n";
        let ports = collect_sva_signals(body);
        assert_eq!(ports[0], "clk");
        assert_eq!(ports[1], "rst_n");
    }

    #[test]
    fn port_order_is_stable_across_runs() {
        let body = "a |-> @(posedge clk) disable iff (!rst_n) b;\n";
        assert_eq!(collect_sva_signals(body), collect_sva_signals(body));
    }

    #[test]
    fn numeric_literals_are_not_ports() {
        let body = "rst_n |-> ##1 full;\n";
        let ports = collect_sva_signals(body);
        assert!(!ports.iter().any(|p| p == "1"));
        assert!(ports.iter().any(|p| p == "full"));
    }
}

/// Why a behavior could not be expressed as an immediate assertion.
#[derive(Debug, Clone, PartialEq)]
pub enum YosysSkip {
    /// `s_eventually` is a liveness property: it asserts that something happens
    /// at some unbounded future time. An immediate assertion evaluates in one
    /// cycle and cannot express it. This is a real expressiveness limit, not a
    /// gap in the translation.
    Liveness,
}

/// Translate one behavior into a Yosys-checkable immediate assertion.
///
/// Yosys's `read_verilog -sv -formal` accepts **neither** named `property`
/// blocks **nor** inline `assert property (@(posedge clk) ...)`. It accepts
/// immediate assertions inside `always`. This emitter targets that subset, so
/// the properties this project already writes can actually be proved by the
/// open-source flow rather than merely emitted for one that cannot read them.
///
/// Translation rules, all measured against Yosys 0.63:
/// - `a |-> b`        becomes `assert (!a || b)`
/// - `a |-> ##N b`    becomes `assert (!$past(a, N) || b)`
/// - `a |-> s_eventually b` has **no** immediate form; reported as a skip.
///
/// The reset guard is `rst_n && $past(rst_n)` for delayed forms: guarding only
/// on the current cycle lets an assertion fire on the first cycle after reset,
/// when the antecedent's history predates the reset. That produced a genuine
/// counterexample while developing this, and the prover was right.
pub fn build_behavior_immediate_assert(
    behavior: &Behavior<'_>,
    index: usize,
) -> Result<String, YosysSkip> {
    let timing = parse_when_clause(behavior.when);
    let antecedent = parse_given_clause_v2(behavior.given);
    let consequent = parse_then_clause_v2(behavior.then);

    let (guard, ante, cons) = match &consequent {
        ConsequentV2::Eventually(_) => return Err(YosysSkip::Liveness),
        ConsequentV2::Plain(expr) => ("rst_n".to_string(), antecedent, expr.clone()),
        ConsequentV2::Delayed { cycles, expr } if *cycles == 0 => {
            ("rst_n".to_string(), antecedent, expr.clone())
        }
        ConsequentV2::Delayed { cycles, expr } => (
            "rst_n && $past(rst_n)".to_string(),
            format!("$past({}, {})", antecedent, cycles),
            expr.clone(),
        ),
    };

    let mut out = String::with_capacity(480);
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("// Behavior: {}\n", behavior.name));
    out.push_str(&format!("// Given:    {}\n", behavior.given));
    out.push_str(&format!("// When:     {}\n", behavior.when));
    out.push_str(&format!("// Then:     {}\n", behavior.then));
    out.push_str("// ----------------------------------------------------------------------------\n");
    out.push_str(&format!("always @({}) begin\n", timing));
    out.push_str(&format!("    if ({}) begin\n", guard));
    out.push_str(&format!(
        "        a_{}_{}: assert (!({}) || ({}));\n",
        index, behavior.name, ante, cons
    ));
    out.push_str("    end\nend\n");
    Ok(out)
}

/// Emit a complete Yosys-checkable module for a behavior set.
///
/// Returns the file text plus the behaviors that could not be translated, so a
/// caller can report them rather than let them vanish. Silently dropping the
/// untranslatable cases would reproduce exactly the failure this project keeps
/// finding: a gate that reports success over a reduced domain without saying
/// the domain shrank.
pub fn build_behavior_yosys_file(behaviors: &[Behavior<'_>]) -> (String, Vec<(String, YosysSkip)>) {
    let mut body = String::new();
    let mut skipped: Vec<(String, YosysSkip)> = Vec::new();
    for (i, b) in behaviors.iter().enumerate() {
        match build_behavior_immediate_assert(b, i) {
            Ok(block) => {
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(&block);
            }
            Err(skip) => skipped.push((b.name.to_string(), skip)),
        }
    }

    let ports = collect_sva_signals(&body);
    let mut out = String::with_capacity(1024 + body.len());
    out.push_str("// ============================================================================\n");
    out.push_str("// Behavior-DSL immediate assertions -- Yosys-checkable subset\n");
    out.push_str("// Generated by t27c gen-behavior-sva-yosys\n");
    out.push_str("//\n");
    out.push_str("// Yosys read_verilog -sv -formal accepts neither `property ... endproperty`\n");
    out.push_str("// nor inline `assert property (@(posedge clk) ...)`. It accepts immediate\n");
    out.push_str("// assertions inside `always`, which is what this file emits, so these\n");
    out.push_str("// properties can actually be proved:\n");
    out.push_str("//\n");
    out.push_str("//   yosys -p 'read_verilog -sv -formal <this>; prep -top behavior_yosys; \\\n");
    out.push_str("//             async2sync; chformal -lower; \\\n");
    out.push_str("//             sat -verify -prove-asserts -seq 8 -tempinduct'\n");
    if !skipped.is_empty() {
        out.push_str("//\n");
        out.push_str("// NOT TRANSLATED (liveness has no immediate form):\n");
        for (name, _) in &skipped {
            out.push_str(&format!("//   - {} (s_eventually)\n", name));
        }
    }
    out.push_str("// phi^2 + 1/phi^2 = 3 | TRINITY\n");
    out.push_str("// ============================================================================\n\n");
    out.push_str("`default_nettype none\n\n");
    out.push_str("module behavior_yosys (\n");
    if ports.is_empty() {
        out.push_str("    // no signals referenced\n");
    } else {
        for (i, p) in ports.iter().enumerate() {
            let comma = if i + 1 == ports.len() { "" } else { "," };
            out.push_str(&format!("    input wire {}{}\n", p, comma));
        }
    }
    out.push_str(");\n\n");
    out.push_str(&body);
    out.push_str("\nendmodule\n\n`default_nettype wire\n");
    (out, skipped)
}

#[cfg(test)]
mod yosys_subset_tests {
    use super::*;

    fn beh<'a>(name: &'a str, given: &'a str, then: &'a str) -> Behavior<'a> {
        Behavior { name, given, when: "posedge clk", then }
    }

    // Yosys accepts immediate assertions inside `always`, and nothing else.
    #[test]
    fn overlapping_implication_becomes_a_disjunction() {
        let b = beh("busy_safety", "running", "set full");
        let out = build_behavior_immediate_assert(&b, 0).expect("translatable");
        assert!(out.contains("always @(posedge clk)"), "{out}");
        assert!(out.contains("assert (!(running) || (full));"), "{out}");
        assert!(!out.contains("property"), "must not emit a property block");
        assert!(!out.contains("|->"), "must not emit the SVA implication operator");
    }

    #[test]
    fn delayed_implication_uses_past_with_depth() {
        let b = beh("after_three", "running", "after 3 cycles set full");
        let out = build_behavior_immediate_assert(&b, 1).expect("translatable");
        assert!(out.contains("$past(running, 3)"), "{out}");
    }

    // Guarding only on the current rst_n lets a delayed assertion fire on the
    // first post-reset cycle, when the antecedent's history predates reset.
    // That produced a genuine counterexample from the prover during
    // development; the prover was right and the guard was wrong.
    #[test]
    fn delayed_form_guards_on_past_reset_too() {
        let b = beh("after_one", "running", "after 1 cycles set full");
        let out = build_behavior_immediate_assert(&b, 0).expect("translatable");
        assert!(out.contains("rst_n && $past(rst_n)"), "{out}");
    }

    #[test]
    fn zero_cycle_delay_does_not_use_past() {
        let b = beh("immediate", "running", "after 0 cycles set full");
        let out = build_behavior_immediate_assert(&b, 0).expect("translatable");
        assert!(!out.contains("$past"), "{out}");
    }

    // Liveness is a real expressiveness limit, not a translation gap: an
    // immediate assertion evaluates in one cycle.
    #[test]
    fn liveness_is_reported_not_silently_dropped() {
        let b = beh("eventually_done", "valid", "eventually set full");
        assert_eq!(
            build_behavior_immediate_assert(&b, 0),
            Err(YosysSkip::Liveness)
        );
    }

    #[test]
    fn skipped_behaviors_are_returned_to_the_caller() {
        let bs = [
            beh("ok_one", "running", "set full"),
            beh("live_one", "valid", "eventually set full"),
        ];
        let (file, skipped) = build_behavior_yosys_file(&bs);
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0].0, "live_one");
        // and the file says so in a comment, so the artefact is self-describing
        assert!(file.contains("NOT TRANSLATED"), "{file}");
        assert!(file.contains("live_one"), "{file}");
    }

    // Regression: the port collector was written for the property form and
    // leaked `always`, `begin`, `if`, `end` and the `a_<i>_` labels as ports.
    #[test]
    fn procedural_keywords_and_labels_are_not_ports() {
        let bs = [beh("busy_safety", "running", "set full")];
        let (file, _) = build_behavior_yosys_file(&bs);
        let header = &file[file.find("module behavior_yosys").unwrap()
            ..file.find(");").unwrap()];
        for bad in ["always", "begin", "end", "if", "a_0_busy_safety"] {
            assert!(!header.contains(&format!("input wire {bad}")), "leaked {bad}: {header}");
        }
        for want in ["clk", "rst_n", "running", "full"] {
            assert!(header.contains(&format!("input wire {want}")), "missing {want}: {header}");
        }
    }

    #[test]
    fn emitted_module_is_default_nettype_safe() {
        let (file, _) = build_behavior_yosys_file(&[beh("x", "running", "set full")]);
        assert!(file.contains("`default_nettype none"));
        assert!(file.trim_end().ends_with("`default_nettype wire"));
    }

    #[test]
    fn an_all_liveness_set_yields_no_assertions_but_still_says_why() {
        let bs = [beh("l1", "valid", "eventually set full")];
        let (file, skipped) = build_behavior_yosys_file(&bs);
        assert_eq!(skipped.len(), 1);
        assert!(!file.contains("assert ("), "no assertions expected");
        assert!(file.contains("NOT TRANSLATED"));
    }
}
