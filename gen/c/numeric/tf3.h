/* Auto-generated from specs/numeric/tf3.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/tf3.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef TF3_H
#define TF3_H

#include <stdint.h>
#include <math.h>

/* ========================================================================== */
/* TF3 -- Ternary Float 3: 8-bit ternary neural network weight format         */
/* Bit layout: [S(1) E(3) M(4)] = [7:7][6:4][3:0]                            */
/* ========================================================================== */

#define TF3_SIGN_SHIFT  7
#define TF3_EXP_SHIFT   4
#define TF3_MANT_SHIFT  0

#define TF3_SIGN_MASK   ((uint8_t)0x80)
#define TF3_EXP_MASK    ((uint8_t)0x70)
#define TF3_MANT_MASK   ((uint8_t)0x0F)

#define TF3_EXP_MAX     ((uint8_t)0x07)
#define TF3_BIAS        3
#define TF3_MANT_BITS   4

/* Special values */
#define TF3_ZERO_POS    ((uint8_t)0x00)
#define TF3_ZERO_NEG    ((uint8_t)0x80)
#define TF3_INF_POS     ((uint8_t)0x70)
#define TF3_INF_NEG     ((uint8_t)0xF0)

typedef uint8_t TF3;

/* Field extraction */
int8_t tf3_extract_sign(TF3 tf3);
int8_t tf3_extract_exponent(TF3 tf3);
int8_t tf3_extract_mantissa(TF3 tf3);
TF3    tf3_from_components(int8_t sign, int8_t exp, int8_t mant);

/* Classification */
int tf3_is_zero(TF3 tf3);
int tf3_is_inf(TF3 tf3);
int tf3_is_negative(TF3 tf3);
int tf3_is_positive(TF3 tf3);

/* Encoding / Decoding */
TF3   tf3_from_f32(float value);
float tf3_to_f32(TF3 tf3);

/* Unary */
TF3 tf3_negate(TF3 tf3);
TF3 tf3_abs_val(TF3 tf3);

/* Arithmetic */
TF3 tf3_add(TF3 a, TF3 b);
TF3 tf3_sub(TF3 a, TF3 b);
TF3 tf3_mul(TF3 a, TF3 b);
TF3 tf3_div(TF3 a, TF3 b);

/* Comparison */
int tf3_eq(TF3 a, TF3 b);
int tf3_lt(TF3 a, TF3 b);
TF3 tf3_max(TF3 a, TF3 b);
TF3 tf3_min(TF3 a, TF3 b);

/* Format properties */
int tf3_validate_format(void);

/* Tests */
void test_tf3_is_zero(void);
void test_tf3_inf_encoding(void);
void test_tf3_zero_roundtrip(void);
void test_tf3_positive_roundtrip(void);
void test_tf3_negate(void);
void test_tf3_validate_format(void);

#endif /* TF3_H */
