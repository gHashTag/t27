/* Auto-generated from specs/numeric/gf12.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf12.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "gf12.h"
#include <assert.h>
#include <stdio.h>

/* ========================================================================== */
/* Helper Functions                                                           */
/* ========================================================================== */

int8_t gf12_floor_log2(float x) {
    if (x <= 0.0f) return -128;
    int8_t e = 0;
    while (x >= 2.0f) { x /= 2.0f; e++; }
    while (x < 1.0f)  { x *= 2.0f; e--; }
    return e;
}

uint8_t gf12_extract_mantissa(float value, int8_t exp, uint8_t mant_bits) {
    float scale = powf(2.0f, (float)exp);
    float normalized = value / scale;
    float frac = normalized - 1.0f;
    uint8_t max_mant = (1 << mant_bits) - 1;
    float val = frac * ((float)max_mant + 1.0f);
    if (val < 0.0f) return 0;
    if (val > (float)max_mant) return max_mant;
    return (uint8_t)val;
}

float gf12_pow(float base, float exp) {
    return powf(base, exp);
}

float gf12_ln_approx(float x) {
    if (x <= 0.0f) return NAN;
    if (x == 1.0f) return 0.0f;
    return logf(x);
}

float gf12_exp_approx(float x) {
    if (x == 0.0f) return 1.0f;
    return expf(x);
}

float gf12_floor(float x) {
    return floorf(x);
}

/* ========================================================================== */
/* Encoding / Decoding                                                        */
/* ========================================================================== */

GF12 gf12_encode(float value) {
    GF12 result;
    if (value == 0.0f) { result.raw = 0; return result; }

    uint16_t sign = (value < 0.0f) ? 1 : 0;
    float abs_val = (value < 0.0f) ? -value : value;

    int8_t exp_unbiased = gf12_floor_log2(abs_val);
    int16_t biased = (int16_t)exp_unbiased + GF12_EXP_BIAS;
    uint8_t exp_biased = (biased < 0) ? 0 : ((biased > 15) ? 15 : (uint8_t)biased);

    uint8_t mant = gf12_extract_mantissa(abs_val, exp_unbiased, GF12_MANT_BITS);

    result.raw = (sign << 11) | ((uint16_t)exp_biased << GF12_MANT_BITS) | (uint16_t)mant;
    return result;
}

float gf12_decode(GF12 gf) {
    uint8_t sign = (uint8_t)(gf.raw >> 11);
    uint8_t exp_biased = (uint8_t)((gf.raw >> GF12_MANT_BITS) & 0x0F);
    uint8_t mant = (uint8_t)(gf.raw & 0x7F);

    if (exp_biased == 0 && mant == 0) return 0.0f;

    int8_t exp_unbiased;
    float mant_normalized;

    if (exp_biased == 0) {
        exp_unbiased = -(int8_t)GF12_EXP_BIAS + 1;
        mant_normalized = (float)mant / 128.0f;
    } else {
        exp_unbiased = (int8_t)exp_biased - (int8_t)GF12_EXP_BIAS;
        mant_normalized = 1.0f + (float)mant / 128.0f;
    }

    float val = mant_normalized * powf(2.0f, (float)exp_unbiased);
    return sign ? -val : val;
}

/* ========================================================================== */
/* Format Properties                                                          */
/* ========================================================================== */

float gf12_max_value(void) {
    float mant_max = 1.0f + 127.0f / 128.0f;
    int8_t exp_max = 15 - (int8_t)GF12_EXP_BIAS;
    return mant_max * powf(2.0f, (float)exp_max);
}

float gf12_min_positive(void) {
    float mant_min = 1.0f / 128.0f;
    int8_t exp_min = -(int8_t)GF12_EXP_BIAS + 1;
    return mant_min * powf(2.0f, (float)exp_min);
}

float gf12_epsilon(void) {
    return 1.0f / 128.0f;
}

int gf12_validate_format(void) {
    return (GF12_BITS == 12) && (GF12_SIGN_BITS == 1) &&
           (GF12_EXP_BITS == 4) && (GF12_MANT_BITS == 7);
}

/* ========================================================================== */
/* Tests                                                                      */
/* ========================================================================== */

void test_gf12_decode_zero(void) {
    GF12 gf = { .raw = 0 };
    assert(gf12_decode(gf) == 0.0f);
    printf("PASS: gf12_decode_zero\n");
}

void test_gf12_encode_zero_roundtrip(void) {
    GF12 enc = gf12_encode(0.0f);
    float dec = gf12_decode(enc);
    assert(dec == 0.0f);
    printf("PASS: gf12_encode_zero_roundtrip\n");
}

void test_gf12_bits_sum(void) {
    assert(GF12_SIGN_BITS + GF12_EXP_BITS + GF12_MANT_BITS == GF12_BITS);
    printf("PASS: gf12_bits_sum\n");
}

void test_gf12_max_value_positive(void) {
    assert(gf12_max_value() > 0.0f);
    printf("PASS: gf12_max_value_positive\n");
}

void test_gf12_validate_format(void) {
    assert(gf12_validate_format());
    printf("PASS: gf12_validate_format\n");
}

void test_gf12_floor_log2_power_of_two(void) {
    assert(gf12_floor_log2(8.0f) == 3);
    printf("PASS: gf12_floor_log2_power_of_two\n");
}
