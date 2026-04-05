/* Auto-generated from specs/base/ops.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/base/ops.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef TRITYPE_OPS_H
#define TRITYPE_OPS_H

#include <stdint.h>
#include <stdbool.h>

/* ========================================================================== */
/* Constants                                                                   */
/* ========================================================================== */

#define OPS_NEGONE  ((int8_t)-1)
#define OPS_ZERO    ((int8_t)0)
#define OPS_ONE     ((int8_t)1)

typedef int8_t Trit;

/* ========================================================================== */
/* Lookup Tables                                                               */
/* ========================================================================== */

extern const int8_t mult_table[9];
extern const int8_t add_table[9];
extern const int8_t carry_table[9];

/* ========================================================================== */
/* Types                                                                       */
/* ========================================================================== */

typedef struct {
    Trit result;
    Trit carry_out;
} AddResult;

/* ========================================================================== */
/* Function Declarations                                                       */
/* ========================================================================== */

Trit      trit_multiply_table(Trit a, Trit b);
Trit      trit_add_table(Trit a, Trit b);
Trit      trit_carry_table_fn(Trit a, Trit b);
AddResult trit_add_with_carry(Trit a, Trit b, Trit carry_in);
int8_t    trit_compare(Trit a, Trit b);
Trit      trit_negate(Trit a);
Trit      trit_abs(Trit a);
Trit      trit_min(Trit a, Trit b);
Trit      trit_max(Trit a, Trit b);
Trit      trit_subtract(Trit a, Trit b);
int8_t    trit_sign(Trit a);
Trit      trit_clamp(Trit a, Trit min_val, Trit max_val);
bool      trit_is_negative(Trit a);
bool      trit_is_zero(Trit a);
bool      trit_is_positive(Trit a);
bool      trit_equal(Trit a, Trit b);
bool      trit_not_equal(Trit a, Trit b);
bool      trit_lt(Trit a, Trit b);
bool      trit_le(Trit a, Trit b);
bool      trit_gt(Trit a, Trit b);
bool      trit_ge(Trit a, Trit b);
AddResult trit_multiply_with_carry(Trit a, Trit b, Trit carry_in);
Trit      trit_reverse(Trit a);
Trit      trit_multiply_by_power_of_two(Trit a, uint8_t power);
Trit      trit_power(Trit a, uint8_t n);
Trit      trit_from_bool(bool b);
bool      trit_to_bool(Trit a);
Trit      trit_abs_diff(Trit a, Trit b);
Trit      trit_cond_swap(Trit cond, Trit a, Trit b);
bool      trit_is_unit(Trit a);
bool      trit_is_identity(Trit a);
bool      trit_is_negated(Trit a, Trit b);

/* Test functions */
void test_trit_multiply_table_all_combinations(void);
void test_trit_add_table_all(void);
void test_trit_carry_table_all(void);
void test_trit_add_with_carry_basic(void);
void test_trit_compare_all(void);
void test_trit_negate_involutive(void);
void test_trit_subtract_all(void);

#endif /* TRITYPE_OPS_H */
