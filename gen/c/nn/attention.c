/* Auto-generated from specs/nn/attention.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/nn/attention.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 29 | Module: SacredAttention | Multi-head attention with phi-RoPE */

#include "attention.h"
#include <math.h>
#include <string.h>

/* ===================================================================== */
/* Module-level state                                                      */
/* ===================================================================== */

static RoPETables rope_tables;

/* ===================================================================== */
/* Initialization: phi-RoPE tables                                         */
/* ===================================================================== */

void sacred_attention_init(void) {
    size_t p, i;
    for (p = 0; p < ATTN_CONTEXT_LEN; p++) {
        for (i = 0; i < ATTN_ROPE_PAIRS; i++) {
            /* freq = phi^(-2i/HEAD_DIM) */
            double freq_exponent = -2.0 * (double)i / (double)ATTN_HEAD_DIM;
            double freq = pow(ATTN_PHI, freq_exponent);
            double angle = (double)p * freq;

            size_t table_offset = p * ATTN_ROPE_PAIRS + i;
            rope_tables.cos_table[table_offset] = cos(angle);
            rope_tables.sin_table[table_offset] = sin(angle);
        }
    }
}

/* ===================================================================== */
/* Ternary Matrix Multiplication                                           */
/* ===================================================================== */

void ternary_matmul(const double *input, const Trit *weights,
                    double *output, size_t in_dim, size_t out_dim) {
    size_t i, j;
    for (i = 0; i < out_dim; i++) {
        double acc = 0.0;
        for (j = 0; j < in_dim; j++) {
            int8_t w = weights[j * out_dim + i];
            if (w == 1) {
                acc += input[j];
            } else if (w == -1) {
                acc -= input[j];
            }
        }
        output[i] = acc;
    }
}

/* ===================================================================== */
/* Q/K/V Projection                                                        */
/* ===================================================================== */

void project_qkv(AttentionBuffers *buffers, const double *input,
                  const Trit *w_q, const Trit *w_k, const Trit *w_v) {
    ternary_matmul(input, w_q, buffers->q_buffer, ATTN_EMBED_DIM, ATTN_EMBED_DIM);
    ternary_matmul(input, w_k, buffers->k_buffer, ATTN_EMBED_DIM, ATTN_EMBED_DIM);
    ternary_matmul(input, w_v, buffers->v_buffer, ATTN_EMBED_DIM, ATTN_EMBED_DIM);
}

/* ===================================================================== */
/* phi-RoPE Application                                                    */
/* ===================================================================== */

void apply_rope_qk(AttentionBuffers *buffers, size_t position) {
    size_t h, pair_idx;
    for (h = 0; h < ATTN_NUM_HEADS; h++) {
        size_t head_offset = h * ATTN_HEAD_DIM;
        for (pair_idx = 0; pair_idx < ATTN_ROPE_PAIRS; pair_idx++) {
            size_t idx0 = head_offset + pair_idx;
            size_t idx1 = head_offset + pair_idx + ATTN_ROPE_PAIRS;

            size_t table_offset = position * ATTN_ROPE_PAIRS + pair_idx;
            double cos_val = rope_tables.cos_table[table_offset];
            double sin_val = rope_tables.sin_table[table_offset];

            /* Rotate Q */
            double q0 = buffers->q_buffer[idx0];
            double q1 = buffers->q_buffer[idx1];
            buffers->q_buffer[idx0] = q0 * cos_val - q1 * sin_val;
            buffers->q_buffer[idx1] = q0 * sin_val + q1 * cos_val;

            /* Rotate K */
            double k0 = buffers->k_buffer[idx0];
            double k1 = buffers->k_buffer[idx1];
            buffers->k_buffer[idx0] = k0 * cos_val - k1 * sin_val;
            buffers->k_buffer[idx1] = k0 * sin_val + k1 * cos_val;
        }
    }
}

/* ===================================================================== */
/* KV Caching                                                              */
/* ===================================================================== */

void cache_kv(AttentionBuffers *buffers, size_t position,
              double *cache_k, double *cache_v) {
    size_t offset = position * ATTN_EMBED_DIM;
    size_t i;
    for (i = 0; i < ATTN_EMBED_DIM; i++) {
        cache_k[offset + i] = buffers->k_buffer[i];
        cache_v[offset + i] = buffers->v_buffer[i];
    }
}

/* ===================================================================== */
/* Score Computation                                                       */
/* ===================================================================== */

void compute_scores(AttentionBuffers *buffers, size_t position,
                    size_t seq_len, const double *cache_k) {
    size_t h, j, d;
    for (h = 0; h < ATTN_NUM_HEADS; h++) {
        size_t head_offset = h * ATTN_HEAD_DIM;
        for (j = 0; j < seq_len; j++) {
            if (j > position) {
                buffers->scores[h * ATTN_CONTEXT_LEN + j] = 0.0;
                continue;
            }

            double score = 0.0;
            for (d = 0; d < ATTN_HEAD_DIM; d++) {
                double q_val = buffers->q_buffer[head_offset + d];
                double k_val = cache_k[j * ATTN_EMBED_DIM + head_offset + d];
                score += q_val * k_val;
            }

            buffers->scores[h * ATTN_CONTEXT_LEN + j] = score * ATTN_SACRED_SCALE;
        }
    }
}

/* ===================================================================== */
/* Softmax                                                                 */
/* ===================================================================== */

void apply_softmax(AttentionBuffers *buffers, size_t seq_len) {
    size_t h, j;
    for (h = 0; h < ATTN_NUM_HEADS; h++) {
        /* Find max */
        double max_score = -1.0e30;
        for (j = 0; j < seq_len; j++) {
            double s = buffers->scores[h * ATTN_CONTEXT_LEN + j];
            if (s > max_score) max_score = s;
        }

        /* Compute exp and sum */
        double sum_exp = 0.0;
        for (j = 0; j < seq_len; j++) {
            double s = exp(buffers->scores[h * ATTN_CONTEXT_LEN + j] - max_score);
            buffers->scores[h * ATTN_CONTEXT_LEN + j] = s;
            sum_exp += s;
        }

        /* Normalize */
        for (j = 0; j < seq_len; j++) {
            buffers->scores[h * ATTN_CONTEXT_LEN + j] /= sum_exp;
        }
    }
}

/* ===================================================================== */
/* Weighted Value Sum                                                      */
/* ===================================================================== */

void weighted_values(AttentionBuffers *buffers, size_t seq_len,
                     const double *cache_v) {
    size_t h, d, j;
    for (h = 0; h < ATTN_NUM_HEADS; h++) {
        size_t head_offset = h * ATTN_HEAD_DIM;
        for (d = 0; d < ATTN_HEAD_DIM; d++) {
            double weighted_sum = 0.0;
            for (j = 0; j < seq_len; j++) {
                double weight = buffers->scores[h * ATTN_CONTEXT_LEN + j];
                double v_val = cache_v[j * ATTN_EMBED_DIM + head_offset + d];
                weighted_sum += weight * v_val;
            }
            buffers->concat[head_offset + d] = weighted_sum;
        }
    }
}

/* ===================================================================== */
/* Output Projection                                                       */
/* ===================================================================== */

void project_output(AttentionBuffers *buffers, const Trit *w_o,
                    double *output) {
    ternary_matmul(buffers->concat, w_o, output, ATTN_EMBED_DIM, ATTN_EMBED_DIM);
}

/* ===================================================================== */
/* Residual Connection                                                     */
/* ===================================================================== */

void add_residual(double *output, const double *input, size_t dim) {
    size_t i;
    for (i = 0; i < dim; i++) {
        output[i] += input[i];
    }
}

/* ===================================================================== */
/* Main Attention Kernel                                                   */
/* ===================================================================== */

void sacred_attention_kernel(
    const double *input,
    const Trit *w_q, const Trit *w_k, const Trit *w_v, const Trit *w_o,
    size_t position, size_t seq_len,
    double *output,
    double *cache_k, double *cache_v)
{
    AttentionBuffers buffers;
    memset(&buffers, 0, sizeof(buffers));

    /* Step 1: Project Q, K, V */
    project_qkv(&buffers, input, w_q, w_k, w_v);

    /* Step 2: Apply phi-RoPE */
    apply_rope_qk(&buffers, position);

    /* Step 3: Cache K and V */
    cache_kv(&buffers, position, cache_k, cache_v);

    /* Step 4: Compute scores (Q @ K^T * SACRED_SCALE) */
    compute_scores(&buffers, position, seq_len, cache_k);

    /* Step 5: Softmax */
    apply_softmax(&buffers, seq_len);

    /* Step 6: Weighted values */
    weighted_values(&buffers, seq_len, cache_v);

    /* Step 7: Output projection */
    project_output(&buffers, w_o, output);

    /* Step 8: Residual connection */
    add_residual(output, input, ATTN_EMBED_DIM);
}
