/* Auto-generated from specs/nn/hslm.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/nn/hslm.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 29 | Module: HSLM | Hierarchical Sacred Learning Model */

#include "hslm.h"
#include <math.h>
#include <string.h>

/* ===================================================================== */
/* Module state                                                            */
/* ===================================================================== */

static uint8_t hslm_mode = 0;

/* ===================================================================== */
/* Ternary Matrix Multiplication                                           */
/* ===================================================================== */

void hslm_ternary_matmul(const double *input, const Trit *weights,
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
/* RMS Normalization                                                       */
/* ===================================================================== */

void hslm_rms_norm_forward(double *x, const double *gamma, size_t dim) {
    double sum_squares = 0.0;
    size_t i;

    for (i = 0; i < dim; i++) {
        sum_squares += x[i] * x[i];
    }

    double mean_sq = sum_squares / (double)dim;
    double rms = sqrt(mean_sq + 1e-6);

    for (i = 0; i < dim; i++) {
        x[i] = (x[i] / rms) * gamma[i];
    }
}

/* ===================================================================== */
/* GELU Activation                                                         */
/* ===================================================================== */

void hslm_gelu_activation(double *x, size_t dim) {
    const double sqrt_2_over_pi = sqrt(2.0 / 3.141592653589793);
    size_t i;

    for (i = 0; i < dim; i++) {
        double val = x[i];
        double cube = val * val * val;
        double inner = sqrt_2_over_pi * (val + 0.044715 * cube);
        double tanh_val = tanh(inner);
        x[i] = 0.5 * val * (1.0 + tanh_val);
    }
}

/* ===================================================================== */
/* Feed-Forward Network                                                    */
/* ===================================================================== */

void hslm_ffn_forward(HSLMLayerBuffers *buffers,
                       const HSLMLayerWeights *weights) {
    /* Step 1: Project to FF_DIM: intermediate = output @ W1 */
    hslm_ternary_matmul(buffers->output, weights->w1,
                         buffers->ffn_intermediate,
                         HSLM_EMBED_DIM, HSLM_FF_DIM);

    /* Step 2: GELU activation */
    hslm_gelu_activation(buffers->ffn_intermediate, HSLM_FF_DIM);

    /* Step 3: Project back to EMBED_DIM: output = intermediate @ W2 */
    hslm_ternary_matmul(buffers->ffn_intermediate, weights->w2,
                         buffers->output,
                         HSLM_FF_DIM, HSLM_EMBED_DIM);
}

/* ===================================================================== */
/* Layer Forward                                                           */
/* ===================================================================== */

void hslm_layer_forward(HSLMLayerBuffers *buffers,
                         const HSLMLayerWeights *weights,
                         size_t position, size_t seq_len,
                         HSLMAttentionCache *cache) {
    size_t i;

    /* Copy input to output for residual */
    for (i = 0; i < HSLM_EMBED_DIM; i++) {
        buffers->output[i] = buffers->input[i];
    }

    /* Step 1: RMSNorm before attention */
    hslm_rms_norm_forward(buffers->output, weights->norm1_gamma, HSLM_EMBED_DIM);

    /* Save for residual */
    for (i = 0; i < HSLM_EMBED_DIM; i++) {
        buffers->temp[i] = buffers->output[i];
    }

    /* Step 2: Sacred attention */
    sacred_attention_kernel(
        buffers->output,
        weights->w_q, weights->w_k, weights->w_v, weights->w_o,
        position, seq_len,
        buffers->output,
        cache->cache_k, cache->cache_v);

    /* Step 3: Residual (attention + input) */
    for (i = 0; i < HSLM_EMBED_DIM; i++) {
        buffers->output[i] += buffers->input[i];
    }

    /* Save attention output for second residual */
    for (i = 0; i < HSLM_EMBED_DIM; i++) {
        buffers->input[i] = buffers->output[i];
    }

    /* Step 4: RMSNorm before FFN */
    hslm_rms_norm_forward(buffers->output, weights->norm2_gamma, HSLM_EMBED_DIM);

    /* Step 5: FFN */
    hslm_ffn_forward(buffers, weights);

    /* Step 6: Residual (ffn + attention_output) */
    for (i = 0; i < HSLM_EMBED_DIM; i++) {
        buffers->output[i] += buffers->input[i];
    }
}

/* ===================================================================== */
/* Full Forward Pass                                                       */
/* ===================================================================== */

void hslm_forward(const double *input, const HSLMWeights *weights,
                   size_t seq_len, double *output,
                   HSLMAttentionCache *caches) {
    size_t position, layer_idx, i;

    for (position = 0; position < seq_len; position++) {
        HSLMLayerBuffers buffers;

        /* Initialize with input embedding */
        for (i = 0; i < HSLM_EMBED_DIM; i++) {
            buffers.input[i] = input[position * HSLM_EMBED_DIM + i];
        }
        memset(buffers.output, 0, sizeof(buffers.output));
        memset(buffers.temp, 0, sizeof(buffers.temp));
        memset(buffers.ffn_intermediate, 0, sizeof(buffers.ffn_intermediate));

        /* Process through all layers */
        for (layer_idx = 0; layer_idx < HSLM_NUM_LAYERS; layer_idx++) {
            hslm_layer_forward(&buffers, &weights->layers[layer_idx],
                                position, seq_len, &caches[layer_idx]);

            /* Copy output to input for next layer */
            for (i = 0; i < HSLM_EMBED_DIM; i++) {
                buffers.input[i] = buffers.output[i];
            }
        }

        /* Store final layer output */
        for (i = 0; i < HSLM_EMBED_DIM; i++) {
            output[position * HSLM_EMBED_DIM + i] = buffers.output[i];
        }
    }
}

/* ===================================================================== */
/* Backward Pass (stubs)                                                   */
/* ===================================================================== */

void hslm_backward(const double *grad_output, const HSLMWeights *weights,
                    size_t seq_len, double *grad_input) {
    (void)grad_output;
    (void)weights;
    (void)seq_len;
    (void)grad_input;
    /* Backward through layers in reverse order */
    /* Placeholder for gradient computation */
}

/* ===================================================================== */
/* Phase Management                                                        */
/* ===================================================================== */

void hslm_set_phase(uint8_t phase) {
    if (phase == HSLM_PHASE_FORWARD) {
        hslm_mode = 1;
    } else if (phase == HSLM_PHASE_BACKWARD) {
        hslm_mode = 2;
    } else if (phase == HSLM_PHASE_UPDATE) {
        hslm_mode = 3;
    }
}

uint8_t hslm_get_mode(void) {
    return hslm_mode;
}
