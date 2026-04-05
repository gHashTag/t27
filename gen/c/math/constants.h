/* Auto-generated from specs/math/constants.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/math/constants.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef CONSTANTS_H
#define CONSTANTS_H

#include <stdint.h>

/* ========================================================================== */
/* Sacred Constants                                                            */
/* ========================================================================== */

/* phi = (1 + sqrt(5)) / 2 -- the golden ratio */
#define PHI             1.61803398874989484820458683436563811772

/* phi^-1 = phi - 1 */
#define PHI_INV         0.61803398874989484820458683436563811772

/* phi^2 */
#define PHI_SQ          (PHI * PHI)

/* (phi^-1)^2 */
#define PHI_INV_SQ      (PHI_INV * PHI_INV)

/* TRINITY = 3.0 (phi^2 + phi^-2 = 3) */
#define TRINITY         3.0

/* pi */
#define T27_PI          3.14159265358979323846264338327950288

/* e (Euler's number) */
#define T27_E           2.7182818284590452353602874713526625

/* ========================================================================== */
/* CODATA 2022 Measurements                                                    */
/* ========================================================================== */

/* Gravitational constant G */
#define G_MEASURED              6.67430e-11

/* Cosmological constant Lambda */
#define LAMBDA_COSMO            1.1056e-52

/* Dark energy density parameter Omega_Lambda */
#define OMEGA_LAMBDA_MEASURED   0.685

/* ========================================================================== */
/* Function Declarations                                                       */
/* ========================================================================== */

double t27_abs(double x);
double t27_pow(double x, double n);
double t27_ln_approx(double x);
double t27_exp_approx(double x);
double t27_floor(double x);

/* Test functions */
void test_phi_squared_plus_inverse_squared_equals_3(void);
void test_phi_inverse_is_phi_minus_one(void);
void test_phi_multiplicative_persistence(void);
void test_trinity_constant_accuracy(void);
void test_pi_range_validity(void);
void test_euler_number_range_validity(void);
void test_pow_zero_exponent_returns_one(void);
void test_pow_positive_integer_exponent(void);
void test_pow_negative_integer_exponent(void);
void test_pow_phi_squared(void);
void test_floor_function_positive(void);
void test_floor_function_negative(void);
void test_floor_function_integer(void);

#endif /* CONSTANTS_H */
