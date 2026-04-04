/* Auto-generated from specs/nn/attention.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/nn/attention.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 29 | Module: SacredAttention | Multi-head attention with phi-RoPE */

#ifndef SACRED_ATTENTION_H
#define SACRED_ATTENTION_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ===================================================================== */
/* Trit type (ternary weight)                                             */
/* ===================================================================== */

typedef int8_t Trit;

#define TRIT_NEG   ((Trit)-1)
#define TRIT_ZERO  ((Trit)0)
#define TRIT_POS   ((Trit)1)

/* ===================================================================== */
/* Constants                                                               */
/* ===================================================================== */

#define ATTN_PHI             1.6180339887498948482
#define ATTN_PHI_INV         0.6180339887498948482

#define ATTN_NUM_HEADS       3
#define ATTN_HEAD_DIM        81      /* 3^4 */
#define ATTN_EMBED_DIM       243     /* 3 * 81 */
#define ATTN_CONTEXT_LEN     81
#define ATTN_ROPE_PAIRS      40      /* 81 / 2 */

/* Sacred scaling: phi^-3 ~ 0.2360679 */
#define ATTN_SACRED_GAMMA    0.2360679774997896964

/* Sacred scale: 81^(-phi^-3) ~ 0.354 */
#define ATTN_SACRED_SCALE    0.35355339059327376

#define ATTN_CAUSAL          0
#define ATTN_BIDIR           1
#define ATTN_SPARSE          2

#define ATTN_PHASE_QUERY     0
#define ATTN_PHASE_KEY       1
#define ATTN_PHASE_VALUE     2
#define ATTN_PHASE_SCORE     3
#define ATTN_PHASE_SOFTMAX   4
#define ATTN_PHASE_WEIGHT    5

/* ===================================================================== */
/* RoPE Tables                                                             */
/* ===================================================================== */

typedef struct {
    double cos_table[ATTN_CONTEXT_LEN * ATTN_ROPE_PAIRS];
    double sin_table[ATTN_CONTEXT_LEN * ATTN_ROPE_PAIRS];
} RoPETables;

/* ===================================================================== */
/* Attention Buffers                                                       */
/* ===================================================================== */

typedef struct {
    double q_buffer[ATTN_EMBED_DIM];
    double k_buffer[ATTN_EMBED_DIM];
    double v_buffer[ATTN_EMBED_DIM];
    double scores[ATTN_NUM_HEADS * ATTN_CONTEXT_LEN];
    double concat[ATTN_EMBED_DIM];
} AttentionBuffers;

/* ===================================================================== */
/* Function declarations                                                   */
/* ===================================================================== */

/* Initialization: build phi-RoPE tables */
void sacred_attention_init(void);

/* Ternary matrix multiplication: output[i] = sum_j input[j] * w[j*out+i] */
void ternary_matmul(const double *input, const Trit *weights,
                    double *output, size_t in_dim, size_t out_dim);

/* Q/K/V projection via ternary matmul */
void project_qkv(AttentionBuffers *buffers, const double *input,
                  const Trit *w_q, const Trit *w_k, const Trit *w_v);

/* Apply phi-RoPE rotation to Q and K */
void apply_rope_qk(AttentionBuffers *buffers, size_t position);

/* Cache K and V at position */
void cache_kv(AttentionBuffers *buffers, size_t position,
              double *cache_k, double *cache_v);

/* Compute attention scores with sacred scaling */
void compute_scores(AttentionBuffers *buffers, size_t position,
                    size_t seq_len, const double *cache_k);

/* Apply softmax to attention scores */
void apply_softmax(AttentionBuffers *buffers, size_t seq_len);

/* Compute weighted value sum */
void weighted_values(AttentionBuffers *buffers, size_t seq_len,
                     const double *cache_v);

/* Output projection via W_o */
void project_output(AttentionBuffers *buffers, const Trit *w_o,
                    double *output);

/* Residual connection: output += input */
void add_residual(double *output, const double *input, size_t dim);

/* Full single-position forward pass */
void sacred_attention_kernel(
    const double *input,
    const Trit *w_q, const Trit *w_k, const Trit *w_v, const Trit *w_o,
    size_t position, size_t seq_len,
    double *output,
    double *cache_k, double *cache_v);

#endif /* SACRED_ATTENTION_H */
