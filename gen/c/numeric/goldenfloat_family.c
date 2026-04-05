/* Auto-generated from specs/numeric/goldenfloat_family.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/numeric/goldenfloat_family.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "goldenfloat_family.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

/* ========================================================================== */
/* Family Array                                                               */
/* ========================================================================== */

const GoldenFloatFormat GOLDEN_FLOAT_FAMILY[FAMILY_SIZE] = {
    { "GF4",  4,  1,  1,  2, 0.5,                  0.1180339887498949, 0 },
    { "GF8",  8,  1,  3,  4, 0.75,                 0.1319660112501052, 0 },
    { "GF12", 12, 1,  4,  7, 0.5714285714285714,   0.0466051228804211, 0 },
    { "GF16", 16, 1,  6,  9, 0.6666666666666667,   0.0486326779167718, 1 },
    { "GF20", 20, 1,  7, 12, 0.5833333333333333,   0.0346993445834384, 0 },
    { "GF24", 24, 1,  9, 14, 0.6428571428571429,   0.0248231541072480, 0 },
    { "GF32", 32, 1, 12, 19, 0.6315789473684210,   0.0135449586185261, 0 },
};

/* ========================================================================== */
/* Query Functions                                                            */
/* ========================================================================== */

const GoldenFloatFormat* gff_get_format_by_name(const char* name) {
    for (int i = 0; i < FAMILY_SIZE; i++) {
        if (strcmp(GOLDEN_FLOAT_FAMILY[i].name, name) == 0) {
            return &GOLDEN_FLOAT_FAMILY[i];
        }
    }
    return NULL;
}

const GoldenFloatFormat* gff_get_format_by_bits(uint8_t bits) {
    for (int i = 0; i < FAMILY_SIZE; i++) {
        if (GOLDEN_FLOAT_FAMILY[i].bits == bits) {
            return &GOLDEN_FLOAT_FAMILY[i];
        }
    }
    return NULL;
}

GoldenFloatFormat gff_get_primary_format(void) {
    return GOLDEN_FLOAT_FAMILY[3]; /* GF16 */
}

/* ========================================================================== */
/* Utility Functions                                                          */
/* ========================================================================== */

double gff_max_value(const GoldenFloatFormat* fmt) {
    double mant_max = 2.0 - pow(2.0, -(double)fmt->mant_bits);
    double exp_max = pow(2.0, (double)fmt->exp_bits) - 1.0;
    return mant_max * pow(2.0, exp_max);
}

double gff_min_positive(const GoldenFloatFormat* fmt) {
    double mant_min = pow(2.0, -(double)fmt->mant_bits);
    double bias = pow(2.0, (double)fmt->exp_bits - 1.0) - 1.0;
    return mant_min * pow(2.0, 1.0 - bias);
}

double gff_memory_efficiency(const GoldenFloatFormat* fmt) {
    return (double)fmt->bits / 32.0;
}

/* ========================================================================== */
/* Verification                                                               */
/* ========================================================================== */

VerificationReport gff_verify_golden_family(void) {
    VerificationReport report;
    int primary_count = 0;
    double best_dist = 1.0;
    const char* best_name = "";
    double total_dist = 0.0;
    int all_bit_sums_valid = 1;
    int all_phi_non_neg = 1;

    for (int i = 0; i < FAMILY_SIZE; i++) {
        const GoldenFloatFormat* fmt = &GOLDEN_FLOAT_FAMILY[i];
        if (fmt->is_primary) primary_count++;
        if (fmt->phi_distance < best_dist) {
            best_dist = fmt->phi_distance;
            best_name = fmt->name;
        }
        total_dist += fmt->phi_distance;
        if (fmt->sign_bits + fmt->exp_bits + fmt->mant_bits != fmt->bits) {
            all_bit_sums_valid = 0;
        }
        if (fmt->phi_distance < 0.0) {
            all_phi_non_neg = 0;
        }
    }

    report.all_valid = all_bit_sums_valid && all_phi_non_neg && (primary_count == 1);
    report.primary_is_gf16 = (primary_count == 1) && GOLDEN_FLOAT_FAMILY[3].is_primary;
    report.phi_distances_ok = best_dist < 0.1;
    report.best_phi_format = best_name;
    report.best_phi_distance = best_dist;
    report.avg_phi_distance = total_dist / 7.0;

    return report;
}

/* ========================================================================== */
/* Tests                                                                      */
/* ========================================================================== */

void test_gff_get_format_by_name_gf16(void) {
    const GoldenFloatFormat* fmt = gff_get_format_by_name("GF16");
    assert(fmt != NULL);
    assert(fmt->bits == 16);
    assert(fmt->is_primary);
    printf("PASS: gff_get_format_by_name_gf16\n");
}

void test_gff_get_format_by_bits_8(void) {
    const GoldenFloatFormat* fmt = gff_get_format_by_bits(8);
    assert(fmt != NULL);
    assert(strcmp(fmt->name, "GF8") == 0);
    printf("PASS: gff_get_format_by_bits_8\n");
}

void test_gff_get_primary_format(void) {
    GoldenFloatFormat primary = gff_get_primary_format();
    assert(strcmp(primary.name, "GF16") == 0);
    assert(primary.is_primary);
    printf("PASS: gff_get_primary_format\n");
}

void test_gff_family_size(void) {
    assert(FAMILY_SIZE == 7);
    printf("PASS: gff_family_size\n");
}

void test_gff_only_gf16_primary(void) {
    int count = 0;
    for (int i = 0; i < FAMILY_SIZE; i++) {
        if (GOLDEN_FLOAT_FAMILY[i].is_primary) count++;
    }
    assert(count == 1);
    printf("PASS: gff_only_gf16_primary\n");
}

void test_gff_verify_all_valid(void) {
    VerificationReport report = gff_verify_golden_family();
    assert(report.all_valid);
    assert(report.primary_is_gf16);
    printf("PASS: gff_verify_all_valid\n");
}

void test_gff_best_phi_format(void) {
    VerificationReport report = gff_verify_golden_family();
    assert(strcmp(report.best_phi_format, "GF32") == 0 ||
           strcmp(report.best_phi_format, "GF12") == 0);
    assert(report.best_phi_distance < 0.05);
    printf("PASS: gff_best_phi_format\n");
}

void test_gff_memory_efficiency_gf8(void) {
    const GoldenFloatFormat* fmt = gff_get_format_by_name("GF8");
    double eff = gff_memory_efficiency(fmt);
    assert(fabs(eff - 0.25) < 0.01);
    printf("PASS: gff_memory_efficiency_gf8\n");
}

void test_gff_unknown_name(void) {
    assert(gff_get_format_by_name("GF999") == NULL);
    printf("PASS: gff_unknown_name\n");
}

void test_gff_all_bits_sum_correct(void) {
    for (int i = 0; i < FAMILY_SIZE; i++) {
        const GoldenFloatFormat* fmt = &GOLDEN_FLOAT_FAMILY[i];
        assert(fmt->sign_bits + fmt->exp_bits + fmt->mant_bits == fmt->bits);
    }
    printf("PASS: gff_all_bits_sum_correct\n");
}
