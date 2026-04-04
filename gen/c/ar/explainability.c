/* AUTO-GENERATED from specs/ar/explainability.t27 — DO NOT EDIT */
/* Ring: 18 | Module: Explainability | phi^2 + 1/phi^2 = 3 */
/* Generator: PHI LOOP manual codegen (bootstrap unavailable) */

#include "explainability.h"
#include <string.h>

/* ═══════════════════════════════════════════════════════════════ */
/* Internal helpers                                                */
/* ═══════════════════════════════════════════════════════════════ */

/** Compare two predicate name buffers for equality. */
static bool names_equal(const uint8_t a[MAX_PREDICATE_NAME],
                        const uint8_t b[MAX_PREDICATE_NAME]) {
    return memcmp(a, b, MAX_PREDICATE_NAME) == 0;
}

/** Compare two FactIds for equality. */
static bool fact_id_equal(FactId a, FactId b) {
    if (!names_equal(a.predicate, b.predicate)) return false;
    for (int i = 0; i < 3; i++) {
        if (a.args[i] != b.args[i]) return false;
    }
    return true;
}

/** Return the character representation of a Trit value. */
static uint8_t trit_char(Trit t) {
    if (t == K_TRUE)  return 'T';
    if (t == K_FALSE) return 'F';
    return '?';
}

/* ═══════════════════════════════════════════════════════════════ */
/* Proof trace helpers (from ar::proof_trace)                      */
/* ═══════════════════════════════════════════════════════════════ */

ProofTrace create_trace(void) {
    ProofTrace trace;
    memset(&trace, 0, sizeof(trace));
    trace.step_count = 0;
    return trace;
}

void append_step(ProofTrace *trace, DerivationStep step) {
    if (trace->step_count < MAX_DERIVATION_STEPS) {
        trace->steps[trace->step_count] = step;
        trace->step_count++;
    }
}

/* ═══════════════════════════════════════════════════════════════ */
/* Internal formatting helpers                                     */
/* ═══════════════════════════════════════════════════════════════ */

/** Write bytes from src into buf at *pos, respecting max. Returns bytes written. */
static size_t write_bytes(uint8_t *buf, size_t max, size_t *pos,
                          const uint8_t *src, size_t len) {
    size_t written = 0;
    for (size_t i = 0; i < len && *pos < max; i++) {
        buf[*pos] = src[i];
        (*pos)++;
        written++;
    }
    return written;
}

/** Write a single byte into buf at *pos if space permits. */
static void write_byte(uint8_t *buf, size_t max, size_t *pos, uint8_t c) {
    if (*pos < max) {
        buf[*pos] = c;
        (*pos)++;
    }
}

/** Format a derivation step in natural style. */
static void format_step_natural(const DerivationStep *step,
                                uint8_t *buf, size_t max, size_t *pos) {
    const uint8_t prefix[] = "Step: ";
    write_bytes(buf, max, pos, prefix, 6);

    size_t name_len = step->rule_name_len;
    if (name_len > MAX_PREDICATE_NAME) name_len = MAX_PREDICATE_NAME;
    write_bytes(buf, max, pos, step->rule_name, name_len);

    const uint8_t arrow[] = " -> ";
    write_bytes(buf, max, pos, arrow, 4);

    write_byte(buf, max, pos, trit_char(step->output_trit));
    write_byte(buf, max, pos, '\n');
}

/** Format a derivation step in Fitch style. */
static void format_step_fitch(const DerivationStep *step, uint8_t idx,
                               uint8_t *buf, size_t max, size_t *pos) {
    write_byte(buf, max, pos, (uint8_t)('0' + (idx % 10)));

    const uint8_t sep[] = ". ";
    write_bytes(buf, max, pos, sep, 2);

    size_t name_len = step->rule_name_len;
    if (name_len > MAX_PREDICATE_NAME) name_len = MAX_PREDICATE_NAME;
    write_bytes(buf, max, pos, step->rule_name, name_len);

    const uint8_t turnstile[] = " |- ";
    write_bytes(buf, max, pos, turnstile, 4);

    write_byte(buf, max, pos, trit_char(step->output_trit));
    write_byte(buf, max, pos, '\n');
}

/** Format a derivation step in compact style. */
static void format_step_compact(const DerivationStep *step,
                                uint8_t *buf, size_t max, size_t *pos) {
    size_t name_len = step->rule_name_len;
    if (name_len > MAX_PREDICATE_NAME) name_len = MAX_PREDICATE_NAME;
    write_bytes(buf, max, pos, step->rule_name, name_len);

    write_byte(buf, max, pos, ':');
    write_byte(buf, max, pos, trit_char(step->output_trit));
    write_byte(buf, max, pos, ';');
}

/* ═══════════════════════════════════════════════════════════════ */
/* Public API                                                      */
/* ═══════════════════════════════════════════════════════════════ */

Explanation explain_fact(const DatalogEngine *engine, FactId fact_id, FormatStyle style) {
    Explanation explanation;
    memset(&explanation, 0, sizeof(explanation));
    explanation.style = style;

    ProofTrace trace = create_trace();
    GF16 best_confidence = 0;

    /* Search engine facts for the requested fact_id */
    for (uint8_t i = 0; i < engine->fact_count; i++) {
        if (fact_id_equal(engine->facts[i], fact_id)) {
            best_confidence = engine->fact_confidences[i];
            break;
        }
    }

    /* Search clauses for derivation steps that conclude this fact */
    for (uint8_t i = 0; i < engine->clause_count; i++) {
        const HornClause *clause = &engine->clauses[i];

        if (names_equal(clause->head_predicate, fact_id.predicate)) {
            bool args_match = true;
            for (int j = 0; j < 3; j++) {
                if (clause->head_args[j] != fact_id.args[j]) {
                    args_match = false;
                    break;
                }
            }
            if (args_match) {
                DerivationStep step;
                memset(&step, 0, sizeof(step));
                memcpy(step.rule_name, clause->head_predicate, MAX_PREDICATE_NAME);
                step.rule_name_len = clause->head_pred_len;
                memcpy(step.input_trits, fact_id.args, sizeof(fact_id.args));
                step.output_trit = K_TRUE;
                step.confidence = best_confidence;
                append_step(&trace, step);
            }
        }
    }

    explanation.trace = trace;
    explanation.confidence = best_confidence;
    explanation.step_count = trace.step_count;
    explain_derivation_chain(trace, style, explanation.human_readable);

    return explanation;
}

void explain_derivation_chain(ProofTrace trace, FormatStyle style, uint8_t *out_buf) {
    memset(out_buf, 0, MAX_HUMAN_READABLE);
    size_t pos = 0;

    for (uint8_t i = 0; i < trace.step_count; i++) {
        if (pos >= MAX_HUMAN_READABLE) break;
        const DerivationStep *step = &trace.steps[i];

        switch (style) {
            case FORMAT_NATURAL:
                format_step_natural(step, out_buf, MAX_HUMAN_READABLE, &pos);
                break;
            case FORMAT_FITCH:
                format_step_fitch(step, i, out_buf, MAX_HUMAN_READABLE, &pos);
                break;
            case FORMAT_COMPACT:
                format_step_compact(step, out_buf, MAX_HUMAN_READABLE, &pos);
                break;
        }
    }
}

Summary summarize_trace(ProofTrace trace) {
    Summary summary;
    memset(&summary, 0, sizeof(summary));

    for (uint8_t i = 0; i < trace.step_count; i++) {
        const DerivationStep *step = &trace.steps[i];

        /* Build a FactId from the step */
        FactId fid;
        memcpy(fid.predicate, step->rule_name, MAX_PREDICATE_NAME);
        memcpy(fid.args, step->input_trits, sizeof(fid.args));

        /* Check if this fact is already in the summary */
        bool found = false;
        for (uint8_t j = 0; j < summary.fact_count; j++) {
            if (fact_id_equal(summary.top_facts[j], fid)) {
                if (step->confidence > summary.top_confidence[j]) {
                    summary.top_confidence[j] = step->confidence;
                }
                found = true;
                break;
            }
        }

        if (!found && summary.fact_count < MAX_SUMMARY_SIZE) {
            summary.top_facts[summary.fact_count] = fid;
            summary.top_confidence[summary.fact_count] = step->confidence;
            summary.fact_count++;
        } else if (!found) {
            /* Replace the lowest-confidence entry if this one is better */
            uint8_t min_idx = 0;
            GF16 min_conf = summary.top_confidence[0];
            for (uint8_t k = 1; k < MAX_SUMMARY_SIZE; k++) {
                if (summary.top_confidence[k] < min_conf) {
                    min_conf = summary.top_confidence[k];
                    min_idx = k;
                }
            }
            if (step->confidence > min_conf) {
                summary.top_facts[min_idx] = fid;
                summary.top_confidence[min_idx] = step->confidence;
            }
        }
    }

    return summary;
}
