/* AUTO-GENERATED from specs/ar/asp_solver.t27 — DO NOT EDIT */
/* Ring: 18 | Module: AspSolver | phi^2 + 1/phi^2 = 3 */
/* Answer Set Programming with Restraint for CLARA */
/* Negation as Failure (NAF) with Kleene K3 semantics */

#include "asp_solver.h"
#include <string.h>

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 Utility Functions                                         */
/* ═══════════════════════════════════════════════════════════════ */

GF16 gf16_encode_f32(float v) {
    if (v <= 0.0f) return 0;
    if (v >= 1.0f) return 65535;
    return (GF16)(v * 65535.0f);
}

float gf16_decode_to_f32(GF16 g) {
    return (float)g / 65535.0f;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Restraint Functions                                            */
/* ═══════════════════════════════════════════════════════════════ */

RestraintParams params_for_quality(QualityLevel quality) {
    RestraintParams params;
    switch (quality) {
        case QUALITY_UNKNOWN:
            params.max_depth = 3;
            params.max_rules = 10;
            params.confidence_threshold = gf16_encode_f32(0.85f);
            params.timeout_ms = 100;
            break;
        case QUALITY_UNSTABLE:
            params.max_depth = 7;
            params.max_rules = 50;
            params.confidence_threshold = gf16_encode_f32(0.75f);
            params.timeout_ms = 1000;
            break;
        case QUALITY_GOOD:
        default:
            params.max_depth = 15;
            params.max_rules = 500;
            params.confidence_threshold = gf16_encode_f32(0.70f);
            params.timeout_ms = 10000;
            break;
    }
    return params;
}

Trit should_continue(ExecutionState state, RestraintParams params) {
    if (state.current_depth >= params.max_depth) return K_FALSE;
    if (state.rules_fired >= params.max_rules) return K_FALSE;
    float conf_value = gf16_decode_to_f32(state.current_confidence);
    float threshold_value = gf16_decode_to_f32(params.confidence_threshold);
    if (conf_value < threshold_value) return K_FALSE;
    return K_TRUE;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Datalog Engine Functions                                       */
/* ═══════════════════════════════════════════════════════════════ */

static bool horn_clause_eq(HornClause a, HornClause b) {
    if (a.name != b.name || a.arg_count != b.arg_count) return false;
    for (uint8_t i = 0; i < a.arg_count; i++) {
        if (a.args[i] != b.args[i]) return false;
    }
    return true;
}

DatalogEngine datalog_init(void) {
    DatalogEngine engine;
    engine.fact_count = 0;
    engine.rule_count = 0;
    engine.solved = false;
    memset(engine.derived_facts, 0, sizeof(engine.derived_facts));
    return engine;
}

bool add_fact(DatalogEngine *engine, HornClause fact) {
    /* Check for duplicates */
    for (size_t i = 0; i < engine->fact_count; i++) {
        if (horn_clause_eq(fact, engine->facts[i])) return false;
    }
    /* Check capacity */
    if (engine->fact_count >= MAX_CLAUSES) return false;
    engine->facts[engine->fact_count] = fact;
    engine->derived_facts[engine->fact_count] = false;
    engine->fact_count++;
    return true;
}

size_t datalog_solve(DatalogEngine *engine) {
    size_t new_facts = 0;
    bool changed = true;
    size_t iterations = 0;

    while (changed && iterations < MAX_CLAUSES) {
        changed = false;
        iterations++;

        for (size_t i = 0; i < engine->rule_count; i++) {
            Rule rule = engine->rules[i];
            size_t current_count = engine->fact_count;
            for (size_t j = 0; j < current_count; j++) {
                Trit fact_trit = engine->facts[j].args[0];
                Trit derived = forward_chain(rule, fact_trit);
                if (derived == K_TRUE) {
                    HornClause new_fact;
                    new_fact.name = (uint16_t)(engine->fact_count + 1000);
                    memset(new_fact.args, 0, sizeof(new_fact.args)); /* K_UNKNOWN = 0 */
                    new_fact.args[0] = derived;
                    new_fact.arg_count = 1;
                    if (add_fact(engine, new_fact)) {
                        new_facts++;
                        changed = true;
                        engine->derived_facts[engine->fact_count - 1] = true;
                    }
                }
            }
        }
    }

    engine->solved = true;
    return new_facts;
}

/* ═══════════════════════════════════════════════════════════════ */
/* NAF (Negation as Failure) Evaluation                           */
/* ═══════════════════════════════════════════════════════════════ */

bool evaluate_naf(DatalogEngine *engine, const uint32_t *naf_ids, size_t count) {
    for (size_t i = 0; i < count; i++) {
        uint32_t fact_id = naf_ids[i];

        /* Search engine facts for matching ID */
        for (size_t j = 0; j < engine->fact_count; j++) {
            HornClause fact = engine->facts[j];
            if (fact.name == (uint16_t)fact_id) {
                /* If fact is K_TRUE, NAF fails */
                if (fact.args[0] == K_TRUE) {
                    return false;
                }
                break;
            }
        }
    }

    /* All NAF conditions satisfied (none are K_TRUE) */
    return true;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Fixed Point Iteration with Restraint                           */
/* ═══════════════════════════════════════════════════════════════ */

uint16_t fixed_point_iteration(AspState *state, const AspRule *rules, size_t rule_count) {
    uint16_t iteration = 0;
    uint16_t max_iter = (state->restraint_params.max_depth > 0 && state->restraint_params.max_depth < ASP_MAX_ITERATIONS)
        ? (uint16_t)state->restraint_params.max_depth
        : ASP_MAX_ITERATIONS;

    while (iteration < max_iter) {
        /* Check restraint before each iteration */
        if (should_continue(state->exec_state, state->restraint_params) == K_FALSE) {
            return iteration;
        }

        state->exec_state.rules_fired++;

        /* Run one step of inference with NAF checking */
        size_t new_derived = 0;
        for (size_t r = 0; r < rule_count; r++) {
            const AspRule *asp_rule = &rules[r];

            /* Evaluate NAF conditions */
            if (asp_rule->naf_count > 0) {
                if (!evaluate_naf(&state->engine, asp_rule->naf_ids, asp_rule->naf_count)) {
                    continue; /* NAF failed, skip rule */
                }
            }

            /* Apply forward chaining */
            size_t current_count = state->engine.fact_count;
            for (size_t f = 0; f < current_count; f++) {
                Trit fact_trit = state->engine.facts[f].args[0];
                Trit derived = forward_chain(asp_rule->base, fact_trit);
                if (derived == K_TRUE) {
                    HornClause new_fact;
                    new_fact.name = (uint16_t)(state->engine.fact_count + 1000);
                    memset(new_fact.args, 0, sizeof(new_fact.args));
                    new_fact.args[0] = derived;
                    new_fact.arg_count = 1;
                    if (add_fact(&state->engine, new_fact)) {
                        new_derived++;
                        state->engine.derived_facts[state->engine.fact_count - 1] = true;
                    }
                }
            }
        }

        iteration++;
        state->exec_state.current_depth++;

        /* Converged if no new facts derived */
        if (new_derived == 0) {
            break;
        }
    }

    return iteration;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Query with Restraint                                           */
/* ═══════════════════════════════════════════════════════════════ */

Trit query_with_restraint(AspState *state, Trit goal) {
    (void)goal;

    /* Check restraint */
    if (should_continue(state->exec_state, state->restraint_params) == K_FALSE) {
        return K_UNKNOWN;
    }

    state->exec_state.rules_fired++;

    /* Search engine facts */
    for (size_t i = 0; i < state->engine.fact_count; i++) {
        if (state->engine.facts[i].args[0] == K_TRUE) {
            return K_TRUE;
        }
    }

    return K_UNKNOWN;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Consistency Checking                                           */
/* ═══════════════════════════════════════════════════════════════ */

bool is_consistent(StableModel model) {
    for (uint8_t i = 0; i < model.fact_count; i++) {
        FactId fact_i = model.facts[i];
        for (uint8_t j = 0; j < model.fact_count; j++) {
            if (i != j) {
                FactId fact_j = model.facts[j];
                if (fact_i.id == fact_j.id) {
                    if ((fact_i.truth == K_TRUE && fact_j.truth == K_FALSE) ||
                        (fact_i.truth == K_FALSE && fact_j.truth == K_TRUE)) {
                        return false;
                    }
                }
            }
        }
    }
    return true;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Stable Model Computation                                       */
/* ═══════════════════════════════════════════════════════════════ */

StableModel compute_stable_model(AspConfig config, const AspRule *rules, size_t rule_count,
                                  const FactId *facts, size_t fact_count) {
    StableModel model;
    memset(&model, 0, sizeof(model));

    /* Initialize engine */
    DatalogEngine engine = datalog_init();

    /* Load initial facts into engine */
    for (size_t fi = 0; fi < fact_count && fi < ASP_MAX_FACTS; fi++) {
        HornClause hc;
        hc.name = (uint16_t)facts[fi].id;
        memset(hc.args, 0, sizeof(hc.args));
        hc.args[0] = facts[fi].truth;
        hc.arg_count = 1;
        add_fact(&engine, hc);
    }

    /* Initialize restraint state */
    RestraintParams r_params = params_for_quality(config.quality);
    ExecutionState exec_state;
    exec_state.current_depth = 0;
    exec_state.rules_fired = 0;
    exec_state.current_confidence = config.initial_confidence;
    exec_state.start_time_ms = 0;
    exec_state.quality = config.quality;

    AspState asp_state;
    asp_state.engine = engine;
    asp_state.exec_state = exec_state;
    asp_state.restraint_params = r_params;

    /* Run fixed point iteration */
    uint16_t iterations = fixed_point_iteration(&asp_state, rules, rule_count);

    /* Check if aborted by restraint */
    bool restraint_triggered = (should_continue(asp_state.exec_state, asp_state.restraint_params) == K_FALSE);
    model.aborted_by_restraint = restraint_triggered && (iterations > 0);
    model.is_complete = !model.aborted_by_restraint;
    model.iterations = iterations;

    /* Extract facts from engine to model */
    size_t ei = 0;
    while (ei < asp_state.engine.fact_count && ei < ASP_MAX_FACTS) {
        HornClause fact = asp_state.engine.facts[ei];
        model.facts[ei].id = (uint32_t)fact.name;
        model.facts[ei].truth = fact.args[0];
        ei++;
    }
    model.fact_count = (uint8_t)ei;

    return model;
}

bool has_stable_model(const AspRule *rules, size_t rule_count) {
    AspConfig config;
    config.max_iterations = ASP_MAX_ITERATIONS;
    config.max_models = 1;
    config.quality = QUALITY_GOOD;
    config.initial_confidence = gf16_encode_f32(1.0f);

    /* Empty program with no facts has no stable model */
    if (rule_count == 0) {
        return false;
    }

    StableModel model = compute_stable_model(config, rules, rule_count, NULL, 0);
    return model.is_complete && is_consistent(model);
}
