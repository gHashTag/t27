// AUTO-GENERATED from specs/ar/composition.t27 — DO NOT EDIT
// Ring: 18 | Module: Composition | ML+AR Composition for CLARA
// Generator: PHI LOOP manual codegen (bootstrap unavailable)

const std = @import("std");

// ═══════════════════════════════════════════════════════════════
// Dependencies — re-exported from ar::ternary_logic
// ═══════════════════════════════════════════════════════════════

pub const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

pub const K_FALSE: Trit = .neg;
pub const K_UNKNOWN: Trit = .zero;
pub const K_TRUE: Trit = .pos;

fn trit_min(a: Trit, b: Trit) Trit {
    const ai = @intFromEnum(a);
    const bi = @intFromEnum(b);
    return if (ai < bi) a else b;
}

fn trit_max(a: Trit, b: Trit) Trit {
    const ai = @intFromEnum(a);
    const bi = @intFromEnum(b);
    return if (ai > bi) a else b;
}

/// Kleene conjunction: minimum of truth values
pub fn k3_and(a: Trit, b: Trit) Trit {
    return trit_min(a, b);
}

/// Kleene disjunction: maximum of truth values
pub fn k3_or(a: Trit, b: Trit) Trit {
    return trit_max(a, b);
}

// ═══════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════

pub const MAX_COMPOSITION_DEPTH: u8 = 5;
pub const GF16_ONE: u16 = 0x3C00;
pub const GF16_ZERO: u16 = 0x0000;

// ═══════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════

/// Composition pattern type - defines how ML and AR components combine
pub const CompositionPattern = enum(u8) {
    /// CNN + Rules: Neural feature extraction -> AR rule evaluation
    CNN_RULES = 0,
    /// MLP + Bayesian: Neural forward pass -> Bayesian inference
    MLP_BAYESIAN = 1,
    /// Transformer + XAI: Self-attention -> <=10 step explanation
    TRANSFORMER_XAI = 2,
    /// RL + Guardrails: Policy network -> AR constraint checking
    RL_GUARDRAILS = 3,
};

/// Explanation format style
pub const FormatStyle = enum(u8) {
    natural = 0,
    fitch = 1,
    compact = 2,
};

/// Horn clause for AR inference
pub const HornClause = struct {
    antecedent: Trit,
    consequent: Trit,
};

/// Derivation step within a proof trace
pub const DerivationStep = struct {
    step_number: u8,
    rule_name: []const u8,
    output_fact: Trit,
    confidence: u16,
    k3_value: Trit,
};

/// Proof trace: bounded derivation chain
pub const MAX_STEPS: u8 = 10;

pub const ProofTrace = struct {
    steps: [MAX_STEPS]DerivationStep,
    step_count: u8,
};

/// Explanation structure
pub const Explanation = struct {
    trace: ProofTrace,
    style: FormatStyle,
    confidence: u16,
    step_count: u8,
};

/// ML component abstraction (pluggable)
pub const MLComponent = struct {
    component_type: []const u8,
    input_shape: [4]u32,
    output_shape: [2]u32,
    confidence: u16,
};

/// AR component abstraction
pub const ARComponent = struct {
    rule_set: []const HornClause,
    rule_count: usize,
    explanation_style: FormatStyle,
    confidence_threshold: u16,
};

/// Composed pipeline - ML + AR combination
pub const ComposedPipeline = struct {
    pattern: CompositionPattern,
    ml_component: MLComponent,
    ar_component: ARComponent,
    pipeline_confidence: u16,
    steps_completed: u8,
    explanation: Explanation,
    terminated: bool,
};

/// Composition result
pub const CompositionResult = struct {
    prediction: Trit,
    confidence: u16,
    explanation: Explanation,
    proof_trace: ProofTrace,
    satisfaction: u16,
};

// ═══════════════════════════════════════════════════════════════
// GF16 helpers (simplified IEEE 754 half-precision)
// ═══════════════════════════════════════════════════════════════

/// Encode f32 to GF16 (u16 half-precision)
pub fn gf16_from_f32(value: f32) u16 {
    // Simplified: clamp to [0,1] and scale to half-precision
    const clamped = if (value > 1.0) @as(f32, 1.0) else if (value < 0.0) @as(f32, 0.0) else value;
    if (clamped == 0.0) return GF16_ZERO;
    if (clamped >= 1.0) return GF16_ONE;
    // Linear approximation in the [0,1] range for GF16
    const scaled: u16 = @intFromFloat(clamped * @as(f32, @floatFromInt(GF16_ONE)));
    return scaled;
}

/// Decode GF16 (u16) to f32
pub fn gf16_to_f32(gf: u16) f32 {
    if (gf == GF16_ZERO) return 0.0;
    if (gf == GF16_ONE) return 1.0;
    return @as(f32, @floatFromInt(gf)) / @as(f32, @floatFromInt(GF16_ONE));
}

// ═══════════════════════════════════════════════════════════════
// Core Functions
// ═══════════════════════════════════════════════════════════════

/// Convert f32 to Trit: >0.5 -> K_TRUE, <-0.5 -> K_FALSE, else K_UNKNOWN
pub fn f32_to_trit(value: f32) Trit {
    if (value > 0.5) return K_TRUE;
    if (value < -0.5) return K_FALSE;
    return K_UNKNOWN;
}

/// Combine two GF16 confidence values.
/// Simplified multiplication: (a * b) >> 16, clamped to valid range.
/// Approximates geometric mean of the two confidences.
pub fn combine_confidence(ml_conf: u16, ar_conf: u16) u16 {
    const a: u32 = @intCast(ml_conf);
    const b: u32 = @intCast(ar_conf);
    const product: u32 = (a * b);
    // Divide by GF16_ONE to keep result in GF16 range
    const result: u32 = product / @as(u32, GF16_ONE);
    if (result > GF16_ONE) return GF16_ONE;
    return @intCast(result);
}

/// Calculate satisfaction of CLARA requirements.
/// Compares result confidence against the pipeline threshold.
/// Returns GF16_ONE if confidence >= threshold, scaled value otherwise.
pub fn calculate_satisfaction(pipeline: *const ComposedPipeline, prediction: Trit, confidence: u16) u16 {
    _ = prediction;
    const threshold = pipeline.ar_component.confidence_threshold;

    // Base satisfaction: confidence meets or exceeds threshold
    if (confidence >= threshold and threshold > 0) {
        // Full satisfaction when confident
        var sat: u16 = GF16_ONE;

        // Bonus capped at GF16_ONE for bounded explanations (<=10 steps)
        if (pipeline.explanation.step_count > MAX_STEPS) {
            // Penalty for exceeding CLARA limit
            sat = combine_confidence(sat, gf16_from_f32(0.5));
        }

        // Penalty for terminated pipelines (restraint triggered)
        if (pipeline.terminated) {
            sat = combine_confidence(sat, gf16_from_f32(0.7));
        }

        return sat;
    }

    // Partial satisfaction: scale by confidence / threshold ratio
    if (threshold == 0) return GF16_ONE;
    const conf_f = gf16_to_f32(confidence);
    const thresh_f = gf16_to_f32(threshold);
    if (thresh_f == 0.0) return GF16_ONE;
    const ratio = conf_f / thresh_f;
    return gf16_from_f32(if (ratio > 1.0) 1.0 else ratio);
}

/// Evaluate AR rules against an input trit value.
/// Returns the combined result of forward-chaining all rules.
pub fn evaluate_ar_rules(ar_comp: *const ARComponent, input_decision: Trit) Trit {
    var result: Trit = K_UNKNOWN;
    var i: usize = 0;
    while (i < ar_comp.rule_count) : (i += 1) {
        const rule = ar_comp.rule_set[i];
        // Forward chain: if input matches antecedent, apply consequent
        if (input_decision == rule.antecedent) {
            result = k3_or(result, rule.consequent);
        }
    }
    return result;
}

/// Create a composition pipeline with the given parameters.
pub fn create_pipeline(
    pattern: CompositionPattern,
    ml_confidence: u16,
    rules: []const HornClause,
    rule_count: usize,
) ComposedPipeline {
    const empty_trace = ProofTrace{
        .steps = undefined,
        .step_count = 0,
    };
    const empty_explanation = Explanation{
        .trace = empty_trace,
        .style = .natural,
        .confidence = GF16_ZERO,
        .step_count = 0,
    };

    return ComposedPipeline{
        .pattern = pattern,
        .ml_component = MLComponent{
            .component_type = "generic",
            .input_shape = [_]u32{ 0, 0, 0, 0 },
            .output_shape = [_]u32{ 0, 0 },
            .confidence = ml_confidence,
        },
        .ar_component = ARComponent{
            .rule_set = rules,
            .rule_count = rule_count,
            .explanation_style = .natural,
            .confidence_threshold = GF16_ZERO,
        },
        .pipeline_confidence = ml_confidence,
        .steps_completed = 0,
        .explanation = empty_explanation,
        .terminated = false,
    };
}

// ═══════════════════════════════════════════════════════════════
// Pattern-specific composition functions
// ═══════════════════════════════════════════════════════════════

/// CNN + Rules: Neural feature extraction -> AR rule evaluation
pub fn compose_cnn_rules(
    cnn_features: []const f32,
    cnn_confidence: u16,
    rules: []const HornClause,
    rule_count: usize,
    confidence_threshold: u16,
) CompositionResult {
    var pipeline = create_pipeline(.CNN_RULES, cnn_confidence, rules, rule_count);
    pipeline.ar_component.confidence_threshold = confidence_threshold;

    // Step 1: Simulate CNN inference — threshold on first feature
    const cnn_decision: Trit = if (cnn_features.len > 0 and cnn_features[0] > 0.5)
        K_TRUE
    else
        K_FALSE;
    pipeline.steps_completed = 1;

    // Step 2: Evaluate AR rules
    const ar_decision = evaluate_ar_rules(&pipeline.ar_component, cnn_decision);
    pipeline.steps_completed = 2;

    // Step 3: Combine confidence
    const combined_conf = combine_confidence(cnn_confidence, GF16_ONE);
    pipeline.steps_completed = 3;

    // Step 4: Final prediction — conjunction of CNN and AR decisions
    const prediction = k3_and(cnn_decision, ar_decision);

    // Step 5: Calculate satisfaction
    const satisfaction = calculate_satisfaction(&pipeline, prediction, combined_conf);

    return CompositionResult{
        .prediction = prediction,
        .confidence = combined_conf,
        .explanation = pipeline.explanation,
        .proof_trace = pipeline.explanation.trace,
        .satisfaction = satisfaction,
    };
}

/// MLP + Bayesian: Neural forward pass -> Bayesian inference
pub fn compose_mlp_bayesian(
    mlp_input: []const f32,
    mlp_confidence: u16,
    bayesian_prior: f32,
    confidence_threshold: u16,
) CompositionResult {
    var pipeline = create_pipeline(.MLP_BAYESIAN, mlp_confidence, &[_]HornClause{}, 0);
    pipeline.ar_component.confidence_threshold = confidence_threshold;

    // Step 1: Simulate MLP forward pass — average of inputs
    var sum: f32 = 0.0;
    for (mlp_input) |v| {
        sum += v;
    }
    const mlp_prob: f32 = if (mlp_input.len > 0)
        sum / @as(f32, @floatFromInt(mlp_input.len))
    else
        0.0;
    pipeline.steps_completed = 1;

    // Step 2: Bayesian update — posterior proportional to prior * likelihood
    const posterior: f32 = bayesian_prior * mlp_prob;
    pipeline.steps_completed = 2;

    // Step 3: Combine confidence
    const bayesian_gf = gf16_from_f32(posterior);
    const combined_conf = combine_confidence(mlp_confidence, bayesian_gf);
    pipeline.steps_completed = 3;

    // Step 4: Final prediction
    const mlp_trit = f32_to_trit(mlp_prob);
    const bayesian_trit = f32_to_trit(posterior);
    const prediction = k3_and(mlp_trit, bayesian_trit);

    // Step 5: Calculate satisfaction
    const satisfaction = calculate_satisfaction(&pipeline, prediction, combined_conf);

    return CompositionResult{
        .prediction = prediction,
        .confidence = combined_conf,
        .explanation = pipeline.explanation,
        .proof_trace = pipeline.explanation.trace,
        .satisfaction = satisfaction,
    };
}

/// Transformer + XAI: Self-attention -> <=10 step explanation
pub fn compose_transformer_xai(
    transformer_input: []const f32,
    transformer_confidence: u16,
    max_steps: u8,
    style: FormatStyle,
) CompositionResult {
    var pipeline = create_pipeline(.TRANSFORMER_XAI, transformer_confidence, &[_]HornClause{}, 0);
    pipeline.ar_component.confidence_threshold = GF16_ZERO;

    // Step 1: Simulate transformer — threshold on first input
    const decision: Trit = if (transformer_input.len > 0 and transformer_input[0] > 0.5)
        K_TRUE
    else
        K_FALSE;
    pipeline.steps_completed = 1;

    // Step 2: Build proof trace (bounded to MAX_STEPS=10 per CLARA)
    var trace = ProofTrace{
        .steps = undefined,
        .step_count = 0,
    };
    const effective_steps = if (max_steps < MAX_STEPS) max_steps else MAX_STEPS;
    var i: u8 = 0;
    while (i < effective_steps) : (i += 1) {
        trace.steps[i] = DerivationStep{
            .step_number = i,
            .rule_name = "attention",
            .output_fact = decision,
            .confidence = transformer_confidence,
            .k3_value = decision,
        };
        trace.step_count += 1;
    }
    pipeline.steps_completed = 2;

    // Step 3: Build explanation
    const explanation = Explanation{
        .trace = trace,
        .style = style,
        .confidence = transformer_confidence,
        .step_count = trace.step_count,
    };
    pipeline.explanation = explanation;

    // Step 4: Calculate satisfaction
    const satisfaction = calculate_satisfaction(&pipeline, decision, transformer_confidence);

    return CompositionResult{
        .prediction = decision,
        .confidence = transformer_confidence,
        .explanation = explanation,
        .proof_trace = trace,
        .satisfaction = satisfaction,
    };
}

/// RL + Guardrails: Policy network -> AR constraint checking
pub fn compose_rl_guardrails(
    state: []const f32,
    policy_confidence: u16,
    guardrails: []const HornClause,
    guardrail_count: usize,
    confidence_threshold: u16,
) CompositionResult {
    var pipeline = create_pipeline(.RL_GUARDRAILS, policy_confidence, guardrails, guardrail_count);
    pipeline.ar_component.confidence_threshold = confidence_threshold;

    // Step 1: Simulate RL policy inference
    const rl_prob: f32 = if (state.len > 0) state[0] else 0.0;
    const rl_trit = f32_to_trit(rl_prob);
    pipeline.steps_completed = 1;

    // Step 2: Evaluate guardrail rules
    const guardrail_decision = evaluate_ar_rules(&pipeline.ar_component, rl_trit);
    pipeline.steps_completed = 2;

    // Step 3: Action allowed only if both RL and guardrails approve
    const prediction = k3_and(rl_trit, guardrail_decision);
    pipeline.steps_completed = 3;

    // Step 4: Combine confidence (lower if guardrails block)
    const combined_conf = if (guardrail_decision == K_FALSE)
        gf16_from_f32(0.3)
    else
        combine_confidence(policy_confidence, GF16_ONE);

    // Step 5: Calculate satisfaction
    const satisfaction = calculate_satisfaction(&pipeline, prediction, combined_conf);

    return CompositionResult{
        .prediction = prediction,
        .confidence = combined_conf,
        .explanation = pipeline.explanation,
        .proof_trace = pipeline.explanation.trace,
        .satisfaction = satisfaction,
    };
}

/// Generic composition dispatch based on pattern.
/// Switches on the CompositionPattern and delegates to the appropriate
/// pattern-specific composition function.
pub fn compose(
    pattern: CompositionPattern,
    ml_input: []const f32,
    ml_confidence: u16,
    ar_rules: []const HornClause,
    ar_rule_count: usize,
    confidence_threshold: u16,
) CompositionResult {
    return switch (pattern) {
        .CNN_RULES => compose_cnn_rules(ml_input, ml_confidence, ar_rules, ar_rule_count, confidence_threshold),
        .MLP_BAYESIAN => compose_mlp_bayesian(ml_input, ml_confidence, 0.5, confidence_threshold),
        .TRANSFORMER_XAI => compose_transformer_xai(ml_input, ml_confidence, MAX_STEPS, .natural),
        .RL_GUARDRAILS => compose_rl_guardrails(ml_input, ml_confidence, ar_rules, ar_rule_count, confidence_threshold),
    };
}

// ═══════════════════════════════════════════════════════════════
// Tests — Conformance vectors from spec
// ═══════════════════════════════════════════════════════════════

test "compose_dispatches_correctly" {
    // CNN_RULES with high feature -> K_TRUE path
    const cnn_input = [_]f32{ 0.8, 0.3, 0.6 };
    const cnn_rules = [_]HornClause{
        .{ .antecedent = K_TRUE, .consequent = K_TRUE },
    };
    const cnn_result = compose(
        .CNN_RULES,
        &cnn_input,
        GF16_ONE,
        &cnn_rules,
        1,
        GF16_ZERO,
    );
    // High feature score with matching rule -> K_TRUE
    try std.testing.expectEqual(K_TRUE, cnn_result.prediction);

    // MLP_BAYESIAN with moderate input
    const mlp_input = [_]f32{ 0.5, 0.5, 0.5 };
    const mlp_result = compose(
        .MLP_BAYESIAN,
        &mlp_input,
        GF16_ONE,
        &[_]HornClause{},
        0,
        GF16_ZERO,
    );
    // MLP average is 0.5, Bayesian posterior = 0.5*0.5 = 0.25 -> both map to K_UNKNOWN/K_FALSE
    // This is intentionally different from the CNN result
    try std.testing.expect(cnn_result.prediction != mlp_result.prediction);
}

test "combine_confidence_identity" {
    // Combining with GF16_ONE should preserve the original value
    const result = combine_confidence(GF16_ONE, GF16_ONE);
    try std.testing.expectEqual(GF16_ONE, result);
}

test "combine_confidence_zero" {
    // Combining with GF16_ZERO should yield zero
    const result = combine_confidence(GF16_ONE, GF16_ZERO);
    try std.testing.expectEqual(GF16_ZERO, result);
}

test "combine_confidence_symmetric" {
    const a: u16 = 0x3800; // ~0.5 in half-precision range
    const b: u16 = 0x3000; // ~0.25 in half-precision range
    try std.testing.expectEqual(combine_confidence(a, b), combine_confidence(b, a));
}

test "combine_confidence_product_range" {
    // Result should be <= both inputs for values in [0, GF16_ONE]
    const a: u16 = 0x3800;
    const b: u16 = 0x3000;
    const result = combine_confidence(a, b);
    try std.testing.expect(result <= a);
    try std.testing.expect(result <= b);
}

test "f32_to_trit_positive" {
    // Values > 0.5 -> K_TRUE
    try std.testing.expectEqual(K_TRUE, f32_to_trit(0.6));
    try std.testing.expectEqual(K_TRUE, f32_to_trit(0.9));
    try std.testing.expectEqual(K_TRUE, f32_to_trit(1.0));
}

test "f32_to_trit_negative" {
    // Values < -0.5 -> K_FALSE
    try std.testing.expectEqual(K_FALSE, f32_to_trit(-0.6));
    try std.testing.expectEqual(K_FALSE, f32_to_trit(-0.9));
    try std.testing.expectEqual(K_FALSE, f32_to_trit(-1.0));
}

test "f32_to_trit_unknown" {
    // Values in [-0.5, 0.5] -> K_UNKNOWN
    try std.testing.expectEqual(K_UNKNOWN, f32_to_trit(0.0));
    try std.testing.expectEqual(K_UNKNOWN, f32_to_trit(0.3));
    try std.testing.expectEqual(K_UNKNOWN, f32_to_trit(-0.3));
    try std.testing.expectEqual(K_UNKNOWN, f32_to_trit(0.5));
    try std.testing.expectEqual(K_UNKNOWN, f32_to_trit(-0.5));
}

test "f32_to_trit_boundary" {
    // Exactly 0.5 -> K_UNKNOWN (not strictly greater)
    try std.testing.expectEqual(K_UNKNOWN, f32_to_trit(0.5));
    // Exactly -0.5 -> K_UNKNOWN (not strictly less)
    try std.testing.expectEqual(K_UNKNOWN, f32_to_trit(-0.5));
    // Just above 0.5 -> K_TRUE
    try std.testing.expectEqual(K_TRUE, f32_to_trit(0.50001));
    // Just below -0.5 -> K_FALSE
    try std.testing.expectEqual(K_FALSE, f32_to_trit(-0.50001));
}

test "satisfaction_with_zero_threshold" {
    // When threshold is zero, satisfaction should be GF16_ONE
    const pipeline = create_pipeline(.CNN_RULES, GF16_ONE, &[_]HornClause{}, 0);
    const sat = calculate_satisfaction(&pipeline, K_TRUE, GF16_ONE);
    try std.testing.expectEqual(GF16_ONE, sat);
}

test "satisfaction_meets_threshold" {
    // When confidence meets threshold, satisfaction should be high
    var pipeline = create_pipeline(.CNN_RULES, GF16_ONE, &[_]HornClause{}, 0);
    pipeline.ar_component.confidence_threshold = 0x3000;
    const sat = calculate_satisfaction(&pipeline, K_TRUE, GF16_ONE);
    try std.testing.expectEqual(GF16_ONE, sat);
}

test "satisfaction_below_threshold" {
    // When confidence is below threshold, satisfaction is partial
    var pipeline = create_pipeline(.CNN_RULES, GF16_ONE, &[_]HornClause{}, 0);
    pipeline.ar_component.confidence_threshold = GF16_ONE;
    const sat = calculate_satisfaction(&pipeline, K_TRUE, 0x3000);
    // Should be less than GF16_ONE
    try std.testing.expect(sat < GF16_ONE);
}

test "satisfaction_penalty_for_terminated_pipeline" {
    // Terminated pipelines get satisfaction penalty
    var pipeline = create_pipeline(.CNN_RULES, GF16_ONE, &[_]HornClause{}, 0);
    pipeline.ar_component.confidence_threshold = 0x1000;
    pipeline.terminated = true;
    const sat_terminated = calculate_satisfaction(&pipeline, K_TRUE, GF16_ONE);

    pipeline.terminated = false;
    const sat_normal = calculate_satisfaction(&pipeline, K_TRUE, GF16_ONE);

    try std.testing.expect(sat_terminated < sat_normal);
}

test "transformer_xai_respects_10_step_limit" {
    // Request 15 steps, should be capped at MAX_STEPS (10)
    const input = [_]f32{0.7} ** 10;
    const result = compose_transformer_xai(&input, GF16_ONE, 15, .natural);
    try std.testing.expect(result.explanation.step_count <= MAX_STEPS);
}

test "composition_pattern_coverage" {
    // All 4 composition patterns are defined and reachable
    const patterns = [_]CompositionPattern{ .CNN_RULES, .MLP_BAYESIAN, .TRANSFORMER_XAI, .RL_GUARDRAILS };
    const input = [_]f32{0.5};
    for (patterns) |p| {
        const result = compose(p, &input, GF16_ONE, &[_]HornClause{}, 0, GF16_ZERO);
        // Each pattern should produce a valid Trit prediction
        const pi = @intFromEnum(result.prediction);
        try std.testing.expect(pi >= -1 and pi <= 1);
    }
}

test "confidence_in_valid_range" {
    // All composition results have confidence in valid GF16 range
    const patterns = [_]CompositionPattern{ .CNN_RULES, .MLP_BAYESIAN, .TRANSFORMER_XAI, .RL_GUARDRAILS };
    const input = [_]f32{0.5};
    for (patterns) |p| {
        const result = compose(p, &input, GF16_ONE, &[_]HornClause{}, 0, GF16_ZERO);
        try std.testing.expect(result.confidence <= GF16_ONE);
    }
}

test "gf16_roundtrip" {
    // GF16 encode/decode roundtrip for key values
    try std.testing.expectEqual(GF16_ZERO, gf16_from_f32(0.0));
    try std.testing.expectEqual(GF16_ONE, gf16_from_f32(1.0));
    try std.testing.expectEqual(@as(f32, 0.0), gf16_to_f32(GF16_ZERO));
    try std.testing.expectEqual(@as(f32, 1.0), gf16_to_f32(GF16_ONE));
}

test "evaluate_ar_rules_matching" {
    const rules = [_]HornClause{
        .{ .antecedent = K_TRUE, .consequent = K_TRUE },
    };
    const ar_comp = ARComponent{
        .rule_set = &rules,
        .rule_count = 1,
        .explanation_style = .natural,
        .confidence_threshold = GF16_ZERO,
    };
    // Input matches antecedent -> consequent returned
    try std.testing.expectEqual(K_TRUE, evaluate_ar_rules(&ar_comp, K_TRUE));
}

test "evaluate_ar_rules_no_match" {
    const rules = [_]HornClause{
        .{ .antecedent = K_TRUE, .consequent = K_TRUE },
    };
    const ar_comp = ARComponent{
        .rule_set = &rules,
        .rule_count = 1,
        .explanation_style = .natural,
        .confidence_threshold = GF16_ZERO,
    };
    // Input does not match any antecedent -> K_UNKNOWN
    try std.testing.expectEqual(K_UNKNOWN, evaluate_ar_rules(&ar_comp, K_FALSE));
}

test "evaluate_ar_rules_empty" {
    const ar_comp = ARComponent{
        .rule_set = &[_]HornClause{},
        .rule_count = 0,
        .explanation_style = .natural,
        .confidence_threshold = GF16_ZERO,
    };
    // No rules -> K_UNKNOWN
    try std.testing.expectEqual(K_UNKNOWN, evaluate_ar_rules(&ar_comp, K_TRUE));
}
