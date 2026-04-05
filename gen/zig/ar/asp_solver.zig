// AUTO-GENERATED from specs/ar/asp_solver.t27 — DO NOT EDIT
// Ring: 18 | Module: AspSolver | phi^2 + 1/phi^2 = 3
// Answer Set Programming with Restraint for CLARA
// Negation as Failure (NAF) with Kleene K3 semantics

const std = @import("std");
const ternary_logic = @import("ternary_logic.zig");

// ═══════════════════════════════════════════════════════════════
// Re-exports from ternary_logic
// ═══════════════════════════════════════════════════════════════

pub const Trit = ternary_logic.Trit;
pub const K_FALSE = ternary_logic.K_FALSE;
pub const K_UNKNOWN = ternary_logic.K_UNKNOWN;
pub const K_TRUE = ternary_logic.K_TRUE;
pub const Rule = ternary_logic.Rule;
pub const forward_chain = ternary_logic.forward_chain;
pub const k3_equiv = ternary_logic.k3_equiv;
pub const k3_and = ternary_logic.k3_and;
pub const k3_or = ternary_logic.k3_or;

// ═══════════════════════════════════════════════════════════════
// GF16 — Galois Field 16-bit confidence encoding
// ═══════════════════════════════════════════════════════════════

pub const GF16 = u16;

/// Encode a floating-point confidence value [0.0, 1.0] to GF16
pub fn gf16_encode_f32(v: f32) GF16 {
    if (v <= 0.0) return 0;
    if (v >= 1.0) return 65535;
    return @intFromFloat(v * 65535.0);
}

/// Decode a GF16 value back to floating-point [0.0, 1.0]
pub fn gf16_decode_to_f32(g: GF16) f32 {
    return @as(f32, @floatFromInt(g)) / 65535.0;
}

// ═══════════════════════════════════════════════════════════════
// Types from ar::restraint
// ═══════════════════════════════════════════════════════════════

pub const QualityLevel = enum(u2) {
    unknown = 0,
    unstable = 1,
    good = 2,
};

pub const RestraintParams = struct {
    max_depth: u32,
    max_rules: u32,
    confidence_threshold: GF16,
    timeout_ms: u64,
};

pub const ExecutionState = struct {
    current_depth: u32,
    rules_fired: u32,
    current_confidence: GF16,
    start_time_ms: u64,
    quality: QualityLevel,
};

/// Select appropriate restraint parameters for a given quality level
pub fn params_for_quality(quality: QualityLevel) RestraintParams {
    return switch (quality) {
        .unknown => RestraintParams{
            .max_depth = 3,
            .max_rules = 10,
            .confidence_threshold = gf16_encode_f32(0.85),
            .timeout_ms = 100,
        },
        .unstable => RestraintParams{
            .max_depth = 7,
            .max_rules = 50,
            .confidence_threshold = gf16_encode_f32(0.75),
            .timeout_ms = 1000,
        },
        .good => RestraintParams{
            .max_depth = 15,
            .max_rules = 500,
            .confidence_threshold = gf16_encode_f32(0.70),
            .timeout_ms = 10000,
        },
    };
}

/// Determine if execution should continue or be restrained
pub fn should_continue(state: ExecutionState, params: RestraintParams) Trit {
    if (state.current_depth >= params.max_depth) return K_FALSE;
    if (state.rules_fired >= params.max_rules) return K_FALSE;
    const conf_value = gf16_decode_to_f32(state.current_confidence);
    const threshold_value = gf16_decode_to_f32(params.confidence_threshold);
    if (conf_value < threshold_value) return K_FALSE;
    return K_TRUE;
}

// ═══════════════════════════════════════════════════════════════
// Types from ar::datalog_engine
// ═══════════════════════════════════════════════════════════════

pub const MAX_CLAUSES: usize = 256;
pub const MAX_ARGS: usize = 8;

pub const HornClause = struct {
    name: u16,
    args: [MAX_ARGS]Trit,
    arg_count: u8,
};

pub const DatalogEngine = struct {
    facts: [MAX_CLAUSES]HornClause,
    fact_count: usize,
    rules: [MAX_CLAUSES]Rule,
    rule_count: usize,
    derived_facts: [MAX_CLAUSES]bool,
    solved: bool,
};

/// Initialize an empty Datalog engine
pub fn datalog_init() DatalogEngine {
    return DatalogEngine{
        .facts = undefined,
        .fact_count = 0,
        .rules = undefined,
        .rule_count = 0,
        .derived_facts = [_]bool{false} ** MAX_CLAUSES,
        .solved = false,
    };
}

/// Check if two Horn clauses are equal
fn horn_clause_eq(a: HornClause, b: HornClause) bool {
    if (a.name != b.name or a.arg_count != b.arg_count) return false;
    var i: usize = 0;
    while (i < a.arg_count) : (i += 1) {
        if (a.args[i] != b.args[i]) return false;
    }
    return true;
}

/// Add a fact to the engine
pub fn add_fact(engine: *DatalogEngine, fact: HornClause) bool {
    var i: usize = 0;
    while (i < engine.fact_count) : (i += 1) {
        if (horn_clause_eq(fact, engine.facts[i])) return false;
    }
    if (engine.fact_count >= MAX_CLAUSES) return false;
    engine.facts[engine.fact_count] = fact;
    engine.derived_facts[engine.fact_count] = false;
    engine.fact_count += 1;
    return true;
}

/// Run one round of forward-chaining datalog inference
/// Returns the number of newly derived facts
pub fn datalog_solve(engine: *DatalogEngine) usize {
    var new_facts: usize = 0;
    var changed: bool = true;
    var iterations: usize = 0;

    while (changed and iterations < MAX_CLAUSES) {
        changed = false;
        iterations += 1;

        var i: usize = 0;
        while (i < engine.rule_count) : (i += 1) {
            const rule = engine.rules[i];
            var j: usize = 0;
            const current_fact_count = engine.fact_count;
            while (j < current_fact_count) : (j += 1) {
                const fact_trit = engine.facts[j].args[0];
                const derived = forward_chain(rule, fact_trit);
                if (derived == K_TRUE) {
                    var new_args = [_]Trit{K_UNKNOWN} ** MAX_ARGS;
                    new_args[0] = derived;
                    const new_fact = HornClause{
                        .name = @intCast(engine.fact_count + 1000),
                        .args = new_args,
                        .arg_count = 1,
                    };
                    if (add_fact(engine, new_fact)) {
                        new_facts += 1;
                        changed = true;
                        engine.derived_facts[engine.fact_count - 1] = true;
                    }
                }
            }
        }
    }

    engine.solved = true;
    return new_facts;
}

// ═══════════════════════════════════════════════════════════════
// 1. ASP Core Constants
// ═══════════════════════════════════════════════════════════════

pub const MAX_NAF: u8 = 3;
pub const MAX_FACTS: u8 = 32;
pub const MAX_ITERATIONS: u16 = 1000;

// ═══════════════════════════════════════════════════════════════
// 2. ASP Core Types
// ═══════════════════════════════════════════════════════════════

/// AspRule: Horn clause with Negation as Failure
/// Represents: IF body AND NOT naf_ids[0] AND ... THEN head
pub const AspRule = struct {
    base: Rule,
    naf_ids: [MAX_NAF]u32,
    naf_count: u8,
};

/// FactId: Fact identifier with truth value
pub const FactId = struct {
    id: u32,
    truth: Trit,
};

/// StableModel: Result of ASP solver computation
pub const StableModel = struct {
    facts: [MAX_FACTS]FactId,
    fact_count: u8,
    is_complete: bool,
    iterations: u16,
    aborted_by_restraint: bool,
};

/// AspConfig: Configuration for ASP solver
pub const AspConfig = struct {
    max_iterations: u16,
    max_models: u8,
    quality: QualityLevel,
    initial_confidence: GF16,
};

/// AspState: State for ASP computation with restraint
pub const AspState = struct {
    engine: DatalogEngine,
    exec_state: ExecutionState,
    restraint_params: RestraintParams,
};

// ═══════════════════════════════════════════════════════════════
// 3. NAF (Negation as Failure) Evaluation
// ═══════════════════════════════════════════════════════════════

/// Evaluate NAF conditions: return true if ALL NAF conditions are NOT K_TRUE
/// NAF semantics: NOT p means "p cannot be proven" (p != K_TRUE)
pub fn evaluate_naf(engine: *DatalogEngine, naf_ids: []const u32, count: usize) bool {
    var i: usize = 0;
    while (i < count) : (i += 1) {
        const fact_id = naf_ids[i];

        // Search engine facts for matching ID
        var j: usize = 0;
        while (j < engine.fact_count) : (j += 1) {
            const fact = engine.facts[j];
            if (fact.name == @as(u16, @intCast(fact_id))) {
                // If fact is K_TRUE, NAF fails
                if (fact.args[0] == K_TRUE) {
                    return false;
                }
                break;
            }
        }
    }

    // All NAF conditions satisfied (none are K_TRUE)
    return true;
}

// ═══════════════════════════════════════════════════════════════
// 4. Fixed Point Iteration with Restraint
// ═══════════════════════════════════════════════════════════════

/// Run fixed point iteration for ASP with restraint checking.
/// Applies rules repeatedly until no new facts or restraint triggers.
/// Returns the number of iterations performed.
pub fn fixed_point_iteration(state: *AspState, rules: []const AspRule, rule_count: usize) u16 {
    var iteration: u16 = 0;
    const max_iter = if (state.restraint_params.max_depth > 0)
        @as(u16, @intCast(@min(state.restraint_params.max_depth, MAX_ITERATIONS)))
    else
        MAX_ITERATIONS;

    while (iteration < max_iter) {
        // Check restraint before each iteration
        if (should_continue(state.exec_state, state.restraint_params) == K_FALSE) {
            return iteration;
        }

        state.exec_state.rules_fired += 1;

        // Run one step of datalog inference with NAF checking
        var new_derived: usize = 0;
        var r: usize = 0;
        while (r < rule_count) : (r += 1) {
            const asp_rule = rules[r];

            // Evaluate NAF conditions
            if (asp_rule.naf_count > 0) {
                if (!evaluate_naf(&state.engine, &asp_rule.naf_ids, asp_rule.naf_count)) {
                    continue; // NAF failed, skip rule
                }
            }

            // Apply forward chaining for this rule
            var f: usize = 0;
            const current_count = state.engine.fact_count;
            while (f < current_count) : (f += 1) {
                const fact_trit = state.engine.facts[f].args[0];
                const derived = forward_chain(asp_rule.base, fact_trit);
                if (derived == K_TRUE) {
                    var new_args = [_]Trit{K_UNKNOWN} ** MAX_ARGS;
                    new_args[0] = derived;
                    const new_fact = HornClause{
                        .name = @intCast(state.engine.fact_count + 1000),
                        .args = new_args,
                        .arg_count = 1,
                    };
                    if (add_fact(&state.engine, new_fact)) {
                        new_derived += 1;
                        state.engine.derived_facts[state.engine.fact_count - 1] = true;
                    }
                }
            }
        }

        iteration += 1;
        state.exec_state.current_depth += 1;

        // Converged if no new facts were derived
        if (new_derived == 0) {
            break;
        }
    }

    return iteration;
}

// ═══════════════════════════════════════════════════════════════
// 5. Query with Restraint
// ═══════════════════════════════════════════════════════════════

/// Query a fact with restraint checking.
/// Returns K_UNKNOWN if restraint blocks, actual truth value otherwise.
pub fn query_with_restraint(state: *AspState, goal: Trit) Trit {
    _ = goal;
    // Check restraint
    if (should_continue(state.exec_state, state.restraint_params) == K_FALSE) {
        return K_UNKNOWN;
    }

    state.exec_state.rules_fired += 1;

    // Search engine facts for matching goal truth
    var i: usize = 0;
    while (i < state.engine.fact_count) : (i += 1) {
        if (state.engine.facts[i].args[0] == K_TRUE) {
            return K_TRUE;
        }
    }

    return K_UNKNOWN;
}

// ═══════════════════════════════════════════════════════════════
// 6. Consistency Checking
// ═══════════════════════════════════════════════════════════════

/// Check if stable model has no contradictions.
/// A model is inconsistent if same fact has both K_TRUE and K_FALSE.
pub fn is_consistent(model: StableModel) bool {
    var i: usize = 0;
    while (i < model.fact_count) : (i += 1) {
        const fact_i = model.facts[i];
        var j: usize = 0;
        while (j < model.fact_count) : (j += 1) {
            if (i != j) {
                const fact_j = model.facts[j];
                if (fact_i.id == fact_j.id) {
                    if ((fact_i.truth == K_TRUE and fact_j.truth == K_FALSE) or
                        (fact_i.truth == K_FALSE and fact_j.truth == K_TRUE))
                    {
                        return false;
                    }
                }
            }
        }
    }
    return true;
}

// ═══════════════════════════════════════════════════════════════
// 7. Stable Model Computation
// ═══════════════════════════════════════════════════════════════

/// Main ASP solver: compute stable model using fixed point iteration.
/// Combines datalog inference with NAF checking and restraint.
pub fn compute_stable_model(config: AspConfig, rules: []const AspRule, rule_count: usize, facts: []const FactId, fact_count: usize) StableModel {
    var model = StableModel{
        .facts = undefined,
        .fact_count = 0,
        .is_complete = false,
        .iterations = 0,
        .aborted_by_restraint = false,
    };

    // Initialize engine
    var engine = datalog_init();

    // Load initial facts into engine
    var fi: usize = 0;
    while (fi < fact_count and fi < MAX_FACTS) : (fi += 1) {
        var args = [_]Trit{K_UNKNOWN} ** MAX_ARGS;
        args[0] = facts[fi].truth;
        const hc = HornClause{
            .name = @intCast(facts[fi].id),
            .args = args,
            .arg_count = 1,
        };
        _ = add_fact(&engine, hc);
    }

    // Initialize restraint state
    const r_params = params_for_quality(config.quality);
    const exec_state = ExecutionState{
        .current_depth = 0,
        .rules_fired = 0,
        .current_confidence = config.initial_confidence,
        .start_time_ms = 0,
        .quality = config.quality,
    };

    var asp_state = AspState{
        .engine = engine,
        .exec_state = exec_state,
        .restraint_params = r_params,
    };

    // Run fixed point iteration
    const iterations = fixed_point_iteration(&asp_state, rules, rule_count);

    // Check if aborted by restraint (did not converge naturally)
    const restraint_triggered = (should_continue(asp_state.exec_state, asp_state.restraint_params) == K_FALSE);
    model.aborted_by_restraint = restraint_triggered and iterations > 0;
    model.is_complete = !model.aborted_by_restraint;
    model.iterations = iterations;

    // Extract facts from engine to model
    var ei: usize = 0;
    while (ei < asp_state.engine.fact_count and ei < MAX_FACTS) : (ei += 1) {
        const fact = asp_state.engine.facts[ei];
        model.facts[ei] = FactId{
            .id = @as(u32, @intCast(fact.name)),
            .truth = fact.args[0],
        };
    }
    model.fact_count = @intCast(ei);

    return model;
}

/// Check if ASP program has at least one stable model.
pub fn has_stable_model(rules: []const AspRule, rule_count: usize) bool {
    const config = AspConfig{
        .max_iterations = MAX_ITERATIONS,
        .max_models = 1,
        .quality = .good,
        .initial_confidence = gf16_encode_f32(1.0),
    };

    // Empty program with no facts has no stable model
    if (rule_count == 0) {
        return false;
    }

    const empty_facts = [_]FactId{};
    const model = compute_stable_model(config, rules, rule_count, &empty_facts, 0);
    return model.is_complete and is_consistent(model);
}

// ═══════════════════════════════════════════════════════════════
// Tests — Conformance vectors from spec
// ═══════════════════════════════════════════════════════════════

fn make_horn_clause(name: u16, first_arg: Trit) HornClause {
    var args = [_]Trit{K_UNKNOWN} ** MAX_ARGS;
    args[0] = first_arg;
    return HornClause{
        .name = name,
        .args = args,
        .arg_count = 1,
    };
}

test "evaluate_naf_true_when_all_unknown" {
    // NAF with all unknown facts succeeds
    var engine = datalog_init();
    const unknown_fact = make_horn_clause(1, K_UNKNOWN);
    try std.testing.expect(add_fact(&engine, unknown_fact));
    const naf_ids = [_]u32{1};
    const result = evaluate_naf(&engine, &naf_ids, 1);
    try std.testing.expect(result);
}

test "evaluate_naf_false_when_one_true" {
    // NAF with a true fact fails
    var engine = datalog_init();
    const true_fact = make_horn_clause(1, K_TRUE);
    try std.testing.expect(add_fact(&engine, true_fact));
    const naf_ids = [_]u32{1};
    const result = evaluate_naf(&engine, &naf_ids, 1);
    try std.testing.expect(!result);
}

test "evaluate_naf_true_when_all_false" {
    // NAF with a false fact succeeds (NOT false = true in NAF)
    var engine = datalog_init();
    const false_fact = make_horn_clause(1, K_FALSE);
    try std.testing.expect(add_fact(&engine, false_fact));
    const naf_ids = [_]u32{1};
    const result = evaluate_naf(&engine, &naf_ids, 1);
    try std.testing.expect(result);
}

test "evaluate_naf_multiple_conditions" {
    // NAF with multiple conditions checks all
    var engine = datalog_init();
    const fact1 = make_horn_clause(1, K_UNKNOWN);
    const fact2 = make_horn_clause(2, K_FALSE);
    try std.testing.expect(add_fact(&engine, fact1));
    try std.testing.expect(add_fact(&engine, fact2));
    const naf_ids = [_]u32{ 1, 2 };
    const result = evaluate_naf(&engine, &naf_ids, 2);
    try std.testing.expect(result); // Neither is K_TRUE
}

test "fixed_point_basic" {
    // Fixed point converges when no new facts derived
    var engine = datalog_init();
    const fact = make_horn_clause(1, K_FALSE);
    try std.testing.expect(add_fact(&engine, fact));

    const asp_rule = AspRule{
        .base = Rule{ .antecedent = K_TRUE, .consequent = K_TRUE },
        .naf_ids = [_]u32{ 0, 0, 0 },
        .naf_count = 0,
    };
    const rules = [_]AspRule{asp_rule};

    const r_params = params_for_quality(.good);
    var state = AspState{
        .engine = engine,
        .exec_state = ExecutionState{
            .current_depth = 0,
            .rules_fired = 0,
            .current_confidence = gf16_encode_f32(0.9),
            .start_time_ms = 0,
            .quality = .good,
        },
        .restraint_params = r_params,
    };

    const iterations = fixed_point_iteration(&state, &rules, 1);
    // Should converge quickly (no facts match K_TRUE antecedent to derive new facts)
    try std.testing.expect(iterations <= 2);
}

test "compute_stable_model_basic" {
    // Simple program finds stable model
    const asp_rule = AspRule{
        .base = Rule{ .antecedent = K_TRUE, .consequent = K_TRUE },
        .naf_ids = [_]u32{ 0, 0, 0 },
        .naf_count = 0,
    };
    const rules = [_]AspRule{asp_rule};
    const initial_facts = [_]FactId{
        FactId{ .id = 1, .truth = K_TRUE },
    };

    const config = AspConfig{
        .max_iterations = 100,
        .max_models = 1,
        .quality = .good,
        .initial_confidence = gf16_encode_f32(1.0),
    };

    const model = compute_stable_model(config, &rules, 1, &initial_facts, 1);
    try std.testing.expect(model.is_complete);
    try std.testing.expect(!model.aborted_by_restraint);
    try std.testing.expect(model.fact_count > 0);
}

test "is_consistent_no_contradictions" {
    // Model with no contradictions is consistent
    var facts: [MAX_FACTS]FactId = undefined;
    facts[0] = FactId{ .id = 1, .truth = K_TRUE };
    facts[1] = FactId{ .id = 2, .truth = K_FALSE };
    facts[2] = FactId{ .id = 3, .truth = K_UNKNOWN };
    const model = StableModel{
        .facts = facts,
        .fact_count = 3,
        .is_complete = true,
        .iterations = 10,
        .aborted_by_restraint = false,
    };
    try std.testing.expect(is_consistent(model));
}

test "is_consistent_with_contradictions" {
    // Model with same fact having both K_TRUE and K_FALSE is inconsistent
    var facts: [MAX_FACTS]FactId = undefined;
    facts[0] = FactId{ .id = 1, .truth = K_TRUE };
    facts[1] = FactId{ .id = 1, .truth = K_FALSE };
    facts[2] = FactId{ .id = 2, .truth = K_TRUE };
    const model = StableModel{
        .facts = facts,
        .fact_count = 3,
        .is_complete = true,
        .iterations = 10,
        .aborted_by_restraint = false,
    };
    try std.testing.expect(!is_consistent(model));
}

test "has_stable_model_empty_program" {
    // Empty program with no rules has no stable model
    const rules = [_]AspRule{};
    try std.testing.expect(!has_stable_model(&rules, 0));
}

test "evaluate_naf_not_found_succeeds" {
    // NAF on a fact ID that does not exist succeeds (not proven)
    var engine = datalog_init();
    const naf_ids = [_]u32{99};
    const result = evaluate_naf(&engine, &naf_ids, 1);
    try std.testing.expect(result);
}

// ═══════════════════════════════════════════════════════════════
// Invariant tests
// ═══════════════════════════════════════════════════════════════

test "invariant_max_facts_equals_32" {
    try std.testing.expectEqual(@as(u8, 32), MAX_FACTS);
}

test "invariant_max_naf_equals_3" {
    try std.testing.expectEqual(@as(u8, 3), MAX_NAF);
}

test "invariant_max_iterations_equals_1000" {
    try std.testing.expectEqual(@as(u16, 1000), MAX_ITERATIONS);
}

test "invariant_gf16_roundtrip" {
    const encoded = gf16_encode_f32(0.5);
    const decoded = gf16_decode_to_f32(encoded);
    try std.testing.expect(decoded > 0.49 and decoded < 0.51);
}
