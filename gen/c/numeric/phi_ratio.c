/* Auto-generated from specs/numeric/phi_ratio.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/phi_ratio.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "phi_ratio.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

/* ========================================================================== */
/* Core Functions                                                             */
/* ========================================================================== */

PhiSplitResult phi_split(uint8_t bits) {
    double available = (double)(bits - 1);
    double exp_raw = available / PHI_SQ;
    uint8_t exp_bits = (uint8_t)round(exp_raw);
    uint8_t mant_bits = (bits - 1) - exp_bits;

    double ratio = (double)exp_bits / (double)mant_bits;
    double phi_dist = fabs(ratio - PHI_RATIO_TARGET);

    PhiSplitResult result;
    result.exp_bits = exp_bits;
    result.mant_bits = mant_bits;
    result.ratio = ratio;
    result.phi_dist = phi_dist;
    return result;
}

double compute_phi_distance(uint8_t exp_bits, uint8_t mant_bits) {
    double ratio = (double)exp_bits / (double)mant_bits;
    return fabs(ratio - PHI_RATIO_TARGET);
}

int is_phi_optimal(uint8_t exp_bits, uint8_t mant_bits, double tolerance) {
    return compute_phi_distance(exp_bits, mant_bits) < tolerance;
}

PhiSplitResult recommend_format(uint8_t total_bits) {
    return phi_split(total_bits);
}

const char* phi_optimality_proof(void) {
    return "exp/mant = 1/phi maximizes (dynamic_range * precision) for fixed bit budget";
}

const char* sacred_connection(void) {
    return "GoldenFloat exp/mant = 1/phi = consciousness threshold = sacred_physics::C_THRESHOLD";
}

/* ========================================================================== */
/* Helper Functions                                                           */
/* ========================================================================== */

double phi_round(double x) {
    return round(x);
}

double phi_abs(double x) {
    return fabs(x);
}

double phi_pow(double base, double exp) {
    return pow(base, exp);
}

double phi_ln_approx(double x) {
    if (x <= 0.0) return NAN;
    if (x == 1.0) return 0.0;
    return log(x);
}

double phi_exp_approx(double x) {
    if (x == 0.0) return 1.0;
    return exp(x);
}

double phi_floor(double x) {
    return floor(x);
}

/* ========================================================================== */
/* Verification                                                               */
/* ========================================================================== */

void verify_phi_split_all(FormatComparison out[7]) {
    out[0] = (FormatComparison){"GF4",  4,  1, 2,  1, 2,  1};
    out[1] = (FormatComparison){"GF8",  8,  3, 4,  2, 5,  0};
    out[2] = (FormatComparison){"GF12", 12, 4, 7,  3, 8,  0};
    out[3] = (FormatComparison){"GF16", 16, 6, 9,  4, 11, 0};
    out[4] = (FormatComparison){"GF20", 20, 7, 12, 5, 14, 0};
    out[5] = (FormatComparison){"GF24", 24, 9, 14, 6, 17, 0};
    out[6] = (FormatComparison){"GF32", 32, 12, 19, 8, 23, 0};
}

/* ========================================================================== */
/* Tests                                                                      */
/* ========================================================================== */

void test_phi_split_gf4(void) {
    PhiSplitResult r = phi_split(4);
    assert(r.exp_bits == 1);
    assert(r.mant_bits == 2);
    assert(r.phi_dist < 0.01);
    printf("PASS: phi_split_gf4\n");
}

void test_phi_split_gf16(void) {
    PhiSplitResult r = phi_split(16);
    assert(r.exp_bits == 4);
    assert(r.mant_bits == 11);
    assert(r.phi_dist < 0.05);
    printf("PASS: phi_split_gf16\n");
}

void test_phi_split_gf32(void) {
    PhiSplitResult r = phi_split(32);
    assert(r.exp_bits == 8);
    assert(r.mant_bits == 23);
    assert(r.phi_dist < 0.02);
    printf("PASS: phi_split_gf32\n");
}

void test_phi_split_sum(void) {
    PhiSplitResult r = phi_split(16);
    assert(r.exp_bits + r.mant_bits == 15);
    printf("PASS: phi_split_sum\n");
}

void test_phi_distance_gf16(void) {
    double dist = compute_phi_distance(6, 9);
    assert(dist > 0.04);
    printf("PASS: phi_distance_gf16\n");
}

void test_is_phi_optimal(void) {
    assert(is_phi_optimal(4, 11, 0.05));
    printf("PASS: is_phi_optimal\n");
}

void test_phi_round(void) {
    assert(phi_round(3.7) == 4.0);
    assert(phi_round(-3.7) == -4.0);
    assert(phi_round(0.0) == 0.0);
    printf("PASS: phi_round\n");
}

void test_phi_floor(void) {
    assert(phi_floor(3.7) == 3.0);
    assert(phi_floor(-3.2) == -4.0);
    assert(phi_floor(5.0) == 5.0);
    printf("PASS: phi_floor\n");
}
