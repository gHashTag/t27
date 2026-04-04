/* Auto-generated from specs/numeric/phi_ratio.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/phi_ratio.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef PHI_RATIO_H
#define PHI_RATIO_H

#include <stdint.h>
#include <math.h>

/* ========================================================================== */
/* PhiRatio -- phi-split derivation for GoldenFloat exp/mantissa allocation    */
/* The ideal exp/mant ratio = 1/phi ~= 0.618                                  */
/* ========================================================================== */

#define PHI           1.6180339887498948
#define PHI_INV       0.6180339887498949
#define PHI_SQ        2.6180339887498949
#define PHI_RATIO_TARGET PHI_INV

typedef struct {
    uint8_t exp_bits;
    uint8_t mant_bits;
    double  ratio;
    double  phi_dist;
} PhiSplitResult;

typedef struct {
    const char* name;
    uint8_t bits;
    uint8_t actual_exp;
    uint8_t actual_mant;
    uint8_t phi_split_exp;
    uint8_t phi_split_mant;
    int     matches_phi_split;
} FormatComparison;

/* Core functions */
PhiSplitResult phi_split(uint8_t bits);
double compute_phi_distance(uint8_t exp_bits, uint8_t mant_bits);
int    is_phi_optimal(uint8_t exp_bits, uint8_t mant_bits, double tolerance);
PhiSplitResult recommend_format(uint8_t total_bits);
const char* phi_optimality_proof(void);
const char* sacred_connection(void);

/* Helper functions */
double phi_round(double x);
double phi_abs(double x);
double phi_pow(double base, double exp);
double phi_ln_approx(double x);
double phi_exp_approx(double x);
double phi_floor(double x);

/* Verification */
void verify_phi_split_all(FormatComparison out[7]);

/* Tests */
void test_phi_split_gf4(void);
void test_phi_split_gf16(void);
void test_phi_split_gf32(void);
void test_phi_split_sum(void);
void test_phi_distance_gf16(void);
void test_is_phi_optimal(void);
void test_phi_round(void);
void test_phi_floor(void);

#endif /* PHI_RATIO_H */
