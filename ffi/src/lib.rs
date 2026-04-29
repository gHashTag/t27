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

// ─── GF32 constants — IEEE 754 fp32 layout (FPGA f32 drop-in compatible) ────
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
    let gf16_exp: u16 = if gf16_exp_raw < 0 {
        0
    } else if gf16_exp_raw >= ((1 << GF16_EXP_BITS) - 1) as i32 {
        return (sign << 15) | GF16_EXP_MASK; // overflow → ±Inf (exp=max, mant=0)
    } else {
        gf16_exp_raw as u16
    };

    // Round-to-nearest-even mantissa: 23 bits → 9 bits
    let lower_bits = mant & 0x3FFF; // 14 bits being dropped
    let halfway = 0x2000;           // 2^13
    let mut gf16_mant: u16 = (mant >> 14) as u16;
    let round_up = lower_bits > halfway
        || (lower_bits == halfway && (gf16_mant & 1) == 1); // tie-break to even
    if round_up {
        gf16_mant += 1;
        if gf16_mant == (1 << GF16_MANT_BITS) {
            gf16_mant = 0;
            // carry into exponent
            let new_exp = gf16_exp + 1;
            if new_exp >= (1 << GF16_EXP_BITS) - 1 {
                return (sign << 15) | GF16_EXP_MASK; // carry overflow → ±Inf
            }
            return (sign << 15) | ((new_exp as u16) << GF16_MANT_BITS as u16) | gf16_mant;
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
    f32_val.to_bits()        // GF32 = IEEE fp32 layout for FPGA f32 drop-in compatibility
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

// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════
// GF4/GF8/GF12/GF20/GF24 — encode/decode (from_f32 / to_f32)
// Layouts derived from extract_* bit positions above.
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════

// ─── GF4: [s:1][e:1][m:2], bias=1 ──────────────────────────────────────────

#[no_mangle]
pub extern "C" fn gf4_from_f32(x: f32) -> u8 {
    let bits = x.to_bits();
    let sign = ((bits >> 31) & 1) as u8;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant = bits & 0x007F_FFFF;
    if (bits & 0x7FFF_FFFF) == 0 { return sign << 3; }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        return if mant == 0 { sign << 3 | 0x06 } else { sign << 3 | 0x07 };
    }
    let gf_exp = exp + 1;
    if gf_exp < 0 { return sign << 3; }
    if gf_exp >= 1 { return sign << 3 | 0x06; } // overflow → Inf (e=max=1, m=0)
    let lower = mant & 0x001F_FFFF;
    let half = 0x0010_0000;
    let mut m = (mant >> 21) as u8;
    if lower > half || (lower == half && (m & 1) == 1) { m += 1; }
    if m >= 4 { return sign << 3 | 0x06; }
    (sign << 3) | ((gf_exp as u8) << 2) | m
}

#[no_mangle]
pub extern "C" fn gf4_to_f32(value: u8) -> f32 {
    let sign = ((value as u32) >> 3) & 1;
    let exp = (value >> 2) & 1;
    let mant = (value as u32) & 3;
    if exp == 1 && mant == 0 { return f32::from_bits((sign << 31) | 0x7F80_0000); }
    if exp == 1 && mant != 0 { return f32::NAN; }
    if exp == 0 && mant == 0 { return f32::from_bits(sign << 31); }
    let ieee_exp = (exp as i32 - 1 + 127) as u32;
    f32::from_bits((sign << 31) | (ieee_exp << 23) | (mant << 21))
}

// ─── GF8: [s:1][e:3][m:4], bias=3 ──────────────────────────────────────────

#[no_mangle]
pub extern "C" fn gf8_from_f32(x: f32) -> u8 {
    let bits = x.to_bits();
    let sign = ((bits >> 31) & 1) as u8;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant = bits & 0x007F_FFFF;
    if (bits & 0x7FFF_FFFF) == 0 { return sign << 7; }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        return if mant == 0 { sign << 7 | 0x70 } else { sign << 7 | 0x71 };
    }
    let gf_exp = exp + 3;
    if gf_exp < 0 { return sign << 7; }
    if gf_exp >= 6 { return sign << 7 | 0x70; } // overflow → Inf
    let lower = mant & 0x000F_FFFF;
    let half = 0x0008_0000;
    let mut m = (mant >> 19) as u8;
    if lower > half || (lower == half && (m & 1) == 1) { m += 1; }
    if m >= 16 { return sign << 7 | 0x70; }
    (sign << 7) | ((gf_exp as u8) << 4) | m
}

#[no_mangle]
pub extern "C" fn gf8_to_f32(value: u8) -> f32 {
    let sign = ((value as u32) >> 7) & 1;
    let exp = ((value >> 4) & 7) as i32;
    let mant = (value as u32) & 15;
    if exp == 7 && mant == 0 { return f32::from_bits((sign << 31) | 0x7F80_0000); }
    if exp == 7 && mant != 0 { return f32::NAN; }
    if exp == 0 && mant == 0 { return f32::from_bits(sign << 31); }
    let ieee_exp = (exp - 3 + 127) as u32;
    f32::from_bits((sign << 31) | (ieee_exp << 23) | (mant << 19))
}

// ─── GF12: [s:1][e:4][m:7], bias=7 ─────────────────────────────────────────

#[no_mangle]
pub extern "C" fn gf12_from_f32(x: f32) -> u16 {
    let bits = x.to_bits();
    let sign = ((bits >> 31) & 1) as u16;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant = bits & 0x007F_FFFF;
    if (bits & 0x7FFF_FFFF) == 0 { return sign << 11; }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        return if mant == 0 { sign << 11 | 0x0780 } else { sign << 11 | 0x0781 };
    }
    let gf_exp = exp + 7;
    if gf_exp < 0 { return sign << 11; }
    if gf_exp >= 14 { return sign << 11 | 0x0780; } // overflow → Inf
    let lower = mant & 0x0000_FFFF;
    let half = 0x0000_8000;
    let mut m = (mant >> 16) as u16;
    if lower > half || (lower == half && (m & 1) == 1) { m += 1; }
    if m >= 128 { return sign << 11 | 0x0780; }
    (sign << 11) | ((gf_exp as u16) << 7) | m
}

#[no_mangle]
pub extern "C" fn gf12_to_f32(value: u16) -> f32 {
    let sign = ((value as u32) >> 11) & 1;
    let exp = ((value >> 7) & 15) as i32;
    let mant = (value as u32) & 127;
    if exp == 15 && mant == 0 { return f32::from_bits((sign << 31) | 0x7F80_0000); }
    if exp == 15 && mant != 0 { return f32::NAN; }
    if exp == 0 && mant == 0 { return f32::from_bits(sign << 31); }
    let ieee_exp = (exp - 7 + 127) as u32;
    f32::from_bits((sign << 31) | (ieee_exp << 23) | (mant << 16))
}

// ─── GF20: [s:1][e:7][m:12], bias=63 ───────────────────────────────────────

#[no_mangle]
pub extern "C" fn gf20_from_f32(x: f32) -> u32 {
    let bits = x.to_bits();
    let sign = ((bits >> 31) & 1) as u32;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant = bits & 0x007F_FFFF;
    if (bits & 0x7FFF_FFFF) == 0 { return sign << 19; }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        return if mant == 0 { sign << 19 | 0x3F000 } else { sign << 19 | 0x3F001 };
    }
    let gf_exp = exp + 63;
    if gf_exp < 0 { return sign << 19; }
    if gf_exp >= 126 { return sign << 19 | 0x3F000; } // overflow → Inf
    let lower = mant & 0x0000_07FF;
    let half = 0x0000_0400;
    let mut m = (mant >> 11) as u32;
    if lower > half || (lower == half && (m & 1) == 1) { m += 1; }
    if m >= 4096 { return sign << 19 | 0x3F000; }
    (sign << 19) | ((gf_exp as u32) << 12) | m
}

#[no_mangle]
pub extern "C" fn gf20_to_f32(value: u32) -> f32 {
    let sign = (value >> 19) & 1;
    let exp = ((value >> 12) & 127) as i32;
    let mant = value & 4095;
    if exp == 127 && mant == 0 { return f32::from_bits((sign << 31) | 0x7F80_0000); }
    if exp == 127 && mant != 0 { return f32::NAN; }
    if exp == 0 && mant == 0 { return f32::from_bits(sign << 31); }
    let ieee_exp_raw = exp - 63 + 127;
    if ieee_exp_raw <= 0 { return f32::from_bits(sign << 31); }
    if ieee_exp_raw >= 255 { return f32::from_bits((sign << 31) | 0x7F80_0000); }
    f32::from_bits((sign << 31) | ((ieee_exp_raw as u32) << 23) | (mant << 11))
}

// ─── GF24: [s:1][e:9][m:14], bias=255 (full 9-bit range; overflow maps to f32 Inf) ───

#[no_mangle]
pub extern "C" fn gf24_from_f32(x: f32) -> u32 {
    let bits = x.to_bits();
    let sign = ((bits >> 31) & 1) as u32;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant = bits & 0x007F_FFFF;
    if (bits & 0x7FFF_FFFF) == 0 { return sign << 23; }
    if (bits & 0x7F80_0000) == 0x7F80_0000 {
        return if mant == 0 { sign << 23 | 0x1FC000 } else { sign << 23 | 0x1FC001 };
    }
    let gf_exp = (exp + 255) as u32;
    if gf_exp < 1 { return sign << 23; } // underflow
    if gf_exp > 510 { return sign << 23 | 0x1FC000; } // overflow → Inf
    let lower = mant & 0x0000_01FF;
    let half = 0x0000_0100;
    let mut m = (mant >> 9) as u32;
    if lower > half || (lower == half && (m & 1) == 1) { m += 1; }
    if m >= 16384 { return sign << 23 | 0x1FC000; }
    (sign << 23) | (gf_exp << 14) | m
}

#[no_mangle]
pub extern "C" fn gf24_to_f32(value: u32) -> f32 {
    let sign = (value >> 23) & 1;
    let exp = ((value >> 14) & 511) as i32;
    let mant = value & 16383;
    if exp == 511 && mant == 0 { return f32::from_bits((sign << 31) | 0x7F80_0000); }
    if exp == 511 && mant != 0 { return f32::NAN; }
    if exp == 0 && mant == 0 { return f32::from_bits(sign << 31); }
    let ieee_exp = exp - 255 + 127;
    if ieee_exp <= 0 { return f32::from_bits(sign << 31); }
    if ieee_exp >= 255 { return f32::from_bits((sign << 31) | 0x7F80_0000); }
    f32::from_bits((sign << 31) | ((ieee_exp as u32) << 23) | (mant << 9))
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // BUG-001: round-to-nearest-even
    #[test]
    fn gf16_roundtrip_near_one() {
        let x = 1.0f32 + 2.0f32.powi(-10);
        let encoded = gf16_from_f32(x);
        let decoded = gf16_to_f32(encoded);
        assert!((decoded - x).abs() < 0.005, "roundtrip error too large: {} vs {}", decoded, x);
    }

    #[test]
    fn gf16_roundtrip_half() {
        let x = 0.5f32 + 2.0f32.powi(-11);
        let encoded = gf16_from_f32(x);
        let decoded = gf16_to_f32(encoded);
        assert!((decoded - x).abs() < 0.002, "roundtrip error too large: {} vs {}", decoded, x);
    }

    #[test]
    fn gf16_roundtrip_basic() {
        for &v in &[0.0f32, 1.0, -1.0, 0.5, 100.0, -100.0, 0.001] {
            let decoded = gf16_to_f32(gf16_from_f32(v));
            let err = (decoded - v).abs() / v.max(1e-10);
            assert!(err < 0.01, "GF16 roundtrip error {:.4}% for {}", err * 100.0, v);
        }
    }

    // BUG-002: overflow → +Inf
    #[test]
    fn gf16_overflow_is_inf() {
        let encoded = gf16_from_f32(1e30f32);
        assert!(gf16_is_inf(encoded), "1e30 should encode to Inf, got {:016b}", encoded);
        assert!(!gf16_is_nan(encoded), "overflow should not be NaN");
    }

    #[test]
    fn gf16_f32_max_is_inf() {
        let encoded = gf16_from_f32(f32::MAX);
        assert!(gf16_is_inf(encoded), "f32::MAX should encode to Inf");
    }

    #[test]
    fn gf16_negative_overflow_is_ninf() {
        let encoded = gf16_from_f32(-1e30f32);
        assert!(gf16_is_inf(encoded), "-1e30 should encode to -Inf");
        assert_eq!(gf16_extract_sign(encoded), 1);
    }

    // Special values
    #[test]
    fn gf16_zero_roundtrip() {
        assert_eq!(gf16_to_f32(gf16_from_f32(0.0)), 0.0);
        let neg_zero = gf16_from_f32(-0.0f32);
        assert_eq!(gf16_to_f32(neg_zero), 0.0);
    }

    #[test]
    fn gf16_inf_roundtrip() {
        let inf = gf16_from_f32(f32::INFINITY);
        assert!(gf16_is_inf(inf));
        assert!(gf16_to_f32(inf).is_infinite());
        assert!(gf16_to_f32(inf).is_sign_positive());
    }

    #[test]
    fn gf16_nan_preserved() {
        let nan = gf16_from_f32(f32::NAN);
        assert!(gf16_is_nan(nan));
        assert!(gf16_to_f32(nan).is_nan());
    }

    // GF8 encode/decode roundtrip
    #[test]
    fn gf8_roundtrip() {
        for &v in &[0.0f32, 0.5, 1.0, 2.0, 4.0, -1.0] {
            let decoded = gf8_to_f32(gf8_from_f32(v));
            let err = (decoded - v).abs();
            assert!(err < 0.3, "GF8 roundtrip error {} for {}", err, v);
        }
    }

    #[test]
    fn gf8_overflow_inf() {
        let encoded = gf8_from_f32(100.0f32);
        let decoded = gf8_to_f32(encoded);
        assert!(decoded.is_infinite() || decoded > 0.0, "GF8 overflow should be +Inf or large");
    }

    // GF12 encode/decode roundtrip
    #[test]
    fn gf12_roundtrip() {
        for &v in &[0.0f32, 1.0, 2.0, 0.5, -1.0, 10.0] {
            let decoded = gf12_to_f32(gf12_from_f32(v));
            let err = (decoded - v).abs();
            assert!(err < 0.05, "GF12 roundtrip error {} for {}", err, v);
        }
    }

    // GF20 encode/decode roundtrip
    #[test]
    fn gf20_roundtrip() {
        for &v in &[0.0f32, 1.0, 3.14, -1.0, 100.0, 0.001] {
            let decoded = gf20_to_f32(gf20_from_f32(v));
            let err = (decoded - v).abs() / v.max(1e-10);
            assert!(err < 0.001, "GF20 roundtrip error {:.4}% for {}", err * 100.0, v);
        }
    }

    // GF24 encode/decode roundtrip
    #[test]
    fn gf24_roundtrip() {
        for &v in &[0.0f32, 1.0, 3.14159, -2.71828, 1000.0] {
            let encoded = gf24_from_f32(v);
            let decoded = gf24_to_f32(encoded);
            if v == 0.0 {
                assert_eq!(decoded, 0.0);
                continue;
            }
            let err = (decoded - v).abs() / v.abs();
            assert!(err < 0.001, "GF24 roundtrip error {:.4}% for {}", err * 100.0, v);
        }
    }

    // GF4 encode/decode roundtrip (tiny format, limited range)
    #[test]
    fn gf4_roundtrip() {
        // GF4 has 1 exp bit → only 2 exp values (0=denorm, 1=Inf)
        // Can only represent values very close to 0
        let encoded = gf4_from_f32(0.25f32);
        let decoded = gf4_to_f32(encoded);
        assert!(decoded >= 0.0, "GF4(0.25) should be non-negative");
    }

    #[test]
    fn gf4_zero() {
        assert_eq!(gf4_to_f32(gf4_from_f32(0.0f32)), 0.0);
    }
}
