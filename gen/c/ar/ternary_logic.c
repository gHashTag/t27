/* AUTO-GENERATED from specs/ar/ternary_logic.t27 — DO NOT EDIT */
/* Ring: 18 | Module: TernaryLogic | phi^2 + 1/phi^2 = 3 */
/* Generator: PHI LOOP manual codegen (bootstrap unavailable) */

#include "ternary_logic.h"
#include <stdbool.h>
#include <stddef.h>
#include <assert.h>

/* ═══════════════════════════════════════════════════════════════ */
/* Base operations (from base::ops)                               */
/* ═══════════════════════════════════════════════════════════════ */

static Trit trit_min(Trit a, Trit b) {
    return (int8_t)a < (int8_t)b ? a : b;
}

static Trit trit_max(Trit a, Trit b) {
    return (int8_t)a > (int8_t)b ? a : b;
}

static Trit trit_not(Trit a) {
    return (Trit)(-(int8_t)a);
}

static int8_t trit_compare(Trit a, Trit b) {
    int8_t ai = (int8_t)a;
    int8_t bi = (int8_t)b;
    if (ai < bi) return -1;
    if (ai > bi) return 1;
    return 0;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Kleene K3 Logical Operations                                   */
/* ═══════════════════════════════════════════════════════════════ */

Trit k3_and(Trit a, Trit b) {
    return trit_min(a, b);
}

Trit k3_or(Trit a, Trit b) {
    return trit_max(a, b);
}

Trit k3_not(Trit a) {
    return trit_not(a);
}

Trit k3_implies(Trit a, Trit b) {
    return k3_or(k3_not(a), b);
}

Trit k3_equiv(Trit a, Trit b) {
    Trit ab = k3_implies(a, b);
    Trit ba = k3_implies(b, a);
    return k3_and(ab, ba);
}

/* ═══════════════════════════════════════════════════════════════ */
/* Automated Reasoning Primitives                                 */
/* ═══════════════════════════════════════════════════════════════ */

Trit forward_chain(Rule rule, Trit fact) {
    Trit fact_matches = k3_equiv(fact, rule.antecedent);
    return k3_and(fact_matches, rule.consequent);
}

Trit backward_chain(Trit goal, const Rule *rules, size_t count) {
    Trit result = K_UNKNOWN;
    for (size_t i = 0; i < count; i++) {
        Trit consequent_matches = k3_equiv(rules[i].consequent, goal);
        Trit support = k3_and(consequent_matches, rules[i].antecedent);
        result = k3_or(result, support);
    }
    return result;
}

void resolve(const Trit *clause_a, const Trit *clause_b, size_t len, Trit *result) {
    for (size_t i = 0; i < len; i++) {
        Trit a = clause_a[i];
        Trit b = clause_b[i];
        if (a == K_TRUE && b == K_FALSE) {
            result[i] = K_UNKNOWN;
        } else if (a == K_FALSE && b == K_TRUE) {
            result[i] = K_UNKNOWN;
        } else {
            result[i] = k3_or(a, b);
        }
    }
}

/* ═══════════════════════════════════════════════════════════════ */
/* Restraint and Bounded Rationality                              */
/* ═══════════════════════════════════════════════════════════════ */

bool is_restraint(Trit t) {
    return t == K_UNKNOWN;
}

void apply_restraint(const Trit *values, size_t len, Trit *result) {
    for (size_t i = 0; i < len; i++) {
        if (is_restraint(values[i])) {
            result[i] = K_FALSE;
        } else {
            result[i] = values[i];
        }
    }
}
