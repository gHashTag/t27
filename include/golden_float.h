#ifndef GOLDEN_FLOAT_H
#define GOLDEN_FLOAT_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <math.h>

/**
 * Encode f32 → GF16 via integer bit manipulation.
 * FPGA-ALLOWED: f32 only at API boundary; .to_bits() extracts u32 immediately.
 */
uint16_t gf16_from_f32(float x);

/**
 * Decode GF16 → f32 via integer bit reconstruction.
 * FPGA-ALLOWED: f32 only at API boundary exit; from_bits() reconstructs from u32.
 */
float gf16_to_f32(uint16_t value);

/**
 * GF16 addition via decode→add→encode using intermediate u32 IEEE 754 bits.
 * No f32/f64 arithmetic — uses from_bits/to_bits only at transition.
 */
uint16_t gf16_add(uint16_t a, uint16_t b);

uint16_t gf16_sub(uint16_t a, uint16_t b);

uint16_t gf16_mul(uint16_t a, uint16_t b);

uint16_t gf16_div(uint16_t a, uint16_t b);

bool gf16_eq(uint16_t a, uint16_t b);

bool gf16_lt(uint16_t a, uint16_t b);

bool gf16_is_zero(uint16_t value);

bool gf16_is_inf(uint16_t value);

bool gf16_is_nan(uint16_t value);

uint8_t gf16_extract_sign(uint16_t value);

uint8_t gf16_extract_exponent(uint16_t value);

int16_t gf16_extract_mantissa(uint16_t value);

/**
 * FPGA-ALLOWED: f64 at API boundary only — converts to f32 via bits, then encodes.
 */
uint16_t gf16_from_f64(double x);

/**
 * FPGA-ALLOWED: f64 at API boundary only.
 */
double gf16_to_f64(uint16_t value);

/**
 * FPGA-ALLOWED: f64 at boundary; extract u64 bits then truncate to f32 range.
 */
uint32_t gf32_from_f64(double x);

/**
 * FPGA-ALLOWED: f64 at API boundary only.
 */
double gf32_to_f64(uint32_t value);

bool gf32_is_zero(uint32_t value);

bool gf32_is_inf(uint32_t value);

bool gf32_is_nan(uint32_t value);

uint8_t gf32_extract_sign(uint32_t value);

uint8_t gf32_extract_exponent(uint32_t value);

int32_t gf32_extract_mantissa(uint32_t value);

uint32_t gf32_add(uint32_t a, uint32_t b);

uint32_t gf32_sub(uint32_t a, uint32_t b);

uint32_t gf32_mul(uint32_t a, uint32_t b);

uint32_t gf32_div(uint32_t a, uint32_t b);

bool gf32_eq(uint32_t a, uint32_t b);

bool gf32_lt(uint32_t a, uint32_t b);

int8_t gf4_extract_sign(uint8_t value);

int8_t gf4_extract_exponent(uint8_t value);

int16_t gf4_extract_mantissa(uint8_t value);

int8_t gf8_extract_sign(uint8_t value);

int8_t gf8_extract_exponent(uint8_t value);

int16_t gf8_extract_mantissa(uint8_t value);

int8_t gf12_extract_sign(uint16_t value);

int8_t gf12_extract_exponent(uint16_t value);

int16_t gf12_extract_mantissa(uint16_t value);

int8_t gf20_extract_sign(uint32_t value);

int8_t gf20_extract_exponent(uint32_t value);

int16_t gf20_extract_mantissa(uint32_t value);

int8_t gf24_extract_sign(uint32_t value);

int8_t gf24_extract_exponent(uint32_t value);

int16_t gf24_extract_mantissa(uint32_t value);

#endif  /* GOLDEN_FLOAT_H */
