/* AUTO-GENERATED from specs/ar/explainability.t27 — DO NOT EDIT */
/* Ring: 18 | Module: Explainability | phi^2 + 1/phi^2 = 3 */

#ifndef EXPLAINABILITY_H
#define EXPLAINABILITY_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include "ternary_logic.h"  /* Trit, K_FALSE, K_UNKNOWN, K_TRUE */

/* ═══════════════════════════════════════════════════════════════ */
/* Constants                                                       */
/* ═══════════════════════════════════════════════════════════════ */

#define MAX_SUMMARY_SIZE       5
#define MAX_PREDICATE_NAME     64
#define MAX_DERIVATION_STEPS   10
#define MAX_HUMAN_READABLE     512

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 alias (from numeric::gf16)                                 */
/* ═══════════════════════════════════════════════════════════════ */

typedef uint16_t GF16;

/* ═══════════════════════════════════════════════════════════════ */
/* Explainability Types                                            */
/* ═══════════════════════════════════════════════════════════════ */

/** Identifies a fact by its predicate name and ternary arguments. */
typedef struct {
    uint8_t predicate[MAX_PREDICATE_NAME];
    Trit    args[3];
} FactId;

/** The format style for human-readable explanations. */
typedef enum {
    FORMAT_NATURAL = 0,
    FORMAT_FITCH   = 1,
    FORMAT_COMPACT = 2
} FormatStyle;

/* ═══════════════════════════════════════════════════════════════ */
/* Forward-declared dependency types from ar::proof_trace           */
/* ═══════════════════════════════════════════════════════════════ */

/** A single step in a derivation proof trace. */
typedef struct {
    uint8_t rule_name[MAX_PREDICATE_NAME];
    uint8_t rule_name_len;
    Trit    input_trits[3];
    Trit    output_trit;
    GF16    confidence;
} DerivationStep;

/** A proof trace recording a sequence of derivation steps. */
typedef struct {
    DerivationStep steps[MAX_DERIVATION_STEPS];
    uint8_t        step_count;
} ProofTrace;

/* ═══════════════════════════════════════════════════════════════ */
/* Forward-declared dependency types from ar::datalog_engine        */
/* ═══════════════════════════════════════════════════════════════ */

/** A Horn clause in the Datalog engine. */
typedef struct {
    uint8_t head_predicate[MAX_PREDICATE_NAME];
    uint8_t head_pred_len;
    Trit    head_args[3];
    uint8_t body_count;
    uint8_t body_predicates[3][MAX_PREDICATE_NAME];
    uint8_t body_pred_lens[3];
    Trit    body_args[3][3];
} HornClause;

/** The Datalog engine holding a set of Horn clauses and derived facts. */
typedef struct {
    HornClause clauses[64];
    uint8_t    clause_count;
    FactId     facts[64];
    GF16       fact_confidences[64];
    uint8_t    fact_count;
} DatalogEngine;

/* ═══════════════════════════════════════════════════════════════ */
/* Explanation and Summary types                                   */
/* ═══════════════════════════════════════════════════════════════ */

/** A full explanation of a derived fact. */
typedef struct {
    ProofTrace  trace;
    FormatStyle style;
    uint8_t     human_readable[MAX_HUMAN_READABLE];
    GF16        confidence;
    uint8_t     step_count;
} Explanation;

/** A summary of the top facts in a proof trace. */
typedef struct {
    FactId  top_facts[MAX_SUMMARY_SIZE];
    GF16    top_confidence[MAX_SUMMARY_SIZE];
    uint8_t fact_count;
} Summary;

/* ═══════════════════════════════════════════════════════════════ */
/* Proof trace helpers (from ar::proof_trace)                      */
/* ═══════════════════════════════════════════════════════════════ */

/** Create an empty proof trace. */
ProofTrace create_trace(void);

/** Append a derivation step to a proof trace (bounded). */
void append_step(ProofTrace *trace, DerivationStep step);

/* ═══════════════════════════════════════════════════════════════ */
/* Public API                                                      */
/* ═══════════════════════════════════════════════════════════════ */

/**
 * Explain a fact by searching the engine for its derivation,
 * building a proof trace, and formatting the explanation.
 */
Explanation explain_fact(const DatalogEngine *engine, FactId fact_id, FormatStyle style);

/**
 * Format a full derivation chain as a human-readable string.
 * Writes result into out_buf (must be >= MAX_HUMAN_READABLE bytes).
 */
void explain_derivation_chain(ProofTrace trace, FormatStyle style, uint8_t *out_buf);

/**
 * Summarize a proof trace, extracting top facts and confidences.
 */
Summary summarize_trace(ProofTrace trace);

#endif /* EXPLAINABILITY_H */
