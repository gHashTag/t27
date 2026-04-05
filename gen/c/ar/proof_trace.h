/* AUTO-GENERATED from specs/ar/proof_trace.t27 — DO NOT EDIT */
/* Ring: 18 | Module: ProofTrace | phi^2 + 1/phi^2 = 3 */

#ifndef PROOF_TRACE_H
#define PROOF_TRACE_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ═══════════════════════════════════════════════════════════════ */
/* Kleene K3 Truth Values — Isomorphism: Trit = Kleene K3        */
/* ═══════════════════════════════════════════════════════════════ */

typedef int8_t Trit;

#define K_FALSE  ((Trit)-1)
#define K_UNKNOWN ((Trit)0)
#define K_TRUE   ((Trit)1)

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 — Galois Field confidence encoded as uint16_t            */
/* ═══════════════════════════════════════════════════════════════ */

typedef uint16_t GF16;

/* ═══════════════════════════════════════════════════════════════ */
/* Constants                                                      */
/* ═══════════════════════════════════════════════════════════════ */

#define MAX_STEPS 10

/* ═══════════════════════════════════════════════════════════════ */
/* Types                                                          */
/* ═══════════════════════════════════════════════════════════════ */

typedef enum {
    FORMAT_NATURAL = 0,
    FORMAT_FITCH   = 1,
    FORMAT_COMPACT = 2
} FormatStyle;

typedef struct {
    uint8_t    step_number;
    const char *rule_name;     /* max 64 bytes */
    Trit       input_facts[3]; /* up to 3 antecedents */
    Trit       output_fact;
    GF16       confidence;     /* [0.0, 1.0] encoded (NUMERIC-STANDARD-001) */
    Trit       k3_value;       /* Kleene truth value of this step */
} DerivationStep;

typedef struct {
    DerivationStep steps[MAX_STEPS];
    uint8_t        step_count;
    Trit           conclusion;
    GF16           total_confidence;  /* Encoded confidence (NUMERIC-STANDARD-001) */
    bool           terminated;        /* true = Restraint cut (MAX_STEPS hit) */
} ProofTrace;

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 encode/decode                                             */
/* ═══════════════════════════════════════════════════════════════ */

GF16 gf16_encode_f32(float val);
float gf16_decode_to_f32(GF16 encoded);

/* ═══════════════════════════════════════════════════════════════ */
/* ProofTrace API                                                 */
/* ═══════════════════════════════════════════════════════════════ */

ProofTrace  proof_trace_create(void);
bool        proof_trace_append_step(ProofTrace *trace, DerivationStep step);
Trit        proof_trace_get_conclusion(ProofTrace trace);
GF16        proof_trace_get_confidence(ProofTrace trace);
size_t      proof_trace_format(ProofTrace trace, FormatStyle style, char *buf, size_t buf_len);

#endif /* PROOF_TRACE_H */
