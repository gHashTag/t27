/* Auto-generated from specs/vsa/ops.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/vsa/ops.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 28 | Module: VSAOps */

#ifndef VSA_OPS_H
#define VSA_OPS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ===================================================================== */
/* Trit type for balanced ternary hypervectors                           */
/* ===================================================================== */

typedef int8_t Trit;

#define TRIT_NEG  ((Trit)-1)
#define TRIT_ZERO ((Trit)0)
#define TRIT_POS  ((Trit)1)

/* ===================================================================== */
/* Constants                                                              */
/* ===================================================================== */

#define VSA_DIM       1024
#define SIMD_WIDTH    32
#define MAX_VECTORS   32

#define SIM_COSINE    0
#define SIM_HAMMING   1
#define SIM_DOT       2

#define BIND_IDENTITY 0
#define BIND_INVERT   1

/* ===================================================================== */
/* Bind / Unbind                                                          */
/* ===================================================================== */

void vsa_bind(const Trit *a, const Trit *b, size_t len, Trit *result);
void vsa_unbind(const Trit *bound, const Trit *key, size_t len, Trit *result);

/* ===================================================================== */
/* Bundle                                                                 */
/* ===================================================================== */

void vsa_bundle2(const Trit *a, const Trit *b, size_t len, Trit *result);
void vsa_bundle3(const Trit *a, const Trit *b, const Trit *c, size_t len, Trit *result);

/* ===================================================================== */
/* Similarity                                                             */
/* ===================================================================== */

double vsa_dot_product(const Trit *a, const Trit *b, size_t len);
double vsa_vector_norm(const Trit *v, size_t len);
size_t vsa_hamming_distance(const Trit *a, const Trit *b, size_t len);
double vsa_cosine_similarity(const Trit *a, const Trit *b, size_t len);
double vsa_hamming_similarity(const Trit *a, const Trit *b, size_t len);
double vsa_similarity(const Trit *a, const Trit *b, size_t len, uint8_t metric);

/* ===================================================================== */
/* Permutation                                                            */
/* ===================================================================== */

void vsa_permute(const Trit *v, size_t len, size_t shift, Trit *result);
void vsa_encode_sequence(const Trit **items, size_t count, size_t item_len, Trit *result);
double vsa_probe_sequence(const Trit *seq, const Trit *candidate, size_t position, size_t len);

/* ===================================================================== */
/* Test entry point                                                       */
/* ===================================================================== */

void test_vsa_ops(void);

#endif /* VSA_OPS_H */
