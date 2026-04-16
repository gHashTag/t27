// SPDX-License-Identifier: Apache-2.0
// Trinity φ-Arithmetic - Ring 001
// Public API for φ arithmetic operations

#ifndef TRINITY_PHI_ARITH_H
#define TRINITY_PHI_ARITH_H

#include <stdint.h>
#include <stdbool.h>

// φ constant
#define PHI 1.6180339887498948482

// Result structures for arithmetic operations
typedef struct {
    uint64_t value;        // Primary result
    bool overflow;         // Did computation overflow?
    bool underflow;        // Did computation underflow?
} PhiArithResult;

typedef struct {
    uint64_t quotient;      // Division result
    uint64_t remainder;     // Division remainder
    bool overflow;         // Division overflow?
} PhiDivResult;

// φ^n: exponentiation
uint64_t phi_pow(uint64_t n);

// Lucas number check (L5: L_n are integers)
bool is_lucas_n(uint64_t n);

// L5 identity: φ^2 + φ^(-2) == 3
bool verify_l5_identity_double(double phi_val);

// Basic arithmetic
PhiArithResult phi_add(uint64_t a, uint64_t b);
PhiArithResult phi_sub(uint64_t a, uint64_t b);
PhiArithResult phi_mul(uint64_t a, uint64_t b);
PhiDivResult phi_div(uint64_t a, uint64_t b);

// Comparison (returns: -1, 0, +1)
int8_t phi_compare(uint64_t a, uint64_t b);

// Kleene operations for ternary logic
uint8_t trit_and(uint8_t a, uint8_t b);
uint8_t trit_or(uint8_t a, uint8_t b);
uint8_t trit_not(uint8_t t);
uint8_t trit_xor(uint8_t a, uint8_t b);

// Consensus: all inputs equal
bool trit_consensus(const uint8_t* inputs, uint8_t count);

#endif // TRINITY_PHI_ARITH_H
