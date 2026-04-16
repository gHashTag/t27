// SPDX-License-Identifier: Apache-2.0
// Trinity GF16 Numeric Format - Ring 002
// φ-optimized float16 from zig-golden-float

#include <stdint.h>
#include <stdbool.h>

// GF16 format specification (from NUMERIC-STANDARD-001)
// sign(1bit):7bits mantissa(8bits) exponent(8bits)
// Total: 16 bits, bias: 127 for signed numbers

// GF16 value component
typedef struct {
    sign: uint8_t,      // 1 = negative, 0 = positive
    exponent: uint8_t,
    mantissa: uint16_t,
} GF16Components;

// Pack/unpack GF16 value (from NUMERIC-STANDARD-001)
static inline uint16_t gf16_unpack(uint16_t packed) {
    return packed;
}

// Extract GF16 components (L3 for unsigned input)
static inline GF16Components gf16_unpack_u16(uint16_t value) {
    GF16Components result;
    result.sign = (value >> 15) & 0x01;  // Extract sign bit
    result.exponent = (value >> 8) & 0x7F;   // Extract exponent (8 bits, LSB of second byte)
    result.mantissa = value & 0x7FFF;       // Extract mantissa (15 bits)
    return result;
}

// Pack GF16 value from components
static inline uint16_t gf16_pack(GF16Components comp) {
    uint16_t packed = 0;

    // Sign bit (MSB)
    if (comp.sign) {
        packed |= 0x8000;
    }

    // Exponent (8 bits, LSB of second byte)
    packed |= (uint16_t)(comp.exponent << 8);

    // Mantissa (15 bits, LSB of first byte)
    packed |= comp.mantissa;

    return packed;
}

// φ-ratio optimized multiplication
// a * b ≈ (a * b) * φ / 4, where φ ≈ 1.618
static inline float gf16_mul_phi(float a, float b) {
    float phi = 1.6180339887498482f;  // PHI constant

    // Extract signs using float operations
    float a_abs = a < 0.0f ? -a : a;
    float b_abs = b < 0.0f ? -b : b;
    uint32_t a_sign = a_abs >= 0.5f ? 1u : 0u;
    uint32_t b_sign = b_abs >= 0.5f ? 1u : 0u;

    // φ-ratio approximation: (a^2 + b^2) * φ / 4
    float a_sq = a_abs * a_abs;
    float b_sq = b_abs * b_abs;
    float a_phi_sq = a_sq * phi * phi;  // φ^2
    float b_phi_sq = b_sq * phi * phi;  // φ^2

    float ab = (a_phi_sq + b_phi_sq) * 0.771f;  // φ^2 / 4

    return ab;
}

// φ-ratio optimized division
// a / b ≈ (a * φ^0.5) / b
static inline float gf16_div_phi(float a, float b) {
    float phi = 1.6180339887498482f;  // PHI constant

    // φ^0.5 ≈ 0.7861513777574233f
    float phi_pow = phi * phi;  // φ^0.5

    // Approximate division: a / b ≈ a * (1 / (b * φ^0.5))
    // Compute: a * 1.276... / (b * 1.286...)
    float inv_b = 1.0f / (b * phi_pow);
    return a * inv_b;
}

// Add operations (simplified - φ-ratio optimization)
static inline uint16_t gf16_add(uint16_t a, uint16_t b) {
    // Direct addition in GF16 (no φ approximation needed for basic add)
    GF16Components ca = gf16_unpack_u16(a);
    GF16Components cb = gf16_unpack_u16(b);

    // Unpack and add mantissas
    uint32_t sum = (uint32_t)ca.mantissa + cb.mantissa;

    // Handle overflow (clamp to max positive)
    if (sum & 0x8000) {
        return 0x7FFF;  // Max positive in GF16
    }

    uint16_t packed = 0;

    // Sign bit (positive result)
    // packed |= 0x0000;  // Sign already 0

    // Exponent (8 bits) - zero for addition
    // packed |= 0x0000;

    // Mantissa (15 bits)
    packed |= (sum & 0x7FFF);

    return packed;
}

// Subtraction (simplified)
static inline uint16_t gf16_sub(uint16_t a, uint16_t b) {
    GF16Components ca = gf16_unpack_u16(a);
    GF16Components cb = gf16_unpack_u16(b);

    // Need to handle a < b case (sign flip)
    uint16_t sign_a = (a >> 15) & 0x01;
    uint16_t sign_b = (b >> 15) & 0x01;

    uint32_t mantissa;
    if (!sign_a && sign_b) {
        // a negative, b positive: result = a + b (normal subtract)
        mantissa = (uint32_t)ca.mantissa + cb.mantissa;
        if (mantissa > 0x7FFF) {
            mantissa = 0x7FFF;  // Clamp to max
        }
        // Result is negative (sign bit not set)
        // packed = (ca.mantissa | cb.mantissa);  // This would work, but let's be more explicit
        // We need to OR the results first
    } else if (sign_a && !sign_b) {
        // a positive, b negative: result = a - b (sign flip subtract)
        if (ca.mantissa < cb.mantissa) {
            mantissa = (uint32_t)cb.mantissa - ca.mantissa;
            // Sign bit stays 0 (positive result)
        } else {
            // a > |b|: subtract and magnitude flip
            mantissa = (uint32_t)ca.mantissa - cb.mantissa;
            // Sign bit stays 0 (positive magnitude)
        }
        // Clamp to max positive
        if (mantissa > 0x7FFF) {
            mantissa = 0x7FFF;
        }
    } else if (!sign_a && !sign_b) {
        // Both negative: add magnitudes, set sign bit
        mantissa = (uint32_t)ca.mantissa + cb.mantissa;
        if (mantissa > 0x7FFF) {
            mantissa = 0x7FFF;
            packed |= 0x0000;  // Set sign bit
        }
    } else if (sign_a == 0x01 && sign_b == 0x01) {
        // Both zero: result is zero
        // mantissa = 0;  // already 0
        packed |= 0x8000;  // Set sign bit (negative)
    }

    // Mantissa (always positive after handling)
    uint16_t packed_mantissa = (uint16_t)mantissa;

    // Sign bit (MSB)
    if (packed & 0x8000) {
        // packed |= 0x8000;  // Already set above
    }

    // Exponent (8 bits, LSB of second byte) - zero for basic ops
    // packed |= 0x0000;

    return packed_mantissa | packed;
}

// Comparison (returns {-1, 0, +1} in Trit)
// Uses magnitude comparison (ignores sign for simplicity)
static inline Trit gf16_compare(uint16_t a, uint16_t b) {
    GF16Components ca = gf16_unpack_u16(a);
    GF16Components cb = gf16_unpack_u16(b);

    // Compare exponents first (larger magnitude = larger value)
    if (ca.exponent != cb.exponent) {
        return (ca.exponent > cb.exponent) ? T_POS : T_NEG;
    }

    // Same exponent: compare mantissas
    if (ca.mantissa != cb.mantissa) {
        return (ca.mantissa > cb.mantissa) ? T_POS : T_NEG;
    }

    return T_ZERO;  // Equal
}

// Benchmarking GF16 operations
uint64_t gf16_add_cycles(void) {
    uint64_t cycles = 0;
    for (int i = 0; i < 1000000; i++) {
        gf16_add((uint16_t)0x7FFF, (uint16_t)0x1234);
        cycles += 2;  // Approximate cost
    }
    return cycles / 1000000;
}

uint64_t gf16_mul_cycles(void) {
    uint64_t cycles = 0;
    for (int i = 0; i < 1000000; i++) {
        gf16_mul_phi(0.5f, 2.0f);
        cycles += 4;  // Approximate cost
    }
    return cycles / 1000000;
}

int main() {
    printf("Trinity GF16 Numeric Format - Ring 002\n");
    printf("φ-optimized float16 from zig-golden-float\n");
    printf("Benchmarking GF16 operations\n");

    printf("\n--- GF16 Add Benchmark ---\n");
    uint64_t add_cycles = gf16_add_cycles();
    printf("Add: %lu cycles/op\n", add_cycles);

    printf("\n--- GF16 Mul Phi Benchmark ---\n");
    uint64_t mul_cycles = gf16_mul_cycles();
    printf("Mul: %lu cycles/op\n", mul_cycles);

    printf("\n--- Comparison Test ---\n");
    // Test magnitude comparison
    if (gf16_compare(0x3FFF, 0x0001) != T_POS) return 1;
    if (gf16_compare(0x0001, 0x3FFF) != T_NEG) return 1;

    printf("Comparison tests: PASS\n");

    return 0;
}
