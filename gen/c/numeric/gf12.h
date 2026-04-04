/* Auto-generated from specs/numeric/gf12.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf12.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GF12_H
#define GF12_H

#include <stdint.h>
#include <math.h>

/* ========================================================================== */
/* GF12 -- GoldenFloat12: 12-bit phi-structured floating point               */
/* Bit layout: [S|EEEE|MMM MMMM]  S:1 E:4 M:7                               */
/* ========================================================================== */

#define GF12_BITS          12
#define GF12_SIGN_BITS     1
#define GF12_EXP_BITS      4
#define GF12_MANT_BITS     7
#define GF12_EXP_BIAS      7
#define GF12_PHI_DISTANCE  0.04660512288042107
#define GF12_MEMORY_RATIO  0.375f

typedef struct {
    uint16_t raw; /* only lower 12 bits used */
} GF12;

/* Helper functions */
int8_t  gf12_floor_log2(float x);
uint8_t gf12_extract_mantissa(float value, int8_t exp, uint8_t mant_bits);
float   gf12_pow(float base, float exp);
float   gf12_ln_approx(float x);
float   gf12_exp_approx(float x);
float   gf12_floor(float x);

/* Encoding / Decoding */
GF12  gf12_encode(float value);
float gf12_decode(GF12 gf);

/* Format Properties */
float gf12_max_value(void);
float gf12_min_positive(void);
float gf12_epsilon(void);
int   gf12_validate_format(void);

/* Tests */
void test_gf12_decode_zero(void);
void test_gf12_encode_zero_roundtrip(void);
void test_gf12_bits_sum(void);
void test_gf12_max_value_positive(void);
void test_gf12_validate_format(void);
void test_gf12_floor_log2_power_of_two(void);

#endif /* GF12_H */
