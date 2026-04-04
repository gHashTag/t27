/* Auto-generated from specs/numeric/gf8.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf8.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "gf8.h"
#include <assert.h>
#include <stdio.h>

/* ========================================================================== */
/* Helper Functions                                                           */
/* ========================================================================== */

int8_t gf8_floor_log2(float x) {
    if (x <= 0.0f) return -128;
    int8_t e = 0;
    while (x >= 2.0f) { x /= 2.0f; e++; }
    while (x < 1.0f)  { x *= 2.0f; e--; }
    return e;
}

uint8_t gf8_extract_mantissa(float value, int8_t exp, uint8_t mant_bits) {
    float scale = powf(2.0f, (float)exp);
    float normalized = value / scale;
    float frac = normalized - 1.0f;
    uint8_t max_mant = (1 << mant_bits) - 1;
    float val = frac * ((float)max_mant + 1.0f);
    if (val < 0.0f) return 0;
    if (val > (float)max_mant) return max_mant;
    return (uint8_t)val;
}

uint8_t gf8_clamp(uint8_t x, uint8_t min_val, uint8_t max_val) {
    if (x < min_val) return min_val;
    if (x > max_val) return max_val;
    return x;
}

float gf8_pow(float base, float exp) {
    return powf(base, exp);
}

float gf8_ln_approx(float x) {
    if (x <= 0.0f) return NAN;
    if (x == 1.0f) return 0.0f;
    return logf(x);
}

float gf8_exp_approx(float x) {
    if (x == 0.0f) return 1.0f;
    return expf(x);
}

float gf8_floor(float x) {
    return floorf(x);
}

/* ========================================================================== */
/* Encoding / Decoding                                                        */
/* ========================================================================== */

GF8 gf8_encode(float value) {
    GF8 result;
    if (value == 0.0f) { result.raw = 0; return result; }

    uint8_t sign = (value < 0.0f) ? 1 : 0;
    float abs_val = (value < 0.0f) ? -value : value;

    int8_t exp_unbiased = gf8_floor_log2(abs_val);
    int16_t biased = (int16_t)exp_unbiased + GF8_EXP_BIAS;
    uint8_t exp_biased = (biased < 0) ? 0 : ((biased > 7) ? 7 : (uint8_t)biased);

    uint8_t mant = gf8_extract_mantissa(abs_val, exp_unbiased, GF8_MANT_BITS);

    result.raw = (sign << 7) | (exp_biased << GF8_MANT_BITS) | mant;
    return result;
}

float gf8_decode(GF8 gf) {
    uint8_t sign = gf.raw >> 7;
    uint8_t exp_biased = (gf.raw >> GF8_MANT_BITS) & 0x07;
    uint8_t mant = gf.raw & 0x0F;

    if (exp_biased == 0 && mant == 0) return 0.0f;

    int8_t exp_unbiased;
    float mant_normalized;

    if (exp_biased == 0) {
        exp_unbiased = -(int8_t)GF8_EXP_BIAS + 1;
        mant_normalized = (float)mant / 16.0f;
    } else {
        exp_unbiased = (int8_t)exp_biased - (int8_t)GF8_EXP_BIAS;
        mant_normalized = 1.0f + (float)mant / 16.0f;
    }

    float val = mant_normalized * powf(2.0f, (float)exp_unbiased);
    return sign ? -val : val;
}

/* ========================================================================== */
/* Format Properties                                                          */
/* ========================================================================== */

float gf8_max_value(void) {
    float mant_max = 1.0f + 15.0f / 16.0f;
    int8_t exp_max = 7 - (int8_t)GF8_EXP_BIAS;
    return mant_max * powf(2.0f, (float)exp_max);
}

float gf8_min_positive(void) {
    float mant_min = 1.0f / 16.0f;
    int8_t exp_min = -(int8_t)GF8_EXP_BIAS + 1;
    return mant_min * powf(2.0f, (float)exp_min);
}

float gf8_epsilon(void) {
    return 1.0f / 16.0f;
}

int gf8_validate_format(void) {
    return (GF8_BITS == 8) && (GF8_SIGN_BITS == 1) &&
           (GF8_EXP_BITS == 3) && (GF8_MANT_BITS == 4);
}

/* ========================================================================== */
/* Tests                                                                      */
/* ========================================================================== */

void test_gf8_decode_zero(void) {
    GF8 gf = { .raw = 0 };
    assert(gf8_decode(gf) == 0.0f);
    printf("PASS: gf8_decode_zero\n");
}

void test_gf8_encode_zero_roundtrip(void) {
    GF8 enc = gf8_encode(0.0f);
    float dec = gf8_decode(enc);
    assert(dec == 0.0f);
    printf("PASS: gf8_encode_zero_roundtrip\n");
}

void test_gf8_bits_sum(void) {
    assert(GF8_SIGN_BITS + GF8_EXP_BITS + GF8_MANT_BITS == GF8_BITS);
    printf("PASS: gf8_bits_sum\n");
}

void test_gf8_max_value_positive(void) {
    assert(gf8_max_value() > 0.0f);
    printf("PASS: gf8_max_value_positive\n");
}

void test_gf8_validate_format(void) {
    assert(gf8_validate_format());
    printf("PASS: gf8_validate_format\n");
}

void test_gf8_pow_zero_exponent(void) {
    assert(fabsf(gf8_pow(2.0f, 0.0f) - 1.0f) < 1e-6f);
    printf("PASS: gf8_pow_zero_exponent\n");
}

void test_gf8_ln_approx_one(void) {
    assert(fabsf(gf8_ln_approx(1.0f)) < 1e-6f);
    printf("PASS: gf8_ln_approx_one\n");
}

void test_gf8_floor_positive(void) {
    assert(fabsf(gf8_floor(3.7f) - 3.0f) < 1e-6f);
    printf("PASS: gf8_floor_positive\n");
}
