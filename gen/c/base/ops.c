/* Auto-generated from specs/base/ops.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/base/ops.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "ops.h"
#include <assert.h>

/* ========================================================================== */
/* Lookup Tables                                                               */
/* ========================================================================== */

/*    -1  0 +1
 * -1 +1  0 -1
 *  0  0  0  0
 * +1 -1  0 +1
 */
const int8_t mult_table[9] = { 1, 0, -1, 0, 0, 0, -1, 0, 1 };

/*    -1  0 +1
 * -1 -1 -1  0
 *  0 -1  0 +1
 * +1  0 +1 +1
 */
const int8_t add_table[9] = { -1, -1, 0, -1, 0, 1, 0, 1, 1 };

/* Carry: -1+-1 gives carry +1, +1++1 gives carry -1, rest 0 */
const int8_t carry_table[9] = { 1, 0, 0, 0, 0, 0, 0, 0, -1 };

/* ========================================================================== */
/* Helper: table index                                                         */
/* ========================================================================== */

static inline int trit_idx(Trit a, Trit b) {
    return (a + 1) * 3 + (b + 1);
}

/* ========================================================================== */
/* Function Implementations                                                    */
/* ========================================================================== */

Trit trit_multiply_table(Trit a, Trit b) {
    return mult_table[trit_idx(a, b)];
}

Trit trit_add_table(Trit a, Trit b) {
    return add_table[trit_idx(a, b)];
}

Trit trit_carry_table_fn(Trit a, Trit b) {
    return carry_table[trit_idx(a, b)];
}

AddResult trit_add_with_carry(Trit a, Trit b, Trit carry_in) {
    AddResult res;
    int8_t sum = a + b;
    Trit carry = 0;
    Trit result = 0;

    if (sum > 1) {
        result = -1;
        carry = 1;
    } else if (sum < -1) {
        result = 1;
        carry = -1;
    } else {
        result = sum;
    }

    sum = result + carry_in;
    if (sum > 1) {
        result = -1;
        carry = 1;
    } else if (sum < -1) {
        result = 1;
        carry = -1;
    } else {
        result = sum;
        carry = 0;
    }

    res.result = result;
    res.carry_out = carry;
    return res;
}

int8_t trit_compare(Trit a, Trit b) {
    if (a == b) return 0;
    if (a == -1 || (a == 0 && b == 1)) return -1;
    return 1;
}

Trit trit_negate(Trit a) {
    switch (a) {
        case -1: return 1;
        case  0: return 0;
        case  1: return -1;
        default: return 0;
    }
}

Trit trit_abs(Trit a) {
    return (a == -1) ? 1 : a;
}

Trit trit_min(Trit a, Trit b) {
    return (a == -1 || (a == 0 && b == 1)) ? a : b;
}

Trit trit_max(Trit a, Trit b) {
    return (a == 1 || (a == 0 && b == -1)) ? a : b;
}

Trit trit_subtract(Trit a, Trit b) {
    return trit_add_table(a, trit_negate(b));
}

int8_t trit_sign(Trit a) {
    return a;
}

Trit trit_clamp(Trit a, Trit min_val, Trit max_val) {
    if (trit_compare(a, min_val) < 0) return min_val;
    if (trit_compare(a, max_val) > 0) return max_val;
    return a;
}

bool trit_is_negative(Trit a) { return a == -1; }
bool trit_is_zero(Trit a)     { return a == 0; }
bool trit_is_positive(Trit a) { return a == 1; }

bool trit_equal(Trit a, Trit b)     { return a == b; }
bool trit_not_equal(Trit a, Trit b) { return a != b; }

bool trit_lt(Trit a, Trit b) { return trit_compare(a, b) < 0; }
bool trit_le(Trit a, Trit b) { return trit_compare(a, b) <= 0; }
bool trit_gt(Trit a, Trit b) { return trit_compare(a, b) > 0; }
bool trit_ge(Trit a, Trit b) { return trit_compare(a, b) >= 0; }

AddResult trit_multiply_with_carry(Trit a, Trit b, Trit carry_in) {
    Trit product = trit_multiply_table(a, b);
    return trit_add_with_carry(product, carry_in, 0);
}

Trit trit_reverse(Trit a) {
    return (a == 0) ? 0 : a;
}

Trit trit_multiply_by_power_of_two(Trit a, uint8_t power) {
    Trit result = a;
    for (uint8_t i = 1; i < power; i++) {
        Trit carry = trit_carry_table_fn(result, a);
        if (carry != 0) {
            result = (carry == 1) ? 1 : -1;
        }
        result = trit_add_table(result, a);
    }
    return result;
}

Trit trit_power(Trit a, uint8_t n) {
    if (n == 0) return 1;
    if (n == 1) return a;
    if (a == 0) return 0;
    if (a == 1) return 1;
    /* a == -1: (-1)^n = -1 if odd, +1 if even */
    return (n % 2 == 1) ? -1 : 1;
}

Trit trit_from_bool(bool b) { return b ? 1 : 0; }
bool trit_to_bool(Trit a)   { return a == 1; }

Trit trit_abs_diff(Trit a, Trit b) {
    return (a == b) ? 0 : 1;
}

Trit trit_cond_swap(Trit cond, Trit a, Trit b) {
    return (cond == 1) ? b : a;
}

bool trit_is_unit(Trit a)     { return a == 1; }
bool trit_is_identity(Trit a) { return a == 0; }

bool trit_is_negated(Trit a, Trit b) {
    return b == trit_negate(a);
}

/* ========================================================================== */
/* Test Functions                                                              */
/* ========================================================================== */

void test_trit_multiply_table_all_combinations(void) {
    assert(trit_multiply_table(-1, -1) ==  1);
    assert(trit_multiply_table(-1,  0) ==  0);
    assert(trit_multiply_table(-1,  1) == -1);
    assert(trit_multiply_table( 0, -1) ==  0);
    assert(trit_multiply_table( 0,  0) ==  0);
    assert(trit_multiply_table( 0,  1) ==  0);
    assert(trit_multiply_table( 1, -1) == -1);
    assert(trit_multiply_table( 1,  0) ==  0);
    assert(trit_multiply_table( 1,  1) ==  1);
}

void test_trit_add_table_all(void) {
    assert(trit_add_table(-1, -1) == -1);
    assert(trit_add_table(-1,  0) == -1);
    assert(trit_add_table(-1,  1) ==  0);
    assert(trit_add_table( 0,  0) ==  0);
    assert(trit_add_table( 0,  1) ==  1);
    assert(trit_add_table( 1,  1) ==  1);
}

void test_trit_carry_table_all(void) {
    assert(trit_carry_table_fn(-1, -1) ==  1);
    assert(trit_carry_table_fn(-1,  0) ==  0);
    assert(trit_carry_table_fn(-1,  1) ==  0);
    assert(trit_carry_table_fn( 0,  0) ==  0);
    assert(trit_carry_table_fn( 0,  1) ==  0);
    assert(trit_carry_table_fn( 1,  1) == -1);
}

void test_trit_add_with_carry_basic(void) {
    AddResult r;

    r = trit_add_with_carry(1, -1, 0);
    assert(r.result == 0 && r.carry_out == 0);

    r = trit_add_with_carry(1, 1, 0);
    assert(r.result == -1 && r.carry_out == 1);

    r = trit_add_with_carry(-1, -1, 0);
    assert(r.result == 1 && r.carry_out == -1);

    r = trit_add_with_carry(1, 1, 1);
    assert(r.result == 0 && r.carry_out == 1);
}

void test_trit_compare_all(void) {
    assert(trit_compare(-1,  0) == -1);
    assert(trit_compare(-1,  1) == -1);
    assert(trit_compare( 0,  1) == -1);
    assert(trit_compare(-1, -1) ==  0);
    assert(trit_compare( 0,  0) ==  0);
    assert(trit_compare( 1,  1) ==  0);
    assert(trit_compare( 1,  0) ==  1);
    assert(trit_compare( 1, -1) ==  1);
    assert(trit_compare( 0, -1) ==  1);
}

void test_trit_negate_involutive(void) {
    Trit trits[] = { -1, 0, 1 };
    for (int i = 0; i < 3; i++) {
        assert(trit_negate(trit_negate(trits[i])) == trits[i]);
    }
}

void test_trit_subtract_all(void) {
    Trit trits[] = { -1, 0, 1 };
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            assert(trit_subtract(trits[i], trits[j]) ==
                   trit_add_table(trits[i], trit_negate(trits[j])));
        }
    }
}
