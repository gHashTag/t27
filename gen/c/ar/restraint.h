/* AUTO-GENERATED from specs/ar/restraint.t27 — DO NOT EDIT */
/* Ring: 18 | Module: Restraint | phi^2 + 1/phi^2 = 3 */
/* Bounded Rationality via Trit=0 (CLARA Restraint) */

#ifndef RESTRAINT_H
#define RESTRAINT_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ═══════════════════════════════════════════════════════════════ */
/* Trit — Kleene K3 Truth Values (from base::types)              */
/* ═══════════════════════════════════════════════════════════════ */

typedef int8_t Trit;

#define K_FALSE   ((Trit)-1)
#define K_UNKNOWN ((Trit)0)
#define K_TRUE    ((Trit)1)

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 — phi-optimized float encoding (from numeric::gf16)      */
/* Raw u16 placeholder values pending full GF16 spec             */
/* ═══════════════════════════════════════════════════════════════ */

typedef uint16_t GF16;

#define GF16_ZERO  ((GF16)0x0000)
#define GF16_085   ((GF16)0x3B33)  /* ~0.85 */
#define GF16_075   ((GF16)0x3A00)  /* ~0.75 */
#define GF16_070   ((GF16)0x3966)  /* ~0.70 */

/* ═══════════════════════════════════════════════════════════════ */
/* 1. Quality Level Enumeration                                   */
/* ═══════════════════════════════════════════════════════════════ */

typedef enum {
    QUALITY_UNKNOWN  = 0,
    QUALITY_UNSTABLE = 1,
    QUALITY_GOOD     = 2,
} QualityLevel;

/* ═══════════════════════════════════════════════════════════════ */
/* 2. Restraint Parameters                                        */
/* ═══════════════════════════════════════════════════════════════ */

typedef struct {
    uint32_t max_depth;             /* Maximum derivation depth */
    uint32_t max_rules;             /* Maximum rules to fire */
    GF16     confidence_threshold;  /* Minimum confidence (GF16) */
    uint64_t timeout_ms;            /* Timeout in milliseconds */
} RestraintParams;

/* ═══════════════════════════════════════════════════════════════ */
/* 3. Execution State                                             */
/* ═══════════════════════════════════════════════════════════════ */

typedef struct {
    uint32_t     current_depth;      /* Current derivation depth */
    uint32_t     rules_fired;        /* Rules fired so far */
    GF16         current_confidence; /* Current confidence (GF16) */
    uint64_t     start_time_ms;      /* Execution start time */
    QualityLevel quality;            /* Current quality level */
} ExecutionState;

/* ═══════════════════════════════════════════════════════════════ */
/* 4. Meta-Rule for HiLog                                         */
/* ═══════════════════════════════════════════════════════════════ */

typedef struct {
    Trit    condition;  /* Condition (Kleene semantics) */
    Trit    action;     /* Action: neg=halt, zero=restraint, pos=continue */
    uint8_t priority;   /* Higher = checked first */
} MetaRule;

/* ═══════════════════════════════════════════════════════════════ */
/* Function Declarations                                          */
/* ═══════════════════════════════════════════════════════════════ */

/* 5. Quality-Level Parameter Selection — O(1) */
RestraintParams params_for_quality(QualityLevel quality);

/* 6. Restraint Decision Function — O(1) */
Trit should_continue(ExecutionState state, RestraintParams params);

/* 7. Restraint Value Detection — O(1) */
bool is_restraint_value(Trit t);

/* 8. Depth Stepping — O(1) */
void step_depth(ExecutionState *state, uint32_t delta);

/* 9. Meta-Rule Application — O(n) */
Trit apply_meta_rules(ExecutionState *state, const MetaRule *meta_rules, size_t count);

/* 10. State Initialization — O(1) */
ExecutionState init_state(QualityLevel quality, uint64_t start_time);

#endif /* RESTRAINT_H */
