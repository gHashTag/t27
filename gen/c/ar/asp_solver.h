/* AUTO-GENERATED from specs/ar/asp_solver.t27 — DO NOT EDIT */
/* Ring: 18 | Module: AspSolver | phi^2 + 1/phi^2 = 3 */

#ifndef ASP_SOLVER_H
#define ASP_SOLVER_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "ternary_logic.h"

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 — Galois Field 16-bit confidence encoding                 */
/* ═══════════════════════════════════════════════════════════════ */

typedef uint16_t GF16;

/* ═══════════════════════════════════════════════════════════════ */
/* Types from ar::restraint                                       */
/* ═══════════════════════════════════════════════════════════════ */

typedef enum {
    QUALITY_UNKNOWN  = 0,
    QUALITY_UNSTABLE = 1,
    QUALITY_GOOD     = 2
} QualityLevel;

typedef struct {
    uint32_t max_depth;
    uint32_t max_rules;
    GF16     confidence_threshold;
    uint64_t timeout_ms;
} RestraintParams;

typedef struct {
    uint32_t     current_depth;
    uint32_t     rules_fired;
    GF16         current_confidence;
    uint64_t     start_time_ms;
    QualityLevel quality;
} ExecutionState;

/* ═══════════════════════════════════════════════════════════════ */
/* Types from ar::datalog_engine                                  */
/* ═══════════════════════════════════════════════════════════════ */

#define MAX_CLAUSES 256
#define MAX_ARGS    8

typedef struct {
    uint16_t name;
    Trit     args[MAX_ARGS];
    uint8_t  arg_count;
} HornClause;

typedef struct {
    HornClause facts[MAX_CLAUSES];
    size_t     fact_count;
    Rule       rules[MAX_CLAUSES];
    size_t     rule_count;
    bool       derived_facts[MAX_CLAUSES];
    bool       solved;
} DatalogEngine;

/* ═══════════════════════════════════════════════════════════════ */
/* ASP Core Constants                                             */
/* ═══════════════════════════════════════════════════════════════ */

#define ASP_MAX_NAF        3
#define ASP_MAX_FACTS      32
#define ASP_MAX_ITERATIONS 1000

/* ═══════════════════════════════════════════════════════════════ */
/* ASP Core Types                                                 */
/* ═══════════════════════════════════════════════════════════════ */

/* AspRule: Horn clause with Negation as Failure */
typedef struct {
    Rule     base;
    uint32_t naf_ids[ASP_MAX_NAF];
    uint8_t  naf_count;
} AspRule;

/* FactId: Fact identifier with truth value */
typedef struct {
    uint32_t id;
    Trit     truth;
} FactId;

/* StableModel: Result of ASP solver computation */
typedef struct {
    FactId   facts[ASP_MAX_FACTS];
    uint8_t  fact_count;
    bool     is_complete;
    uint16_t iterations;
    bool     aborted_by_restraint;
} StableModel;

/* AspConfig: Configuration for ASP solver */
typedef struct {
    uint16_t     max_iterations;
    uint8_t      max_models;
    QualityLevel quality;
    GF16         initial_confidence;
} AspConfig;

/* AspState: State for ASP computation with restraint */
typedef struct {
    DatalogEngine  engine;
    ExecutionState exec_state;
    RestraintParams restraint_params;
} AspState;

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 Utility Functions                                         */
/* ═══════════════════════════════════════════════════════════════ */

GF16  gf16_encode_f32(float v);
float gf16_decode_to_f32(GF16 g);

/* ═══════════════════════════════════════════════════════════════ */
/* Restraint Functions                                            */
/* ═══════════════════════════════════════════════════════════════ */

RestraintParams params_for_quality(QualityLevel quality);
Trit should_continue(ExecutionState state, RestraintParams params);

/* ═══════════════════════════════════════════════════════════════ */
/* Datalog Engine Functions                                       */
/* ═══════════════════════════════════════════════════════════════ */

DatalogEngine datalog_init(void);
bool add_fact(DatalogEngine *engine, HornClause fact);
size_t datalog_solve(DatalogEngine *engine);

/* ═══════════════════════════════════════════════════════════════ */
/* ASP Solver Functions                                           */
/* ═══════════════════════════════════════════════════════════════ */

bool evaluate_naf(DatalogEngine *engine, const uint32_t *naf_ids, size_t count);
uint16_t fixed_point_iteration(AspState *state, const AspRule *rules, size_t rule_count);
Trit query_with_restraint(AspState *state, Trit goal);
bool is_consistent(StableModel model);
StableModel compute_stable_model(AspConfig config, const AspRule *rules, size_t rule_count,
                                  const FactId *facts, size_t fact_count);
bool has_stable_model(const AspRule *rules, size_t rule_count);

#endif /* ASP_SOLVER_H */
