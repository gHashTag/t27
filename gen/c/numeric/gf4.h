/* Auto-generated from specs/numeric/gf4.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf4.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GF4_H
#define GF4_H

#include <stdint.h>
#include <math.h>

/* ========================================================================== */
/* GF4 -- GoldenFloat4: 4-bit phi-structured floating point                  */
/* Bit layout: [S|E|MM]  S:1 E:1 M:2                                         */
/* ========================================================================== */

#define GF4_BITS          4
#define GF4_SIGN_BITS     1
#define GF4_EXP_BITS      1
#define GF4_MANT_BITS     2
#define GF4_EXP_BIAS      0
#define GF4_PHI_DISTANCE  0.1180339887498949
#define GF4_MEMORY_RATIO  0.125f

typedef struct {
    uint8_t raw; /* only lower 4 bits used */
} GF4;

/* Encoding / Decoding */
GF4   gf4_encode(float value);
float gf4_decode(GF4 gf);

/* Format Properties */
float gf4_max_value(void);
float gf4_min_positive(void);
float gf4_epsilon(void);
int   gf4_validate_format(void);

/* Tests */
void test_gf4_decode_zero(void);
void test_gf4_decode_positive_max(void);
void test_gf4_encode_zero_roundtrip(void);
void test_gf4_encode_0_25(void);
void test_gf4_encode_0_5(void);
void test_gf4_encode_1_0(void);
void test_gf4_encode_1_5(void);
void test_gf4_encode_negative(void);
void test_gf4_bits_sum(void);
void test_gf4_validate_format(void);

#endif /* GF4_H */
