/* Auto-generated from specs/numeric/gf24.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf24.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "gf24.h"
#include <math.h>
#include <assert.h>
#include <stdio.h>

/* ===================================================================== */
/* Helper Functions                                                       */
/* ===================================================================== */

static int32_t gf24_clamp_i32(int32_t x, int32_t min_val, int32_t max_val) {
    if (x < min_val) return min_val;
    if (x > max_val) return max_val;
    return x;
}

int16_t gf24_floor_log2(float x) {
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

static uint32_t gf24_extract_mantissa(float value, int16_t exp_val, uint8_t mant_bits) {
    float normalized = value / powf(2.0f, (float)exp_val);
    float frac = normalized - 1.0f;
    uint32_t max_mant = (1u << mant_bits) - 1;
    return (uint32_t)(frac * (float)(max_mant + 1));
}

float gf24_pow(float base, float exponent) {
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
    float ln_val = gf24_ln_approx(base);
    return gf24_exp_approx(exponent * ln_val);
}

float gf24_ln_approx(float x) {
    if (x <= 0.0f) return NAN;
    if (x == 1.0f) return 0.0f;

    float t = (x - 1.0f) / (x + 1.0f);
    float t2 = t * t;
    float t3 = t2 * t;
    float t5 = t3 * t2;
    float t7 = t5 * t2;

    return 2.0f * (t + t3 / 3.0f + t5 / 5.0f + t7 / 7.0f);
}

float gf24_exp_approx(float x) {
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
        float e5 = gf24_exp_approx(5.0f);
        for (int i = 0; i < k; i++) {
            result *= e5;
        }
    } else if (k < 0) {
        float e5 = gf24_exp_approx(5.0f);
        for (int i = k; i < 0; i++) {
            result /= e5;
        }
    }

    return result;
}

float gf24_floor(float x) {
    return floorf(x);
}

/* ===================================================================== */
/* Encoding / Decoding                                                    */
/* ===================================================================== */

GF24 gf24_encode(float value) {
    GF24 gf = { 0 };

    if (value == 0.0f) {
        return gf;
    }

    uint32_t sign = (value < 0.0f) ? 1u : 0u;
    float abs_val = (value < 0.0f) ? -value : value;

    /* Extract exponent (unbiased) */
    int16_t exp_unbiased = gf24_floor_log2(abs_val);
    int32_t exp_biased = (int32_t)exp_unbiased + (int32_t)GF24_EXP_BIAS;

    /* Clamp exponent */
    int32_t exp_clamped = gf24_clamp_i32(exp_biased, 0, (int32_t)GF24_EXP_MASK);

    /* Extract mantissa (14 bits) */
    uint32_t mant = gf24_extract_mantissa(abs_val, exp_unbiased, GF24_MANT_BITS);

    gf.raw = (sign << 23) |
             ((uint32_t)exp_clamped << GF24_MANT_BITS) |
             (mant & GF24_MANT_MASK);

    return gf;
}

float gf24_decode(GF24 gf) {
    uint8_t sign = (uint8_t)((gf.raw >> 23) & 1);
    uint16_t exp_biased = (uint16_t)((gf.raw >> GF24_MANT_BITS) & GF24_EXP_MASK);
    uint32_t mant = gf.raw & GF24_MANT_MASK;

    /* Zero */
    if (exp_biased == 0 && mant == 0) {
        return 0.0f;
    }

    /* Exponent */
    int16_t exp_unbiased;
    if (exp_biased == 0) {
        exp_unbiased = -(int16_t)GF24_EXP_BIAS + 1;
    } else {
        exp_unbiased = (int16_t)exp_biased - (int16_t)GF24_EXP_BIAS;
    }

    /* Mantissa */
    float mant_normalized;
    if (exp_biased == 0) {
        mant_normalized = (float)mant / GF24_MANT_SCALE;
    } else {
        mant_normalized = 1.0f + (float)mant / GF24_MANT_SCALE;
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

float gf24_max_value(void) {
    float mant_max = 1.0f + 16383.0f / GF24_MANT_SCALE;
    int16_t exp_max = (int16_t)GF24_EXP_MASK - (int16_t)GF24_EXP_BIAS;
    return mant_max * powf(2.0f, (float)exp_max);
}

float gf24_min_positive(void) {
    float mant_min = 1.0f / GF24_MANT_SCALE;
    int16_t exp_min = -(int16_t)GF24_EXP_BIAS + 1;
    return mant_min * powf(2.0f, (float)exp_min);
}

float gf24_epsilon(void) {
    return 1.0f / GF24_MANT_SCALE; /* 0.00006103515625 */
}

bool gf24_validate_format(void) {
    return (GF24_BITS == GF24_SIGN_BITS + GF24_EXP_BITS + GF24_MANT_BITS) &&
           (GF24_SIGN_BITS == 1) &&
           (GF24_EXP_BITS == 9) &&
           (GF24_MANT_BITS == 14);
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_gf24(void) {
    /* test gf24_decode_zero */
    {
        GF24 gf = { .raw = 0 };
        float value = gf24_decode(gf);
        assert(value == 0.0f);
    }

    /* test gf24_encode_zero_roundtrip */
    {
        float original = 0.0f;
        GF24 encoded = gf24_encode(original);
        float decoded = gf24_decode(encoded);
        assert(decoded == original);
    }

    /* test gf24_bits_sum_correct */
    {
        int total = GF24_SIGN_BITS + GF24_EXP_BITS + GF24_MANT_BITS;
        assert(total == GF24_BITS);
    }

    /* test gf24_max_value_positive */
    {
        float max_val = gf24_max_value();
        assert(max_val > 0.0f);
    }

    /* test gf24_min_positive_greater_than_zero */
    {
        float min_pos = gf24_min_positive();
        assert(min_pos > 0.0f);
    }

    /* test gf24_epsilon_positive */
    {
        float eps = gf24_epsilon();
        assert(eps > 0.0f);
    }

    /* test gf24_phi_distance_within_tolerance */
    {
        assert(GF24_PHI_DISTANCE < 0.03);
    }

    /* test gf24_memory_ratio_vs_fp32 */
    {
        assert(fabsf(GF24_MEMORY_RATIO - 0.75f) < 0.01f);
    }

    /* test gf24_validate_format_success */
    {
        assert(gf24_validate_format());
    }

    /* test gf24_pow_zero_exponent_returns_one */
    {
        float result = gf24_pow(2.0f, 0.0f);
        assert(fabsf(result - 1.0f) < 1e-6f);
    }

    /* test gf24_pow_one_exponent_returns_base */
    {
        float result = gf24_pow(5.0f, 1.0f);
        assert(fabsf(result - 5.0f) < 1e-6f);
    }

    /* test gf24_pow_positive_integer_exponent */
    {
        float result = gf24_pow(2.0f, 5.0f);
        assert(fabsf(result - 32.0f) < 1e-5f);
    }

    /* test gf24_pow_negative_integer_exponent */
    {
        float result = gf24_pow(2.0f, -3.0f);
        assert(fabsf(result - 0.125f) < 1e-5f);
    }

    /* test gf24_pow_fractional_exponent */
    {
        float result = gf24_pow(4.0f, 0.5f);
        assert(fabsf(result - 2.0f) < 1e-4f);
    }

    /* test gf24_pow_zero_base_positive_exponent */
    {
        float result = gf24_pow(0.0f, 5.0f);
        assert(result == 0.0f);
    }

    /* test gf24_pow_one_base_any_exponent */
    {
        float r1 = gf24_pow(1.0f, 10.0f);
        float r2 = gf24_pow(1.0f, -5.0f);
        assert(fabsf(r1 - 1.0f) < 1e-6f);
        assert(fabsf(r2 - 1.0f) < 1e-6f);
    }

    /* test gf24_ln_approx_of_one */
    {
        float result = gf24_ln_approx(1.0f);
        assert(fabsf(result) < 1e-6f);
    }

    /* test gf24_ln_approx_of_e */
    {
        float e = 2.718281828459045f;
        float result = gf24_ln_approx(e);
        assert(fabsf(result - 1.0f) < 0.01f);
    }

    /* test gf24_ln_approx_negative_returns_nan */
    {
        float result = gf24_ln_approx(-1.0f);
        assert(result != result); /* NaN check */
    }

    /* test gf24_exp_approx_zero */
    {
        float result = gf24_exp_approx(0.0f);
        assert(fabsf(result - 1.0f) < 1e-6f);
    }

    /* test gf24_exp_approx_one */
    {
        float e = 2.718281828459045f;
        float result = gf24_exp_approx(1.0f);
        assert(fabsf(result - e) < 0.01f);
    }

    /* test gf24_exp_approx_negative */
    {
        float result = gf24_exp_approx(-1.0f);
        float expected = 1.0f / 2.718281828459045f;
        assert(fabsf(result - expected) < 0.01f);
    }

    /* test gf24_floor_positive */
    {
        float result = gf24_floor(3.7f);
        assert(fabsf(result - 3.0f) < 1e-6f);
    }

    /* test gf24_floor_negative */
    {
        float result = gf24_floor(-3.2f);
        assert(fabsf(result - (-4.0f)) < 1e-6f);
    }

    /* test gf24_floor_integer */
    {
        float result = gf24_floor(5.0f);
        assert(fabsf(result - 5.0f) < 1e-6f);
    }

    /* Invariant: max >= min_positive */
    {
        assert(gf24_max_value() >= gf24_min_positive());
    }

    /* Invariant: ln/exp inversion */
    {
        float x = 2.0f;
        float y = gf24_ln_approx(x);
        assert(fabsf(gf24_exp_approx(y) - x) < 0.01f);
    }

    /* Invariant: floor monotonic */
    {
        assert(gf24_floor(2.5f) <= gf24_floor(3.5f));
    }

    printf("GF24: all tests passed\n");
}
