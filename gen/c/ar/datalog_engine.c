/* AUTO-GENERATED from specs/ar/datalog_engine.t27 — DO NOT EDIT */
/* Ring: 18 | Module: DatalogEngine | phi^2 + 1/phi^2 = 3 */
/* Generator: PHI LOOP manual codegen (bootstrap unavailable) */

#include "datalog_engine.h"
#include <stdbool.h>
#include <stddef.h>
#include <string.h>

/* ═══════════════════════════════════════════════════════════════ */
/* Helpers                                                         */
/* ═══════════════════════════════════════════════════════════════ */

bool horn_clause_eq(const HornClause *a, const HornClause *b) {
    if (a->name != b->name) return false;
    if (a->arg_count != b->arg_count) return false;
    for (uint8_t i = 0; i < a->arg_count; i++) {
        if (a->args[i] != b->args[i]) return false;
    }
    return true;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Core Functions                                                  */
/* ═══════════════════════════════════════════════════════════════ */

DatalogEngine datalog_init(void) {
    DatalogEngine engine;
    engine.fact_count = 0;
    engine.rule_count = 0;
    engine.solved = false;
    memset(engine.derived_facts, 0, sizeof(engine.derived_facts));
    return engine;
}

bool add_fact(DatalogEngine *engine, HornClause fact) {
    /* Check for duplicate */
    for (size_t i = 0; i < engine->fact_count; i++) {
        if (horn_clause_eq(&engine->facts[i], &fact)) return false;
    }
    /* Check capacity */
    if (engine->fact_count >= MAX_CLAUSES) return false;
    engine->facts[engine->fact_count] = fact;
    engine->fact_count++;
    engine->solved = false;
    return true;
}

bool has_fact(DatalogEngine *engine, HornClause fact) {
    for (size_t i = 0; i < engine->fact_count; i++) {
        if (horn_clause_eq(&engine->facts[i], &fact)) return true;
    }
    return false;
}

bool add_rule(DatalogEngine *engine, Rule rule) {
    if (engine->rule_count >= MAX_CLAUSES) return false;
    engine->rules[engine->rule_count] = rule;
    engine->rule_count++;
    engine->solved = false;
    return true;
}

void datalog_solve(DatalogEngine *engine) {
    bool changed = true;
    while (changed) {
        changed = false;
        for (size_t ri = 0; ri < engine->rule_count; ri++) {
            Rule rule = engine->rules[ri];
            /* Snapshot current count to iterate only existing facts */
            size_t current_count = engine->fact_count;
            for (size_t fi = 0; fi < current_count; fi++) {
                HornClause fact = engine->facts[fi];
                /* Use first arg as the trit value for forward chaining */
                Trit fact_trit = (fact.arg_count > 0) ? fact.args[0] : K_UNKNOWN;
                Trit result = forward_chain(rule, fact_trit);
                if (result == K_TRUE) {
                    /* Derive a new fact: same name, first arg = consequent */
                    HornClause derived;
                    derived.name = fact.name;
                    derived.arg_count = 1;
                    derived.args[0] = rule.consequent;
                    for (uint8_t ai = 1; ai < MAX_ARGS; ai++) {
                        derived.args[ai] = K_UNKNOWN;
                    }
                    if (!has_fact(engine, derived)) {
                        if (engine->fact_count < MAX_CLAUSES) {
                            engine->facts[engine->fact_count] = derived;
                            engine->derived_facts[engine->fact_count] = true;
                            engine->fact_count++;
                            changed = true;
                        }
                    }
                }
            }
        }
    }
    engine->solved = true;
}
