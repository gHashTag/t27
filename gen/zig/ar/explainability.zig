// AUTO-GENERATED from specs/ar/explainability.t27 — DO NOT EDIT
// Ring: 18 | Module: Explainability | phi^2 + 1/phi^2 = 3
// Generator: PHI LOOP manual codegen (bootstrap unavailable)

const std = @import("std");
const ternary_logic = @import("ternary_logic.zig");

const Trit = ternary_logic.Trit;
const K_FALSE = ternary_logic.K_FALSE;
const K_UNKNOWN = ternary_logic.K_UNKNOWN;
const K_TRUE = ternary_logic.K_TRUE;

// ═══════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════

pub const MAX_SUMMARY_SIZE: usize = 5;
pub const MAX_PREDICATE_NAME: usize = 64;
pub const MAX_DERIVATION_STEPS: usize = 10;

// ═══════════════════════════════════════════════════════════════
// Forward-declared dependency types from ar::proof_trace
// ═══════════════════════════════════════════════════════════════

/// A single step in a derivation proof trace.
/// Forward-declared from ar::proof_trace (DerivationStep).
pub const DerivationStep = struct {
    rule_name: [MAX_PREDICATE_NAME]u8,
    rule_name_len: u8,
    input_trits: [3]Trit,
    output_trit: Trit,
    confidence: u16, // GF16
};

/// A proof trace recording a sequence of derivation steps.
/// Forward-declared from ar::proof_trace (ProofTrace).
pub const ProofTrace = struct {
    steps: [MAX_DERIVATION_STEPS]DerivationStep,
    step_count: u8,
};

// ═══════════════════════════════════════════════════════════════
// Forward-declared dependency types from ar::datalog_engine
// ═══════════════════════════════════════════════════════════════

/// A Horn clause in the Datalog engine.
/// Forward-declared from ar::datalog_engine (HornClause).
pub const HornClause = struct {
    head_predicate: [MAX_PREDICATE_NAME]u8,
    head_pred_len: u8,
    head_args: [3]Trit,
    body_count: u8,
    body_predicates: [3][MAX_PREDICATE_NAME]u8,
    body_pred_lens: [3]u8,
    body_args: [3][3]Trit,
};

/// The Datalog engine holding a set of Horn clauses and derived facts.
/// Forward-declared from ar::datalog_engine (DatalogEngine).
pub const DatalogEngine = struct {
    clauses: [64]HornClause,
    clause_count: u8,
    facts: [64]FactId,
    fact_confidences: [64]u16,
    fact_count: u8,
};

// ═══════════════════════════════════════════════════════════════
// GF16 alias (from numeric::gf16)
// ═══════════════════════════════════════════════════════════════

pub const GF16 = u16;

// ═══════════════════════════════════════════════════════════════
// Explainability Types
// ═══════════════════════════════════════════════════════════════

/// Identifies a fact by its predicate name and ternary arguments.
pub const FactId = struct {
    predicate: [MAX_PREDICATE_NAME]u8,
    args: [3]Trit,
};

/// The format style for human-readable explanations.
pub const FormatStyle = enum(u8) {
    natural = 0,
    fitch = 1,
    compact = 2,
};

/// A full explanation of a derived fact, including the proof trace,
/// format style, human-readable string, confidence, and step count.
pub const Explanation = struct {
    trace: ProofTrace,
    style: FormatStyle,
    human_readable: [512]u8,
    confidence: GF16,
    step_count: u8,
};

/// A summary of the top facts in a proof trace.
pub const Summary = struct {
    top_facts: [MAX_SUMMARY_SIZE]FactId,
    top_confidence: [MAX_SUMMARY_SIZE]GF16,
    fact_count: u8,
};

// ═══════════════════════════════════════════════════════════════
// Helper: create_trace / append_step (from ar::proof_trace)
// ═══════════════════════════════════════════════════════════════

/// Create an empty proof trace.
pub fn create_trace() ProofTrace {
    return ProofTrace{
        .steps = undefined,
        .step_count = 0,
    };
}

/// Append a derivation step to a proof trace (bounded by MAX_DERIVATION_STEPS).
pub fn append_step(trace: *ProofTrace, step: DerivationStep) void {
    if (trace.step_count < MAX_DERIVATION_STEPS) {
        trace.steps[trace.step_count] = step;
        trace.step_count += 1;
    }
}

// ═══════════════════════════════════════════════════════════════
// Internal helpers
// ═══════════════════════════════════════════════════════════════

/// Copy a null-terminated or length-bounded name into a fixed buffer.
fn copy_name(dst: *[MAX_PREDICATE_NAME]u8, src: []const u8) u8 {
    const len = if (src.len > MAX_PREDICATE_NAME) MAX_PREDICATE_NAME else src.len;
    for (src[0..len], 0..) |c, i| {
        dst[i] = c;
    }
    // Zero-fill remainder
    var i: usize = len;
    while (i < MAX_PREDICATE_NAME) : (i += 1) {
        dst[i] = 0;
    }
    return @intCast(len);
}

/// Compare two predicate names (fixed-size buffers).
fn names_equal(a: [MAX_PREDICATE_NAME]u8, b: [MAX_PREDICATE_NAME]u8) bool {
    for (a, b) |ca, cb| {
        if (ca != cb) return false;
    }
    return true;
}

/// Compare two FactIds for equality.
fn fact_id_equal(a: FactId, b: FactId) bool {
    if (!names_equal(a.predicate, b.predicate)) return false;
    for (a.args, b.args) |aa, ba| {
        if (aa != ba) return false;
    }
    return true;
}

/// Format a single derivation step as a human-readable line.
/// Returns the number of bytes written.
fn format_step_natural(step: DerivationStep, buf: []u8, offset: usize) usize {
    var pos = offset;
    // Write "Step: <rule_name> ("
    const prefix = "Step: ";
    for (prefix) |c| {
        if (pos >= buf.len) return pos - offset;
        buf[pos] = c;
        pos += 1;
    }
    // Write rule name
    var name_len: usize = step.rule_name_len;
    if (name_len > MAX_PREDICATE_NAME) name_len = MAX_PREDICATE_NAME;
    for (step.rule_name[0..name_len]) |c| {
        if (pos >= buf.len) return pos - offset;
        buf[pos] = c;
        pos += 1;
    }
    // Write " -> "
    const arrow = " -> ";
    for (arrow) |c| {
        if (pos >= buf.len) return pos - offset;
        buf[pos] = c;
        pos += 1;
    }
    // Write output trit symbol
    const trit_char: u8 = switch (step.output_trit) {
        K_TRUE => 'T',
        K_FALSE => 'F',
        K_UNKNOWN => '?',
        else => '?',
    };
    if (pos < buf.len) {
        buf[pos] = trit_char;
        pos += 1;
    }
    // Newline
    if (pos < buf.len) {
        buf[pos] = '\n';
        pos += 1;
    }
    return pos - offset;
}

/// Format a derivation step in Fitch-style notation.
fn format_step_fitch(step: DerivationStep, idx: u8, buf: []u8, offset: usize) usize {
    var pos = offset;
    // Write index digit
    if (pos < buf.len) {
        buf[pos] = '0' + (idx % 10);
        pos += 1;
    }
    const sep = ". ";
    for (sep) |c| {
        if (pos >= buf.len) return pos - offset;
        buf[pos] = c;
        pos += 1;
    }
    // Write rule name
    var name_len: usize = step.rule_name_len;
    if (name_len > MAX_PREDICATE_NAME) name_len = MAX_PREDICATE_NAME;
    for (step.rule_name[0..name_len]) |c| {
        if (pos >= buf.len) return pos - offset;
        buf[pos] = c;
        pos += 1;
    }
    // Write " |- "
    const turnstile = " |- ";
    for (turnstile) |c| {
        if (pos >= buf.len) return pos - offset;
        buf[pos] = c;
        pos += 1;
    }
    const trit_char: u8 = switch (step.output_trit) {
        K_TRUE => 'T',
        K_FALSE => 'F',
        K_UNKNOWN => '?',
        else => '?',
    };
    if (pos < buf.len) {
        buf[pos] = trit_char;
        pos += 1;
    }
    if (pos < buf.len) {
        buf[pos] = '\n';
        pos += 1;
    }
    return pos - offset;
}

/// Format a derivation step in compact notation.
fn format_step_compact(step: DerivationStep, buf: []u8, offset: usize) usize {
    var pos = offset;
    // Write rule_name:trit;
    var name_len: usize = step.rule_name_len;
    if (name_len > MAX_PREDICATE_NAME) name_len = MAX_PREDICATE_NAME;
    for (step.rule_name[0..name_len]) |c| {
        if (pos >= buf.len) return pos - offset;
        buf[pos] = c;
        pos += 1;
    }
    if (pos < buf.len) {
        buf[pos] = ':';
        pos += 1;
    }
    const trit_char: u8 = switch (step.output_trit) {
        K_TRUE => 'T',
        K_FALSE => 'F',
        K_UNKNOWN => '?',
        else => '?',
    };
    if (pos < buf.len) {
        buf[pos] = trit_char;
        pos += 1;
    }
    if (pos < buf.len) {
        buf[pos] = ';';
        pos += 1;
    }
    return pos - offset;
}

// ═══════════════════════════════════════════════════════════════
// Public Functions
// ═══════════════════════════════════════════════════════════════

/// Explain a fact by searching the engine for its derivation,
/// building a proof trace, and formatting the explanation.
pub fn explain_fact(engine: *const DatalogEngine, fact_id: FactId, style: FormatStyle) Explanation {
    var trace = create_trace();
    var best_confidence: GF16 = 0;

    // Search engine facts for the requested fact_id
    var i: u8 = 0;
    while (i < engine.fact_count) : (i += 1) {
        if (fact_id_equal(engine.facts[i], fact_id)) {
            best_confidence = engine.fact_confidences[i];
            break;
        }
    }

    // Search clauses for derivation steps that conclude this fact
    i = 0;
    while (i < engine.clause_count) : (i += 1) {
        const clause = engine.clauses[i];
        // Check if clause head matches the requested fact
        if (names_equal(clause.head_predicate, fact_id.predicate)) {
            var args_match = true;
            for (clause.head_args, fact_id.args) |ca, fa| {
                if (ca != fa) {
                    args_match = false;
                    break;
                }
            }
            if (args_match) {
                var step: DerivationStep = undefined;
                step.rule_name = clause.head_predicate;
                step.rule_name_len = clause.head_pred_len;
                step.input_trits = fact_id.args;
                step.output_trit = K_TRUE;
                step.confidence = best_confidence;
                append_step(&trace, step);
            }
        }
    }

    // Format the explanation
    var explanation: Explanation = undefined;
    explanation.trace = trace;
    explanation.style = style;
    explanation.confidence = best_confidence;
    explanation.step_count = trace.step_count;
    explanation.human_readable = explain_derivation_chain(trace, style);
    return explanation;
}

/// Format a full derivation chain as a human-readable string
/// given a proof trace and a format style.
pub fn explain_derivation_chain(trace: ProofTrace, style: FormatStyle) [512]u8 {
    var buf: [512]u8 = [_]u8{0} ** 512;
    var pos: usize = 0;

    var i: u8 = 0;
    while (i < trace.step_count) : (i += 1) {
        if (pos >= 512) break;
        const step = trace.steps[i];
        const written = switch (style) {
            .natural => format_step_natural(step, &buf, pos),
            .fitch => format_step_fitch(step, i, &buf, pos),
            .compact => format_step_compact(step, &buf, pos),
        };
        pos += written;
    }
    return buf;
}

/// Summarize a proof trace, extracting the top facts and their
/// confidences, bounded by MAX_SUMMARY_SIZE.
pub fn summarize(trace: ProofTrace) Summary {
    var summary: Summary = undefined;
    summary.fact_count = 0;

    // Zero-initialize top arrays
    for (&summary.top_facts) |*f| {
        f.predicate = [_]u8{0} ** MAX_PREDICATE_NAME;
        f.args = [_]Trit{ K_UNKNOWN, K_UNKNOWN, K_UNKNOWN };
    }
    for (&summary.top_confidence) |*c| {
        c.* = 0;
    }

    // Collect unique facts from trace steps, keeping top by confidence
    var i: u8 = 0;
    while (i < trace.step_count) : (i += 1) {
        const step = trace.steps[i];

        // Build a FactId from the step
        var fid: FactId = undefined;
        fid.predicate = step.rule_name;
        fid.args = step.input_trits;

        // Check if this fact is already in the summary
        var found = false;
        var j: u8 = 0;
        while (j < summary.fact_count) : (j += 1) {
            if (fact_id_equal(summary.top_facts[j], fid)) {
                // Update confidence if higher
                if (step.confidence > summary.top_confidence[j]) {
                    summary.top_confidence[j] = step.confidence;
                }
                found = true;
                break;
            }
        }

        if (!found and summary.fact_count < MAX_SUMMARY_SIZE) {
            summary.top_facts[summary.fact_count] = fid;
            summary.top_confidence[summary.fact_count] = step.confidence;
            summary.fact_count += 1;
        } else if (!found) {
            // Replace the lowest-confidence entry if this one is better
            var min_idx: u8 = 0;
            var min_conf: GF16 = summary.top_confidence[0];
            var k: u8 = 1;
            while (k < MAX_SUMMARY_SIZE) : (k += 1) {
                if (summary.top_confidence[k] < min_conf) {
                    min_conf = summary.top_confidence[k];
                    min_idx = k;
                }
            }
            if (step.confidence > min_conf) {
                summary.top_facts[min_idx] = fid;
                summary.top_confidence[min_idx] = step.confidence;
            }
        }
    }

    return summary;
}

// ═══════════════════════════════════════════════════════════════
// Tests — Conformance vectors from spec
// ═══════════════════════════════════════════════════════════════

fn make_test_step(name: []const u8, out: Trit, conf: u16) DerivationStep {
    var step: DerivationStep = undefined;
    step.rule_name = [_]u8{0} ** MAX_PREDICATE_NAME;
    _ = copy_name(&step.rule_name, name);
    step.rule_name_len = @intCast(name.len);
    step.input_trits = [_]Trit{ K_TRUE, K_FALSE, K_UNKNOWN };
    step.output_trit = out;
    step.confidence = conf;
    return step;
}

fn make_empty_engine() DatalogEngine {
    var engine: DatalogEngine = undefined;
    engine.clause_count = 0;
    engine.fact_count = 0;
    return engine;
}

fn make_fact_id(name: []const u8, args: [3]Trit) FactId {
    var fid: FactId = undefined;
    fid.predicate = [_]u8{0} ** MAX_PREDICATE_NAME;
    _ = copy_name(&fid.predicate, name);
    fid.args = args;
    return fid;
}

test "explain_fact_basic" {
    // Set up an engine with one clause and one matching fact
    var engine = make_empty_engine();

    const fid = make_fact_id("parent", [_]Trit{ K_TRUE, K_FALSE, K_UNKNOWN });

    // Add a matching fact
    engine.facts[0] = fid;
    engine.fact_confidences[0] = 42;
    engine.fact_count = 1;

    // Add a matching clause
    engine.clauses[0].head_predicate = fid.predicate;
    engine.clauses[0].head_pred_len = 6; // "parent"
    engine.clauses[0].head_args = fid.args;
    engine.clauses[0].body_count = 0;
    engine.clause_count = 1;

    const explanation = explain_fact(&engine, fid, .natural);

    // Should have found the derivation step
    try std.testing.expect(explanation.step_count > 0);
    try std.testing.expectEqual(@as(GF16, 42), explanation.confidence);
    try std.testing.expectEqual(FormatStyle.natural, explanation.style);
    // Human readable should contain "Step: parent"
    try std.testing.expect(explanation.human_readable[0] == 'S');
    try std.testing.expect(explanation.human_readable[1] == 't');
}

test "explain_empty_engine" {
    // An empty engine should produce an explanation with zero steps
    var engine = make_empty_engine();

    const fid = make_fact_id("unknown_fact", [_]Trit{ K_UNKNOWN, K_UNKNOWN, K_UNKNOWN });

    const explanation = explain_fact(&engine, fid, .compact);

    try std.testing.expectEqual(@as(u8, 0), explanation.step_count);
    try std.testing.expectEqual(@as(GF16, 0), explanation.confidence);
    try std.testing.expectEqual(FormatStyle.compact, explanation.style);
}

test "summarize_with_steps" {
    // Build a trace with several steps and summarize
    var trace = create_trace();

    const step1 = make_test_step("alpha", K_TRUE, 100);
    const step2 = make_test_step("beta", K_FALSE, 200);
    const step3 = make_test_step("gamma", K_TRUE, 150);

    append_step(&trace, step1);
    append_step(&trace, step2);
    append_step(&trace, step3);

    try std.testing.expectEqual(@as(u8, 3), trace.step_count);

    const summary = summarize(trace);

    try std.testing.expectEqual(@as(u8, 3), summary.fact_count);
    // Confidences should match the steps
    try std.testing.expectEqual(@as(GF16, 100), summary.top_confidence[0]);
    try std.testing.expectEqual(@as(GF16, 200), summary.top_confidence[1]);
    try std.testing.expectEqual(@as(GF16, 150), summary.top_confidence[2]);
}

test "explain_derivation_chain_fitch" {
    var trace = create_trace();
    const step = make_test_step("modus", K_TRUE, 50);
    append_step(&trace, step);

    const buf = explain_derivation_chain(trace, .fitch);
    // Fitch format: "0. modus |- T\n"
    try std.testing.expectEqual(@as(u8, '0'), buf[0]);
    try std.testing.expectEqual(@as(u8, '.'), buf[1]);
}

test "explain_derivation_chain_compact" {
    var trace = create_trace();
    const step = make_test_step("rule", K_FALSE, 10);
    append_step(&trace, step);

    const buf = explain_derivation_chain(trace, .compact);
    // Compact format: "rule:F;"
    try std.testing.expectEqual(@as(u8, 'r'), buf[0]);
    try std.testing.expectEqual(@as(u8, 'u'), buf[1]);
    try std.testing.expectEqual(@as(u8, 'l'), buf[2]);
    try std.testing.expectEqual(@as(u8, 'e'), buf[3]);
    try std.testing.expectEqual(@as(u8, ':'), buf[4]);
    try std.testing.expectEqual(@as(u8, 'F'), buf[5]);
    try std.testing.expectEqual(@as(u8, ';'), buf[6]);
}

test "summarize_empty_trace" {
    const trace = create_trace();
    const summary = summarize(trace);
    try std.testing.expectEqual(@as(u8, 0), summary.fact_count);
}

test "summarize_overflow_max_summary_size" {
    // Add more unique steps than MAX_SUMMARY_SIZE to test eviction
    var trace = create_trace();

    const names = [_][]const u8{ "a", "b", "c", "d", "e", "f", "g" };
    const confs = [_]u16{ 10, 20, 30, 40, 50, 60, 70 };

    for (names, confs) |name, conf| {
        var step = make_test_step(name, K_TRUE, conf);
        // Ensure unique args per step by setting first arg differently
        step.input_trits[0] = if (conf < 40) K_FALSE else K_TRUE;
        step.input_trits[1] = if (conf < 20) K_TRUE else K_UNKNOWN;
        append_step(&trace, step);
    }

    const summary = summarize(trace);
    // Should be capped at MAX_SUMMARY_SIZE
    try std.testing.expectEqual(@as(u8, MAX_SUMMARY_SIZE), summary.fact_count);
}

test "create_trace_and_append" {
    var trace = create_trace();
    try std.testing.expectEqual(@as(u8, 0), trace.step_count);

    const step = make_test_step("test", K_TRUE, 1);
    append_step(&trace, step);
    try std.testing.expectEqual(@as(u8, 1), trace.step_count);
}

test "append_step_bounded" {
    var trace = create_trace();
    const step = make_test_step("x", K_TRUE, 1);

    // Fill to max
    var i: usize = 0;
    while (i < MAX_DERIVATION_STEPS + 5) : (i += 1) {
        append_step(&trace, step);
    }
    // Should be capped at MAX_DERIVATION_STEPS
    try std.testing.expectEqual(@as(u8, MAX_DERIVATION_STEPS), trace.step_count);
}
