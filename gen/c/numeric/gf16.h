/* Auto-generated from specs/numeric/gf16.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf16.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GF16_H
#define GF16_H

#include <stdint.h>
#include <math.h>

/* ========================================================================== */
/* GF16 -- GoldenFloat16: 16-bit phi-structured floating point (PRIMARY)      */
/* Bit layout: [S(1) E(6) M(9)] = [15:15][14:9][8:0]                         */
/* ========================================================================== */

#define GF16_SIGN_SHIFT    15
#define GF16_EXP_SHIFT     9
#define GF16_MANT_SHIFT    0

#define GF16_SIGN_MASK     ((uint16_t)0x8000)
#define GF16_EXP_MASK      ((uint16_t)0x7E00)
#define GF16_MANT_MASK     ((uint16_t)0x01FF)

#define GF16_EXP_MAX       ((uint8_t)0x3F)
#define GF16_BIAS          31
#define GF16_SPECIAL_EXP   ((uint8_t)0x3F)
#define GF16_MANT_DIVISOR  512

#define GF16_PHI_BIAS      60

/* GF16 special values */
#define GF16_ZERO_POS      ((uint16_t)0x0000)
#define GF16_ZERO_NEG      ((uint16_t)0x8000)
#define GF16_INF_POS       ((uint16_t)0x7E00)
#define GF16_INF_NEG       ((uint16_t)0xFE00)
#define GF16_NAN_VAL       ((uint16_t)0xFE01)

typedef uint16_t GF16;

/* Field extraction */
int8_t  gf16_extract_sign(GF16 gf16);
int8_t  gf16_extract_exponent(GF16 gf16);
int16_t gf16_extract_mantissa(GF16 gf16);
GF16    gf16_from_components(int8_t sign, int8_t exp, int16_t mant);

/* Classification */
int gf16_is_zero(GF16 gf16);
int gf16_is_special(GF16 gf16);
int gf16_is_inf(GF16 gf16);
int gf16_is_nan(GF16 gf16);
int gf16_is_negative(GF16 gf16);
int gf16_is_positive(GF16 gf16);
int gf16_is_finite(GF16 gf16);

/* Encoding / Decoding */
GF16  gf16_encode_f32(float value);
float gf16_decode_to_f32(GF16 gf16);
GF16  gf16_round_phi(float value);

/* Unary operations */
GF16 gf16_negate(GF16 gf16);
GF16 gf16_abs_val(GF16 gf16);

/* Arithmetic */
GF16 gf16_add(GF16 a, GF16 b);
GF16 gf16_sub(GF16 a, GF16 b);
GF16 gf16_mul(GF16 a, GF16 b);
GF16 gf16_div(GF16 a, GF16 b);
GF16 gf16_fma(GF16 a, GF16 b, GF16 c);
GF16 gf16_sqrt_val(GF16 a);

/* Comparison */
int  gf16_eq(GF16 a, GF16 b);
int  gf16_lt(GF16 a, GF16 b);
int  gf16_le(GF16 a, GF16 b);
GF16 gf16_max(GF16 a, GF16 b);
GF16 gf16_min(GF16 a, GF16 b);

/* Rounding */
GF16 gf16_floor_val(GF16 a);
GF16 gf16_ceil_val(GF16 a);
GF16 gf16_round_val(GF16 a);

/* Format properties */
float gf16_max_value(void);
float gf16_min_positive(void);
float gf16_epsilon(void);
int   gf16_validate_format(void);

/* Tests */
void test_gf16_zero_encoding(void);
void test_gf16_encode_decode_roundtrip(void);
void test_gf16_add_basic(void);
void test_gf16_mul_basic(void);
void test_gf16_is_nan(void);
void test_gf16_is_inf(void);
void test_gf16_validate_format(void);

#endif /* GF16_H */
