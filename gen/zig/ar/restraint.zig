// AUTO-GENERATED from specs/ar/restraint.t27 — DO NOT EDIT
// Ring: 18 | Module: Restraint | φ² + 1/φ² = 3
// Bounded Rationality via Trit=0 (CLARA Restraint)
// Agent C: Most pure agent in CLARA AR family

const std = @import("std");

// ═══════════════════════════════════════════════════════════════
// Trit — Kleene K3 Truth Values (from base::types)
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
// GF16 — phi-optimized float encoding (from numeric::gf16)
// Raw u16 placeholder values pending full GF16 spec
// ═══════════════════════════════════════════════════════════════

pub const GF16 = u16;

/// GF16 placeholder encode: raw u16 bit patterns
/// Real implementation depends on numeric::gf16 spec
fn gf16_from_raw(raw: u16) GF16 {
    return raw;
}

/// GF16 placeholder comparison: unsigned compare on raw bits
/// For the phi-optimized encoding, higher raw value = higher float
fn gf16_gt(a: GF16, b: GF16) bool {
    return a > b;
}

fn gf16_lt(a: GF16, b: GF16) bool {
    return a < b;
}

// GF16 raw placeholder constants
const GF16_ZERO: GF16 = 0x0000;
const GF16_085: GF16 = 0x3B33; // ~0.85
const GF16_075: GF16 = 0x3A00; // ~0.75
const GF16_070: GF16 = 0x3966; // ~0.70

// ═══════════════════════════════════════════════════════════════
// 1. Quality Level Enumeration
// ═══════════════════════════════════════════════════════════════

pub const QualityLevel = enum(u2) {
    /// Unknown quality: no confidence bounds, minimal resources
    unknown = 0,
    /// Unstable quality: low confidence, restricted resources
    unstable = 1,
    /// Good quality: high confidence, full resources
    good = 2,
};

// ═══════════════════════════════════════════════════════════════
// 2. Restraint Parameters
// ═══════════════════════════════════════════════════════════════

pub const RestraintParams = struct {
    /// Maximum derivation depth (prevents infinite loops)
    max_depth: u32,
    /// Maximum number of rules to fire (prevents combinatorial explosion)
    max_rules: u32,
    /// Minimum confidence threshold (GF16 phi-optimized encoding)
    confidence_threshold: GF16,
    /// Timeout in milliseconds (prevents runaway inference)
    timeout_ms: u64,
};

// ═══════════════════════════════════════════════════════════════
// 3. Execution State
// ═══════════════════════════════════════════════════════════════

pub const ExecutionState = struct {
    /// Current derivation depth (starts at 0)
    current_depth: u32,
    /// Number of rules fired so far
    rules_fired: u32,
    /// Current confidence level (GF16 phi-optimized encoding)
    current_confidence: GF16,
    /// Execution start time in milliseconds
    start_time_ms: u64,
    /// Current quality level
    quality: QualityLevel,
};

// ═══════════════════════════════════════════════════════════════
// 4. Meta-Rule for HiLog
// ═══════════════════════════════════════════════════════════════

pub const MetaRule = struct {
    /// Condition under which this meta-rule applies (Kleene semantics)
    condition: Trit,
    /// Action: .neg=halt, .zero=restraint, .pos=continue
    action: Trit,
    /// Priority: higher priority meta-rules checked first
    priority: u8,
};

// ═══════════════════════════════════════════════════════════════
// 5. Quality-Level Parameter Selection — O(1)
// ═══════════════════════════════════════════════════════════════

/// Select appropriate restraint parameters for a given quality level.
/// Per CLARA requirements: unknown=conservative, good=permissive.
pub fn params_for_quality(quality: QualityLevel) RestraintParams {
    return switch (quality) {
        .unknown => RestraintParams{
            .max_depth = 3,
            .max_rules = 10,
            .confidence_threshold = GF16_085, // 0.85 — high threshold for unknown
            .timeout_ms = 100,
        },
        .unstable => RestraintParams{
            .max_depth = 7,
            .max_rules = 50,
            .confidence_threshold = GF16_075, // 0.75 — medium threshold
            .timeout_ms = 1000,
        },
        .good => RestraintParams{
            .max_depth = 15,
            .max_rules = 500,
            .confidence_threshold = GF16_070, // 0.70 — lower threshold for good
            .timeout_ms = 10000,
        },
    };
}

// ═══════════════════════════════════════════════════════════════
// 6. Restraint Decision Function — O(1)
// ═══════════════════════════════════════════════════════════════

/// Determine if execution should continue (K_TRUE) or be restrained (K_FALSE).
/// Returns K_FALSE when ANY restraint condition is triggered.
/// This is CORE of CLARA bounded rationality.
pub fn should_continue(state: ExecutionState, params: RestraintParams) Trit {
    // Check depth limit
    if (state.current_depth >= params.max_depth) {
        return K_FALSE; // Depth restraint
    }

    // Check rule limit
    if (state.rules_fired >= params.max_rules) {
        return K_FALSE; // Rule count restraint
    }

    // Check confidence threshold using GF16 comparison
    if (gf16_lt(state.current_confidence, params.confidence_threshold)) {
        return K_FALSE; // Confidence restraint
    }

    // All checks passed — continue
    return K_TRUE;
}

// ═══════════════════════════════════════════════════════════════
// 7. Restraint Value Detection — O(1)
// ═══════════════════════════════════════════════════════════════

/// Check if a Trit value represents restraint.
/// Trit.zero = K_UNKNOWN = "don't-care" = restraint.
pub fn is_restraint_value(t: Trit) bool {
    return t == K_UNKNOWN;
}

// ═══════════════════════════════════════════════════════════════
// 8. Depth Stepping — O(1)
// ═══════════════════════════════════════════════════════════════

/// Increment execution depth by delta.
/// Used for entering/exiting derivation levels.
pub fn step_depth(state: *ExecutionState, delta: u32) void {
    state.current_depth += delta;
}

// ═══════════════════════════════════════════════════════════════
// 9. Meta-Rule Application — O(n)
// ═══════════════════════════════════════════════════════════════

/// Apply HiLog meta-rules in priority order.
/// Returns K_FALSE if any triggered meta-rule says halt.
/// Returns K_TRUE if all triggered rules say continue.
/// Returns K_UNKNOWN if partial/conflicting meta-information.
pub fn apply_meta_rules(state: *ExecutionState, meta_rules: []const MetaRule, count: usize) Trit {
    _ = state;
    var result: Trit = K_TRUE; // Default: continue

    var i: usize = 0;
    while (i < count) : (i += 1) {
        const rule = meta_rules[i];

        // Check if rule condition matches (K_TRUE = always trigger)
        if (rule.condition == K_TRUE) {
            // K_FALSE action = halt (strongest)
            if (rule.action == K_FALSE) {
                return K_FALSE; // Immediate halt
            }
            // K_UNKNOWN action = restraint (downgrade)
            if (rule.action == K_UNKNOWN) {
                result = K_UNKNOWN;
            }
            // K_TRUE action = continue (no change)
        }
    }

    return result;
}

// ═══════════════════════════════════════════════════════════════
// 10. State Initialization — O(1)
// ═══════════════════════════════════════════════════════════════

/// Initialize execution state for a new derivation.
pub fn init_state(quality: QualityLevel, start_time: u64) ExecutionState {
    return ExecutionState{
        .current_depth = 0,
        .rules_fired = 0,
        .current_confidence = GF16_ZERO, // Start with zero confidence
        .start_time_ms = start_time,
        .quality = quality,
    };
}

// ═══════════════════════════════════════════════════════════════
// Tests — Conformance vectors from spec
// ═══════════════════════════════════════════════════════════════

test "params_for_quality_unknown_conservative" {
    const p = params_for_quality(.unknown);
    try std.testing.expectEqual(@as(u32, 3), p.max_depth);
    try std.testing.expectEqual(@as(u32, 10), p.max_rules);
    try std.testing.expectEqual(@as(u64, 100), p.timeout_ms);
    try std.testing.expectEqual(GF16_085, p.confidence_threshold);
}

test "params_for_quality_unstable_moderate" {
    const p = params_for_quality(.unstable);
    try std.testing.expectEqual(@as(u32, 7), p.max_depth);
    try std.testing.expectEqual(@as(u32, 50), p.max_rules);
    try std.testing.expectEqual(@as(u64, 1000), p.timeout_ms);
    try std.testing.expectEqual(GF16_075, p.confidence_threshold);
}

test "params_for_quality_good_permissive" {
    const p = params_for_quality(.good);
    try std.testing.expectEqual(@as(u32, 15), p.max_depth);
    try std.testing.expectEqual(@as(u32, 500), p.max_rules);
    try std.testing.expectEqual(@as(u64, 10000), p.timeout_ms);
    try std.testing.expectEqual(GF16_070, p.confidence_threshold);
}

test "should_continue_blocks_on_depth_limit" {
    const params = RestraintParams{
        .max_depth = 2,
        .max_rules = 100,
        .confidence_threshold = 0x0001,
        .timeout_ms = 1000,
    };
    const state = ExecutionState{
        .current_depth = 3,
        .rules_fired = 0,
        .current_confidence = 0xFFFF, // Very high
        .start_time_ms = 0,
        .quality = .good,
    };
    try std.testing.expectEqual(K_FALSE, should_continue(state, params));
}

test "should_continue_blocks_on_rule_limit" {
    const params = RestraintParams{
        .max_depth = 100,
        .max_rules = 5,
        .confidence_threshold = 0x0001,
        .timeout_ms = 1000,
    };
    const state = ExecutionState{
        .current_depth = 0,
        .rules_fired = 10,
        .current_confidence = 0xFFFF,
        .start_time_ms = 0,
        .quality = .good,
    };
    try std.testing.expectEqual(K_FALSE, should_continue(state, params));
}

test "should_continue_blocks_on_low_confidence" {
    const params = RestraintParams{
        .max_depth = 100,
        .max_rules = 100,
        .confidence_threshold = GF16_070,
        .timeout_ms = 1000,
    };
    const state = ExecutionState{
        .current_depth = 0,
        .rules_fired = 0,
        .current_confidence = 0x0010, // Very low
        .start_time_ms = 0,
        .quality = .good,
    };
    try std.testing.expectEqual(K_FALSE, should_continue(state, params));
}

test "should_continue_allows_sufficient_confidence" {
    const params = RestraintParams{
        .max_depth = 100,
        .max_rules = 100,
        .confidence_threshold = GF16_070,
        .timeout_ms = 1000,
    };
    const state = ExecutionState{
        .current_depth = 0,
        .rules_fired = 0,
        .current_confidence = 0xFFFF, // Above threshold
        .start_time_ms = 0,
        .quality = .good,
    };
    try std.testing.expectEqual(K_TRUE, should_continue(state, params));
}

test "is_restraint_value_true_for_zero" {
    try std.testing.expect(is_restraint_value(K_UNKNOWN));
}

test "is_restraint_value_false_for_neg" {
    try std.testing.expect(!is_restraint_value(K_FALSE));
}

test "is_restraint_value_false_for_pos" {
    try std.testing.expect(!is_restraint_value(K_TRUE));
}

test "step_depth_increments" {
    var state = init_state(.good, 0);
    const initial = state.current_depth;
    step_depth(&state, 1);
    try std.testing.expectEqual(initial + 1, state.current_depth);
}

test "step_depth_multiple" {
    var state = init_state(.good, 0);
    step_depth(&state, 5);
    try std.testing.expectEqual(@as(u32, 5), state.current_depth);
}

test "apply_meta_rules_empty" {
    var state = init_state(.good, 0);
    const rules = [_]MetaRule{};
    try std.testing.expectEqual(K_TRUE, apply_meta_rules(&state, &rules, 0));
}

test "apply_meta_rules_halt_on_false_action" {
    var state = init_state(.good, 0);
    const rules = [_]MetaRule{
        .{ .condition = K_TRUE, .action = K_FALSE, .priority = 0 },
    };
    try std.testing.expectEqual(K_FALSE, apply_meta_rules(&state, &rules, 1));
}

test "apply_meta_rules_restraint_on_unknown_action" {
    var state = init_state(.good, 0);
    const rules = [_]MetaRule{
        .{ .condition = K_TRUE, .action = K_UNKNOWN, .priority = 0 },
    };
    try std.testing.expectEqual(K_UNKNOWN, apply_meta_rules(&state, &rules, 1));
}

test "apply_meta_rules_condition_mismatch" {
    var state = init_state(.good, 0);
    const rules = [_]MetaRule{
        .{ .condition = K_FALSE, .action = K_FALSE, .priority = 0 },
    };
    try std.testing.expectEqual(K_TRUE, apply_meta_rules(&state, &rules, 1));
}

test "init_state_sets_defaults" {
    const state = init_state(.unstable, 12345);
    try std.testing.expectEqual(@as(u32, 0), state.current_depth);
    try std.testing.expectEqual(@as(u32, 0), state.rules_fired);
    try std.testing.expectEqual(GF16_ZERO, state.current_confidence);
    try std.testing.expectEqual(@as(u64, 12345), state.start_time_ms);
    try std.testing.expectEqual(QualityLevel.unstable, state.quality);
}

// ═══════════════════════════════════════════════════════════════
// Invariant tests
// ═══════════════════════════════════════════════════════════════

test "invariant_quality_level_coverage" {
    const q_unknown = params_for_quality(.unknown);
    const q_unstable = params_for_quality(.unstable);
    const q_good = params_for_quality(.good);
    try std.testing.expect(q_unknown.max_depth < q_unstable.max_depth);
    try std.testing.expect(q_unstable.max_depth < q_good.max_depth);
    try std.testing.expect(q_unknown.max_rules < q_unstable.max_rules);
    try std.testing.expect(q_unstable.max_rules < q_good.max_rules);
}

test "invariant_quality_thresholds_decrease" {
    // Higher quality = lower threshold (easier to continue)
    const q_unknown = params_for_quality(.unknown);
    const q_unstable = params_for_quality(.unstable);
    const q_good = params_for_quality(.good);
    try std.testing.expect(gf16_gt(q_unknown.confidence_threshold, q_unstable.confidence_threshold));
    try std.testing.expect(gf16_gt(q_unstable.confidence_threshold, q_good.confidence_threshold));
}

test "invariant_restraint_params_nonzero" {
    const q = params_for_quality(.unknown);
    try std.testing.expect(q.max_depth > 0);
    try std.testing.expect(q.max_rules > 0);
    try std.testing.expect(q.timeout_ms > 0);
}

test "invariant_restraint_value_isomorphism" {
    try std.testing.expect(is_restraint_value(.zero));
    try std.testing.expect(!is_restraint_value(.neg));
    try std.testing.expect(!is_restraint_value(.pos));
}
