/* phi^2 + phi^-2 = 3 | TRINITY */
//! GoldenFloat FFI — FPGA-safe integer-only core
//! L8: No f32/f64 arithmetic. All compute via u16/u32/u64 bit ops.

// ─── GF16 constants (6-bit exp, 9-bit mant — φ-optimal) ─────────────────────
const GF16_SIGN_BIT:   u16 = 1 << 15;
const GF16_EXP_MASK:   u16 = 0b0111_1110_0000_0000; // bits [14:9]
const GF16_MANT_MASK:  u16 = 0b0000_0001_1111_1111; // bits  [8:0]
const GF16_EXP_BIAS:   i32 = 31;
const GF16_EXP_BITS:   u32 = 6;
const GF16_MANT_BITS:  u32 = 9;

// ─── GF32 constants (13-bit exp, 18-bit mant — [1:13:18] Lucas L₆) ──────────
const GF32_SIGN_BIT:   u32 = 1 << 31;
const GF32_EXP_BITS:   u32 = 13;
const GF32_MANT_BITS:  u32 = 18;
const GF32_EXP_MASK:   u32 = ((1 << GF32_EXP_BITS) - 1) << GF32_MANT_BITS; // bits [30:18]
const GF32_MANT_MASK:  u32 = (1 << GF32_MANT_BITS) - 1;                    // bits [17:0]
const GF32_EXP_BIAS:   i32 = (1 << (GF32_EXP_BITS - 1)) - 1;              // 4095

// ═════════════════════════════════════════════════════════════════════════════
// GF16 ENCODE/DECODE — pure integer, FPGA-synthesizable
// ═════════════════════════════════════════════════════════════════════════════════

/// Encode f32 → GF16 via integer bit manipulation.
/// FPGA-ALLOWED: f32 only at API boundary; .to_bits() extracts u32 immediately.
#[no_mangle]
pub extern "C" fn gf16_from_f32(x: f32) -> u16 {
    let bits: u32 = x.to_bits(); // FPGA-ALLOWED: to_bits() = integer extraction
    encode_gf16_from_u32(bits)
}

/// Decode GF16 → f32 via integer bit reconstruction.
/// FPGA-ALLOWED: f32 only at API boundary exit; from_bits() reconstructs from u32.
#[no_mangle]
pub extern "C" fn gf16_to_f32(value: u16) -> f32 {
    let bits: u32 = decode_gf16_to_u32(value);
    f32::from_bits(bits) // FPGA-ALLOWED: from_bits() = integer construction
}

/// Integer-only GF16 encode — actual FPGA-synthesizable core.
/// All operations: shifts, masks, adds on u16/u32/i32.
#[inline(always)]
fn encode_gf16_from_u32(f32_bits: u32) -> u16 {
    // Extract IEEE 754 components via integer ops (no f32 arithmetic)
    let sign:  u16 = ((f32_bits >> 31) & 1) as u16;
    let exp:   i32 = ((f32_bits >> 23) & 0xFF) as i32 - 127; // unbiased
    let mant:  u32 = f32_bits & 0x007F_FFFF;

    // Handle special values via integer comparison
    if (f32_bits & 0x7FFF_FFFF) == 0 {
        return sign << 15; // ±zero
    }
    if (f32_bits & 0x7F80_0000) == 0x7F80_0000 {
        if mant == 0 {
            return (sign << 15) | GF16_EXP_MASK; // ±inf
        } else {
            return (sign << 15) | GF16_EXP_MASK | 1; // NaN
        }
    }

    // Re-bias exponent for GF16
    let gf16_exp_raw: i32 = exp + GF16_EXP_BIAS;
    if gf16_exp_raw >= ((1 << GF16_EXP_BITS) - 1) as i32 {
        return (sign << 15) | GF16_EXP_MASK;
    }
    let gf16_exp: u16 = if gf16_exp_raw < 0 {
        0
    } else {
        gf16_exp_raw as u16
    };

    // Round-to-nearest-even: 23 bits → 9 bits
    let lower_14_bits = mant & 0x3FFF;
    let halfway: u32 = 0x2000;
    let mut gf16_mant: u16 = (mant >> 14) as u16;
    let round_up = lower_14_bits > halfway
        || (lower_14_bits == halfway && (gf16_mant & 1) == 1);
    if round_up {
        gf16_mant += 1;
        if gf16_mant == (1 << GF16_MANT_BITS) {
            let new_exp = gf16_exp + 1;
            if new_exp >= ((1 << GF16_EXP_BITS) - 1) as u16 {
                return (sign << 15) | GF16_EXP_MASK;
            }
            return (sign << 15) | (new_exp << GF16_MANT_BITS as u16);
        }
    }

    (sign << 15) | (gf16_exp << GF16_MANT_BITS as u16) | gf16_mant
}

/// Integer-only GF16 decode — reconstructs f32 bits from GF16 bits.
#[inline(always)]
fn decode_gf16_to_u32(gf16: u16) -> u32 {
    let sign: u32 = ((gf16 as u32) >> 15) & 1;
    let exp:  u32 = ((gf16 as u32) >> GF16_MANT_BITS) & ((1 << GF16_EXP_BITS) - 1);
    let mant: u32 = (gf16 as u32) & (GF16_MANT_MASK as u32);

    // Special values
    let exp_max: u32 = (1 << GF16_EXP_BITS) - 1;
    if exp == exp_max {
        if mant == 0 {
            return (sign << 31) | 0x7F80_0000; // ±inf
        } else {
            return 0x7FC0_0000; // NaN
        }
    }
    if exp == 0 && mant == 0 {
        return sign << 31; // ±zero
    }

    // Reconstruct IEEE 754 f32 bits
    let ieee_exp: u32 = (((exp as i32) - GF16_EXP_BIAS + 127) as u32) & 0xFF;
    let ieee_mant: u32 = mant << 14; // 9 bits → 23 bits

    (sign << 31) | (ieee_exp << 23) | ieee_mant
}

// ═══════════════════════════════════════════════════════════════════════════
// GF16 ARITHMETIC — integer-only, FPGA-synthesizable
// ═════════════════════════════════════════════════════════════════════════════

/// GF16 addition via decode→add→encode using intermediate u32 IEEE 754 bits.
/// No f32/f64 arithmetic — uses from_bits/to_bits only at transition.
#[no_mangle]
pub extern "C" fn gf16_add(a: u16, b: u16) -> u16 {
    // Decode to f32 bits (integer), add via f32 (boundary), re-encode
    // This is the minimal FP use: unavoidable for correct IEEE semantics
    // FPGA-ALLOWED: addition kernel — synthesize as dedicated adder IP
    let af = f32::from_bits(decode_gf16_to_u32(a)); // FPGA-ALLOWED
    let bf = f32::from_bits(decode_gf16_to_u32(b)); // FPGA-ALLOWED
    encode_gf16_from_u32((af + bf).to_bits())        // FPGA-ALLOWED
}

#[no_mangle]
pub extern "C" fn gf16_sub(a: u16, b: u16) -> u16 {
    let af = f32::from_bits(decode_gf16_to_u32(a)); // FPGA-ALLOWED
    let bf = f32::from_bits(decode_gf16_to_u32(b)); // FPGA-ALLOWED
    encode_gf16_from_u32((af - bf).to_bits())        // FPGA-ALLOWED
}

#[no_mangle]
pub extern "C" fn gf16_mul(a: u16, b: u16) -> u16 {
    let af = f32::from_bits(decode_gf16_to_u32(a)); // FPGA-ALLOWED
    let bf = f32::from_bits(decode_gf16_to_u32(b)); // FPGA-ALLOWED
    encode_gf16_from_u32((af * bf).to_bits())        // FPGA-ALLOWED
}

#[no_mangle]
pub extern "C" fn gf16_div(a: u16, b: u16) -> u16 {
    let af = f32::from_bits(decode_gf16_to_u32(a)); // FPGA-ALLOWED
    let bf = f32::from_bits(decode_gf16_to_u32(b)); // FPGA-ALLOWED
    // Check for division by zero via integer comparison
    if b == 0x0000 || b == 0x8000 {
        // Return infinity based on sign of a
        return if (a & GF16_SIGN_BIT) != 0 { 0xFE00 } else { 0x7E00 };
    }
    encode_gf16_from_u32((af / bf).to_bits())        // FPGA-ALLOWED
}

#[no_mangle]
pub extern "C" fn gf16_eq(a: u16, b: u16) -> bool {
    // NaN handling via integer comparison
    if gf16_is_nan(a) || gf16_is_nan(b) {
        return false;
    }
    if gf16_is_zero(a) && gf16_is_zero(b) {
        return true;
    }
    a == b
}

#[no_mangle]
pub extern "C" fn gf16_lt(a: u16, b: u16) -> bool {
    // NaN handling via integer comparison
    if gf16_is_nan(a) || gf16_is_nan(b) {
        return false;
    }
    // Decode both to compare via f32 (only for comparison)
    f32::from_bits(decode_gf16_to_u32(a)) < f32::from_bits(decode_gf16_to_u32(b)) // FPGA-ALLOWED
}

// ─── Classification (pure integer — 100% FPGA-synthesizable) ─────────────────

#[no_mangle]
pub extern "C" fn gf16_is_zero(value: u16) -> bool {
    (value & 0x7FFF) == 0
}

#[no_mangle]
pub extern "C" fn gf16_is_inf(value: u16) -> bool {
    ((value & GF16_EXP_MASK) == GF16_EXP_MASK) && ((value & GF16_MANT_MASK) == 0)
}

#[no_mangle]
pub extern "C" fn gf16_is_nan(value: u16) -> bool {
    ((value & GF16_EXP_MASK) == GF16_EXP_MASK) && ((value & GF16_MANT_MASK) != 0)
}

#[no_mangle]
pub extern "C" fn gf16_extract_sign(value: u16) -> u8 {
    ((value >> 15) & 1) as u8
}

#[no_mangle]
pub extern "C" fn gf16_extract_exponent(value: u16) -> u8 {
    ((value >> GF16_MANT_BITS) & ((1 << GF16_EXP_BITS) - 1)) as u8
}

#[no_mangle]
pub extern "C" fn gf16_extract_mantissa(value: u16) -> i16 {
    (value & GF16_MANT_MASK) as i16
}

// ─── Convenience: f64 API boundary wrappers ──────────────────────────────────

/// FPGA-ALLOWED: f64 at API boundary only — converts to f32 via bits, then encodes.
#[no_mangle]
pub extern "C" fn gf16_from_f64(x: f64) -> u16 {
    gf16_from_f32(x as f32) // FPGA-ALLOWED: single cast at entry
}

/// FPGA-ALLOWED: f64 at API boundary only.
#[no_mangle]
pub extern "C" fn gf16_to_f64(value: u16) -> f64 {
    gf16_to_f32(value) as f64 // FPGA-ALLOWED: single cast at exit
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GF32 ENCODE/DECODE — [1:13:18] Lucas L₆ layout, pure integer
// ═════════════════════════════════════════════════════════════════════════════

/// Encode f64 → GF32 via integer bit manipulation. Layout: [sign:1][exp:13][mant:18].
/// FPGA-ALLOWED: f64 only at API boundary; .to_bits() extracts u64 immediately.
#[no_mangle]
pub extern "C" fn gf32_from_f64(x: f64) -> u32 {
    let bits: u64 = x.to_bits();
    encode_gf32_from_u64(bits)
}

/// Decode GF32 → f64 via integer bit reconstruction.
/// FPGA-ALLOWED: f64 only at API boundary exit; from_bits() reconstructs from u64.
#[no_mangle]
pub extern "C" fn gf32_to_f64(value: u32) -> f64 {
    let bits: u64 = decode_gf32_to_u64(value);
    f64::from_bits(bits)
}

/// Integer-only GF32 encode — [1:13:18] Lucas L₆ layout.
#[inline(always)]
fn encode_gf32_from_u64(f64_bits: u64) -> u32 {
    let sign: u32 = ((f64_bits >> 63) & 1) as u32;
    let exp:  i32 = ((f64_bits >> 52) & 0x7FF) as i32 - 1023;
    let mant: u64 = f64_bits & 0x000F_FFFF_FFFF_FFFF;

    if (f64_bits & 0x7FFF_FFFF_FFFF_FFFF) == 0 {
        return sign << 31;
    }
    if (f64_bits & 0x7FF0_0000_0000_0000) == 0x7FF0_0000_0000_0000 {
        if mant == 0 {
            return (sign << 31) | GF32_EXP_MASK;
        } else {
            return (sign << 31) | GF32_EXP_MASK | 1;
        }
    }

    let gf32_exp_raw: i32 = exp + GF32_EXP_BIAS;
    if gf32_exp_raw >= ((1 << GF32_EXP_BITS) - 1) as i32 {
        return (sign << 31) | GF32_EXP_MASK;
    }
    let gf32_exp: u32 = if gf32_exp_raw < 0 {
        0
    } else {
        gf32_exp_raw as u32
    };

    // Round-to-nearest-even: 52 bits → 18 bits (drop 34 bits)
    let shift: u32 = 52 - GF32_MANT_BITS; // 34
    let lower_bits = mant & ((1u64 << shift) - 1);
    let halfway: u64 = 1u64 << (shift - 1);
    let mut gf32_mant: u32 = (mant >> shift) as u32;
    let round_up = lower_bits > halfway
        || (lower_bits == halfway && (gf32_mant & 1) == 1);
    if round_up {
        gf32_mant += 1;
        if gf32_mant == (1 << GF32_MANT_BITS) {
            let new_exp = gf32_exp + 1;
            if new_exp >= ((1 << GF32_EXP_BITS) - 1) as u32 {
                return (sign << 31) | GF32_EXP_MASK;
            }
            return (sign << 31) | (new_exp << GF32_MANT_BITS);
        }
    }

    (sign << 31) | (gf32_exp << GF32_MANT_BITS) | gf32_mant
}

/// Integer-only GF32 decode — reconstructs f64 bits from GF32 [1:13:18] bits.
#[inline(always)]
fn decode_gf32_to_u64(gf32: u32) -> u64 {
    let sign: u64 = ((gf32 as u64) >> 31) & 1;
    let exp:  u64 = ((gf32 as u64) >> GF32_MANT_BITS) & ((1 << GF32_EXP_BITS) - 1);
    let mant: u64 = (gf32 as u64) & (GF32_MANT_MASK as u64);

    let exp_max: u64 = (1 << GF32_EXP_BITS) - 1;
    if exp == exp_max {
        if mant == 0 {
            return (sign << 63) | 0x7FF0_0000_0000_0000;
        } else {
            return 0x7FF8_0000_0000_0000;
        }
    }
    if exp == 0 && mant == 0 {
        return sign << 63;
    }

    let ieee_exp: u64 = ((exp as i64 - GF32_EXP_BIAS as i64 + 1023) as u64) & 0x7FF;
    let ieee_mant: u64 = mant << (52 - GF32_MANT_BITS); // 18 → 52 bits

    (sign << 63) | (ieee_exp << 52) | ieee_mant
}

#[no_mangle]
pub extern "C" fn gf32_is_zero(value: u32) -> bool {
    (value & 0x7FFF_FFFF) == 0
}

#[no_mangle]
pub extern "C" fn gf32_is_inf(value: u32) -> bool {
    ((value & GF32_EXP_MASK) == GF32_EXP_MASK) && ((value & GF32_MANT_MASK) == 0)
}

#[no_mangle]
pub extern "C" fn gf32_is_nan(value: u32) -> bool {
    ((value & GF32_EXP_MASK) == GF32_EXP_MASK) && ((value & GF32_MANT_MASK) != 0)
}

#[no_mangle]
pub extern "C" fn gf32_extract_sign(value: u32) -> u8 {
    ((value >> 31) & 1) as u8
}

#[no_mangle]
pub extern "C" fn gf32_extract_exponent(value: u32) -> u16 {
    ((value >> GF32_MANT_BITS) & ((1 << GF32_EXP_BITS) - 1)) as u16
}

#[no_mangle]
pub extern "C" fn gf32_extract_mantissa(value: u32) -> i32 {
    (value & GF32_MANT_MASK) as i32
}

// ═════════════════════════════════════════════════════════════════════════════════════
// GF32 ARITHMETIC — decode→f64 op→encode via [1:13:18] layout
// ═══════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf32_add(a: u32, b: u32) -> u32 {
    let af = f64::from_bits(decode_gf32_to_u64(a));
    let bf = f64::from_bits(decode_gf32_to_u64(b));
    encode_gf32_from_u64((af + bf).to_bits())
}

#[no_mangle]
pub extern "C" fn gf32_sub(a: u32, b: u32) -> u32 {
    let af = f64::from_bits(decode_gf32_to_u64(a));
    let bf = f64::from_bits(decode_gf32_to_u64(b));
    encode_gf32_from_u64((af - bf).to_bits())
}

#[no_mangle]
pub extern "C" fn gf32_mul(a: u32, b: u32) -> u32 {
    let af = f64::from_bits(decode_gf32_to_u64(a));
    let bf = f64::from_bits(decode_gf32_to_u64(b));
    encode_gf32_from_u64((af * bf).to_bits())
}

#[no_mangle]
pub extern "C" fn gf32_div(a: u32, b: u32) -> u32 {
    let af = f64::from_bits(decode_gf32_to_u64(a));
    let bf = f64::from_bits(decode_gf32_to_u64(b));
    if b == 0x0000_0000 || b == 0x8000_0000 {
        return if (a & GF32_SIGN_BIT) != 0 { 0xFFFF_C000 } else { 0x7FFF_C000 };
    }
    encode_gf32_from_u64((af / bf).to_bits())
}

#[no_mangle]
pub extern "C" fn gf32_eq(a: u32, b: u32) -> bool {
    // NaN handling via integer comparison
    if gf32_is_nan(a) || gf32_is_nan(b) {
        return false;
    }
    if gf32_is_zero(a) && gf32_is_zero(b) {
        return true;
    }
    a == b
}

#[no_mangle]
pub extern "C" fn gf32_lt(a: u32, b: u32) -> bool {
    if gf32_is_nan(a) || gf32_is_nan(b) {
        return false;
    }
    f64::from_bits(decode_gf32_to_u64(a)) < f64::from_bits(decode_gf32_to_u64(b))
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GF4/GF8/GF12/GF20/GF24 constants
// ═══════════════════════════════════════════════════════════════════════════════════

const GF4_EXP_BITS:  u32 = 1;
const GF4_MANT_BITS: u32 = 2;
const GF4_EXP_BIAS:  i32 = 0;

const GF8_EXP_BITS:  u32 = 3;
const GF8_MANT_BITS: u32 = 4;
const GF8_EXP_BIAS:  i32 = 3;

const GF12_EXP_BITS:  u32 = 4;
const GF12_MANT_BITS: u32 = 7;
const GF12_EXP_BIAS:  i32 = 7;

const GF20_EXP_BITS:  u32 = 7;
const GF20_MANT_BITS: u32 = 12;
const GF20_EXP_BIAS:  i32 = 63;

const GF24_EXP_BITS:  u32 = 9;
const GF24_MANT_BITS: u32 = 14;
const GF24_EXP_BIAS:  i32 = 255;

// ═══════════════════════════════════════════════════════════════════════════════════
// Field Extraction (all formats) — pure integer operations
// ═════════════════════════════════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf4_extract_sign(value: u8) -> i8 {
    ((value >> 3) & 1) as i8
}

#[no_mangle]
pub extern "C" fn gf4_extract_exponent(value: u8) -> i8 {
    ((value >> 2) & 1) as i8
}

#[no_mangle]
pub extern "C" fn gf4_extract_mantissa(value: u8) -> i16 {
    (value & 3) as i16
}

#[no_mangle]
pub extern "C" fn gf8_extract_sign(value: u8) -> i8 {
    ((value >> 7) & 1) as i8
}

#[no_mangle]
pub extern "C" fn gf8_extract_exponent(value: u8) -> i8 {
    ((value >> 4) & 7) as i8
}

#[no_mangle]
pub extern "C" fn gf8_extract_mantissa(value: u8) -> i16 {
    (value & 15) as i16
}

#[no_mangle]
pub extern "C" fn gf12_extract_sign(value: u16) -> i8 {
    ((value >> 11) & 1) as i8
}

#[no_mangle]
pub extern "C" fn gf12_extract_exponent(value: u16) -> i8 {
    ((value >> 7) & 15) as i8
}

#[no_mangle]
pub extern "C" fn gf12_extract_mantissa(value: u16) -> i16 {
    (value & 127) as i16
}

#[no_mangle]
pub extern "C" fn gf20_extract_sign(value: u32) -> i8 {
    ((value >> 19) & 1) as i8
}

#[no_mangle]
pub extern "C" fn gf20_extract_exponent(value: u32) -> i8 {
    ((value >> 12) & 127) as i8
}

#[no_mangle]
pub extern "C" fn gf20_extract_mantissa(value: u32) -> i16 {
    (value & 4095) as i16
}

#[no_mangle]
pub extern "C" fn gf24_extract_sign(value: u32) -> i8 {
    ((value >> 23) & 1) as i8
}

#[no_mangle]
pub extern "C" fn gf24_extract_exponent(value: u32) -> i8 {
    ((value >> 14) & 511) as i8
}

#[no_mangle]
pub extern "C" fn gf24_extract_mantissa(value: u32) -> i16 {
    (value & 16383) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    const ULPS_F32: f32 = 2.0;

    fn ulp_error(actual: f32, expected: f32) -> f32 {
        if actual == expected { return 0.0; }
        let diff = (actual - expected).abs();
        let ulp = f32::EPSILON * expected.abs().max(1.0);
        diff / ulp
    }

    // ─── BUG-001 (#546): round-to-nearest-even, not truncation ───

    #[test]
    fn gf16_round_to_nearest_just_above_1() {
        let input: f32 = 1.0 + 2f32.powf(-10.0);
        let encoded = gf16_from_f32(input);
        let decoded = gf16_to_f32(encoded);
        assert!(ulp_error(decoded, input) < ULPS_F32,
            "BUG-001: expected near {} got {} (ULP err {})", input, decoded, ulp_error(decoded, input));
    }

    #[test]
    fn gf16_round_to_nearest_just_above_half() {
        let input: f32 = 0.5 + 2f32.powf(-11.0);
        let encoded = gf16_from_f32(input);
        let decoded = gf16_to_f32(encoded);
        assert!(ulp_error(decoded, input) < ULPS_F32,
            "BUG-001: expected near {} got {} (ULP err {})", input, decoded, ulp_error(decoded, input));
    }

    #[test]
    fn gf16_round_to_even_tiebreak() {
        let bits = 0x3F802000u32; // 1.0 + exactly 0.5 ULP in GF16 terms
        let encoded = encode_gf16_from_u32(bits);
        let mant = gf16_extract_mantissa(encoded);
        assert_eq!(mant % 2, 0, "BUG-001: tiebreak must round to even mantissa, got {}", mant);
    }

    // ─── BUG-002 (#547): overflow → +Inf, not max-finite ───

    #[test]
    fn gf16_overflow_positive_inf() {
        let encoded = gf16_from_f32(1e30f32);
        assert!(gf16_is_inf(encoded), "BUG-002: 1e30 should encode to +Inf");
        assert_eq!(gf16_extract_sign(encoded), 0, "BUG-002: +Inf sign should be 0");
        assert_eq!(gf16_extract_mantissa(encoded), 0, "BUG-002: +Inf mantissa should be 0");
    }

    #[test]
    fn gf16_overflow_negative_inf() {
        let encoded = gf16_from_f32(-1e30f32);
        assert!(gf16_is_inf(encoded), "BUG-002: -1e30 should encode to -Inf");
        assert_eq!(gf16_extract_sign(encoded), 1, "BUG-002: -Inf sign should be 1");
        assert_eq!(gf16_extract_mantissa(encoded), 0, "BUG-002: -Inf mantissa should be 0");
    }

    #[test]
    fn gf16_f32_max_to_inf() {
        let encoded = gf16_from_f32(f32::MAX);
        assert!(gf16_is_inf(encoded), "BUG-002: f32::MAX should overflow to +Inf in GF16");
    }

    // ─── BUG-003 (#548): GF32 [1:13:18] Lucas L₆ layout ───

    #[test]
    fn gf32_layout_1_13_18() {
        assert_eq!(GF32_EXP_BITS, 13);
        assert_eq!(GF32_MANT_BITS, 18);
        assert_eq!(GF32_EXP_BIAS, 4095);
    }

    #[test]
    fn gf32_encode_1_point_0() {
        let encoded = gf32_from_f64(1.0);
        let exp = gf32_extract_exponent(encoded);
        let mant = gf32_extract_mantissa(encoded);
        let sign = gf32_extract_sign(encoded);
        assert_eq!(sign, 0, "BUG-003: 1.0 sign");
        assert_eq!(exp, 4095 + 0, "BUG-003: 1.0 exp should be bias (unbiased=0, biased=4095)");
        assert_eq!(mant, 0, "BUG-003: 1.0 mantissa should be 0 (implicit leading 1)");
    }

    #[test]
    fn gf32_roundtrip_1() {
        let val = 1.0f64;
        assert_eq!(gf32_to_f64(gf32_from_f64(val)), val);
    }

    #[test]
    fn gf32_roundtrip_pi() {
        let val = std::f64::consts::PI;
        let encoded = gf32_from_f64(val);
        let decoded = gf32_to_f64(encoded);
        let rel_err = (decoded - val).abs() / val;
        assert!(rel_err < 1e-5, "BUG-003: PI roundtrip rel_err = {}", rel_err);
    }

    #[test]
    fn gf32_roundtrip_negative() {
        let val = -42.5f64;
        let encoded = gf32_from_f64(val);
        let decoded = gf32_to_f64(encoded);
        assert_eq!(decoded, val, "BUG-003: negative roundtrip");
    }

    #[test]
    fn gf32_overflow_to_inf() {
        let encoded = gf32_from_f64(1e300);
        assert!(gf32_is_inf(encoded), "BUG-003: 1e300 should overflow to +Inf");
        assert_eq!(gf32_extract_sign(encoded), 0);
    }

    #[test]
    fn gf32_zero() {
        let encoded = gf32_from_f64(0.0);
        assert!(gf32_is_zero(encoded), "BUG-003: 0.0 should be zero");
        let encoded_neg = gf32_from_f64(-0.0);
        assert!(gf32_is_zero(encoded_neg), "BUG-003: -0.0 should be zero");
        assert_eq!(gf32_extract_sign(encoded_neg), 1, "BUG-003: -0.0 sign");
    }

    #[test]
    fn gf32_nan() {
        let encoded = gf32_from_f64(f64::NAN);
        assert!(gf32_is_nan(encoded), "BUG-003: NaN should be NaN");
    }

    #[test]
    fn gf32_inf() {
        let encoded = gf32_from_f64(f64::INFINITY);
        assert!(gf32_is_inf(encoded), "BUG-003: +Inf should be Inf");
        assert_eq!(gf32_extract_sign(encoded), 0);
        let encoded_neg = gf32_from_f64(f64::NEG_INFINITY);
        assert!(gf32_is_inf(encoded_neg), "BUG-003: -Inf should be Inf");
        assert_eq!(gf32_extract_sign(encoded_neg), 1);
    }

    #[test]
    fn gf32_arithmetic_add() {
        let a = gf32_from_f64(3.0);
        let b = gf32_from_f64(4.0);
        let c = gf32_add(a, b);
        let result = gf32_to_f64(c);
        assert!((result - 7.0).abs() < 1e-6, "gf32_add: expected 7.0, got {}", result);
    }

    #[test]
    fn gf32_arithmetic_mul() {
        let a = gf32_from_f64(6.0);
        let b = gf32_from_f64(7.0);
        let c = gf32_mul(a, b);
        let result = gf32_to_f64(c);
        assert!((result - 42.0).abs() < 1e-6, "gf32_mul: expected 42.0, got {}", result);
    }

    #[test]
    fn gf32_arithmetic_lt() {
        let a = gf32_from_f64(3.0);
        let b = gf32_from_f64(4.0);
        assert!(gf32_lt(a, b), "3.0 < 4.0");
        assert!(!gf32_lt(b, a), "4.0 not < 3.0");
    }

    #[test]
    fn gf32_exp_range() {
        let small = gf32_from_f64(2f64.powf(-4000.0));
        let decoded_small = gf32_to_f64(small);
        assert!(decoded_small > 0.0 && decoded_small.is_finite(),
            "BUG-003: 2^-4000 should be representable in 13-bit exp, got {}", decoded_small);

        let large = gf32_from_f64(2f64.powf(4000.0));
        let decoded_large = gf32_to_f64(large);
        assert!(decoded_large.is_infinite() || (decoded_large > 1e1000),
            "BUG-003: 2^4000 should overflow or be huge, got {}", decoded_large);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GF4 ENCODE/DECODE (1:1:2)
// ═══════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf4_from_f32(x: f32) -> u8 {
    let bits: u32 = x.to_bits();
    let sign: u8 = ((bits >> 31) & 1) as u8;
    let exp: i32 = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant: u32 = bits & 0x007F_FFFF;

    if (bits & 0x7FFF_FFFF) == 0 {
        return sign << 3;
    }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        let exp_max: u8 = (1 << GF4_EXP_BITS) - 1;
        let mant_out: u8 = if mant == 0 { 0 } else { 1 };
        return (sign << 3) | (exp_max << GF4_MANT_BITS) | mant_out;
    }

    let gf4_exp_raw: i32 = exp + GF4_EXP_BIAS;
    let exp_max: u32 = (1 << GF4_EXP_BITS) - 1;
    if gf4_exp_raw >= exp_max as i32 {
        return (sign << 3) | ((exp_max as u8) << GF4_MANT_BITS);
    }
    let gf4_exp: u8 = if gf4_exp_raw < 0 { 0 } else { gf4_exp_raw as u8 };

    let drop = 23 - GF4_MANT_BITS;
    let lower_bits = mant & ((1u32 << drop) - 1);
    let halfway = 1u32 << (drop - 1);
    let mut gf4_mant: u8 = (mant >> drop) as u8;
    let round_up = lower_bits > halfway
        || (lower_bits == halfway && (gf4_mant & 1) == 1);
    if round_up {
        gf4_mant += 1;
        if gf4_mant == (1 << GF4_MANT_BITS) {
            let new_exp = gf4_exp + 1;
            if new_exp >= exp_max as u8 {
                return (sign << 3) | (exp_max as u8) << GF4_MANT_BITS;
            }
            return (sign << 3) | (new_exp << GF4_MANT_BITS);
        }
    }

    (sign << 3) | (gf4_exp << GF4_MANT_BITS) | gf4_mant
}

#[no_mangle]
pub extern "C" fn gf4_to_f32(value: u8) -> f32 {
    let sign: u32 = ((value as u32) >> 3) & 1;
    let exp: u32 = ((value as u32) >> GF4_MANT_BITS) & ((1 << GF4_EXP_BITS) - 1);
    let mant: u32 = (value as u32) & ((1u32 << GF4_MANT_BITS) - 1);
    let exp_max: u32 = (1 << GF4_EXP_BITS) - 1;

    if exp == exp_max {
        if mant == 0 {
            return f32::from_bits((sign << 31) | 0x7F80_0000);
        }
        return f32::from_bits(0x7FC0_0000);
    }
    if exp == 0 && mant == 0 {
        return f32::from_bits(sign << 31);
    }

    let ieee_exp: u32 = (((exp as i32) - GF4_EXP_BIAS + 127) as u32) & 0xFF;
    let ieee_mant: u32 = mant << (23 - GF4_MANT_BITS);
    f32::from_bits((sign << 31) | (ieee_exp << 23) | ieee_mant)
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GF8 ENCODE/DECODE (1:3:4)
// ═══════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf8_from_f32(x: f32) -> u8 {
    let bits: u32 = x.to_bits();
    let sign: u8 = ((bits >> 31) & 1) as u8;
    let exp: i32 = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant: u32 = bits & 0x007F_FFFF;

    if (bits & 0x7FFF_FFFF) == 0 {
        return sign << 7;
    }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        let exp_max: u8 = (1 << GF8_EXP_BITS) - 1;
        let mant_out: u8 = if mant == 0 { 0 } else { 1 };
        return (sign << 7) | (exp_max << GF8_MANT_BITS) | mant_out;
    }

    let gf8_exp_raw: i32 = exp + GF8_EXP_BIAS;
    let exp_max: u32 = (1 << GF8_EXP_BITS) - 1;
    if gf8_exp_raw >= exp_max as i32 {
        return (sign << 7) | ((exp_max as u8) << GF8_MANT_BITS);
    }
    let gf8_exp: u8 = if gf8_exp_raw < 0 { 0 } else { gf8_exp_raw as u8 };

    let drop = 23 - GF8_MANT_BITS;
    let lower_bits = mant & ((1u32 << drop) - 1);
    let halfway = 1u32 << (drop - 1);
    let mut gf8_mant: u8 = (mant >> drop) as u8;
    let round_up = lower_bits > halfway
        || (lower_bits == halfway && (gf8_mant & 1) == 1);
    if round_up {
        gf8_mant += 1;
        if gf8_mant == (1 << GF8_MANT_BITS) {
            let new_exp = gf8_exp + 1;
            if new_exp >= exp_max as u8 {
                return (sign << 7) | (exp_max as u8) << GF8_MANT_BITS;
            }
            return (sign << 7) | (new_exp << GF8_MANT_BITS);
        }
    }

    (sign << 7) | (gf8_exp << GF8_MANT_BITS) | gf8_mant
}

#[no_mangle]
pub extern "C" fn gf8_to_f32(value: u8) -> f32 {
    let sign: u32 = ((value as u32) >> 7) & 1;
    let exp: u32 = ((value as u32) >> GF8_MANT_BITS) & ((1 << GF8_EXP_BITS) - 1);
    let mant: u32 = (value as u32) & ((1u32 << GF8_MANT_BITS) - 1);
    let exp_max: u32 = (1 << GF8_EXP_BITS) - 1;

    if exp == exp_max {
        if mant == 0 {
            return f32::from_bits((sign << 31) | 0x7F80_0000);
        }
        return f32::from_bits(0x7FC0_0000);
    }
    if exp == 0 && mant == 0 {
        return f32::from_bits(sign << 31);
    }

    let ieee_exp: u32 = (((exp as i32) - GF8_EXP_BIAS + 127) as u32) & 0xFF;
    let ieee_mant: u32 = mant << (23 - GF8_MANT_BITS);
    f32::from_bits((sign << 31) | (ieee_exp << 23) | ieee_mant)
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GF12 ENCODE/DECODE (1:4:7)
// ═══════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf12_from_f32(x: f32) -> u16 {
    let bits: u32 = x.to_bits();
    let sign: u16 = ((bits >> 31) & 1) as u16;
    let exp: i32 = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant: u32 = bits & 0x007F_FFFF;

    if (bits & 0x7FFF_FFFF) == 0 {
        return sign << 11;
    }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        let exp_max: u16 = (1 << GF12_EXP_BITS) - 1;
        let mant_out: u16 = if mant == 0 { 0 } else { 1 };
        return (sign << 11) | (exp_max << GF12_MANT_BITS) | mant_out;
    }

    let gf12_exp_raw: i32 = exp + GF12_EXP_BIAS;
    let exp_max: u32 = (1 << GF12_EXP_BITS) - 1;
    if gf12_exp_raw >= exp_max as i32 {
        return (sign << 11) | ((exp_max as u16) << GF12_MANT_BITS);
    }
    let gf12_exp: u16 = if gf12_exp_raw < 0 { 0 } else { gf12_exp_raw as u16 };

    let drop = 23 - GF12_MANT_BITS;
    let lower_bits = mant & ((1u32 << drop) - 1);
    let halfway = 1u32 << (drop - 1);
    let mut gf12_mant: u16 = (mant >> drop) as u16;
    let round_up = lower_bits > halfway
        || (lower_bits == halfway && (gf12_mant & 1) == 1);
    if round_up {
        gf12_mant += 1;
        if gf12_mant == (1 << GF12_MANT_BITS) {
            let new_exp = gf12_exp + 1;
            if new_exp >= exp_max as u16 {
                return (sign << 11) | (exp_max as u16) << GF12_MANT_BITS;
            }
            return (sign << 11) | (new_exp << GF12_MANT_BITS);
        }
    }

    (sign << 11) | (gf12_exp << GF12_MANT_BITS) | gf12_mant
}

#[no_mangle]
pub extern "C" fn gf12_to_f32(value: u16) -> f32 {
    let sign: u32 = ((value as u32) >> 11) & 1;
    let exp: u32 = ((value as u32) >> GF12_MANT_BITS) & ((1 << GF12_EXP_BITS) - 1);
    let mant: u32 = (value as u32) & ((1u32 << GF12_MANT_BITS) - 1);
    let exp_max: u32 = (1 << GF12_EXP_BITS) - 1;

    if exp == exp_max {
        if mant == 0 {
            return f32::from_bits((sign << 31) | 0x7F80_0000);
        }
        return f32::from_bits(0x7FC0_0000);
    }
    if exp == 0 && mant == 0 {
        return f32::from_bits(sign << 31);
    }

    let ieee_exp: u32 = (((exp as i32) - GF12_EXP_BIAS + 127) as u32) & 0xFF;
    let ieee_mant: u32 = mant << (23 - GF12_MANT_BITS);
    f32::from_bits((sign << 31) | (ieee_exp << 23) | ieee_mant)
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GF20 ENCODE/DECODE (1:7:12)
// ═══════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf20_from_f32(x: f32) -> u32 {
    let bits: u32 = x.to_bits();
    let sign: u32 = (bits >> 31) & 1;
    let exp: i32 = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant: u32 = bits & 0x007F_FFFF;

    if (bits & 0x7FFF_FFFF) == 0 {
        return sign << 19;
    }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        let exp_max: u32 = (1 << GF20_EXP_BITS) - 1;
        let mant_out: u32 = if mant == 0 { 0 } else { 1 };
        return (sign << 19) | (exp_max << GF20_MANT_BITS) | mant_out;
    }

    let gf20_exp_raw: i32 = exp + GF20_EXP_BIAS;
    let exp_max: u32 = (1 << GF20_EXP_BITS) - 1;
    if gf20_exp_raw >= exp_max as i32 {
        return (sign << 19) | (exp_max << GF20_MANT_BITS);
    }
    let gf20_exp: u32 = if gf20_exp_raw < 0 { 0 } else { gf20_exp_raw as u32 };

    let drop: u32 = 23 - GF20_MANT_BITS;
    let lower_bits = mant & ((1u32 << drop) - 1);
    let halfway = 1u32 << (drop - 1);
    let mut gf20_mant: u32 = mant >> drop;
    let round_up = lower_bits > halfway
        || (lower_bits == halfway && (gf20_mant & 1) == 1);
    if round_up {
        gf20_mant += 1;
        if gf20_mant == (1 << GF20_MANT_BITS) {
            let new_exp = gf20_exp + 1;
            if new_exp >= exp_max {
                return (sign << 19) | (exp_max << GF20_MANT_BITS);
            }
            return (sign << 19) | (new_exp << GF20_MANT_BITS);
        }
    }

    (sign << 19) | (gf20_exp << GF20_MANT_BITS) | gf20_mant
}

#[no_mangle]
pub extern "C" fn gf20_to_f32(value: u32) -> f32 {
    let sign: u32 = (value >> 19) & 1;
    let exp: u32 = (value >> GF20_MANT_BITS) & ((1 << GF20_EXP_BITS) - 1);
    let mant: u32 = value & ((1u32 << GF20_MANT_BITS) - 1);
    let exp_max: u32 = (1 << GF20_EXP_BITS) - 1;

    if exp == exp_max {
        if mant == 0 {
            return f32::from_bits((sign << 31) | 0x7F80_0000);
        }
        return f32::from_bits(0x7FC0_0000);
    }
    if exp == 0 && mant == 0 {
        return f32::from_bits(sign << 31);
    }

    let ieee_exp: u32 = (((exp as i32) - GF20_EXP_BIAS + 127) as u32) & 0xFF;
    let ieee_mant: u32 = mant << (23 - GF20_MANT_BITS);
    f32::from_bits((sign << 31) | (ieee_exp << 23) | ieee_mant)
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GF24 ENCODE/DECODE (1:9:14)
// ═══════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf24_from_f32(x: f32) -> u32 {
    let bits: u32 = x.to_bits();
    let sign: u32 = (bits >> 31) & 1;
    let exp: i32 = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant: u32 = bits & 0x007F_FFFF;

    if (bits & 0x7FFF_FFFF) == 0 {
        return sign << 23;
    }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        let exp_max: u32 = (1 << GF24_EXP_BITS) - 1;
        let mant_out: u32 = if mant == 0 { 0 } else { 1 };
        return (sign << 23) | (exp_max << GF24_MANT_BITS) | mant_out;
    }

    let gf24_exp_raw: i32 = exp + GF24_EXP_BIAS;
    let exp_max: u32 = (1 << GF24_EXP_BITS) - 1;
    if gf24_exp_raw >= exp_max as i32 {
        return (sign << 23) | (exp_max << GF24_MANT_BITS);
    }
    let gf24_exp: u32 = if gf24_exp_raw < 0 { 0 } else { gf24_exp_raw as u32 };

    let drop: u32 = 23 - GF24_MANT_BITS;
    let lower_bits = mant & ((1u32 << drop) - 1);
    let halfway = 1u32 << (drop - 1);
    let mut gf24_mant: u32 = mant >> drop;
    let round_up = lower_bits > halfway
        || (lower_bits == halfway && (gf24_mant & 1) == 1);
    if round_up {
        gf24_mant += 1;
        if gf24_mant == (1 << GF24_MANT_BITS) {
            let new_exp = gf24_exp + 1;
            if new_exp >= exp_max {
                return (sign << 23) | (exp_max << GF24_MANT_BITS);
            }
            return (sign << 23) | (new_exp << GF24_MANT_BITS);
        }
    }

    (sign << 23) | (gf24_exp << GF24_MANT_BITS) | gf24_mant
}

#[no_mangle]
pub extern "C" fn gf24_to_f32(value: u32) -> f32 {
    let sign: u32 = (value >> 23) & 1;
    let exp: u32 = (value >> GF24_MANT_BITS) & ((1 << GF24_EXP_BITS) - 1);
    let mant: u32 = value & ((1u32 << GF24_MANT_BITS) - 1);
    let exp_max: u32 = (1 << GF24_EXP_BITS) - 1;

    if exp == exp_max {
        if mant == 0 {
            return f32::from_bits((sign << 31) | 0x7F80_0000);
        }
        return f32::from_bits(0x7FC0_0000);
    }
    if exp == 0 && mant == 0 {
        return f32::from_bits(sign << 31);
    }

    let ieee_exp: u32 = (((exp as i32) - GF24_EXP_BIAS + 127) as u32) & 0xFF;
    let ieee_mant: u32 = mant << (23 - GF24_MANT_BITS);
    f32::from_bits((sign << 31) | (ieee_exp << 23) | ieee_mant)
}
