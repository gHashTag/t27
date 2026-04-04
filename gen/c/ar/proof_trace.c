/* AUTO-GENERATED from specs/ar/proof_trace.t27 — DO NOT EDIT */
/* Ring: 18 | Module: ProofTrace | phi^2 + 1/phi^2 = 3 */
/* Generator: PHI LOOP manual codegen (bootstrap unavailable) */
/* Bounded Proof Traces for CLARA Explainability (<=10 steps) */

#include "proof_trace.h"
#include <string.h>
#include <stdio.h>

/* ═══════════════════════════════════════════════════════════════ */
/* GF16 encode/decode                                             */
/* ═══════════════════════════════════════════════════════════════ */

GF16 gf16_encode_f32(float val) {
    if (val < 0.0f) val = 0.0f;
    if (val > 1.0f) val = 1.0f;
    return (GF16)(val * 65535.0f);
}

float gf16_decode_to_f32(GF16 encoded) {
    return (float)encoded / 65535.0f;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Internal helpers                                               */
/* ═══════════════════════════════════════════════════════════════ */

static const char *trit_symbol(Trit t) {
    switch (t) {
        case K_FALSE:   return "F";
        case K_TRUE:    return "T";
        default:        return "?";
    }
}

/* ═══════════════════════════════════════════════════════════════ */
/* ProofTrace API                                                 */
/* ═══════════════════════════════════════════════════════════════ */

ProofTrace proof_trace_create(void) {
    ProofTrace trace;
    memset(&trace, 0, sizeof(trace));
    trace.step_count       = 0;
    trace.conclusion       = K_UNKNOWN;
    trace.total_confidence = gf16_encode_f32(1.0f);
    trace.terminated       = false;
    return trace;
}

bool proof_trace_append_step(ProofTrace *trace, DerivationStep step) {
    /* Returns false = Restraint triggered (MAX_STEPS exceeded) */
    if (trace->step_count >= MAX_STEPS) {
        trace->terminated = true;
        return false;
    }
    trace->steps[trace->step_count] = step;
    trace->step_count += 1;
    trace->conclusion = step.output_fact;
    /* Multiply confidence values via GF16 encode/decode */
    float current_f = gf16_decode_to_f32(trace->total_confidence);
    float step_f    = gf16_decode_to_f32(step.confidence);
    trace->total_confidence = gf16_encode_f32(current_f * step_f);
    return true;
}

Trit proof_trace_get_conclusion(ProofTrace trace) {
    return trace.conclusion;
}

GF16 proof_trace_get_confidence(ProofTrace trace) {
    return trace.total_confidence;
}

/* ═══════════════════════════════════════════════════════════════ */
/* Formatting                                                     */
/* ═══════════════════════════════════════════════════════════════ */

static size_t format_natural(ProofTrace trace, char *buf, size_t buf_len) {
    size_t offset = 0;
    for (uint8_t i = 0; i < trace.step_count && offset < buf_len; i++) {
        DerivationStep step = trace.steps[i];
        int written = snprintf(buf + offset, buf_len - offset,
            "Step %d: %s -> %s\n",
            (int)(i + 1),
            step.rule_name ? step.rule_name : "unknown",
            trit_symbol(step.output_fact));
        if (written > 0) offset += (size_t)written;
    }
    return offset;
}

static size_t format_fitch(ProofTrace trace, char *buf, size_t buf_len) {
    size_t offset = 0;
    for (uint8_t i = 0; i < trace.step_count && offset < buf_len; i++) {
        DerivationStep step = trace.steps[i];
        int written = snprintf(buf + offset, buf_len - offset,
            "| %d. %s    %s\n",
            (int)(i + 1),
            trit_symbol(step.output_fact),
            step.rule_name ? step.rule_name : "unknown");
        if (written > 0) offset += (size_t)written;
    }
    return offset;
}

static size_t format_compact(ProofTrace trace, char *buf, size_t buf_len) {
    int written = snprintf(buf, buf_len,
        "%d steps | conclusion=%s | terminated=%s",
        (int)trace.step_count,
        trit_symbol(trace.conclusion),
        trace.terminated ? "true" : "false");
    return (written > 0) ? (size_t)written : 0;
}

size_t proof_trace_format(ProofTrace trace, FormatStyle style, char *buf, size_t buf_len) {
    if (!buf || buf_len == 0) return 0;
    buf[0] = '\0';
    switch (style) {
        case FORMAT_NATURAL: return format_natural(trace, buf, buf_len);
        case FORMAT_FITCH:   return format_fitch(trace, buf, buf_len);
        case FORMAT_COMPACT: return format_compact(trace, buf, buf_len);
        default:             return 0;
    }
}
