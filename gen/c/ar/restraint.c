/* AUTO-GENERATED from specs/ar/restraint.t27 — DO NOT EDIT */
/* Ring: 18 | Module: Restraint | phi^2 + 1/phi^2 = 3 */
/* Bounded Rationality via Trit=0 (CLARA Restraint) */
/* Agent C: Most pure agent in CLARA AR family */

#include "restraint.h"

/* ═══════════════════════════════════════════════════════════════ */
/* 5. Quality-Level Parameter Selection — O(1)                    */
/* Select appropriate restraint parameters for a given quality.   */
/* Per CLARA: unknown=conservative, good=permissive.              */
/* ═══════════════════════════════════════════════════════════════ */

RestraintParams params_for_quality(QualityLevel quality) {
    RestraintParams p;
    switch (quality) {
        case QUALITY_UNKNOWN:
            /* Conservative: minimal resources for unknown quality */
            p.max_depth             = 3;
            p.max_rules             = 10;
            p.confidence_threshold  = GF16_085;  /* 0.85 — high threshold */
            p.timeout_ms            = 100;
            break;
        case QUALITY_UNSTABLE:
            /* Moderate: restricted resources for unstable quality */
            p.max_depth             = 7;
            p.max_rules             = 50;
            p.confidence_threshold  = GF16_075;  /* 0.75 — medium threshold */
            p.timeout_ms            = 1000;
            break;
        case QUALITY_GOOD:
            /* Permissive: full resources for good quality */
            p.max_depth             = 15;
            p.max_rules             = 500;
            p.confidence_threshold  = GF16_070;  /* 0.70 — lower threshold */
            p.timeout_ms            = 10000;
            break;
        default:
            /* Fallback to most conservative */
            p.max_depth             = 3;
            p.max_rules             = 10;
            p.confidence_threshold  = GF16_085;
            p.timeout_ms            = 100;
            break;
    }
    return p;
}

/* ═══════════════════════════════════════════════════════════════ */
/* 6. Restraint Decision Function — O(1)                          */
/* Returns K_FALSE when ANY restraint condition is triggered.      */
/* This is CORE of CLARA bounded rationality.                     */
/* ═══════════════════════════════════════════════════════════════ */

Trit should_continue(ExecutionState state, RestraintParams params) {
    /* Check depth limit */
    if (state.current_depth >= params.max_depth) {
        return K_FALSE;  /* Depth restraint */
    }

    /* Check rule limit */
    if (state.rules_fired >= params.max_rules) {
        return K_FALSE;  /* Rule count restraint */
    }

    /* Check confidence threshold using GF16 comparison */
    if (state.current_confidence < params.confidence_threshold) {
        return K_FALSE;  /* Confidence restraint */
    }

    /* All checks passed — continue */
    return K_TRUE;
}

/* ═══════════════════════════════════════════════════════════════ */
/* 7. Restraint Value Detection — O(1)                            */
/* Trit.zero = K_UNKNOWN = "don't-care" = restraint               */
/* ═══════════════════════════════════════════════════════════════ */

bool is_restraint_value(Trit t) {
    return t == K_UNKNOWN;
}

/* ═══════════════════════════════════════════════════════════════ */
/* 8. Depth Stepping — O(1)                                       */
/* Increment execution depth for entering derivation levels.      */
/* ═══════════════════════════════════════════════════════════════ */

void step_depth(ExecutionState *state, uint32_t delta) {
    state->current_depth += delta;
}

/* ═══════════════════════════════════════════════════════════════ */
/* 9. Meta-Rule Application — O(n)                                */
/* Apply HiLog meta-rules in priority order.                      */
/* K_FALSE action = halt, K_UNKNOWN = restraint, K_TRUE = go.     */
/* ═══════════════════════════════════════════════════════════════ */

Trit apply_meta_rules(ExecutionState *state, const MetaRule *meta_rules, size_t count) {
    (void)state;  /* State reserved for future confidence-based meta-rules */
    Trit result = K_TRUE;  /* Default: continue */

    for (size_t i = 0; i < count; i++) {
        const MetaRule *rule = &meta_rules[i];

        /* Check if rule condition matches (K_TRUE = always trigger) */
        if (rule->condition == K_TRUE) {
            /* K_FALSE action = halt (strongest) */
            if (rule->action == K_FALSE) {
                return K_FALSE;  /* Immediate halt */
            }
            /* K_UNKNOWN action = restraint (downgrade) */
            if (rule->action == K_UNKNOWN) {
                result = K_UNKNOWN;
            }
            /* K_TRUE action = continue (no change) */
        }
    }

    return result;
}

/* ═══════════════════════════════════════════════════════════════ */
/* 10. State Initialization — O(1)                                */
/* Initialize execution state for a new derivation.               */
/* ═══════════════════════════════════════════════════════════════ */

ExecutionState init_state(QualityLevel quality, uint64_t start_time) {
    ExecutionState state;
    state.current_depth      = 0;
    state.rules_fired        = 0;
    state.current_confidence = GF16_ZERO;  /* Start with zero confidence */
    state.start_time_ms      = start_time;
    state.quality            = quality;
    return state;
}
