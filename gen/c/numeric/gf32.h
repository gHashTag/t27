/* Auto-generated from specs/numeric/gf32.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf32.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GF32_H
#define GF32_H

#include <stdint.h>
#include <stdbool.h>

/* ===================================================================== */
/* GoldenFloat32 -- 32-bit phi-structured floating point                  */
/* Format: [S:1 | E:12 | M:19] -- phi_distance = 0.01354                 */
/* Same memory as FP32, near-optimal phi-ratio                            */
/* 12-bit exponent (vs IEEE 8-bit) for wider dynamic range                */
/* ===================================================================== */

#define GF32_BITS          32
#define GF32_SIGN_BITS     1
#define GF32_EXP_BITS      12
#define GF32_MANT_BITS     19
#define GF32_EXP_BIAS      2047
#define GF32_PHI_DISTANCE  0.01354495894042812
#define GF32_MEMORY_RATIO  1.0f

#define GF32_EXP_MASK      0xFFFu     /* (1 << 12) - 1 = 4095 */
#define GF32_MANT_MASK     0x7FFFFu   /* (1 << 19) - 1 = 524287 */
#define GF32_MANT_SCALE    524288.0f  /* 2^19 */

/* GF32 type: 32-bit raw value */
typedef struct {
    uint32_t raw;
} GF32;

/* Encoding / Decoding */
GF32  gf32_encode(float value);
float gf32_decode(GF32 gf);

/* Format Properties */
float gf32_max_value(void);
float gf32_min_positive(void);
float gf32_epsilon(void);
bool  gf32_validate_format(void);

/* Helper Functions */
int16_t gf32_floor_log2(float x);
float   gf32_pow(float base, float exponent);
float   gf32_ln_approx(float x);
float   gf32_exp_approx(float x);
float   gf32_floor(float x);

/* Test suite */
void test_gf32(void);

#endif /* GF32_H */
