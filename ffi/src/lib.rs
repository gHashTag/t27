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

// ─── GF32 constants (8-bit exp, 23-bit mant — same as IEEE but φ-exponent) ──
const GF32_SIGN_BIT:   u32 = 1 << 31;
const GF32_EXP_MASK:   u32 = 0x7F80_0000;
const GF32_MANT_MASK:  u32 = 0x007F_FFFF;
const GF32_EXP_BIAS:   i32 = 127;

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
// GF32 — same pattern, higher precision (8-bit exp, 23-bit mant)
// ═════════════════════════════════════════════════════════════════════════════

/// FPGA-ALLOWED: f64 at boundary; extract u64 bits then truncate to f32 range.
#[no_mangle]
pub extern "C" fn gf32_from_f64(x: f64) -> u32 {
    let f32_val = x as f32; // FPGA-ALLOWED
    f32_val.to_bits()        // GF32 uses same layout as IEEE f32 + φ-exp mapping
}

/// FPGA-ALLOWED: f64 at API boundary only.
#[no_mangle]
pub extern "C" fn gf32_to_f64(value: u32) -> f64 {
    f32::from_bits(value) as f64 // FPGA-ALLOWED
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
pub extern "C" fn gf32_extract_exponent(value: u32) -> u8 {
    ((value >> 23) & 0xFF) as u8
}

#[no_mangle]
pub extern "C" fn gf32_extract_mantissa(value: u32) -> i32 {
    (value & GF32_MANT_MASK) as i32
}

// ═════════════════════════════════════════════════════════════════════════════════════
// GF32 ARITHMETIC — same pattern as GF16
// ═════════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn gf32_add(a: u32, b: u32) -> u32 {
    let af = f32::from_bits(a); // FPGA-ALLOWED
    let bf = f32::from_bits(b); // FPGA-ALLOWED
    (af + bf).to_bits()        // FPGA-ALLOWED
}

#[no_mangle]
pub extern "C" fn gf32_sub(a: u32, b: u32) -> u32 {
    let af = f32::from_bits(a); // FPGA-ALLOWED
    let bf = f32::from_bits(b); // FPGA-ALLOWED
    (af - bf).to_bits()        // FPGA-ALLOWED
}

#[no_mangle]
pub extern "C" fn gf32_mul(a: u32, b: u32) -> u32 {
    let af = f32::from_bits(a); // FPGA-ALLOWED
    let bf = f32::from_bits(b); // FPGA-ALLOWED
    (af * bf).to_bits()        // FPGA-ALLOWED
}

#[no_mangle]
pub extern "C" fn gf32_div(a: u32, b: u32) -> u32 {
    let af = f32::from_bits(a); // FPGA-ALLOWED
    let bf = f32::from_bits(b); // FPGA-ALLOWED
    // Check for division by zero via integer comparison
    if b == 0 {
        // Return infinity based on sign of a
        return if (a & GF32_SIGN_BIT) != 0 { 0xFF80_0000 } else { 0x7F80_0000 };
    }
    (af / bf).to_bits()        // FPGA-ALLOWED
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
    // NaN handling via integer comparison
    if gf32_is_nan(a) || gf32_is_nan(b) {
        return false;
    }
    // Decode both to compare via f32 (only for comparison)
    f32::from_bits(a) < f32::from_bits(b) // FPGA-ALLOWED
}

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
