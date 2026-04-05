// Auto-generated from specs/math/constants.t27
// DO NOT EDIT -- regenerate with: tri gen specs/math/constants.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const math = std.math;

// ============================================================================
// Sacred Constants
// ============================================================================

/// phi = (1 + sqrt(5)) / 2 -- the golden ratio
pub const PHI: f64 = 1.61803398874989484820458683436563811772;

/// phi^-1 = phi - 1 -- the inverse golden ratio
pub const PHI_INV: f64 = 0.61803398874989484820458683436563811772;

/// phi^2
pub const PHI_SQ: f64 = PHI * PHI;

/// (phi^-1)^2
pub const PHI_INV_SQ: f64 = PHI_INV * PHI_INV;

/// TRINITY = 3.0 (phi^2 + phi^-2 = 3)
pub const TRINITY: f64 = 3.0;

/// pi
pub const PI: f64 = 3.14159265358979323846264338327950288;

/// e (Euler's number)
pub const E: f64 = 2.7182818284590452353602874713526625;

// ============================================================================
// CODATA 2022 Measurements
// ============================================================================

/// Gravitational constant G = 6.67430e-11 m^3 kg^-1 s^-2
pub const G_MEASURED: f64 = 6.67430e-11;

/// Cosmological constant Lambda ~ 1.1056e-52 m^-2
pub const LAMBDA_COSMO: f64 = 1.1056e-52;

/// Dark energy density parameter Omega_Lambda ~ 0.685
pub const OMEGA_LAMBDA_MEASURED: f64 = 0.685;

// ============================================================================
// Helper Functions
// ============================================================================

/// Absolute value
pub fn abs(x: f64) f64 {
    return if (x < 0.0) -x else x;
}

/// Power function (integer and fractional exponents)
pub fn pow(x: f64, n: f64) f64 {
    // Handle negative base with fractional exponent -> NaN
    if (x < 0.0 and n != floor(n)) {
        return math.nan(f64);
    }

    // Handle zero base
    if (x == 0.0) {
        if (n > 0.0) {
            return 0.0;
        } else if (n == 0.0) {
            return 1.0;
        }
        return math.inf(f64);
    }

    // Handle n = 0
    if (n == 0.0) {
        return 1.0;
    }

    // Handle negative exponent
    const negative = n < 0.0;
    const exp_val = if (negative) -n else n;

    // Check if exponent is integer
    const is_integer = exp_val == floor(exp_val);

    if (is_integer) {
        // Integer exponent: binary exponentiation
        var exp_int: i64 = @intFromFloat(exp_val);
        var result: f64 = 1.0;
        var base: f64 = x;

        while (exp_int > 0) {
            if (@rem(exp_int, 2) == 1) {
                result = result * base;
            }
            base = base * base;
            exp_int = @divTrunc(exp_int, 2);
        }

        if (negative) {
            result = 1.0 / result;
        }
        return result;
    }

    // Fractional exponent: exp(y * ln(x))
    const ln_x = ln_approx(x);
    var result = exp_approx(exp_val * ln_x);

    if (negative) {
        result = 1.0 / result;
    }
    return result;
}

/// Natural logarithm approximation
pub fn ln_approx(x: f64) f64 {
    if (x <= 0.0) {
        return math.nan(f64);
    }
    if (x == 1.0) {
        return 0.0;
    }

    const t = (x - 1.0) / (x + 1.0);
    const t2 = t * t;
    const t3 = t2 * t;
    const t5 = t3 * t2;
    const t7 = t5 * t2;

    return 2.0 * (t + t3 / 3.0 + t5 / 5.0 + t7 / 7.0);
}

/// Exponential approximation (Taylor series)
pub fn exp_approx(x: f64) f64 {
    if (x == 0.0) {
        return 1.0;
    }

    var exp_x = x;
    var scale: f64 = 1.0;

    if (x > 10.0) {
        const k: i64 = @intFromFloat(floor(x / 10.0));
        exp_x = x - @as(f64, @floatFromInt(k)) * 10.0;
        scale = pow(E, @as(f64, @floatFromInt(k)) * 10.0);
    } else if (x < -10.0) {
        const k: i64 = @intFromFloat(floor(-x / 10.0));
        exp_x = x + @as(f64, @floatFromInt(k)) * 10.0;
        scale = 1.0 / pow(E, @as(f64, @floatFromInt(k)) * 10.0);
    }

    var result: f64 = 1.0;
    var term: f64 = 1.0;

    for (1..11) |i| {
        term = term * exp_x / @as(f64, @floatFromInt(i));
        result = result + term;
    }

    return result * scale;
}

/// Floor function
pub fn floor(x: f64) f64 {
    const xi: i64 = @intFromFloat(x);
    if (x >= 0.0 or x == @as(f64, @floatFromInt(xi))) {
        return @as(f64, @floatFromInt(xi));
    }
    return @as(f64, @floatFromInt(xi - 1));
}

// ============================================================================
// Tests
// ============================================================================

test "test_phi_squared_plus_inverse_squared_equals_3" {
    const phi_sq = PHI * PHI;
    const phi_inv_sq = PHI_INV * PHI_INV;
    const sum = phi_sq + phi_inv_sq;
    try std.testing.expect(abs(sum - TRINITY) < 1e-12);
}

test "test_phi_inverse_is_phi_minus_one" {
    const expected = PHI - 1.0;
    try std.testing.expect(abs(PHI_INV - expected) < 1e-15);
}

test "test_phi_multiplicative_persistence" {
    const squared = PHI * PHI;
    const result = squared - PHI;
    try std.testing.expect(abs(result - 1.0) < 1e-12);
}

test "test_trinity_constant_accuracy" {
    try std.testing.expect(abs(TRINITY - 3.0) < 1e-15);
}

test "test_pi_range_validity" {
    try std.testing.expect(PI >= 3.1415926535 and PI <= 3.1415926536);
}

test "test_euler_number_range_validity" {
    try std.testing.expect(E >= 2.7182818284 and E <= 2.7182818285);
}

test "test_pow_zero_exponent_returns_one" {
    const result = pow(2.0, 0.0);
    try std.testing.expect(abs(result - 1.0) < 1e-15);
}

test "test_pow_one_exponent_returns_base" {
    const result = pow(5.0, 1.0);
    try std.testing.expect(abs(result - 5.0) < 1e-15);
}

test "test_pow_positive_integer_exponent" {
    const result = pow(2.0, 10.0);
    const expected: f64 = 1024.0;
    try std.testing.expect(abs(result - expected) < 1e-10);
}

test "test_pow_negative_integer_exponent" {
    const result = pow(2.0, -3.0);
    const expected: f64 = 0.125;
    try std.testing.expect(abs(result - expected) < 1e-10);
}

test "test_pow_fractional_exponent" {
    const result = pow(4.0, 0.5);
    const expected: f64 = 2.0;
    try std.testing.expect(abs(result - expected) < 1e-6);
}

test "test_pow_phi_squared" {
    const result = pow(PHI, 2.0);
    const expected = PHI * PHI;
    try std.testing.expect(abs(result - expected) < 1e-10);
}

test "test_pow_zero_base_positive_exponent" {
    const result = pow(0.0, 5.0);
    try std.testing.expect(result == 0.0);
}

test "test_pow_one_base_any_exponent" {
    const result1 = pow(1.0, 10.0);
    const result2 = pow(1.0, -5.0);
    const result3 = pow(1.0, 0.5);
    try std.testing.expect(abs(result1 - 1.0) < 1e-15 and abs(result2 - 1.0) < 1e-15 and abs(result3 - 1.0) < 1e-6);
}

test "test_floor_function_positive" {
    const result = floor(3.7);
    try std.testing.expect(result == 3.0);
}

test "test_floor_function_negative" {
    const result = floor(-3.2);
    try std.testing.expect(result == -4.0);
}

test "test_floor_function_integer" {
    const result = floor(5.0);
    try std.testing.expect(result == 5.0);
}
