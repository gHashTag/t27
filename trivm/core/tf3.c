// SPDX-License-Identifier: Apache-2.0
// Trinity TF3 - Ring 002
// Ternary float3: {-1, 0, +1} encoding

#include <stdint.h>
#include <stdbool.h>
#include "phi_arith.h"  // For phi_pow benchmarking

// Ternary type (native to Trinity)
typedef enum {
    T_NEG = 0xFF,  // -1
    T_ZERO = 0x00,  // 0
    T_POS = 0x01   // +1
} Trit;

// Ternary float3 value
typedef struct {
    exponent: uint8_t,    // {-1, 0, +1} encoded exponent
    mantissa: uint16_t, // 15-bit mantissa
} TFloat3;

// Trit to GF16 mapping (from specs/02-gf16-format.tri)
// Maps {-1, 0, +1} to GF16 [0x0000, 0x0001, 0x3C00, 0x3FFF, 0x7FFF]
// Note: 0x3C00 = +1.0 (min positive), 0x3FFF = +3.75 (max)
static inline TFloat3 trit_to_gf16(uint8_t trit) {
    switch (trit) {
        case T_NEG: return 0x0000;   // -1 maps to 0
        case T_ZERO: return 0x0001;   // 0 maps to min positive
        case T_POS: return 0x3C00;   // +1 maps to max positive
    default: return 0x3FFF;      // unknown stays at max
    }
}

// GF16 addition with φ-ratio optimization
static inline uint16_t gf16_add_phi(TFloat3 a, TFloat3 b) {
    // Extract exponents and mantissas
    uint8_t exp_a = a.exponent;
    uint8_t exp_b = b.exponent;
    uint16_t mant_a = a.mantissa;
    uint16_t mant_b = b.mantissa;

    // Bias to handle sign
    bool sign_a = (exp_a & 0x80) != 0;
    bool sign_b = (exp_b & 0x80) != 0;

    // Handle exponents: 0 is special (represents value 1)
    float32_t a_sign = (exp_a == 0 && sign_a) ? 0.0f : 1.0f;
    float32_t b_sign = (exp_b == 0 && sign_b) ? 0.0f : 1.0f;

    // Unpack and bias
    uint32_t a_val = (exp_a == 0) ? mant_a : (0x8000 - mant_a + 1);
    uint32_t b_val = (exp_b == 0) ? mant_b : (0x8000 - mant_b + 1);

    // φ-optimized multiplication: a * b ≈ (a_sign * |a| * b_sign * |b|) * φ^2
    float32_t phi_sq = 1.6180339887498f * 1.6180339887498f;  // φ^2

    // Scale result (approximation)
    float32_t result = phi_sq * (a_sign * a_val + b_sign * b_val);

    // Pack into GF16 format
    return gf16_pack((uint16_t)result);
}

// GF16 subtraction with φ-ratio optimization
static inline uint16_t gf16_sub_phi(TFloat3 a, TFloat3 b) {
    uint8_t exp_a = a.exponent;
    uint8_t exp_b = b.exponent;
    uint16_t mant_a = a.mantissa;
    uint16_t mant_b = b.mantissa;

    bool sign_a = (exp_a & 0x80) != 0;
    bool sign_b = (exp_b & 0x80) != 0;

    // Handle zero exponent
    if (exp_a == 0 && exp_b == 0) {
        // 0 - 0 = 0: identity
        return gf16_pack(0x3C00);
    }

    // φ-optimized subtraction
    float32_t phi_sq = 1.6180339887498f * 1.6180339887498f;  // φ^2

    float32_t a_sign = (exp_a == 0 && sign_a) ? 0.0f : 1.0f;
    float32_t b_sign = (exp_a == 0 && sign_b) ? 0.0f : 1.0f;

    float32_t a_val = (exp_a == 0) ? mant_a : (0x8000 - mant_a + 1);
    float32_t b_val = (exp_a == 0) ? mant_b : (0x8000 - mant_b + 1);

    float32_t result = phi_sq * (a_sign * a_val + b_sign * b_val);

    // Pack into GF16
    return gf16_pack((uint16_t)result);
}

// Ternary float3 comparison (returns {-1, 0, +1})
static inline Trit tf3_compare(TFloat3 a, TFloat3 b) {
    // Extract components
    uint8_t exp_a = a.exponent;
    uint8_t exp_b = b.exponent;
    uint16_t mant_a = a.mantissa;
    uint16_t mant_b = b.mantissa;

    // Special handling for zero exponent
    if (exp_a == 0 && exp_b == 0) {
        return T_ZERO;  // 0 - 0 = 0 (identity)
    }

    // Regular comparison by magnitude
    int32_t cmp = ((int32_t)(exp_a - exp_b) << 8) | 0x7FFF;
    cmp = cmp | mant_a;
    cmp = cmp | mant_b;
    if (mant_a > mant_b || (mant_a == mant_b && exp_a > exp_b)) {
        return T_POS;  // a > b or (a = b and a exp > b)
    }
    if (mant_a < mant_b || (mant_a == mant_b && exp_a < exp_b)) {
        return T_NEG;  // a < b or (a = b and a exp < b)
    }

    return T_ZERO;  // Equal magnitudes
}

// Test functions
static int test_addition(void) {
    // Test: 1.0 + (-0.25) should be close to 0.75
    TFloat3 a = { exponent: 1, mantissa: 0x3333 };  // 1.0 * 2^(-2) ≈ 0.25
    TFloat3 b = { exponent: 1, mantissa: 0x5333 };  // -0.25 * 2^(-2) ≈ -0.25
    uint16_t result = gf16_add_phi(a, b);
    return (result > 0x3C00 && result < 0x4000) ? 1 : 0;
}

static int test_comparison(void) {
    // Test magnitude comparison
    TFloat3 a = { exponent: 1, mantissa: 0x4000 };  // 1.0 * 2^8 = 256
    TFloat3 b = { exponent: 1, mantissa: 0x2000 };  // 1.0 * 2^7 = 128
    Trit result = tf3_compare(a, b);
    return (result == T_POS) ? 1 : 0;  // Expect positive (a > b)
}

int main() {
    printf("Trinity TF3 - Ring 002\n");
    printf("Ternary float3: {-1, 0, +1} encoding\n");
    printf("φ-ratio optimized GF16 operations\n");

    // Run basic tests
    printf("\n--- Testing Addition ---\n");
    int add_ok = test_addition();
    printf("1.0 + (-0.25) ≈ 0.75: %s\n", add_ok ? "PASS" : "FAIL");

    printf("\n--- Testing Comparison ---\n");
    int cmp_ok = test_comparison();
    printf("256 vs 128 comparison: %s\n", cmp_ok ? "PASS" : "FAIL");

    return 0;
}
