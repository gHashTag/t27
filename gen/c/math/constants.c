/* Auto-generated from specs/math/constants.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/math/constants.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "constants.h"
#include <assert.h>
#include <math.h>

/* ========================================================================== */
/* Function Implementations                                                    */
/* ========================================================================== */

double t27_abs(double x) {
    return (x < 0.0) ? -x : x;
}

double t27_floor(double x) {
    long xi = (long)x;
    if (x >= 0.0 || x == (double)xi) {
        return (double)xi;
    }
    return (double)(xi - 1);
}

double t27_ln_approx(double x) {
    if (x <= 0.0) {
        return 0.0 / 0.0; /* NaN */
    }
    if (x == 1.0) {
        return 0.0;
    }

    double t = (x - 1.0) / (x + 1.0);
    double t2 = t * t;
    double t3 = t2 * t;
    double t5 = t3 * t2;
    double t7 = t5 * t2;

    return 2.0 * (t + t3 / 3.0 + t5 / 5.0 + t7 / 7.0);
}

double t27_exp_approx(double x) {
    if (x == 0.0) {
        return 1.0;
    }

    double exp_x = x;
    double scale = 1.0;

    if (x > 10.0) {
        long k = (long)t27_floor(x / 10.0);
        exp_x = x - (double)k * 10.0;
        scale = t27_pow(T27_E, (double)k * 10.0);
    } else if (x < -10.0) {
        long k = (long)t27_floor(-x / 10.0);
        exp_x = x + (double)k * 10.0;
        scale = 1.0 / t27_pow(T27_E, (double)k * 10.0);
    }

    double result = 1.0;
    double term = 1.0;

    for (int i = 1; i <= 10; i++) {
        term = term * exp_x / (double)i;
        result = result + term;
    }

    return result * scale;
}

double t27_pow(double x, double n) {
    /* Handle negative base with fractional exponent -> NaN */
    if (x < 0.0 && n != t27_floor(n)) {
        return 0.0 / 0.0; /* NaN */
    }

    /* Handle zero base */
    if (x == 0.0) {
        if (n > 0.0) return 0.0;
        if (n == 0.0) return 1.0;
        return 1.0 / 0.0; /* Infinity */
    }

    /* Handle n = 0 */
    if (n == 0.0) {
        return 1.0;
    }

    /* Handle negative exponent */
    int negative = (n < 0.0);
    double exp_val = negative ? -n : n;

    /* Check if exponent is integer */
    int is_integer = (exp_val == t27_floor(exp_val));

    if (is_integer) {
        /* Integer exponent: binary exponentiation */
        long exp_int = (long)exp_val;
        double result = 1.0;
        double base = x;

        while (exp_int > 0) {
            if (exp_int % 2 == 1) {
                result = result * base;
            }
            base = base * base;
            exp_int = exp_int / 2;
        }

        if (negative) {
            result = 1.0 / result;
        }
        return result;
    }

    /* Fractional exponent: exp(y * ln(x)) */
    double ln_x = t27_ln_approx(x);
    double result = t27_exp_approx(exp_val * ln_x);

    if (negative) {
        result = 1.0 / result;
    }
    return result;
}

/* ========================================================================== */
/* Test Functions                                                              */
/* ========================================================================== */

void test_phi_squared_plus_inverse_squared_equals_3(void) {
    double phi_sq = PHI * PHI;
    double phi_inv_sq = PHI_INV * PHI_INV;
    double sum = phi_sq + phi_inv_sq;
    assert(t27_abs(sum - TRINITY) < 1e-12);
}

void test_phi_inverse_is_phi_minus_one(void) {
    double expected = PHI - 1.0;
    assert(t27_abs(PHI_INV - expected) < 1e-15);
}

void test_phi_multiplicative_persistence(void) {
    double squared = PHI * PHI;
    double result = squared - PHI;
    assert(t27_abs(result - 1.0) < 1e-12);
}

void test_trinity_constant_accuracy(void) {
    assert(t27_abs(TRINITY - 3.0) < 1e-15);
}

void test_pi_range_validity(void) {
    assert(T27_PI >= 3.1415926535 && T27_PI <= 3.1415926536);
}

void test_euler_number_range_validity(void) {
    assert(T27_E >= 2.7182818284 && T27_E <= 2.7182818285);
}

void test_pow_zero_exponent_returns_one(void) {
    double result = t27_pow(2.0, 0.0);
    assert(t27_abs(result - 1.0) < 1e-15);
}

void test_pow_positive_integer_exponent(void) {
    double result = t27_pow(2.0, 10.0);
    assert(t27_abs(result - 1024.0) < 1e-10);
}

void test_pow_negative_integer_exponent(void) {
    double result = t27_pow(2.0, -3.0);
    assert(t27_abs(result - 0.125) < 1e-10);
}

void test_pow_phi_squared(void) {
    double result = t27_pow(PHI, 2.0);
    double expected = PHI * PHI;
    assert(t27_abs(result - expected) < 1e-10);
}

void test_floor_function_positive(void) {
    assert(t27_floor(3.7) == 3.0);
}

void test_floor_function_negative(void) {
    assert(t27_floor(-3.2) == -4.0);
}

void test_floor_function_integer(void) {
    assert(t27_floor(5.0) == 5.0);
}
