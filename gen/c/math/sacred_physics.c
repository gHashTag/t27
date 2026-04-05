/* Auto-generated from specs/math/sacred_physics.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/math/sacred_physics.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 28 | Module: SacredPhysics */

#include "sacred_physics.h"
#include <assert.h>
#include <math.h>

/* ===================================================================== */
/* 1.5 phi power helper                                                   */
/* ===================================================================== */

double sp_phi_pow(int64_t n) {
    if (n == 0) return 1.0;
    if (n > 0) {
        double result = 1.0;
        double base = SP_PHI;
        int64_t exp = n;
        while (exp > 0) {
            if (exp % 2 == 1) {
                result *= base;
            }
            base *= base;
            exp /= 2;
        }
        return result;
    }
    /* n < 0 */
    {
        double result = 1.0;
        double base = SP_PHI_INV;
        int64_t exp = -n;
        while (exp > 0) {
            if (exp % 2 == 1) {
                result *= base;
            }
            base *= base;
            exp /= 2;
        }
        return result;
    }
}

/* ===================================================================== */
/* Neural gamma band center                                               */
/* ===================================================================== */

double sp_neural_gamma_center(double pi) {
    double phi_cubed = SP_PHI * SP_PHI * SP_PHI;
    return (phi_cubed * pi) / SP_GAMMA_LQG;
}

/* ===================================================================== */
/* 2. Gravity & dark energy from TRINITY                                  */
/* ===================================================================== */

double sp_sacred_gravity(double pi) {
    double pi_sq  = pi * pi;
    double pi_cub = pi_sq * pi;
    double g2     = SP_GAMMA_LQG * SP_GAMMA_LQG;
    return (pi_cub * g2) / SP_PHI;
}

double sp_sacred_dark_energy(double pi) {
    double gamma4 = SP_GAMMA_LQG * SP_GAMMA_LQG * SP_GAMMA_LQG * SP_GAMMA_LQG;
    double gamma8 = gamma4 * gamma4;
    double pi2    = pi * pi;
    double pi4    = pi2 * pi2;
    return (gamma8 * pi4) / SP_PHI_SQ;
}

/* ===================================================================== */
/* 3. Verification API                                                    */
/* ===================================================================== */

SacredPhysicsReport sp_verify_sacred_physics(void) {
    SacredPhysicsReport report;

    report.trinity_value = SP_TRINITY;
    report.trinity_ok    = fabs(SP_TRINITY - 3.0) < SP_MAX_ABS_ERROR_TRINITY;

    report.gamma_value  = SP_GAMMA_LQG;
    report.c_threshold  = SP_C_THRESHOLD;
    report.t_present_ms = SP_T_PRESENT_MS;

    report.g_pred    = sp_sacred_gravity(SP_PI);
    report.g_measured = SP_G_MEASURED;
    report.g_rel_error = fabs(report.g_pred - report.g_measured) / report.g_measured;
    report.g_ok = report.g_rel_error <= SP_MAX_REL_ERROR_G;

    report.omega_pred    = sp_sacred_dark_energy(SP_PI);
    report.omega_measured = SP_OMEGA_LAMBDA_MEASURED;
    report.omega_rel_error = fabs(report.omega_pred - report.omega_measured) / report.omega_measured;
    report.omega_ok = report.omega_rel_error <= SP_MAX_REL_ERROR_OMEGA;

    report.f_gamma_pred = sp_neural_gamma_center(SP_PI);

    return report;
}

/* ===================================================================== */
/* 4. TRINITY Verification                                                */
/* ===================================================================== */

TrinityVerification sp_verify_trinity(double tolerance) {
    TrinityVerification result;

    double phi_sq     = SP_PHI * SP_PHI;
    double phi_inv_sq = SP_PHI_INV * SP_PHI_INV;
    double trinity    = phi_sq + phi_inv_sq;
    double target     = 3.0;
    double abs_err    = fabs(trinity - target);
    double rel_err    = abs_err / target;

    result.phi_value      = SP_PHI;
    result.phi_sq_value   = phi_sq;
    result.phi_inv_sq     = phi_inv_sq;
    result.trinity_value  = trinity;
    result.target         = target;
    result.absolute_error = abs_err;
    result.relative_error = rel_err;
    result.passes         = abs_err <= tolerance;
    result.tolerance      = tolerance;

    return result;
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_sacred_physics(void) {
    /* test trinity_identity_holds */
    assert(fabs(SP_TRINITY - 3.0) < SP_MAX_ABS_ERROR_TRINITY);

    /* test phi_squared_plus_inverse_squared */
    {
        double phi_sq = SP_PHI * SP_PHI;
        double phi_inv_sq = SP_PHI_INV * SP_PHI_INV;
        double trinity = phi_sq + phi_inv_sq;
        assert(fabs(trinity - 3.0) < 1e-12);
    }

    /* test gamma_from_phi_inverse_cubed */
    {
        double gamma_expected = SP_PHI_INV * SP_PHI_INV * SP_PHI_INV;
        assert(fabs(gamma_expected - SP_GAMMA_LQG) < 1e-15);
    }

    /* test consciousness_threshold_equals_phi_inverse */
    assert(fabs(SP_C_THRESHOLD - SP_PHI_INV) < 1e-15);

    /* test specious_present_in_milliseconds */
    assert(fabs(SP_T_PRESENT_MS - SP_T_PRESENT_SEC * 1000.0) < 1e-12);

    /* test neural_gamma_center_around_40hz */
    {
        double f_gamma = sp_neural_gamma_center(SP_PI);
        assert(f_gamma > 30.0 && f_gamma < 50.0);
    }

    /* test sacred_gravity_close_to_measured */
    {
        SacredPhysicsReport report = sp_verify_sacred_physics();
        assert(report.g_rel_error < SP_MAX_REL_ERROR_G);
    }

    /* test sacred_dark_energy_close_to_measured */
    {
        SacredPhysicsReport report = sp_verify_sacred_physics();
        assert(report.omega_rel_error < SP_MAX_REL_ERROR_OMEGA);
    }

    /* test verify_report_contains_all_fields */
    {
        SacredPhysicsReport report = sp_verify_sacred_physics();
        assert(report.trinity_ok == true);
        assert(report.f_gamma_pred > 0.0);
        assert(report.t_present_ms > 0.0);
    }

    /* test verify_trinity_with_strict_tolerance */
    {
        TrinityVerification result = sp_verify_trinity(1e-15);
        assert(result.passes == true);
        assert(result.trinity_value > 2.99);
        assert(result.trinity_value < 3.01);
    }

    /* test verify_trinity_with_loose_tolerance */
    {
        TrinityVerification result = sp_verify_trinity(0.1);
        assert(result.passes == true);
        assert(result.tolerance == 0.1);
    }

    /* test verify_trinity_phi_components */
    {
        TrinityVerification result = sp_verify_trinity(1e-12);
        assert(result.phi_sq_value > 2.6);
        assert(result.phi_sq_value < 2.7);
        assert(result.phi_inv_sq > 0.38);
        assert(result.phi_inv_sq < 0.39);
    }

    /* test phi_pow_zero_equals_one */
    assert(fabs(sp_phi_pow(0) - 1.0) < 1e-15);

    /* test phi_pow_one_equals_phi */
    assert(fabs(sp_phi_pow(1) - SP_PHI) < 1e-15);

    /* test phi_pow_negative_one_equals_phi_inverse */
    assert(fabs(sp_phi_pow(-1) - SP_PHI_INV) < 1e-15);

    /* test phi_pow_two_equals_phi_plus_one */
    {
        double result = sp_phi_pow(2);
        double expected = SP_PHI + 1.0;
        assert(fabs(result - expected) < 1e-15);
    }

    /* test phi_pow_three_matches_multiplication */
    {
        double result = sp_phi_pow(3);
        double expected = SP_PHI * SP_PHI * SP_PHI;
        assert(fabs(result - expected) < 1e-15);
    }

    /* test phi_pow_negative_two_matches_inverse_square */
    {
        double result = sp_phi_pow(-2);
        double expected = SP_PHI_INV * SP_PHI_INV;
        assert(fabs(result - expected) < 1e-15);
    }

    /* test phi_pow_positive_returns_greater_than_one */
    assert(sp_phi_pow(5) > 1.0);

    /* test phi_pow_negative_returns_less_than_one */
    {
        double result = sp_phi_pow(-5);
        assert(result > 0.0 && result < 1.0);
    }
}
