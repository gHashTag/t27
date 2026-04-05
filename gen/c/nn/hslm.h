/* Auto-generated from specs/nn/hslm.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/nn/hslm.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 29 | Module: HSLM | Hierarchical Sacred Learning Model */

#ifndef HSLM_H
#define HSLM_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "attention.h"

/* ===================================================================== */
/* Constants                                                               */
/* ===================================================================== */

#define HSLM_NUM_LAYERS      6
#define HSLM_NUM_HEADS       3
#define HSLM_HEAD_DIM        81      /* 3^4 */
#define HSLM_EMBED_DIM       243     /* 3 * 81 */
#define HSLM_FF_DIM          972     /* 4 * EMBED_DIM */
#define HSLM_CONTEXT_LEN     81
#define HSLM_VSA_DIM         1024

/* Layer phases */
#define HSLM_PHASE_NORM      0
#define HSLM_PHASE_ATTN      1
#define HSLM_PHASE_FFN       2
#define HSLM_PHASE_RESIDUAL  3

/* Activation functions */
#define HSLM_ACT_RELU        0
#define HSLM_ACT_GELU        1
#define HSLM_ACT_SWISH       2
#define HSLM_ACT_TERNARY     3

/* Training phases */
#define HSLM_PHASE_FORWARD   0
#define HSLM_PHASE_BACKWARD  1
#define HSLM_PHASE_UPDATE    2

/* ===================================================================== */
/* Types                                                                   */
/* ===================================================================== */

typedef struct {
    double input[HSLM_EMBED_DIM];
    double output[HSLM_EMBED_DIM];
    double temp[HSLM_EMBED_DIM];
    double ffn_intermediate[HSLM_FF_DIM];
} HSLMLayerBuffers;

typedef struct {
    double cache_k[HSLM_CONTEXT_LEN * HSLM_EMBED_DIM];
    double cache_v[HSLM_CONTEXT_LEN * HSLM_EMBED_DIM];
} HSLMAttentionCache;

typedef struct {
    Trit w_q[HSLM_EMBED_DIM * HSLM_EMBED_DIM];
    Trit w_k[HSLM_EMBED_DIM * HSLM_EMBED_DIM];
    Trit w_v[HSLM_EMBED_DIM * HSLM_EMBED_DIM];
    Trit w_o[HSLM_EMBED_DIM * HSLM_EMBED_DIM];
    Trit w1[HSLM_EMBED_DIM * HSLM_FF_DIM];
    Trit w2[HSLM_FF_DIM * HSLM_EMBED_DIM];
    double norm1_gamma[HSLM_EMBED_DIM];
    double norm2_gamma[HSLM_EMBED_DIM];
} HSLMLayerWeights;

typedef struct {
    HSLMLayerWeights layers[HSLM_NUM_LAYERS];
} HSLMWeights;

/* ===================================================================== */
/* Function declarations                                                   */
/* ===================================================================== */

/* Ternary matrix multiplication */
void hslm_ternary_matmul(const double *input, const Trit *weights,
                          double *output, size_t in_dim, size_t out_dim);

/* RMS normalization (in-place) */
void hslm_rms_norm_forward(double *x, const double *gamma, size_t dim);

/* GELU activation (in-place) */
void hslm_gelu_activation(double *x, size_t dim);

/* Feed-forward network */
void hslm_ffn_forward(HSLMLayerBuffers *buffers,
                       const HSLMLayerWeights *weights);

/* Single layer forward pass */
void hslm_layer_forward(HSLMLayerBuffers *buffers,
                         const HSLMLayerWeights *weights,
                         size_t position, size_t seq_len,
                         HSLMAttentionCache *cache);

/* Full HSLM forward pass */
void hslm_forward(const double *input, const HSLMWeights *weights,
                   size_t seq_len, double *output,
                   HSLMAttentionCache *caches);

/* Backward pass (gradient computation) */
void hslm_backward(const double *grad_output, const HSLMWeights *weights,
                    size_t seq_len, double *grad_input);

/* Phase management */
void hslm_set_phase(uint8_t phase);
uint8_t hslm_get_mode(void);

#endif /* HSLM_H */
