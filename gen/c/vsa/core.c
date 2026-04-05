/* Auto-generated from specs/vsa/core.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/vsa/core.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: VSACore */

#include "core.h"
#include <math.h>
#include <assert.h>
#include <string.h>

/* ===================================================================== */
/* 3. Random Hypervector Generation                                       */
/* ===================================================================== */

void vsa_core_random_hypervector(uint64_t seed, HyperVector result) {
    uint64_t state = seed;
    size_t i;
    for (i = 0; i < DIMENSION; i++) {
        /* xorshift64 */
        state ^= (state << 13);
        state ^= (state >> 7);
        state ^= (state << 17);

        uint64_t r = state % 3;
        if (r == 0)      result[i] = TRIT_NEG;
        else if (r == 1)  result[i] = TRIT_ZERO;
        else              result[i] = TRIT_POS;
    }
}

/* ===================================================================== */
/* 4. Predicate-Argument Encoding                                         */
/* ===================================================================== */

void vsa_core_encode_predicate(const Trit *predicate, const Trit **args,
                               size_t arg_count, HyperVector result) {
    HyperVector permuted_args[MAX_PREDICATE_ARGS];
    size_t i;

    /* Permute each argument by its 1-based position */
    for (i = 0; i < arg_count; i++) {
        vsa_permute(args[i], DIMENSION, i + 1, permuted_args[i]);
    }

    /* Bundle all permuted arguments via iterative majority vote */
    HyperVector bundled;
    memcpy(bundled, permuted_args[0], DIMENSION * sizeof(Trit));
    for (i = 1; i < arg_count; i++) {
        HyperVector temp;
        memcpy(temp, bundled, DIMENSION * sizeof(Trit));
        vsa_bundle2(temp, permuted_args[i], DIMENSION, bundled);
    }

    /* Bind predicate with bundled arguments */
    vsa_bind(predicate, bundled, DIMENSION, result);
}

void vsa_core_decode_argument(const Trit *encoded, const Trit *predicate,
                              size_t position, HyperVector result) {
    /* Unbind predicate to get bundled arguments */
    HyperVector unbound;
    vsa_unbind(encoded, predicate, DIMENSION, unbound);

    /* Inverse permute to extract the argument at position */
    size_t shift = position + 1;
    size_t inverse_shift = DIMENSION - (shift % DIMENSION);
    vsa_permute(unbound, DIMENSION, inverse_shift, result);
}

/* ===================================================================== */
/* 5. Codebook (Cleanup Memory)                                           */
/* ===================================================================== */

bool vsa_core_codebook_add(Codebook *cb, const Trit *vector, uint32_t label) {
    if (cb->count >= CODEBOOK_CAPACITY) {
        return false;
    }

    memcpy(cb->entries[cb->count], vector, DIMENSION * sizeof(Trit));
    cb->labels[cb->count] = label;
    cb->count = cb->count + 1;
    return true;
}

uint32_t vsa_core_codebook_lookup(const Codebook *cb, const Trit *query) {
    if (cb->count == 0) {
        return 0xFFFFFFFF;
    }

    uint32_t best_label = cb->labels[0];
    double best_sim = -2.0;
    size_t i;

    for (i = 0; i < cb->count; i++) {
        double sim = vsa_cosine_similarity(query, cb->entries[i], DIMENSION);
        if (sim > best_sim) {
            best_sim = sim;
            best_label = cb->labels[i];
        }
    }

    return best_label;
}

void vsa_core_codebook_cleanup(const Codebook *cb, const Trit *noisy,
                               HyperVector result) {
    if (cb->count == 0) {
        memcpy(result, noisy, DIMENSION * sizeof(Trit));
        return;
    }

    size_t best_idx = 0;
    double best_sim = -2.0;
    size_t i;

    for (i = 0; i < cb->count; i++) {
        double sim = vsa_cosine_similarity(noisy, cb->entries[i], DIMENSION);
        if (sim > best_sim) {
            best_sim = sim;
            best_idx = i;
        }
    }

    if (best_sim < SIMILARITY_THRESHOLD) {
        memcpy(result, noisy, DIMENSION * sizeof(Trit));
        return;
    }

    memcpy(result, cb->entries[best_idx], DIMENSION * sizeof(Trit));
}

/* ===================================================================== */
/* 6. Compositional Query Operations                                      */
/* ===================================================================== */

void vsa_core_query_role(const Trit *structure, const Trit *filler,
                         HyperVector result) {
    vsa_unbind(structure, filler, DIMENSION, result);
}

void vsa_core_query_filler(const Trit *structure, const Trit *role,
                           HyperVector result) {
    vsa_unbind(structure, role, DIMENSION, result);
}

void vsa_core_analogy(const Trit *a, const Trit *b, const Trit *c,
                      HyperVector result) {
    HyperVector relation;
    vsa_unbind(b, a, DIMENSION, relation);
    vsa_bind(relation, c, DIMENSION, result);
}

/* ===================================================================== */
/* 7. Resonator Network                                                   */
/* ===================================================================== */

void vsa_core_resonator_step(const Trit **estimates, const Trit *target,
                             const Codebook *codebooks, size_t factor_idx,
                             size_t factor_count, HyperVector result) {
    /* Unbind all factors except factor_idx */
    HyperVector remainder;
    memcpy(remainder, target, DIMENSION * sizeof(Trit));

    size_t i;
    for (i = 0; i < factor_count; i++) {
        if (i != factor_idx) {
            HyperVector temp;
            memcpy(temp, remainder, DIMENSION * sizeof(Trit));
            vsa_unbind(temp, estimates[i], DIMENSION, remainder);
        }
    }

    /* Cleanup via codebook */
    vsa_core_codebook_cleanup(&codebooks[factor_idx], remainder, result);
}

void vsa_core_resonator_solve(const Trit *target, const Codebook *codebooks,
                              size_t factor_count, size_t max_iters,
                              HyperVector *results) {
    /* Initialize estimates with first entry of each codebook */
    HyperVector estimates[MAX_PREDICATE_ARGS];
    size_t i;
    for (i = 0; i < factor_count; i++) {
        memcpy(estimates[i], codebooks[i].entries[0], DIMENSION * sizeof(Trit));
    }

    size_t iter;
    for (iter = 0; iter < max_iters; iter++) {
        int converged = 1;

        for (i = 0; i < factor_count; i++) {
            HyperVector old_estimate;
            memcpy(old_estimate, estimates[i], DIMENSION * sizeof(Trit));

            /* Build pointer array for current estimates */
            const Trit *est_ptrs[MAX_PREDICATE_ARGS];
            size_t s;
            for (s = 0; s < factor_count; s++) {
                est_ptrs[s] = estimates[s];
            }

            vsa_core_resonator_step(est_ptrs, target, codebooks, i,
                                    factor_count, estimates[i]);

            size_t dist = vsa_hamming_distance(old_estimate, estimates[i],
                                               DIMENSION);
            if (dist > 0) {
                converged = 0;
            }
        }

        if (converged) {
            break;
        }
    }

    /* Copy results out */
    for (i = 0; i < factor_count; i++) {
        memcpy(results[i], estimates[i], DIMENSION * sizeof(Trit));
    }
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_vsa_core(void) {
    /* test vsa_core_bind_self_inverse */
    {
        HyperVector a, b, bound, recovered;
        vsa_core_random_hypervector(42, a);
        vsa_core_random_hypervector(43, b);
        vsa_bind(a, b, DIMENSION, bound);
        vsa_unbind(bound, b, DIMENSION, recovered);
        double sim = vsa_cosine_similarity(a, recovered, DIMENSION);
        assert(sim > 0.99);
    }

    /* test vsa_core_orthogonality_random_vectors */
    {
        HyperVector a, b;
        vsa_core_random_hypervector(100, a);
        vsa_core_random_hypervector(200, b);
        double sim = vsa_cosine_similarity(a, b, DIMENSION);
        assert(fabs(sim) < 0.1);
    }

    /* test vsa_core_codebook_store_and_retrieve */
    {
        Codebook cb;
        cb.count = 0;
        HyperVector v1, v2;
        vsa_core_random_hypervector(50, v1);
        vsa_core_random_hypervector(51, v2);
        vsa_core_codebook_add(&cb, v1, 1);
        vsa_core_codebook_add(&cb, v2, 2);
        uint32_t label1 = vsa_core_codebook_lookup(&cb, v1);
        uint32_t label2 = vsa_core_codebook_lookup(&cb, v2);
        assert(label1 == 1);
        assert(label2 == 2);
    }

    /* test vsa_core_codebook_cleanup_returns_nearest */
    {
        Codebook cb;
        cb.count = 0;
        HyperVector clean, noise, noisy, cleaned;
        vsa_core_random_hypervector(60, clean);
        vsa_core_codebook_add(&cb, clean, 1);
        vsa_core_random_hypervector(61, noise);
        vsa_bundle2(clean, noise, DIMENSION, noisy);
        vsa_core_codebook_cleanup(&cb, noisy, cleaned);
        double sim = vsa_cosine_similarity(cleaned, clean, DIMENSION);
        assert(sim > SIMILARITY_THRESHOLD);
    }

    /* test vsa_core_query_role_filler */
    {
        HyperVector role, filler, structure, recovered_filler, recovered_role;
        vsa_core_random_hypervector(20, role);
        vsa_core_random_hypervector(21, filler);
        vsa_bind(role, filler, DIMENSION, structure);
        vsa_core_query_filler(structure, role, recovered_filler);
        vsa_core_query_role(structure, filler, recovered_role);
        double sim_f = vsa_cosine_similarity(recovered_filler, filler, DIMENSION);
        double sim_r = vsa_cosine_similarity(recovered_role, role, DIMENSION);
        assert(sim_f > 0.99);
        assert(sim_r > 0.99);
    }

    /* test vsa_core_analogy */
    {
        HyperVector king, queen, royalty, queen_reconstructed;
        vsa_core_random_hypervector(30, king);
        vsa_core_random_hypervector(31, queen);
        vsa_unbind(queen, king, DIMENSION, royalty);
        vsa_bind(royalty, king, DIMENSION, queen_reconstructed);
        double sim = vsa_cosine_similarity(queen_reconstructed, queen, DIMENSION);
        assert(sim > 0.99);
    }

    /* test vsa_core_random_vectors_dense */
    {
        HyperVector v;
        vsa_core_random_hypervector(99, v);
        double norm = vsa_vector_norm(v, DIMENSION);
        assert(norm > 0.0);
    }

    /* test vsa_core_permute_preserves_information */
    {
        HyperVector v, shifted, unshifted;
        vsa_core_random_hypervector(70, v);
        vsa_permute(v, DIMENSION, 5, shifted);
        vsa_permute(shifted, DIMENSION, DIMENSION - 5, unshifted);
        double sim = vsa_cosine_similarity(v, unshifted, DIMENSION);
        assert(sim > 0.99);
    }

    /* test vsa_core_permute_decorrelates */
    {
        HyperVector v, shifted;
        vsa_core_random_hypervector(80, v);
        vsa_permute(v, DIMENSION, 1, shifted);
        double sim = vsa_cosine_similarity(v, shifted, DIMENSION);
        assert(fabs(sim) < 0.15);
    }

    /* test vsa_core_codebook_empty_returns_sentinel */
    {
        Codebook cb;
        cb.count = 0;
        HyperVector query;
        vsa_core_random_hypervector(55, query);
        uint32_t label = vsa_core_codebook_lookup(&cb, query);
        assert(label == 0xFFFFFFFF);
    }

    /* test vsa_core_codebook_full_rejects_add */
    {
        Codebook cb;
        cb.count = CODEBOOK_CAPACITY;
        HyperVector v;
        vsa_core_random_hypervector(56, v);
        bool ok = vsa_core_codebook_add(&cb, v, 999);
        assert(ok == false);
    }
}
