/* SPDX-License-Identifier: Apache-2.0
 * conformance/trinity/abi/abi_fixture.c -- the ABI fixture of specs/api/c_abi.t27 (S05 of
 * gHashTag/trinity#988). Compiled against src/libs/c/libtrinityvsa/include/trinity_vsa.h of
 * gHashTag/trinity by tools/trinity_c_abi.py (`cc -fsyntax-only`), which proves the prototypes
 * this fixture uses are the header's. Linked against libtrinity-vsa it exercises ownership
 * (every handle freed once), NULL safety, the clamped dimension, the trit normalization and the
 * bind/unbind round trip, and exits nonzero on the first mismatch. It has not been linked at
 * the pin: src/c_api.zig does not parse there (the spec's FINDINGS). */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "trinity_vsa.h"

static int failures = 0;
static void check(const char *name, int ok) {
    printf("[ABI] %s : %s\n", name, ok ? "PASSED" : "FAILED");
    if (!ok) failures++;
}

int main(void) {
    const char *version = trinity_vsa_version();
    check("version is 0.2.0", version != NULL && strcmp(version, "0.2.0") == 0);
    check("max_dim is 59049", trinity_vsa_max_dim() == 59049);

    /* NULL safety: every accessor tolerates a NULL handle. */
    trinity_vsa_vector_free(NULL);
    check("get_dim(NULL) is 0", trinity_vsa_get_dim(NULL) == 0);
    check("get_trit(NULL) is 0", trinity_vsa_get_trit(NULL, 0) == 0);
    check("cosine(NULL, NULL) is 0.0", trinity_vsa_cosine_similarity(NULL, NULL) == 0.0);
    check("hamming(NULL, NULL) is 0", trinity_vsa_hamming_distance(NULL, NULL) == 0);
    check("dot(NULL, NULL) is 0", trinity_vsa_dot_product(NULL, NULL) == 0);
    check("clone(NULL) is NULL", trinity_vsa_vector_clone(NULL) == NULL);
    check("bind(NULL, NULL) is NULL", trinity_vsa_bind(NULL, NULL) == NULL);

    /* Ownership: a handle is created by the library and released by the caller, once. */
    trinity_vsa_vector_t zeros = trinity_vsa_vector_zeros(8);
    check("zeros(8) is a handle", zeros != NULL);
    check("zeros(8) has dimension 8", trinity_vsa_get_dim(zeros) == 8);
    check("zeros(8) is all zero", trinity_vsa_get_trit(zeros, 0) == 0 && trinity_vsa_get_trit(zeros, 7) == 0);
    check("get_trit beyond the length is 0", trinity_vsa_get_trit(zeros, 8) == 0);

    /* The clamped dimension. */
    trinity_vsa_vector_t big = trinity_vsa_vector_zeros(70000);
    check("zeros(70000) is clamped to 59049", big != NULL && trinity_vsa_get_dim(big) == 59049);
    trinity_vsa_vector_free(big);

    /* Trit normalization across from_array / to_array. */
    const int8_t raw[8] = { 5, -5, 0, 1, -1, 2, -2, 0 };
    const int8_t want[8] = { 1, -1, 0, 1, -1, 1, -1, 0 };
    trinity_vsa_vector_t v = trinity_vsa_from_array(raw, 8);
    int8_t out[8] = { 9, 9, 9, 9, 9, 9, 9, 9 };
    size_t n = trinity_vsa_to_array(v, out, 8);
    check("to_array writes eight trits", n == 8);
    check("from_array normalizes to -1, 0, 1", memcmp(out, want, 8) == 0);
    check("to_array stops at the shorter length", trinity_vsa_to_array(v, out, 3) == 3);

    /* set_trit in place; an index beyond the length is ignored. */
    trinity_vsa_set_trit(v, 2, 7);
    check("set_trit normalizes and writes in place", trinity_vsa_get_trit(v, 2) == 1);
    trinity_vsa_set_trit(v, 8, 1);
    check("set_trit beyond the length is ignored", trinity_vsa_get_dim(v) == 8);

    /* The bind/unbind round trip on nonzero trits (S04: bind is its own inverse there). */
    const int8_t key_raw[8] = { 1, -1, 1, -1, 1, 1, -1, -1 };
    const int8_t val_raw[8] = { 1, 1, -1, -1, 1, -1, 1, -1 };
    trinity_vsa_vector_t key = trinity_vsa_from_array(key_raw, 8);
    trinity_vsa_vector_t val = trinity_vsa_from_array(val_raw, 8);
    trinity_vsa_vector_t bound = trinity_vsa_bind(key, val);
    trinity_vsa_vector_t back = trinity_vsa_unbind(bound, key);
    check("bind returns a new handle", bound != NULL && bound != key && bound != val);
    check("unbind(bind(k, v), k) is v", back != NULL && trinity_vsa_hamming_distance(back, val) == 0);
    check("cosine(v, v) is 1.0", trinity_vsa_cosine_similarity(val, val) == 1.0);
    check("dot(v, v) counts the nonzero trits", trinity_vsa_dot_product(val, val) == 8);
    trinity_vsa_vector_t clone = trinity_vsa_vector_clone(val);
    check("clone is equal and distinct", clone != NULL && clone != val && trinity_vsa_hamming_distance(clone, val) == 0);

    trinity_vsa_vector_free(clone);
    trinity_vsa_vector_free(back);
    trinity_vsa_vector_free(bound);
    trinity_vsa_vector_free(val);
    trinity_vsa_vector_free(key);
    trinity_vsa_vector_free(v);
    trinity_vsa_vector_free(zeros);

    printf("abi fixture: %d failed\n", failures);
    return failures ? 1 : 0;
}
