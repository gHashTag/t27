// AUTO-GENERATED from specs/ar/proof_trace.t27 — DO NOT EDIT
// Ring: 18 | Module: ProofTrace | phi^2 + 1/phi^2 = 3
// Generator: PHI LOOP manual codegen (bootstrap unavailable)
// Bounded Proof Traces for CLARA Explainability (<=10 steps)

const std = @import("std");

// ═══════════════════════════════════════════════════════════════
// Re-export Kleene K3 Truth Values from ternary_logic
// ═══════════════════════════════════════════════════════════════

pub const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

pub const K_FALSE: Trit = .neg;
pub const K_UNKNOWN: Trit = .zero;
pub const K_TRUE: Trit = .pos;

// ═══════════════════════════════════════════════════════════════
// GF16 — Galois Field confidence encoded as u16
// ═══════════════════════════════════════════════════════════════

pub const GF16 = u16;

/// Encode a float [0.0, 1.0] into GF16 (u16 range 0..65535)
fn gf16_encode_f32(val: f32) GF16 {
    const clamped = @max(0.0, @min(1.0, val));
    return @intFromFloat(clamped * 65535.0);
}

/// Decode a GF16 back to float [0.0, 1.0]
fn gf16_decode_to_f32(encoded: GF16) f32 {
    return @as(f32, @floatFromInt(encoded)) / 65535.0;
}

// ═══════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════

pub const MAX_STEPS: u8 = 10;

// ═══════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════

pub const FormatStyle = enum {
    natural,
    fitch,
    compact,
};

pub const DerivationStep = struct {
    step_number: u8,
    rule_name: []const u8,
    input_facts: [3]Trit,
    output_fact: Trit,
    confidence: GF16,
    k3_value: Trit,
};

pub const ProofTrace = struct {
    steps: [MAX_STEPS]DerivationStep,
    step_count: u8,
    conclusion: Trit,
    total_confidence: GF16,
    terminated: bool,
};

// ═══════════════════════════════════════════════════════════════
// Functions
// ═══════════════════════════════════════════════════════════════

/// Create a fresh empty proof trace with K_UNKNOWN conclusion
/// and full confidence (1.0 encoded as GF16).
pub fn create_trace() ProofTrace {
    return ProofTrace{
        .steps = undefined,
        .step_count = 0,
        .conclusion = K_UNKNOWN,
        .total_confidence = gf16_encode_f32(1.0),
        .terminated = false,
    };
}

/// Append a derivation step to the trace.
/// Returns false if MAX_STEPS has been reached (Restraint triggered).
pub fn append_step(trace: *ProofTrace, step: DerivationStep) bool {
    if (trace.step_count >= MAX_STEPS) {
        trace.terminated = true;
        return false;
    }
    trace.steps[trace.step_count] = step;
    trace.step_count += 1;
    trace.conclusion = step.output_fact;
    // Multiply confidence values via GF16 encode/decode
    const current_f = gf16_decode_to_f32(trace.total_confidence);
    const step_f = gf16_decode_to_f32(step.confidence);
    trace.total_confidence = gf16_encode_f32(current_f * step_f);
    return true;
}

/// Return the current conclusion trit of the trace.
pub fn get_conclusion(trace: ProofTrace) Trit {
    return trace.conclusion;
}

/// Return the total accumulated confidence as GF16.
pub fn get_confidence(trace: ProofTrace) GF16 {
    return trace.total_confidence;
}

/// Format a proof trace according to the given style.
pub fn format(trace: ProofTrace, style: FormatStyle) []const u8 {
    return switch (style) {
        .natural => format_natural(trace),
        .fitch => format_fitch(trace),
        .compact => format_compact(trace),
    };
}

// ═══════════════════════════════════════════════════════════════
// Internal formatting helpers
// ═══════════════════════════════════════════════════════════════

fn trit_symbol(t: Trit) []const u8 {
    return switch (t) {
        .neg => "F",
        .zero => "?",
        .pos => "T",
    };
}

var natural_buf: [512]u8 = undefined;

fn format_natural(trace: ProofTrace) []const u8 {
    var offset: usize = 0;
    var i: u8 = 0;
    while (i < trace.step_count) : (i += 1) {
        const step = trace.steps[i];
        // "Step N: RULE -> FACT\n"
        const prefix = "Step ";
        const digit = [1]u8{'0' + i + 1};
        const mid = ": ";
        const arrow = " -> ";
        const fact = trit_symbol(step.output_fact);
        const nl = "\n";

        for (prefix) |c| {
            if (offset < natural_buf.len) {
                natural_buf[offset] = c;
                offset += 1;
            }
        }
        if (offset < natural_buf.len) {
            natural_buf[offset] = digit[0];
            offset += 1;
        }
        for (mid) |c| {
            if (offset < natural_buf.len) {
                natural_buf[offset] = c;
                offset += 1;
            }
        }
        for (step.rule_name) |c| {
            if (offset < natural_buf.len) {
                natural_buf[offset] = c;
                offset += 1;
            }
        }
        for (arrow) |c| {
            if (offset < natural_buf.len) {
                natural_buf[offset] = c;
                offset += 1;
            }
        }
        for (fact) |c| {
            if (offset < natural_buf.len) {
                natural_buf[offset] = c;
                offset += 1;
            }
        }
        for (nl) |c| {
            if (offset < natural_buf.len) {
                natural_buf[offset] = c;
                offset += 1;
            }
        }
    }
    return natural_buf[0..offset];
}

var fitch_buf: [512]u8 = undefined;

fn format_fitch(trace: ProofTrace) []const u8 {
    var offset: usize = 0;
    var i: u8 = 0;
    while (i < trace.step_count) : (i += 1) {
        const step = trace.steps[i];
        // "| N. FACT    RULE\n"
        const prefix = "| ";
        const digit = [1]u8{'0' + i + 1};
        const dot = ". ";
        const fact = trit_symbol(step.output_fact);
        const pad = "    ";
        const nl = "\n";

        for (prefix) |c| {
            if (offset < fitch_buf.len) {
                fitch_buf[offset] = c;
                offset += 1;
            }
        }
        if (offset < fitch_buf.len) {
            fitch_buf[offset] = digit[0];
            offset += 1;
        }
        for (dot) |c| {
            if (offset < fitch_buf.len) {
                fitch_buf[offset] = c;
                offset += 1;
            }
        }
        for (fact) |c| {
            if (offset < fitch_buf.len) {
                fitch_buf[offset] = c;
                offset += 1;
            }
        }
        for (pad) |c| {
            if (offset < fitch_buf.len) {
                fitch_buf[offset] = c;
                offset += 1;
            }
        }
        for (step.rule_name) |c| {
            if (offset < fitch_buf.len) {
                fitch_buf[offset] = c;
                offset += 1;
            }
        }
        for (nl) |c| {
            if (offset < fitch_buf.len) {
                fitch_buf[offset] = c;
                offset += 1;
            }
        }
    }
    return fitch_buf[0..offset];
}

var compact_buf: [128]u8 = undefined;

fn format_compact(trace: ProofTrace) []const u8 {
    // "N steps | conclusion=X | terminated=Y"
    var offset: usize = 0;
    const digit = [1]u8{'0' + trace.step_count};
    const suffix = " steps | conclusion=";
    const concl = trit_symbol(trace.conclusion);
    const mid = " | terminated=";
    const term_str: []const u8 = if (trace.terminated) "true" else "false";

    if (offset < compact_buf.len) {
        compact_buf[offset] = digit[0];
        offset += 1;
    }
    for (suffix) |c| {
        if (offset < compact_buf.len) {
            compact_buf[offset] = c;
            offset += 1;
        }
    }
    for (concl) |c| {
        if (offset < compact_buf.len) {
            compact_buf[offset] = c;
            offset += 1;
        }
    }
    for (mid) |c| {
        if (offset < compact_buf.len) {
            compact_buf[offset] = c;
            offset += 1;
        }
    }
    for (term_str) |c| {
        if (offset < compact_buf.len) {
            compact_buf[offset] = c;
            offset += 1;
        }
    }
    return compact_buf[0..offset];
}

// ═══════════════════════════════════════════════════════════════
// Helper — create a test derivation step
// ═══════════════════════════════════════════════════════════════

fn make_test_step(number: u8, output: Trit, conf_f32: f32) DerivationStep {
    return DerivationStep{
        .step_number = number,
        .rule_name = "test_rule",
        .input_facts = [3]Trit{ K_UNKNOWN, K_UNKNOWN, K_UNKNOWN },
        .output_fact = output,
        .confidence = gf16_encode_f32(conf_f32),
        .k3_value = output,
    };
}

// ═══════════════════════════════════════════════════════════════
// Tests — Conformance vectors from spec
// ═══════════════════════════════════════════════════════════════

test "trace_unknown_initial_conclusion" {
    const trace = create_trace();
    try std.testing.expectEqual(K_UNKNOWN, get_conclusion(trace));
}

test "trace_conclusion_tracks_last_step" {
    var trace = create_trace();
    const step = make_test_step(1, K_TRUE, 0.9);
    _ = append_step(&trace, step);
    try std.testing.expectEqual(K_TRUE, get_conclusion(trace));

    // Append another step with K_FALSE — conclusion should update
    const step2 = make_test_step(2, K_FALSE, 0.8);
    _ = append_step(&trace, step2);
    try std.testing.expectEqual(K_FALSE, get_conclusion(trace));
}

test "trace_respects_max_steps" {
    var trace = create_trace();
    // Fill to MAX_STEPS
    var i: u8 = 0;
    while (i < MAX_STEPS) : (i += 1) {
        const step = make_test_step(i + 1, K_TRUE, 0.95);
        const ok = append_step(&trace, step);
        try std.testing.expect(ok);
    }
    try std.testing.expectEqual(MAX_STEPS, trace.step_count);

    // 11th append must fail
    const overflow_step = make_test_step(MAX_STEPS + 1, K_TRUE, 0.95);
    const rejected = append_step(&trace, overflow_step);
    try std.testing.expect(!rejected);
    try std.testing.expectEqual(MAX_STEPS, trace.step_count);
    try std.testing.expect(trace.terminated);
}

test "trace_confidence_is_product" {
    var trace = create_trace();
    const step1 = make_test_step(1, K_TRUE, 0.9);
    _ = append_step(&trace, step1);
    const step2 = make_test_step(2, K_TRUE, 0.8);
    _ = append_step(&trace, step2);

    // total_confidence should approximate 0.9 * 0.8 = 0.72
    const decoded = gf16_decode_to_f32(get_confidence(trace));
    const diff = @abs(decoded - 0.72);
    try std.testing.expect(diff < 0.01);
}

test "trace_bounded_by_clara_invariant" {
    var trace = create_trace();
    // Try to add more than MAX_STEPS
    var i: u8 = 0;
    while (i < MAX_STEPS + 5) : (i += 1) {
        const step = make_test_step(i + 1, K_TRUE, 1.0);
        _ = append_step(&trace, step);
    }
    // step_count must never exceed MAX_STEPS
    try std.testing.expect(trace.step_count <= MAX_STEPS);
}

test "format_natural_contains_step_numbers" {
    var trace = create_trace();
    var i: u8 = 0;
    while (i < 3) : (i += 1) {
        const step = make_test_step(i + 1, K_TRUE, 0.9);
        _ = append_step(&trace, step);
    }
    const output = format(trace, .natural);
    // Should contain Step 1, Step 2, Step 3
    var has_s1 = false;
    var has_s2 = false;
    var has_s3 = false;
    var j: usize = 0;
    while (j + 5 < output.len) : (j += 1) {
        if (std.mem.eql(u8, output[j .. j + 6], "Step 1")) has_s1 = true;
        if (std.mem.eql(u8, output[j .. j + 6], "Step 2")) has_s2 = true;
        if (std.mem.eql(u8, output[j .. j + 6], "Step 3")) has_s3 = true;
    }
    try std.testing.expect(has_s1);
    try std.testing.expect(has_s2);
    try std.testing.expect(has_s3);
}

test "format_compact_single_line" {
    var trace = create_trace();
    var i: u8 = 0;
    while (i < 5) : (i += 1) {
        const step = make_test_step(i + 1, K_TRUE, 1.0);
        _ = append_step(&trace, step);
    }
    const output = format(trace, .compact);
    // Should not contain newline
    for (output) |c| {
        try std.testing.expect(c != '\n');
    }
}
