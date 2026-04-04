/* Auto-generated from specs/numeric/gf20.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf20.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GF20_H
#define GF20_H

#include <stdint.h>
#include <stdbool.h>

/* ===================================================================== */
/* GoldenFloat20 -- 20-bit phi-structured floating point                  */
/* Format: [S:1 | E:7 | M:12] -- phi_distance = 0.03463                  */
/* 38% memory savings vs FP32                                             */
/* ===================================================================== */

#define GF20_BITS          20
#define GF20_SIGN_BITS     1
#define GF20_EXP_BITS      7
#define GF20_MANT_BITS     12
#define GF20_EXP_BIAS      63
#define GF20_PHI_DISTANCE  0.03463264154356299
#define GF20_MEMORY_RATIO  0.625f

#define GF20_EXP_MASK      0x7Fu   /* (1 << 7) - 1 = 127 */
#define GF20_MANT_MASK     0xFFFu  /* (1 << 12) - 1 = 4095 */
#define GF20_MANT_SCALE    4096.0f /* 2^12 */

/* GF20 type: 20-bit value stored in uint32_t */
typedef struct {
    uint32_t raw;
} GF20;

/* Encoding / Decoding */
GF20  gf20_encode(float value);
float gf20_decode(GF20 gf);

/* Format Properties */
float gf20_max_value(void);
float gf20_min_positive(void);
float gf20_epsilon(void);
bool  gf20_validate_format(void);

/* Helper Functions */
int16_t gf20_floor_log2(float x);
float   gf20_pow(float base, float exponent);
float   gf20_ln_approx(float x);
float   gf20_exp_approx(float x);
float   gf20_floor(float x);

/* Test suite */
void test_gf20(void);

#endif /* GF20_H */
