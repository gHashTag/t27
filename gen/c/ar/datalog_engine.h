/* AUTO-GENERATED from specs/ar/datalog_engine.t27 — DO NOT EDIT */
/* Ring: 18 | Module: DatalogEngine | phi^2 + 1/phi^2 = 3 */

#ifndef DATALOG_ENGINE_H
#define DATALOG_ENGINE_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "ternary_logic.h"

/* ═══════════════════════════════════════════════════════════════ */
/* Constants                                                       */
/* ═══════════════════════════════════════════════════════════════ */

#define MAX_CLAUSES 256
#define MAX_ARGS    8

/* ═══════════════════════════════════════════════════════════════ */
/* Types                                                           */
/* ═══════════════════════════════════════════════════════════════ */

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
/* Functions                                                       */
/* ═══════════════════════════════════════════════════════════════ */

/* Compare two HornClauses for equality */
bool horn_clause_eq(const HornClause *a, const HornClause *b);

/* Initialize a new empty DatalogEngine */
DatalogEngine datalog_init(void);

/* Add a fact. Returns true on success, false if full or duplicate. */
bool add_fact(DatalogEngine *engine, HornClause fact);

/* Check whether a fact exists in the engine */
bool has_fact(DatalogEngine *engine, HornClause fact);

/* Add a rule. Returns true on success, false if full. */
bool add_rule(DatalogEngine *engine, Rule rule);

/* Forward-chaining solver: iterate until fixed point */
void datalog_solve(DatalogEngine *engine);

#endif /* DATALOG_ENGINE_H */
