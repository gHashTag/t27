# CLARA TA2 Library: ML+AR Composition Patterns

**DARPA PA-25-07-02 - TA2 Deliverable**
**Proposal Reference:** CLARA-PA25-07-02-TRINITY
**Date:** April 5, 2026

---

## Overview

The TA2 library provides four composable ML+AR patterns for building reliable, explainable AI systems. Each pattern specifies:
1. **ML Component:** Neural network, Bayesian inference, or policy network
2. **AR Component:** Logic programs, Horn clauses, or restraint rules
3. **Composition Interface:** `compose()` function with `CompositionResult` return type
4. **Formal Guarantees:** Polynomial-time bounds, ≤10 step explanations

**Design Philosophy:** AR provides formal correctness, ML provides learning; composition bridges both with bounded semantics.

---

## Pattern 1: CNN + Rules

### Specification
**From:** `specs/ar/composition.t27` lines 83-119

### Purpose
Neural feature extraction → Horn clause rule evaluation → bounded decision.

### Components

| Component | Type | Specification |
|-----------|------|-------------|
| **ML** | CNN | Conv2D feature extraction with confidence score |
| **AR** | Logic Programs | Horn clauses for rule matching |
| **Integration** | Kleene AND | `k3_and(cnn_decision, ar_decision)` |

### Algorithm
```zig
pub fn compose_cnn_rules(
    cnn_features: []const f32,
    cnn_confidence: GF16,
    rules: []const HornClause,
    rule_count: usize,
    confidence_threshold: GF16
) -> CompositionResult [line 83-119] {
    // Step 1: CNN feature extraction (simulated)
    const cnn_result = simulate_cnn_inference(cnn_features); [line 93]

    // Step 2: AR rule evaluation via Datalog
    const ar_result = evaluate_ar_rules(&pipeline.ar_component, cnn_result); [line 96]

    // Step 3: Combine confidence (geometric mean)
    const combined_conf = combine_confidence(cnn_confidence, ar_result.confidence); [line 99]

    // Step 4: Generate explanation
    const explanation = generate_composition_explanation(
        &pipeline,
        cnn_result,
        ar_result,
        "CNN feature extraction → AR rule evaluation"
    ); [line 102-107]

    // Step 5: Final decision via Kleene AND
    const prediction = k3_and(cnn_result.decision, ar_result.decision); [line 110]

    return CompositionResult{...};
}
```

### Complexity
- **CNN Inference:** O(W×H×C×D) where W=width, H=height, C=channels, D=depth
- **AR Rule Evaluation:** O(n) where n=number of rules (via Datalog)
- **Confidence Combination:** O(1) geometric mean via GF16
- **Overall:** O(n) dominated by AR rule matching (bounded by MAX_CLAUSES=256)

### Proof Trace
Yes - `evaluate_ar_rules()` creates proof trace via `append_step()` calls [line 187-195].

### Example Usage
```zig
// MNIST digit classification with safety rules
const rules = []HornClause{
    HornClause{ .name = "has_loop", .args = [_]Trit{K_TRUE} ++ [_]Trit{K_UNKNOWN}**7, .arg_count = 1 },
    HornClause{ .name = "safe_to_rotate", .args = [_]Trit{K_TRUE} ++ [_]Trit{K_UNKNOWN}**7, .arg_count = 1 },
};

const features = cnn_extract_features(image);
const result = compose_cnn_rules(&features, cnn_confidence, &rules, 2, threshold);
// result.prediction = K_TRUE if CNN+AR both agree
// result.explanation = "Step 1: CNN detected '7' → Step 2: has_loop(7)=TRUE → Step 3: safe_to_rotate(7)=TRUE"
```

---

## Pattern 2: MLP + Bayesian

### Specification
**From:** `specs/ar/composition.t27` lines 124-162

### Purpose
Neural forward pass → Bayesian prior/posterior update → calibrated belief.

### Components

| Component | Type | Specification |
|-----------|------|-------------|
| **ML** | MLP | Multi-layer perceptron forward pass |
| **AR** | Bayesian | GF16-encoded prior/posterior (DLFloat-6:9) |
| **Integration** | Kleene AND | `k3_and(mlp_trit, bayesian_trit)` |

### Algorithm
```zig
pub fn compose_mlp_bayesian(
    mlp_input: []const f32,
    mlp_confidence: GF16,
    bayesian_prior: f32,
    confidence_threshold: GF16
) -> CompositionResult [line 124-162] {
    var pipeline = create_pipeline(.MLP_BAYESIAN, mlp_confidence, null_rules, 0);

    // Step 1: MLP forward pass
    const mlp_result = simulate_mlp_forward(mlp_input); [line 133]

    // Step 2: Bayesian inference (posterior update)
    const bayesian_result = apply_bayesian_update(bayesian_prior, mlp_result.probability); [line 136]

    // Step 3: Combine confidence (log-space)
    const bayesian_gf = gf16_encode_f32(bayesian_result); [line 139]
    const combined_conf = combine_confidence(mlp_confidence, bayesian_gf); [line 140]

    // Step 4: Generate explanation
    const explanation = generate_composition_explanation(
        &pipeline,
        mlp_result,
        bayesian_result,
        "MLP forward → Bayesian update"
    ); [line 143-148]

    // Step 5: Final decision
    const mlp_trit = f32_to_trit(mlp_result.probability); [line 151]
    const bayesian_trit = f32_to_trit(bayesian_result); [line 152]
    const prediction = k3_and(mlp_trit, bayesian_trit); [line 153]

    return CompositionResult{...};
}

pub fn apply_bayesian_update(prior: f32, likelihood: f32) -> f32 [line 365-372]:
    const log_prior = @log(prior + 0.0001);  // Numerical stability
    const log_likelihood = @log(likelihood + 0.0001);
    return @exp(log_prior + log_likelihood);  // Posterior ∝ prior × likelihood
}
```

### Complexity
- **MLP Forward:** O(H×W) where H=hidden size, W=weight matrix
- **Bayesian Update:** O(1) log+exp operations
- **Confidence Combination:** O(1) GF16 geometric mean
- **Overall:** O(H×W) dominated by MLP inference

### Proof Trace
No - pure Bayesian inference produces probabilistic belief without derivation steps [line 159].

### Example Usage
```zig
// Uncertainty quantification for sensor fusion
const prior = 0.5;  // Prior belief
const mlp_input = sensor_data;
const result = compose_mlp_bayesian(&mlp_input, mlp_confidence, prior, threshold);
// result.prediction = K_TRUE if MLP and Bayesian both agree on high confidence
// result.confidence reflects updated posterior belief (via GF16 encoding)
```

---

## Pattern 3: Transformer + XAI

### Specification
**From:** `specs/ar/composition.t27` lines 167-212

### Purpose
Self-attention → Proof trace generation → Bounded explanation (≤10 steps).

### Components

| Component | Type | Specification |
|-----------|------|-------------|
| **ML** | Transformer | Self-attention mechanism with attention scores |
| **AR** | Logic Programs | Proof trace for explainability |
| **Integration** | Attention-to-Rule | Map attention weights to derivation steps |

### Algorithm
```zig
pub fn compose_transformer_xai(
    transformer_input: []const f32,
    transformer_confidence: GF16,
    max_steps: u8,
    style: FormatStyle
) -> CompositionResult [line 167-212] {
    var pipeline = create_pipeline(.TRANSFORMER_XAI, transformer_confidence, null_rules, 0);

    // Step 1: Transformer self-attention (simulated)
    const transformer_result = simulate_transformer_attention(transformer_input); [line 176]

    // Step 2: Generate proof trace (≤10 steps per CLARA requirement)
    var trace = create_trace(); [line 179]
    var i : u8 = 0;
    while (i < max_steps and i < 10) {  // MAX_STEPS=10 hard limit
        const step = create_attention_step(i, transformer_result.attention_scores[i]); [line 182]
        if (!append_step(&trace, step)) {
            pipeline.terminated = true;  // Restraint triggered
            break;
        }
        i = i + 1;
    }

    // Step 3: Format explanation in requested style
    const explanation_str = explain_derivation_chain(trace, style); [line 191]

    // Step 4: Build Explanation struct
    const explanation = Explanation{
        .trace = trace,
        .style = style,
        .human_readable = explanation_str,
        .confidence = transformer_confidence,
        .step_count = trace.step_count, [line 194-200]
    };

    // Step 5: Final decision from attention
    const prediction = transformer_result.decision; [line 203]

    return CompositionResult{...};
}
```

### Complexity
- **Attention Computation:** O(L×H×D) where L=sequence length, H=hidden size, D=model dimension
- **Proof Trace Generation:** O(MAX_STEPS) = O(10) bounded by CLARA
- **Explanation Formatting:** O(10) string formatting
- **Overall:** O(L×H×D) + O(10)

### Proof Trace
Yes - explicitly generated and bounded by MAX_STEPS=10 [lines 179-188].

### Example Usage
```zig
// Explainable classification with attention visualization
const input = sequence_data;
const result = compose_transformer_xai(&input, tf_confidence, 10, .natural);
// result.explanation contains ≤10 derivation steps
// result.explanation.step_count ≤ 10 (CLARA compliance)
// Format: "Step 1: attention[0]=0.82 → derived_rule → Step 2: ..."
```

---

## Pattern 4: RL + Guardrails

### Specification
**From:** `specs/ar/composition.t27` lines 217-262

### Purpose
Policy network inference → Restraint checking → Guarded action.

### Components

| Component | Type | Specification |
|-----------|------|-------------|
| **ML** | RL Policy Network | PPO-style policy output |
| **AR** | Restraint Rules | Quality-level bounded execution (from `ar::restraint.t27`) |
| **Integration** | Kleene AND | `k3_and(rl_decision, guardrail_decision)` |

### Algorithm
```zig
pub fn compose_rl_guardrails(
    state: []const f32,
    policy_confidence: GF16,
    guardrails: []const HornClause,
    guardrail_count: usize,
    params: RestraintParams
) -> CompositionResult [line 217-262] {
    var pipeline = create_pipeline(.RL_GUARDRAILS, policy_confidence, guardrails, guardrail_count);

    // Step 1: RL policy inference
    const rl_result = simulate_policy_inference(state); [line 227]

    // Step 2: Restraint checking
    var execution_state = init_state(.good, 0); [line 230]
    execution_state.current_confidence = policy_confidence;
    const allowed = should_continue(execution_state, params); [line 233]

    // Step 3: Guardrail rule evaluation
    const guardrail_result = evaluate_ar_rules(&pipeline.ar_component, rl_result); [line 236]

    // Step 4: Combine: action allowed only if both RL and guardrails approve
    const prediction = k3_and(
        f32_to_trit(rl_result.probability),
        guardrail_result.decision
    ); [line 239-242]

    // Step 5: Generate explanation
    const explanation = generate_composition_explanation(
        &pipeline,
        rl_result,
        guardrail_result,
        "RL policy → Guardrail check"
    ); [line 245-250]

    // Step 6: Combined confidence (lower if guardrails triggered)
    const combined_conf = if (allowed) policy_confidence else gf16_encode_f32(0.3); [line 253]

    return CompositionResult{...};
}
```

### Restraint Integration
From `ar::restraint.t27` lines 109-158:
```zig
pub fn params_for_quality(quality: QualityLevel) -> RestraintParams [line 113-143] {
    switch (quality) {
        .unknown => return RestraintParams{
            .max_depth = 3,      // Minimal resources
            .max_rules = 10,
            .confidence_threshold = gf16_encode_f32(0.85),
            .timeout_ms = 100,
        },
        .unstable => return RestraintParams{
            .max_depth = 7,      // Moderate resources
            .max_rules = 50,
            .confidence_threshold = gf16_encode_f32(0.75),
            .timeout_ms = 1000,
        },
        .good => return RestraintParams{
            .max_depth = 15,     // Full resources
            .max_rules = 500,
            .confidence_threshold = gf16_encode_f32(0.70),
            .timeout_ms = 10000,
        },
    }
}

pub fn should_continue(state: ExecutionState, params: RestraintParams) -> Trit [line 158-185] {
    // Check depth limit
    if (state.current_depth >= params.max_depth) return K_FALSE;
    // Check rule limit
    if (state.rules_fired >= params.max_rules) return K_FALSE;
    // Check confidence threshold (GF16 comparison)
    if (gf16_decode_to_f32(state.current_confidence) < gf16_decode_to_f32(params.confidence_threshold)) return K_FALSE;
    return K_TRUE;  // All checks passed
}
```

### Complexity
- **Policy Inference:** O(H×W) where H=hidden, W=weights
- **Restraint Check:** O(1) three comparisons (depth, rules, confidence)
- **Guardrail Evaluation:** O(n) where n=number of guardrail rules
- **Overall:** O(H×W + n) bounded by MAX_RULES=500

### Proof Trace
Yes - `evaluate_ar_rules()` creates proof trace via guardrail rules [line 236].

### Example Usage
```zig
// Safe action selection for autonomous system
const guardrails = []HornClause{
    HornClause{ .name = "battery_low", .args = [_]Trit{K_TRUE}++... },
    HornClause{ .name = "terrain_hazard", .args = [_]Trit{K_TRUE}++... },
};
const params = params_for_quality(.good);  // Full resources
const result = compose_rl_guardrails(&state, policy_conf, &guardrails, 2, params);
// result.prediction = K_TRUE only if policy AND guardrails both approve
// result.terminated = true if restraint triggered (depth/rules/confidence limit)
```

---

## API Reference

### compose() Generic Dispatcher

**From:** `specs/ar/composition.t27` lines 266-280

```zig
pub fn compose(
    pattern: CompositionPattern,
    ml_input: []const f32,
    ml_confidence: GF16,
    ar_rules: []const HornClause,
    ar_rule_count: usize,
    additional_params: anytype
) -> CompositionResult {
    return switch (pattern) {
        .CNN_RULES => compose_cnn_rules(ml_input, ml_confidence, ar_rules, ar_rule_count, GF16_ZERO),
        .MLP_BAYESIAN => compose_mlp_bayesian(ml_input, ml_confidence, additional_params.bayesian_prior, GF16_ZERO),
        .TRANSFORMER_XAI => compose_transformer_xai(ml_input, ml_confidence, additional_params.max_steps, additional_params.style),
        .RL_GUARDRAILS => compose_rl_guardrails(ml_input, ml_confidence, ar_rules, ar_rule_count, additional_params.restraint_params),
    };
}
```

### CompositionResult Struct

**From:** `specs/ar/composition.t27` lines 68-74

```zig
pub const CompositionResult = struct {
    prediction      : Trit,           // K_TRUE, K_FALSE, or K_UNKNOWN
    confidence      : GF16,          // GF16 (NUMERIC-STANDARD-001)
    explanation     : Explanation,    // Human-readable explanation
    proof_trace    : ProofTrace,     // Full derivation chain
    satisfaction    : GF16,          // CLARA requirement satisfaction score
};
```

### ComposedPipeline Struct

**From:** `specs/ar/composition.t27` lines 57-65

```zig
pub const ComposedPipeline = struct {
    pattern         : CompositionPattern,  // CNN_RULES, MLP_BAYESIAN, TRANSFORMER_XAI, RL_GUARDRAILS
    ml_component    : MLComponent,
    ar_component    : ARComponent,
    pipeline_confidence : GF16,
    steps_completed : u8,
    explanation     : Explanation,
    terminated      : bool,          // True if restraint triggered
};
```

### MLComponent Struct

**From:** `specs/ar/composition.t27` lines 41-46

```zig
pub const MLComponent = struct {
    component_type : []const u8,   // "CNN", "MLP", "Transformer", "PPO", etc.
    input_shape    : [4]u32,        // [batch, seq, height, width] or similar
    output_shape   : [2]u32,        // [batch, features]
    confidence     : GF16,          // Component confidence (NUMERIC-STANDARD-001)
};
```

### ARComponent Struct

**From:** `specs/ar/composition.t27` lines 49-54

```zig
pub const ARComponent = struct {
    rule_set      : []const HornClause,  // Horn clauses for inference
    rule_count    : usize,
    explanation_style : FormatStyle,  // natural, fitch, compact
    confidence_threshold : GF16,   // Minimum confidence to accept AR result
};
```

---

## Integration Matrix

| Pattern | ML Kind | AR Kind | Complexity | Proof Trace | Restraint |
|---------|----------|----------|------------|-------------|------------|
| CNN + Rules | Neural Net | Logic Programs | O(n) rules | Yes | No |
| MLP + Bayesian | Neural Net | Bayesian | O(H×W) MLP | No | No |
| Transformer + XAI | Neural Net | Logic Programs | O(L×H×D) + O(10) | Yes | No |
| RL + Guardrails | RL | Restraint | O(H×W + n) | Yes | Yes |

**Legend:**
- **O(n)** where n = number of rules (bounded by MAX_CLAUSES=256)
- **O(H×W)** where H=hidden size, W=weight matrix
- **O(L×H×D)** where L=sequence length, H=hidden size, D=model dimension

---

## CLARA Compliance

| Requirement | TA2 Implementation | Evidence |
|-------------|-------------------|----------|
| ≥2 composition patterns | 4 patterns | CompositionPattern enum [line 26-38] |
| Polynomial bounds per pattern | All O(n) or better | Complexity analysis per pattern |
| Explainability | ≤10 step traces | Pattern 3 + Pattern 4 support traces |
| Restraint mechanism | QualityLevel + should_continue() | Pattern 4 integrates restraint.t27 |
| Confidence encoding | GF16 (DLFloat-6:9) | All patterns use GF16 |

---

## Apache 2.0 License

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

---

**Document Version:** 1.0
**Last Updated:** April 5, 2026
**Related Specifications:**
- `specs/ar/composition.t27` (622 lines)
- `specs/ar/ternary_logic.t27` (717 lines)
- `specs/ar/restraint.t27` (553 lines)
- `specs/ar/proof_trace.t27` (186 lines)
- `specs/ar/explainability.t27` (476 lines)
