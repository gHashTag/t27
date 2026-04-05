/* Auto-generated from specs/vsa/core.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/vsa/core.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: VSACore */

#ifndef VSA_CORE_H
#define VSA_CORE_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include "ops.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ===================================================================== */
/* Constants                                                              */
/* ===================================================================== */

#define DIMENSION             1024
#define SIMILARITY_THRESHOLD  0.15
#define CODEBOOK_CAPACITY     256
#define MAX_PREDICATE_ARGS    8

/* ===================================================================== */
/* Core Types                                                             */
/* ===================================================================== */

/* HyperVector: array of DIMENSION trits {-1, 0, +1} */
typedef Trit HyperVector[DIMENSION];

/* Codebook: cleanup memory / item memory for nearest-neighbor lookup */
typedef struct {
    HyperVector entries[CODEBOOK_CAPACITY];
    uint32_t    labels[CODEBOOK_CAPACITY];
    size_t      count;
} Codebook;

/* PredicateEncoding: result of encoding a predicate-argument structure */
typedef struct {
    HyperVector vector;
    HyperVector predicate;
    size_t      arg_count;
} PredicateEncoding;

/* ===================================================================== */
/* Random Hypervector Generation                                          */
/* ===================================================================== */

void vsa_core_random_hypervector(uint64_t seed, HyperVector result);

/* ===================================================================== */
/* Predicate-Argument Encoding                                            */
/* ===================================================================== */

void vsa_core_encode_predicate(const Trit *predicate, const Trit **args,
                               size_t arg_count, HyperVector result);
void vsa_core_decode_argument(const Trit *encoded, const Trit *predicate,
                              size_t position, HyperVector result);

/* ===================================================================== */
/* Codebook (Cleanup Memory)                                              */
/* ===================================================================== */

bool     vsa_core_codebook_add(Codebook *cb, const Trit *vector, uint32_t label);
uint32_t vsa_core_codebook_lookup(const Codebook *cb, const Trit *query);
void     vsa_core_codebook_cleanup(const Codebook *cb, const Trit *noisy,
                                   HyperVector result);

/* ===================================================================== */
/* Compositional Query Operations                                         */
/* ===================================================================== */

void vsa_core_query_role(const Trit *structure, const Trit *filler,
                         HyperVector result);
void vsa_core_query_filler(const Trit *structure, const Trit *role,
                           HyperVector result);
void vsa_core_analogy(const Trit *a, const Trit *b, const Trit *c,
                      HyperVector result);

/* ===================================================================== */
/* Resonator Network                                                      */
/* ===================================================================== */

void vsa_core_resonator_step(const Trit **estimates, const Trit *target,
                             const Codebook *codebooks, size_t factor_idx,
                             size_t factor_count, HyperVector result);
void vsa_core_resonator_solve(const Trit *target, const Codebook *codebooks,
                              size_t factor_count, size_t max_iters,
                              HyperVector *results);

/* ===================================================================== */
/* Test entry point                                                       */
/* ===================================================================== */

void test_vsa_core(void);

#ifdef __cplusplus
}
#endif

#endif /* VSA_CORE_H */
