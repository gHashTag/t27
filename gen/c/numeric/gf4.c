/* Auto-generated from specs/numeric/gf4.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf4.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "gf4.h"
#include <assert.h>
#include <stdio.h>

/* ========================================================================== */
/* Encoding / Decoding                                                        */
/* ========================================================================== */

GF4 gf4_encode(float value) {
    GF4 result;
    if (value == 0.0f) {
        result.raw = 0x00;
        return result;
    }
    if (value < 0.0f) {
        GF4 pos = gf4_encode(-value);
        result.raw = pos.raw | 0x08;
        return result;
    }
    /* Quantize to: 0.25, 0.5, 0.75, 1.0, 1.5 */
    if (value <= 0.375f) {
        result.raw = 0x01; /* 0.25 */
    } else if (value <= 0.625f) {
        result.raw = 0x02; /* 0.5  */
    } else if (value <= 0.875f) {
        result.raw = 0x03; /* 0.75 */
    } else if (value <= 1.25f) {
        result.raw = 0x05; /* 1.0  */
    } else {
        result.raw = 0x07; /* 1.5  */
    }
    return result;
}

float gf4_decode(GF4 gf) {
    uint8_t raw = gf.raw & 0x0F;
    if (raw == 0) return 0.0f;

    int sign_bit = (raw & 0x08) != 0;
    int exp_bit  = (raw & 0x04) != 0;
    int mant     = raw & 0x03;

    float m = (float)mant / 4.0f;
    float e = exp_bit ? 2.0f : 1.0f;
    float val = m * e;

    return sign_bit ? -val : val;
}

/* ========================================================================== */
/* Format Properties                                                          */
/* ========================================================================== */

float gf4_max_value(void) {
    return 1.5f;
}

float gf4_min_positive(void) {
    return 0.25f;
}

float gf4_epsilon(void) {
    return 0.25f;
}

int gf4_validate_format(void) {
    return (GF4_BITS == 4) && (GF4_SIGN_BITS == 1) &&
           (GF4_EXP_BITS == 1) && (GF4_MANT_BITS == 2);
}

/* ========================================================================== */
/* Tests                                                                      */
/* ========================================================================== */

void test_gf4_decode_zero(void) {
    GF4 gf = { .raw = 0x00 };
    assert(gf4_decode(gf) == 0.0f);
    printf("PASS: gf4_decode_zero\n");
}

void test_gf4_decode_positive_max(void) {
    GF4 gf = { .raw = 0x07 };
    assert(gf4_decode(gf) == 1.5f);
    printf("PASS: gf4_decode_positive_max\n");
}

void test_gf4_encode_zero_roundtrip(void) {
    GF4 enc = gf4_encode(0.0f);
    float dec = gf4_decode(enc);
    assert(dec == 0.0f);
    printf("PASS: gf4_encode_zero_roundtrip\n");
}

void test_gf4_encode_0_25(void) {
    GF4 enc = gf4_encode(0.25f);
    float dec = gf4_decode(enc);
    assert(fabsf(dec - 0.25f) < 0.01f);
    printf("PASS: gf4_encode_0_25\n");
}

void test_gf4_encode_0_5(void) {
    GF4 enc = gf4_encode(0.5f);
    float dec = gf4_decode(enc);
    assert(fabsf(dec - 0.5f) < 0.01f);
    printf("PASS: gf4_encode_0_5\n");
}

void test_gf4_encode_1_0(void) {
    GF4 enc = gf4_encode(1.0f);
    float dec = gf4_decode(enc);
    assert(fabsf(dec - 1.0f) < 0.01f);
    printf("PASS: gf4_encode_1_0\n");
}

void test_gf4_encode_1_5(void) {
    GF4 enc = gf4_encode(1.5f);
    float dec = gf4_decode(enc);
    assert(fabsf(dec - 1.5f) < 0.01f);
    printf("PASS: gf4_encode_1_5\n");
}

void test_gf4_encode_negative(void) {
    GF4 enc = gf4_encode(-0.5f);
    float dec = gf4_decode(enc);
    assert(dec < 0.0f);
    assert(fabsf(dec - (-0.5f)) < 0.01f);
    printf("PASS: gf4_encode_negative\n");
}

void test_gf4_bits_sum(void) {
    assert(GF4_SIGN_BITS + GF4_EXP_BITS + GF4_MANT_BITS == GF4_BITS);
    printf("PASS: gf4_bits_sum\n");
}

void test_gf4_validate_format(void) {
    assert(gf4_validate_format());
    printf("PASS: gf4_validate_format\n");
}
