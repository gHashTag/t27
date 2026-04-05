/* Auto-generated from specs/numeric/gf24.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf24.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GF24_H
#define GF24_H

#include <stdint.h>
#include <stdbool.h>

/* ===================================================================== */
/* GoldenFloat24 -- 24-bit phi-structured floating point                  */
/* Format: [S:1 | E:9 | M:14] -- phi_distance = 0.02482                  */
/* 25% memory savings vs FP32                                             */
/* ===================================================================== */

#define GF24_BITS          24
#define GF24_SIGN_BITS     1
#define GF24_EXP_BITS      9
#define GF24_MANT_BITS     14
#define GF24_EXP_BIAS      255
#define GF24_PHI_DISTANCE  0.02482317991669112
#define GF24_MEMORY_RATIO  0.75f

#define GF24_EXP_MASK      0x1FFu    /* (1 << 9) - 1 = 511 */
#define GF24_MANT_MASK     0x3FFFu   /* (1 << 14) - 1 = 16383 */
#define GF24_MANT_SCALE    16384.0f  /* 2^14 */

/* GF24 type: 24-bit value stored in uint32_t */
typedef struct {
    uint32_t raw;
} GF24;

/* Encoding / Decoding */
GF24  gf24_encode(float value);
float gf24_decode(GF24 gf);

/* Format Properties */
float gf24_max_value(void);
float gf24_min_positive(void);
float gf24_epsilon(void);
bool  gf24_validate_format(void);

/* Helper Functions */
int16_t gf24_floor_log2(float x);
float   gf24_pow(float base, float exponent);
float   gf24_ln_approx(float x);
float   gf24_exp_approx(float x);
float   gf24_floor(float x);

/* Test suite */
void test_gf24(void);

#endif /* GF24_H */
