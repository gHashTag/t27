/* Auto-generated from specs/vsa/ops.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/vsa/ops.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 28 | Module: VSAOps */

#include "ops.h"
#include <math.h>
#include <assert.h>
#include <string.h>

/* ===================================================================== */
/* 2. Bind Operation                                                      */
/* ===================================================================== */

void vsa_bind(const Trit *a, const Trit *b, size_t len, Trit *result) {
    size_t i;
    for (i = 0; i < len; i++) {
        Trit ai = a[i];
        Trit bi = b[i];
        if (ai == TRIT_ZERO) {
            result[i] = bi;
        } else if (bi == TRIT_ZERO) {
            result[i] = ai;
        } else {
            /* Both non-zero: multiply (both are +/-1) */
            result[i] = (ai == bi) ? TRIT_POS : TRIT_NEG;
        }
    }
}

/* ===================================================================== */
/* 3. Unbind Operation                                                    */
/* ===================================================================== */

void vsa_unbind(const Trit *bound, const Trit *key, size_t len, Trit *result) {
    /* unbind = bind for XOR-like implementation */
    vsa_bind(bound, key, len, result);
}

/* ===================================================================== */
/* 4. Bundle Operations                                                   */
/* ===================================================================== */

void vsa_bundle2(const Trit *a, const Trit *b, size_t len, Trit *result) {
    size_t i;
    for (i = 0; i < len; i++) {
        Trit ai = a[i];
        Trit bi = b[i];
        if (ai == TRIT_ZERO) {
            result[i] = bi;
        } else if (bi == TRIT_ZERO) {
            result[i] = ai;
        } else {
            int sum = (int)ai + (int)bi;
            if (sum > 0) result[i] = TRIT_POS;
            else if (sum < 0) result[i] = TRIT_NEG;
            else result[i] = TRIT_ZERO;
        }
    }
}

void vsa_bundle3(const Trit *a, const Trit *b, const Trit *c, size_t len, Trit *result) {
    size_t i;
    for (i = 0; i < len; i++) {
        int sum = (int)a[i] + (int)b[i] + (int)c[i];
        if (sum > 0) result[i] = TRIT_POS;
        else if (sum < 0) result[i] = TRIT_NEG;
        else result[i] = TRIT_ZERO;
    }
}

/* ===================================================================== */
/* 5. Similarity Operations                                               */
/* ===================================================================== */

double vsa_dot_product(const Trit *a, const Trit *b, size_t len) {
    int64_t acc = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        acc += (int16_t)a[i] * (int16_t)b[i];
    }
    return (double)acc;
}

double vsa_vector_norm(const Trit *v, size_t len) {
    size_t nonzero_count = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        if (v[i] != TRIT_ZERO) {
            nonzero_count++;
        }
    }
    return sqrt((double)nonzero_count);
}

size_t vsa_hamming_distance(const Trit *a, const Trit *b, size_t len) {
    size_t distance = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        if (a[i] != b[i]) {
            distance++;
        }
    }
    return distance;
}

double vsa_cosine_similarity(const Trit *a, const Trit *b, size_t len) {
    double dot = vsa_dot_product(a, b, len);
    double norm_a = vsa_vector_norm(a, len);
    double norm_b = vsa_vector_norm(b, len);
    if (norm_a == 0.0 || norm_b == 0.0) {
        return 0.0;
    }
    return dot / (norm_a * norm_b);
}

double vsa_hamming_similarity(const Trit *a, const Trit *b, size_t len) {
    size_t dist = vsa_hamming_distance(a, b, len);
    return 1.0 - ((double)dist / (double)len);
}

double vsa_similarity(const Trit *a, const Trit *b, size_t len, uint8_t metric) {
    if (metric == SIM_COSINE) {
        return vsa_cosine_similarity(a, b, len);
    } else if (metric == SIM_HAMMING) {
        return vsa_hamming_similarity(a, b, len);
    }
    return vsa_dot_product(a, b, len);
}

/* ===================================================================== */
/* 6. Permutation Operations                                              */
/* ===================================================================== */

void vsa_permute(const Trit *v, size_t len, size_t shift, Trit *result) {
    size_t normalized_shift = shift % len;
    size_t i;
    for (i = 0; i < len; i++) {
        size_t src_idx = (i + normalized_shift) % len;
        result[i] = v[src_idx];
    }
}

void vsa_encode_sequence(const Trit **items, size_t count, size_t item_len, Trit *result) {
    Trit permuted[VSA_DIM];
    Trit temp[VSA_DIM];
    size_t i;

    /* Copy items[0] into result */
    memcpy(result, items[0], item_len * sizeof(Trit));

    for (i = 1; i < count; i++) {
        vsa_permute(items[i], item_len, i, permuted);
        memcpy(temp, result, item_len * sizeof(Trit));
        vsa_bundle2(temp, permuted, item_len, result);
    }
}

double vsa_probe_sequence(const Trit *seq, const Trit *candidate, size_t position, size_t len) {
    Trit permuted[VSA_DIM];
    vsa_permute(candidate, len, position, permuted);
    return vsa_similarity(seq, permuted, len, SIM_COSINE);
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_vsa_ops(void) {
    /* test vsa_bind_with_zeros */
    {
        Trit a[] = {TRIT_ZERO, TRIT_POS, TRIT_NEG};
        Trit b[] = {TRIT_POS, TRIT_ZERO, TRIT_NEG};
        Trit result[3];
        vsa_bind(a, b, 3, result);
        assert(result[0] == TRIT_POS);
        assert(result[1] == TRIT_POS);
        assert(result[2] == TRIT_POS);
    }

    /* test vsa_bind_nonzero_multiply */
    {
        Trit a[] = {TRIT_POS, TRIT_POS, TRIT_NEG, TRIT_NEG};
        Trit b[] = {TRIT_POS, TRIT_NEG, TRIT_POS, TRIT_NEG};
        Trit result[4];
        vsa_bind(a, b, 4, result);
        assert(result[0] == TRIT_POS);
        assert(result[1] == TRIT_NEG);
        assert(result[2] == TRIT_NEG);
        assert(result[3] == TRIT_POS);
    }

    /* test vsa_bundle2_with_zero */
    {
        Trit a[] = {TRIT_ZERO, TRIT_POS, TRIT_NEG};
        Trit b[] = {TRIT_POS, TRIT_ZERO, TRIT_NEG};
        Trit result[3];
        vsa_bundle2(a, b, 3, result);
        assert(result[0] == TRIT_POS);
        assert(result[1] == TRIT_POS);
        assert(result[2] == TRIT_NEG);
    }

    /* test vsa_bundle2_majority_vote */
    {
        Trit a[] = {TRIT_POS, TRIT_NEG, TRIT_POS};
        Trit b[] = {TRIT_NEG, TRIT_NEG, TRIT_NEG};
        Trit result[3];
        vsa_bundle2(a, b, 3, result);
        assert(result[0] == TRIT_ZERO);
        assert(result[1] == TRIT_NEG);
        assert(result[2] == TRIT_ZERO);
    }

    /* test vsa_bundle3_consensus */
    {
        Trit a[] = {TRIT_POS, TRIT_POS, TRIT_NEG};
        Trit b[] = {TRIT_POS, TRIT_NEG, TRIT_POS};
        Trit c[] = {TRIT_POS, TRIT_POS, TRIT_POS};
        Trit result[3];
        vsa_bundle3(a, b, c, 3, result);
        assert(result[0] == TRIT_POS);
        assert(result[1] == TRIT_POS);
        assert(result[2] == TRIT_POS);
    }

    /* test vsa_dot_product_identical */
    {
        Trit a[] = {TRIT_POS, TRIT_NEG, TRIT_POS, TRIT_ZERO};
        Trit b[] = {TRIT_POS, TRIT_NEG, TRIT_POS, TRIT_ZERO};
        double result = vsa_dot_product(a, b, 4);
        assert(result == 3.0);
    }

    /* test vsa_hamming_distance_identical */
    {
        Trit a[] = {TRIT_POS, TRIT_NEG, TRIT_ZERO};
        Trit b[] = {TRIT_POS, TRIT_NEG, TRIT_ZERO};
        assert(vsa_hamming_distance(a, b, 3) == 0);
    }

    /* test vsa_hamming_distance_different */
    {
        Trit a[] = {TRIT_POS, TRIT_POS, TRIT_POS};
        Trit b[] = {TRIT_NEG, TRIT_NEG, TRIT_NEG};
        assert(vsa_hamming_distance(a, b, 3) == 3);
    }

    /* test vsa_permute_shift_by_len_returns_original */
    {
        Trit v[] = {TRIT_POS, TRIT_NEG, TRIT_ZERO};
        Trit result[3];
        vsa_permute(v, 3, 3, result);
        assert(result[0] == TRIT_POS);
        assert(result[1] == TRIT_NEG);
        assert(result[2] == TRIT_ZERO);
    }

    /* test vsa_bind_unbind_identity */
    {
        Trit x[] = {TRIT_POS, TRIT_NEG, TRIT_ZERO, TRIT_POS};
        Trit key[] = {TRIT_NEG, TRIT_POS, TRIT_POS, TRIT_NEG};
        Trit bound[4], unbound[4];
        vsa_bind(x, key, 4, bound);
        vsa_unbind(bound, key, 4, unbound);
        double sim = vsa_similarity(x, unbound, 4, SIM_COSINE);
        assert(sim > 0.95);
    }
}
