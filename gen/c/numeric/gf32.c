/* Auto-generated from specs/numeric/gf32.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf32.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "gf32.h"
#include <math.h>
#include <assert.h>
#include <stdio.h>

/* ===================================================================== */
/* Helper Functions                                                       */
/* ===================================================================== */

static int32_t gf32_clamp_i32(int32_t x, int32_t min_val, int32_t max_val) {
    if (x < min_val) return min_val;
    if (x > max_val) return max_val;
    return x;
}

int16_t gf32_floor_log2(float x) {
    if (x <= 0.0f) return -32768;
    int16_t e = 0;
    while (x >= 2.0f) {
        x /= 2.0f;
        e++;
    }
    while (x < 1.0f) {
        x *= 2.0f;
        e--;
    }
    return e;
}

static uint32_t gf32_extract_mantissa(float value, int16_t exp_val, uint8_t mant_bits) {
    float normalized = value / powf(2.0f, (float)exp_val);
    float frac = normalized - 1.0f;
    uint32_t max_mant = (1u << mant_bits) - 1;
    return (uint32_t)(frac * (float)(max_mant + 1));
}

float gf32_pow(float base, float exponent) {
    if (exponent == 0.0f) return 1.0f;
    if (base == 0.0f && exponent > 0.0f) return 0.0f;
    if (base <= 0.0f) return NAN;

    float fl = floorf(exponent);
    if (exponent == fl) {
        /* Integer exponent: binary exponentiation */
        int32_t e = (int32_t)exponent;
        float result = 1.0f;
        float base_acc = base;
        if (e < 0) {
            e = -e;
            base_acc = 1.0f / base_acc;
        }
        while (e > 0) {
            if (e % 2 == 1) {
                result *= base_acc;
            }
            base_acc *= base_acc;
            e /= 2;
        }
        return result;
    }

    /* Fractional exponent: x^y = exp(y * ln(x)) */
    float ln_val = gf32_ln_approx(base);
    return gf32_exp_approx(exponent * ln_val);
}

float gf32_ln_approx(float x) {
    if (x <= 0.0f) return NAN;
    if (x == 1.0f) return 0.0f;

    float t = (x - 1.0f) / (x + 1.0f);
    float t2 = t * t;
    float t3 = t2 * t;
    float t5 = t3 * t2;
    float t7 = t5 * t2;

    return 2.0f * (t + t3 / 3.0f + t5 / 5.0f + t7 / 7.0f);
}

float gf32_exp_approx(float x) {
    if (x == 0.0f) return 1.0f;

    float exp_x = x;
    int32_t k = 0;

    if (exp_x > 5.0f || exp_x < -5.0f) {
        k = (int32_t)floorf(exp_x / 5.0f);
        exp_x = exp_x - (float)k * 5.0f;
    }

    float result = 1.0f;
    float term = 1.0f;
    for (int i = 1; i <= 8; i++) {
        term = term * exp_x / (float)i;
        result += term;
    }

    if (k > 0) {
        float e5 = gf32_exp_approx(5.0f);
        for (int i = 0; i < k; i++) {
            result *= e5;
        }
    } else if (k < 0) {
        float e5 = gf32_exp_approx(5.0f);
        for (int i = k; i < 0; i++) {
            result /= e5;
        }
    }

    return result;
}

float gf32_floor(float x) {
    return floorf(x);
}

/* ===================================================================== */
/* Encoding / Decoding                                                    */
/* ===================================================================== */

GF32 gf32_encode(float value) {
    GF32 gf = { 0 };

    if (value == 0.0f) {
        return gf;
    }

    uint32_t sign = (value < 0.0f) ? 1u : 0u;
    float abs_val = (value < 0.0f) ? -value : value;

    /* Extract exponent (unbiased) */
    int16_t exp_unbiased = gf32_floor_log2(abs_val);
    int32_t exp_biased = (int32_t)exp_unbiased + (int32_t)GF32_EXP_BIAS;

    /* Clamp exponent */
    int32_t exp_clamped = gf32_clamp_i32(exp_biased, 0, (int32_t)GF32_EXP_MASK);

    /* Extract mantissa (19 bits) */
    uint32_t mant = gf32_extract_mantissa(abs_val, exp_unbiased, GF32_MANT_BITS);

    gf.raw = (sign << 31) |
             ((uint32_t)exp_clamped << GF32_MANT_BITS) |
             (mant & GF32_MANT_MASK);

    return gf;
}

float gf32_decode(GF32 gf) {
    uint8_t sign = (uint8_t)((gf.raw >> 31) & 1);
    uint16_t exp_biased = (uint16_t)((gf.raw >> GF32_MANT_BITS) & GF32_EXP_MASK);
    uint32_t mant = gf.raw & GF32_MANT_MASK;

    /* Zero */
    if (exp_biased == 0 && mant == 0) {
        return 0.0f;
    }

    /* Exponent */
    int16_t exp_unbiased;
    if (exp_biased == 0) {
        exp_unbiased = -(int16_t)GF32_EXP_BIAS + 1;
    } else {
        exp_unbiased = (int16_t)exp_biased - (int16_t)GF32_EXP_BIAS;
    }

    /* Mantissa */
    float mant_normalized;
    if (exp_biased == 0) {
        mant_normalized = (float)mant / GF32_MANT_SCALE;
    } else {
        mant_normalized = 1.0f + (float)mant / GF32_MANT_SCALE;
    }

    float result = mant_normalized * powf(2.0f, (float)exp_unbiased);

    if (sign != 0) {
        return -result;
    }
    return result;
}

/* ===================================================================== */
/* Format Properties                                                      */
/* ===================================================================== */

float gf32_max_value(void) {
    float mant_max = 1.0f + 524287.0f / GF32_MANT_SCALE;
    int16_t exp_max = (int16_t)GF32_EXP_MASK - (int16_t)GF32_EXP_BIAS;
    return mant_max * powf(2.0f, (float)exp_max);
}

float gf32_min_positive(void) {
    float mant_min = 1.0f / GF32_MANT_SCALE;
    int16_t exp_min = -(int16_t)GF32_EXP_BIAS + 1;
    return mant_min * powf(2.0f, (float)exp_min);
}

float gf32_epsilon(void) {
    return 1.0f / GF32_MANT_SCALE; /* 0.000001907348633 */
}

bool gf32_validate_format(void) {
    return (GF32_BITS == GF32_SIGN_BITS + GF32_EXP_BITS + GF32_MANT_BITS) &&
           (GF32_SIGN_BITS == 1) &&
           (GF32_EXP_BITS == 12) &&
           (GF32_MANT_BITS == 19);
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_gf32(void) {
    /* test gf32_decode_zero */
    {
        GF32 gf = { .raw = 0 };
        float value = gf32_decode(gf);
        assert(value == 0.0f);
    }

    /* test gf32_encode_zero_roundtrip */
    {
        float original = 0.0f;
        GF32 encoded = gf32_encode(original);
        float decoded = gf32_decode(encoded);
        assert(decoded == original);
    }

    /* test gf32_bits_sum_correct */
    {
        int total = GF32_SIGN_BITS + GF32_EXP_BITS + GF32_MANT_BITS;
        assert(total == GF32_BITS);
    }

    /* test gf32_max_value_positive */
    {
        float max_val = gf32_max_value();
        assert(max_val > 0.0f);
    }

    /* test gf32_min_positive_greater_than_zero */
    {
        float min_pos = gf32_min_positive();
        assert(min_pos > 0.0f);
    }

    /* test gf32_epsilon_positive */
    {
        float eps = gf32_epsilon();
        assert(eps > 0.0f);
    }

    /* test gf32_phi_distance_near_optimal */
    {
        assert(GF32_PHI_DISTANCE < 0.015);
    }

    /* test gf32_memory_ratio_equals_one */
    {
        assert(GF32_MEMORY_RATIO == 1.0f);
    }

    /* test gf32_validate_format_success */
    {
        assert(gf32_validate_format());
    }

    /* test gf32_pow_zero_exponent_returns_one */
    {
        float result = gf32_pow(2.0f, 0.0f);
        assert(fabsf(result - 1.0f) < 1e-6f);
    }

    /* test gf32_pow_one_exponent_returns_base */
    {
        float result = gf32_pow(5.0f, 1.0f);
        assert(fabsf(result - 5.0f) < 1e-6f);
    }

    /* test gf32_pow_positive_integer_exponent */
    {
        float result = gf32_pow(2.0f, 5.0f);
        assert(fabsf(result - 32.0f) < 1e-5f);
    }

    /* test gf32_pow_negative_integer_exponent */
    {
        float result = gf32_pow(2.0f, -3.0f);
        assert(fabsf(result - 0.125f) < 1e-5f);
    }

    /* test gf32_pow_fractional_exponent */
    {
        float result = gf32_pow(4.0f, 0.5f);
        assert(fabsf(result - 2.0f) < 1e-4f);
    }

    /* test gf32_pow_zero_base_positive_exponent */
    {
        float result = gf32_pow(0.0f, 5.0f);
        assert(result == 0.0f);
    }

    /* test gf32_pow_one_base_any_exponent */
    {
        float r1 = gf32_pow(1.0f, 10.0f);
        float r2 = gf32_pow(1.0f, -5.0f);
        assert(fabsf(r1 - 1.0f) < 1e-6f);
        assert(fabsf(r2 - 1.0f) < 1e-6f);
    }

    /* test gf32_ln_approx_of_one */
    {
        float result = gf32_ln_approx(1.0f);
        assert(fabsf(result) < 1e-6f);
    }

    /* test gf32_ln_approx_of_e */
    {
        float e = 2.718281828459045f;
        float result = gf32_ln_approx(e);
        assert(fabsf(result - 1.0f) < 0.01f);
    }

    /* test gf32_ln_approx_negative_returns_nan */
    {
        float result = gf32_ln_approx(-1.0f);
        assert(result != result); /* NaN check */
    }

    /* test gf32_exp_approx_zero */
    {
        float result = gf32_exp_approx(0.0f);
        assert(fabsf(result - 1.0f) < 1e-6f);
    }

    /* test gf32_exp_approx_one */
    {
        float e = 2.718281828459045f;
        float result = gf32_exp_approx(1.0f);
        assert(fabsf(result - e) < 0.01f);
    }

    /* test gf32_exp_approx_negative */
    {
        float result = gf32_exp_approx(-1.0f);
        float expected = 1.0f / 2.718281828459045f;
        assert(fabsf(result - expected) < 0.01f);
    }

    /* test gf32_floor_positive */
    {
        float result = gf32_floor(3.7f);
        assert(fabsf(result - 3.0f) < 1e-6f);
    }

    /* test gf32_floor_negative */
    {
        float result = gf32_floor(-3.2f);
        assert(fabsf(result - (-4.0f)) < 1e-6f);
    }

    /* test gf32_floor_integer */
    {
        float result = gf32_floor(5.0f);
        assert(fabsf(result - 5.0f) < 1e-6f);
    }

    /* Invariant: max >= min_positive */
    {
        assert(gf32_max_value() >= gf32_min_positive());
    }

    /* Invariant: exp wider than IEEE */
    {
        assert(GF32_EXP_BITS > 8);
    }

    /* Invariant: mant narrower than IEEE */
    {
        assert(GF32_MANT_BITS < 23);
    }

    /* Invariant: ln/exp inversion */
    {
        float x = 2.0f;
        float y = gf32_ln_approx(x);
        assert(fabsf(gf32_exp_approx(y) - x) < 0.01f);
    }

    /* Invariant: floor monotonic */
    {
        assert(gf32_floor(2.5f) <= gf32_floor(3.5f));
    }

    printf("GF32: all tests passed\n");
}
