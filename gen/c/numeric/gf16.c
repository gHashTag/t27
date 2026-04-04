/* Auto-generated from specs/numeric/gf16.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf16.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "gf16.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

/* ========================================================================== */
/* Field Extraction                                                           */
/* ========================================================================== */

int8_t gf16_extract_sign(GF16 gf16) {
    return ((gf16 >> GF16_SIGN_SHIFT) & 1) ? -1 : 0;
}

int8_t gf16_extract_exponent(GF16 gf16) {
    return (int8_t)((gf16 >> GF16_EXP_SHIFT) & 0x3F);
}

int16_t gf16_extract_mantissa(GF16 gf16) {
    return (int16_t)(gf16 & GF16_MANT_MASK);
}

GF16 gf16_from_components(int8_t sign, int8_t exp, int16_t mant) {
    uint16_t sign_bit = (sign < 0) ? 1 : 0;
    return (sign_bit << GF16_SIGN_SHIFT) |
           ((uint16_t)(uint8_t)exp << GF16_EXP_SHIFT) |
           (uint16_t)(mant & GF16_MANT_MASK);
}

/* ========================================================================== */
/* Classification                                                             */
/* ========================================================================== */

int gf16_is_zero(GF16 gf16) {
    return (gf16 == GF16_ZERO_POS) || (gf16 == GF16_ZERO_NEG);
}

int gf16_is_special(GF16 gf16) {
    return gf16_extract_exponent(gf16) == GF16_EXP_MAX;
}

int gf16_is_inf(GF16 gf16) {
    return (gf16_extract_exponent(gf16) == GF16_EXP_MAX) &&
           (gf16_extract_mantissa(gf16) == 0);
}

int gf16_is_nan(GF16 gf16) {
    return (gf16_extract_exponent(gf16) == GF16_EXP_MAX) &&
           (gf16_extract_mantissa(gf16) != 0);
}

int gf16_is_negative(GF16 gf16) {
    return (gf16_extract_sign(gf16) < 0) && !gf16_is_zero(gf16);
}

int gf16_is_positive(GF16 gf16) {
    return (gf16_extract_sign(gf16) >= 0) && !gf16_is_zero(gf16);
}

int gf16_is_finite(GF16 gf16) {
    return !gf16_is_nan(gf16) && !gf16_is_inf(gf16);
}

/* ========================================================================== */
/* Encoding / Decoding                                                        */
/* ========================================================================== */

GF16 gf16_encode_f32(float value) {
    if (value == 0.0f) {
        uint32_t bits;
        memcpy(&bits, &value, sizeof(bits));
        return (bits >> 31) ? GF16_ZERO_NEG : GF16_ZERO_POS;
    }

    int8_t sign = (value < 0.0f) ? -1 : 0;
    float abs_value = (value < 0.0f) ? -value : value;

    uint32_t f32_bits;
    memcpy(&f32_bits, &abs_value, sizeof(f32_bits));
    int16_t f32_exp = (int16_t)((f32_bits >> 23) & 0xFF) - 127;
    uint32_t f32_mant = f32_bits & 0x7FFFFF;

    int16_t gf16_exp = f32_exp + GF16_BIAS;
    if (gf16_exp < 0) gf16_exp = 0;
    if (gf16_exp > GF16_EXP_MAX) gf16_exp = GF16_EXP_MAX;

    uint16_t mant = (uint16_t)(f32_mant >> 14);

    /* Round-to-nearest */
    uint32_t discarded = f32_mant & 0x3FFF;
    if (discarded & 0x2000) {
        mant++;
        if (mant > GF16_MANT_MASK) {
            mant = 0;
            if (gf16_exp < GF16_EXP_MAX) gf16_exp++;
        }
    }

    return gf16_from_components(sign, (int8_t)gf16_exp, (int16_t)mant);
}

float gf16_decode_to_f32(GF16 gf16) {
    if (gf16_is_zero(gf16)) {
        return (gf16_extract_sign(gf16) < 0) ? -0.0f : 0.0f;
    }

    if (gf16_is_special(gf16)) {
        if (gf16_extract_mantissa(gf16) == 0) {
            return (gf16_extract_sign(gf16) < 0) ? -INFINITY : INFINITY;
        }
        return NAN;
    }

    int8_t sign = gf16_extract_sign(gf16);
    int8_t exp = gf16_extract_exponent(gf16);
    int16_t mant = gf16_extract_mantissa(gf16);

    float sign_mult = (sign < 0) ? -1.0f : 1.0f;
    float mant_mult = 1.0f + (float)mant / 512.0f;
    float exp_mult = powf(2.0f, (float)(exp - GF16_BIAS));

    return sign_mult * mant_mult * exp_mult;
}

GF16 gf16_round_phi(float value) {
    if (value == 0.0f) {
        uint32_t bits;
        memcpy(&bits, &value, sizeof(bits));
        return (bits >> 31) ? GF16_ZERO_NEG : GF16_ZERO_POS;
    }

    int8_t sign = (value < 0.0f) ? -1 : 0;
    float abs_value = (value < 0.0f) ? -value : value;

    uint32_t f32_bits;
    memcpy(&f32_bits, &abs_value, sizeof(f32_bits));
    int16_t f32_exp = (int16_t)((f32_bits >> 23) & 0xFF) - 127;
    uint32_t f32_mant = f32_bits & 0x7FFFFF;

    int16_t gf16_exp = f32_exp + GF16_BIAS;
    if (gf16_exp < 0) gf16_exp = 0;
    if (gf16_exp > GF16_EXP_MAX) gf16_exp = GF16_EXP_MAX;

    uint32_t normalized_mant = f32_mant | 0x00800000;
    uint16_t mant = (uint16_t)((normalized_mant >> 15) + GF16_PHI_BIAS);

    if (mant > GF16_MANT_MASK) {
        mant = 0;
        if (gf16_exp < GF16_EXP_MAX) gf16_exp++;
        else gf16_exp = GF16_EXP_MAX;
    }

    return gf16_from_components(sign, (int8_t)gf16_exp, (int16_t)mant);
}

/* ========================================================================== */
/* Unary Operations                                                           */
/* ========================================================================== */

GF16 gf16_negate(GF16 gf16) {
    return gf16 ^ GF16_SIGN_MASK;
}

GF16 gf16_abs_val(GF16 gf16) {
    return gf16 & ~GF16_SIGN_MASK;
}

/* ========================================================================== */
/* Arithmetic                                                                 */
/* ========================================================================== */

GF16 gf16_add(GF16 a, GF16 b) {
    if (gf16_is_nan(a) || gf16_is_nan(b)) return GF16_NAN_VAL;
    if (gf16_is_inf(a) && gf16_is_inf(b)) {
        return (gf16_extract_sign(a) == gf16_extract_sign(b)) ? a : GF16_NAN_VAL;
    }
    if (gf16_is_inf(a)) return a;
    if (gf16_is_inf(b)) return b;

    return gf16_encode_f32(gf16_decode_to_f32(a) + gf16_decode_to_f32(b));
}

GF16 gf16_sub(GF16 a, GF16 b) {
    if (gf16_is_nan(a) || gf16_is_nan(b)) return GF16_NAN_VAL;
    if (gf16_is_inf(a) && gf16_is_inf(b)) return GF16_NAN_VAL;
    if (gf16_is_inf(a)) return a;
    if (gf16_is_inf(b)) return gf16_negate(b);

    return gf16_encode_f32(gf16_decode_to_f32(a) - gf16_decode_to_f32(b));
}

GF16 gf16_mul(GF16 a, GF16 b) {
    if (gf16_is_nan(a) || gf16_is_nan(b)) return GF16_NAN_VAL;
    if (gf16_is_zero(a) || gf16_is_zero(b)) {
        int8_t rs = gf16_extract_sign(a) ^ gf16_extract_sign(b);
        return (rs != 0) ? GF16_ZERO_NEG : GF16_ZERO_POS;
    }
    if (gf16_is_inf(a) || gf16_is_inf(b)) {
        int8_t rs = gf16_extract_sign(a) ^ gf16_extract_sign(b);
        return (rs != 0) ? GF16_INF_NEG : GF16_INF_POS;
    }

    return gf16_encode_f32(gf16_decode_to_f32(a) * gf16_decode_to_f32(b));
}

GF16 gf16_div(GF16 a, GF16 b) {
    if (gf16_is_nan(a) || gf16_is_nan(b)) return GF16_NAN_VAL;
    if (gf16_is_zero(b)) {
        return (gf16_extract_sign(a) != 0) ? GF16_INF_NEG : GF16_INF_POS;
    }
    if (gf16_is_inf(a)) {
        int8_t rs = gf16_extract_sign(a) ^ gf16_extract_sign(b);
        return (rs != 0) ? GF16_INF_NEG : GF16_INF_POS;
    }
    if (gf16_is_inf(b)) {
        int8_t rs = gf16_extract_sign(a) ^ gf16_extract_sign(b);
        return (rs != 0) ? GF16_ZERO_NEG : GF16_ZERO_POS;
    }

    return gf16_encode_f32(gf16_decode_to_f32(a) / gf16_decode_to_f32(b));
}

GF16 gf16_fma(GF16 a, GF16 b, GF16 c) {
    if (gf16_is_nan(a) || gf16_is_nan(b) || gf16_is_nan(c)) return GF16_NAN_VAL;
    if (gf16_is_zero(a) || gf16_is_zero(b)) return gf16_add(c, gf16_encode_f32(0.0f));

    float av = gf16_decode_to_f32(a);
    float bv = gf16_decode_to_f32(b);
    float cv = gf16_decode_to_f32(c);
    return gf16_encode_f32(av * bv + cv);
}

GF16 gf16_sqrt_val(GF16 a) {
    if (gf16_is_nan(a)) return GF16_NAN_VAL;
    if (gf16_is_inf(a) && !gf16_is_negative(a)) return a;
    if (gf16_is_inf(a)) return GF16_NAN_VAL;
    if (gf16_is_zero(a)) return a;
    if (gf16_is_negative(a)) return GF16_NAN_VAL;

    return gf16_encode_f32(sqrtf(gf16_decode_to_f32(a)));
}

/* ========================================================================== */
/* Comparison                                                                 */
/* ========================================================================== */

int gf16_eq(GF16 a, GF16 b) {
    if (gf16_is_nan(a) || gf16_is_nan(b)) return 0;
    if (gf16_is_zero(a) && gf16_is_zero(b)) return 1;
    return a == b;
}

int gf16_lt(GF16 a, GF16 b) {
    if (gf16_is_nan(a) || gf16_is_nan(b)) return 0;
    return gf16_decode_to_f32(a) < gf16_decode_to_f32(b);
}

int gf16_le(GF16 a, GF16 b) {
    if (gf16_is_nan(a) || gf16_is_nan(b)) return 0;
    return gf16_decode_to_f32(a) <= gf16_decode_to_f32(b);
}

GF16 gf16_max(GF16 a, GF16 b) {
    if (gf16_is_nan(a)) return b;
    if (gf16_is_nan(b)) return a;
    return (gf16_decode_to_f32(a) >= gf16_decode_to_f32(b)) ? a : b;
}

GF16 gf16_min(GF16 a, GF16 b) {
    if (gf16_is_nan(a)) return b;
    if (gf16_is_nan(b)) return a;
    return (gf16_decode_to_f32(a) <= gf16_decode_to_f32(b)) ? a : b;
}

/* ========================================================================== */
/* Rounding                                                                   */
/* ========================================================================== */

GF16 gf16_floor_val(GF16 a) {
    if (gf16_is_nan(a)) return GF16_NAN_VAL;
    if (gf16_is_inf(a) || gf16_is_zero(a)) return a;
    return gf16_encode_f32(floorf(gf16_decode_to_f32(a)));
}

GF16 gf16_ceil_val(GF16 a) {
    if (gf16_is_nan(a)) return GF16_NAN_VAL;
    if (gf16_is_inf(a) || gf16_is_zero(a)) return a;
    return gf16_encode_f32(ceilf(gf16_decode_to_f32(a)));
}

GF16 gf16_round_val(GF16 a) {
    if (gf16_is_nan(a)) return GF16_NAN_VAL;
    if (gf16_is_inf(a) || gf16_is_zero(a)) return a;
    return gf16_encode_f32(roundf(gf16_decode_to_f32(a)));
}

/* ========================================================================== */
/* Format Properties                                                          */
/* ========================================================================== */

float gf16_max_value(void) {
    float mant_max = 1.0f + 511.0f / 512.0f;
    float exp_max = (float)(GF16_EXP_MAX - 1 - GF16_BIAS);
    return mant_max * powf(2.0f, exp_max);
}

float gf16_min_positive(void) {
    float mant_min = 1.0f / 512.0f;
    float exp_min = (float)(1 - GF16_BIAS);
    return mant_min * powf(2.0f, exp_min);
}

float gf16_epsilon(void) {
    return 1.0f / 512.0f;
}

int gf16_validate_format(void) {
    return (GF16_SIGN_MASK == 0x8000) &&
           (GF16_EXP_MASK == 0x7E00) &&
           (GF16_MANT_MASK == 0x01FF);
}

/* ========================================================================== */
/* Tests                                                                      */
/* ========================================================================== */

void test_gf16_zero_encoding(void) {
    assert(gf16_is_zero(GF16_ZERO_POS));
    assert(gf16_is_zero(GF16_ZERO_NEG));
    printf("PASS: gf16_zero_encoding\n");
}

void test_gf16_encode_decode_roundtrip(void) {
    GF16 enc = gf16_encode_f32(1.0f);
    float dec = gf16_decode_to_f32(enc);
    assert(fabsf(dec - 1.0f) < 0.01f);
    printf("PASS: gf16_encode_decode_roundtrip\n");
}

void test_gf16_add_basic(void) {
    GF16 a = gf16_encode_f32(1.0f);
    GF16 b = gf16_encode_f32(2.0f);
    float result = gf16_decode_to_f32(gf16_add(a, b));
    assert(fabsf(result - 3.0f) < 0.1f);
    printf("PASS: gf16_add_basic\n");
}

void test_gf16_mul_basic(void) {
    GF16 a = gf16_encode_f32(2.0f);
    GF16 b = gf16_encode_f32(3.0f);
    float result = gf16_decode_to_f32(gf16_mul(a, b));
    assert(fabsf(result - 6.0f) < 0.1f);
    printf("PASS: gf16_mul_basic\n");
}

void test_gf16_is_nan(void) {
    assert(gf16_is_nan(GF16_NAN_VAL));
    assert(!gf16_is_nan(GF16_ZERO_POS));
    printf("PASS: gf16_is_nan\n");
}

void test_gf16_is_inf(void) {
    assert(gf16_is_inf(GF16_INF_POS));
    assert(gf16_is_inf(GF16_INF_NEG));
    assert(!gf16_is_inf(GF16_ZERO_POS));
    printf("PASS: gf16_is_inf\n");
}

void test_gf16_validate_format(void) {
    assert(gf16_validate_format());
    printf("PASS: gf16_validate_format\n");
}
