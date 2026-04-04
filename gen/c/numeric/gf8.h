/* Auto-generated from specs/numeric/gf8.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf8.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GF8_H
#define GF8_H

#include <stdint.h>
#include <math.h>

/* ========================================================================== */
/* GF8 -- GoldenFloat8: 8-bit phi-structured floating point                  */
/* Bit layout: [S|EEE|MMMM]  S:1 E:3 M:4                                    */
/* ========================================================================== */

#define GF8_BITS          8
#define GF8_SIGN_BITS     1
#define GF8_EXP_BITS      3
#define GF8_MANT_BITS     4
#define GF8_EXP_BIAS      3
#define GF8_PHI_DISTANCE  0.1319660112501052
#define GF8_MEMORY_RATIO  0.25f

typedef struct {
    uint8_t raw;
} GF8;

/* Helper functions */
int8_t  gf8_floor_log2(float x);
uint8_t gf8_extract_mantissa(float value, int8_t exp, uint8_t mant_bits);
uint8_t gf8_clamp(uint8_t x, uint8_t min_val, uint8_t max_val);
float   gf8_pow(float base, float exp);
float   gf8_ln_approx(float x);
float   gf8_exp_approx(float x);
float   gf8_floor(float x);

/* Encoding / Decoding */
GF8   gf8_encode(float value);
float gf8_decode(GF8 gf);

/* Format Properties */
float gf8_max_value(void);
float gf8_min_positive(void);
float gf8_epsilon(void);
int   gf8_validate_format(void);

/* Tests */
void test_gf8_decode_zero(void);
void test_gf8_encode_zero_roundtrip(void);
void test_gf8_bits_sum(void);
void test_gf8_max_value_positive(void);
void test_gf8_validate_format(void);
void test_gf8_pow_zero_exponent(void);
void test_gf8_ln_approx_one(void);
void test_gf8_floor_positive(void);

#endif /* GF8_H */
