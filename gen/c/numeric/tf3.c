/* Auto-generated from specs/numeric/tf3.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/tf3.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "tf3.h"
#include <assert.h>
#include <stdio.h>

/* ========================================================================== */
/* Field Extraction                                                           */
/* ========================================================================== */

int8_t tf3_extract_sign(TF3 tf3) {
    return (int8_t)((tf3 & TF3_SIGN_MASK) >> TF3_SIGN_SHIFT);
}

int8_t tf3_extract_exponent(TF3 tf3) {
    return (int8_t)((tf3 & TF3_EXP_MASK) >> TF3_EXP_SHIFT);
}

int8_t tf3_extract_mantissa(TF3 tf3) {
    return (int8_t)(tf3 & TF3_MANT_MASK);
}

TF3 tf3_from_components(int8_t sign, int8_t exp, int8_t mant) {
    return ((uint8_t)sign << TF3_SIGN_SHIFT) |
           ((uint8_t)exp << TF3_EXP_SHIFT) |
           ((uint8_t)mant << TF3_MANT_SHIFT);
}

/* ========================================================================== */
/* Classification                                                             */
/* ========================================================================== */

int tf3_is_zero(TF3 tf3) {
    return (tf3 == TF3_ZERO_POS) || (tf3 == TF3_ZERO_NEG);
}

int tf3_is_inf(TF3 tf3) {
    int8_t exp = tf3_extract_exponent(tf3);
    int8_t mant = tf3_extract_mantissa(tf3);
    return (exp == TF3_EXP_MAX) && (mant == 0);
}

int tf3_is_negative(TF3 tf3) {
    return (tf3_extract_sign(tf3) != 0) && !tf3_is_zero(tf3);
}

int tf3_is_positive(TF3 tf3) {
    return (tf3_extract_sign(tf3) == 0) && !tf3_is_zero(tf3);
}

/* ========================================================================== */
/* Encoding / Decoding                                                        */
/* ========================================================================== */

TF3 tf3_from_f32(float value) {
    if (value == 0.0f) {
        return (value < 0.0f) ? TF3_ZERO_NEG : TF3_ZERO_POS;
    }

    int8_t sign = (value < 0.0f) ? 1 : 0;
    float abs_value = (value < 0.0f) ? -value : value;
    float clamped = (abs_value < 8.0f) ? abs_value : 8.0f;

    int8_t exp = 0;
    float scaled = clamped;
    while (scaled >= 2.0f && exp < 7) {
        scaled /= 2.0f;
        exp++;
    }

    float mantissa_raw = (scaled - 1.0f) * 16.0f;
    if (mantissa_raw < 0.0f) mantissa_raw = 0.0f;
    if (mantissa_raw > 15.0f) mantissa_raw = 15.0f;
    int8_t mant = (int8_t)roundf(mantissa_raw);

    return tf3_from_components(sign, exp + TF3_BIAS, mant);
}

float tf3_to_f32(TF3 tf3) {
    if (tf3_is_zero(tf3)) {
        return (tf3_extract_sign(tf3) != 0) ? -0.0f : 0.0f;
    }
    if (tf3_is_inf(tf3)) {
        return (tf3_extract_sign(tf3) != 0) ? -INFINITY : INFINITY;
    }

    int8_t sign = tf3_extract_sign(tf3);
    int8_t exp = tf3_extract_exponent(tf3);
    int8_t mant = tf3_extract_mantissa(tf3);

    float sign_mult = (sign != 0) ? -1.0f : 1.0f;
    float mant_mult = 1.0f + (float)mant / 16.0f;
    float exp_mult = powf(2.0f, (float)(exp - TF3_BIAS));

    return sign_mult * mant_mult * exp_mult;
}

/* ========================================================================== */
/* Unary Operations                                                           */
/* ========================================================================== */

TF3 tf3_negate(TF3 tf3) {
    return tf3 ^ TF3_SIGN_MASK;
}

TF3 tf3_abs_val(TF3 tf3) {
    return tf3 & ~TF3_SIGN_MASK;
}

/* ========================================================================== */
/* Arithmetic                                                                 */
/* ========================================================================== */

TF3 tf3_add(TF3 a, TF3 b) {
    if (tf3_is_inf(a)) return a;
    if (tf3_is_inf(b)) return b;
    return tf3_from_f32(tf3_to_f32(a) + tf3_to_f32(b));
}

TF3 tf3_sub(TF3 a, TF3 b) {
    if (tf3_is_inf(a)) return a;
    if (tf3_is_inf(b)) return tf3_is_negative(b) ? TF3_INF_POS : TF3_INF_NEG;
    return tf3_from_f32(tf3_to_f32(a) - tf3_to_f32(b));
}

TF3 tf3_mul(TF3 a, TF3 b) {
    if (tf3_is_zero(a) || tf3_is_zero(b)) {
        int8_t rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return (rs != 0) ? TF3_ZERO_NEG : TF3_ZERO_POS;
    }
    if (tf3_is_inf(a) || tf3_is_inf(b)) {
        int8_t rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return (rs != 0) ? TF3_INF_NEG : TF3_INF_POS;
    }
    return tf3_from_f32(tf3_to_f32(a) * tf3_to_f32(b));
}

TF3 tf3_div(TF3 a, TF3 b) {
    if (tf3_is_zero(b)) {
        return (tf3_extract_sign(a) != 0) ? TF3_INF_NEG : TF3_INF_POS;
    }
    if (tf3_is_inf(a)) {
        int8_t rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return (rs != 0) ? TF3_INF_NEG : TF3_INF_POS;
    }
    if (tf3_is_inf(b)) {
        int8_t rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return (rs != 0) ? TF3_ZERO_NEG : TF3_ZERO_POS;
    }
    return tf3_from_f32(tf3_to_f32(a) / tf3_to_f32(b));
}

/* ========================================================================== */
/* Comparison                                                                 */
/* ========================================================================== */

int tf3_eq(TF3 a, TF3 b) {
    if (tf3_is_zero(a) && tf3_is_zero(b)) return 1;
    return a == b;
}

int tf3_lt(TF3 a, TF3 b) {
    return tf3_to_f32(a) < tf3_to_f32(b);
}

TF3 tf3_max(TF3 a, TF3 b) {
    return (tf3_to_f32(a) >= tf3_to_f32(b)) ? a : b;
}

TF3 tf3_min(TF3 a, TF3 b) {
    return (tf3_to_f32(a) <= tf3_to_f32(b)) ? a : b;
}

/* ========================================================================== */
/* Format Properties                                                          */
/* ========================================================================== */

int tf3_validate_format(void) {
    return (TF3_SIGN_MASK | TF3_EXP_MASK | TF3_MANT_MASK) == 0xFF;
}

/* ========================================================================== */
/* Tests                                                                      */
/* ========================================================================== */

void test_tf3_is_zero(void) {
    assert(tf3_is_zero(TF3_ZERO_POS));
    assert(tf3_is_zero(TF3_ZERO_NEG));
    assert(!tf3_is_zero(0x01));
    printf("PASS: tf3_is_zero\n");
}

void test_tf3_inf_encoding(void) {
    assert(tf3_extract_exponent(TF3_INF_POS) == TF3_EXP_MAX);
    assert(tf3_extract_mantissa(TF3_INF_POS) == 0);
    printf("PASS: tf3_inf_encoding\n");
}

void test_tf3_zero_roundtrip(void) {
    TF3 enc = tf3_from_f32(0.0f);
    float dec = tf3_to_f32(enc);
    assert(fabsf(dec) < 0.001f);
    printf("PASS: tf3_zero_roundtrip\n");
}

void test_tf3_positive_roundtrip(void) {
    TF3 enc = tf3_from_f32(1.5f);
    float dec = tf3_to_f32(enc);
    assert(fabsf(dec - 1.5f) < 0.5f);
    printf("PASS: tf3_positive_roundtrip\n");
}

void test_tf3_negate(void) {
    TF3 pos = tf3_from_f32(1.5f);
    TF3 neg = tf3_negate(pos);
    assert(tf3_to_f32(neg) < 0.0f);
    printf("PASS: tf3_negate\n");
}

void test_tf3_validate_format(void) {
    assert(tf3_validate_format());
    printf("PASS: tf3_validate_format\n");
}
