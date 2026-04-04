/* Auto-generated from specs/numeric/goldenfloat_family.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/goldenfloat_family.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef GOLDENFLOAT_FAMILY_H
#define GOLDENFLOAT_FAMILY_H

#include <stdint.h>
#include <math.h>

/* ========================================================================== */
/* GoldenFloatFamily -- phi-structured floating point format registry          */
/* Contains all 7 GF formats: GF4, GF8, GF12, GF16, GF20, GF24, GF32        */
/* ========================================================================== */

#define PHI_RATIO_TARGET  0.6180339887498949
#define FAMILY_SIZE       7

typedef struct {
    const char* name;
    uint8_t     bits;
    uint8_t     sign_bits;
    uint8_t     exp_bits;
    uint8_t     mant_bits;
    double      exp_mant_ratio;
    double      phi_distance;
    int         is_primary;
} GoldenFloatFormat;

typedef struct {
    int         all_valid;
    int         primary_is_gf16;
    int         phi_distances_ok;
    const char* best_phi_format;
    double      best_phi_distance;
    double      avg_phi_distance;
} VerificationReport;

/* Family array (defined in .c) */
extern const GoldenFloatFormat GOLDEN_FLOAT_FAMILY[FAMILY_SIZE];

/* Query functions */
const GoldenFloatFormat* gff_get_format_by_name(const char* name);
const GoldenFloatFormat* gff_get_format_by_bits(uint8_t bits);
GoldenFloatFormat gff_get_primary_format(void);

/* Utility functions */
double gff_max_value(const GoldenFloatFormat* fmt);
double gff_min_positive(const GoldenFloatFormat* fmt);
double gff_memory_efficiency(const GoldenFloatFormat* fmt);

/* Verification */
VerificationReport gff_verify_golden_family(void);

/* Tests */
void test_gff_get_format_by_name_gf16(void);
void test_gff_get_format_by_bits_8(void);
void test_gff_get_primary_format(void);
void test_gff_family_size(void);
void test_gff_only_gf16_primary(void);
void test_gff_verify_all_valid(void);
void test_gff_best_phi_format(void);
void test_gff_memory_efficiency_gf8(void);
void test_gff_unknown_name(void);
void test_gff_all_bits_sum_correct(void);

#endif /* GOLDENFLOAT_FAMILY_H */
