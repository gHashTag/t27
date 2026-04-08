/* phi^2 + phi^-2 = 3 | TRINITY */

use wasm_bindgen::prelude::*;

// ============================================================================
// Helper Functions (implementing GF logic directly)
// ============================================================================

fn gf16_encode(value: f64) -> u16 {
    if value == 0.0 {
        return if value.is_sign_negative() { 0x8000u16 } else { 0u16 };
    }

    let sign = if value < 0.0 { 0x8000u16 } else { 0u16 };
    let abs_value = if value < 0.0 { -value } else { value };

    // Get IEEE 754 binary representation
    let bits = abs_value.to_bits();
    let ieee_exp = ((bits >> 52) & 0x7FF) as i32 - 1023;
    let ieee_mant = bits & 0x000FFFFFFFFFFFFF;

    let mut gf16_exp = ieee_exp + 31; // GF16_BIAS = 31
    if gf16_exp < 0 { gf16_exp = 0; }
    if gf16_exp > 62 { gf16_exp = 62; }

    // Convert from 52-bit IEEE mantissa to 9-bit GF16 mantissa
    let mut gf16_mant = (ieee_mant >> 43) as u16;

    // Round to nearest
    let discarded = ieee_mant & 0x7FFFFFFFFFF;
    if discarded & 0x4000000000 != 0 {
        gf16_mant += 1;
        if gf16_mant > 511 {
            gf16_mant = 0;
            if gf16_exp < 62 { gf16_exp += 1; }
        }
    }

    sign | ((gf16_exp as u16) << 9) | gf16_mant
}

fn gf16_decode(value: u16) -> f64 {
    if value == 0x0000 || value == 0x8000 {
        return if value == 0x8000 { -0.0f64 } else { 0.0f64 };
    }

    let sign_f = if (value & 0x8000) != 0 { -1.0f64 } else { 1.0f64 };
    let exp = ((value >> 9) & 0x3F) as i32;
    let mant = (value & 0x01FF) as f64;

    if exp == 63 {
        // Special values
        return if mant == 0.0 {
            if sign_f < 0.0 { f64::NEG_INFINITY } else { f64::INFINITY }
        } else {
            f64::NAN
        };
    }

    let mant_norm = 1.0f64 + mant / 512.0f64;
    let exp_adj = exp - 31;
    sign_f * mant_norm * 2.0_f64.powi(exp_adj)
}

fn gf16_is_zero(value: u16) -> bool {
    value == 0x0000 || value == 0x8000
}

fn gf16_is_inf(value: u16) -> bool {
    let exp = ((value >> 9) & 0x3F) as i32;
    let mant = (value & 0x01FF) as i32;
    exp == 63 && mant == 0
}

fn gf16_is_nan(value: u16) -> bool {
    let exp = ((value >> 9) & 0x3F) as i32;
    let mant = (value & 0x01FF) as i32;
    exp == 63 && mant != 0
}

fn gf32_encode(value: f64) -> u32 {
    if value == 0.0 {
        return if value.is_sign_negative() { 0x80000000u32 } else { 0u32 };
    }

    let sign = if value < 0.0 { 0x80000000u32 } else { 0u32 };
    let abs_value = if value < 0.0 { -value } else { value };

    let bits = abs_value.to_bits();
    let ieee_exp = ((bits >> 52) & 0x7FF) as i32 - 1023;
    let ieee_mant = bits & 0x000FFFFFFFFFFFFF;

    let mut gf32_exp = ieee_exp + 2047; // GF32_BIAS = 2047
    if gf32_exp < 0 { gf32_exp = 0; }
    if gf32_exp > 4094 { gf32_exp = 4094; }

    // Convert from 52-bit IEEE mantissa to 19-bit GF32 mantissa
    let gf32_mant = (ieee_mant >> 33) & 0x7FFFF;

    (((sign as u64) | ((gf32_exp as u64) << 19) | (gf32_mant as u64)) & 0xFFFFFFFF) as u32
}

fn gf32_decode(value: u32) -> f64 {
    if value == 0 {
        return 0.0f64;
    }

    let sign_f = if (value & 0x80000000) != 0 { -1.0f64 } else { 1.0f64 };
    let exp = ((value >> 19) & 0xFFF) as i32;
    let mant = (value & 0x7FFFF) as f64;

    if exp == 4095 {
        return if mant == 0.0 {
            if sign_f < 0.0 { f64::NEG_INFINITY } else { f64::INFINITY }
        } else {
            f64::NAN
        };
    }

    let mant_norm = 1.0f64 + mant / 524288.0f64;
    let exp_adj = exp - 2047;
    sign_f * mant_norm * 2.0_f64.powi(exp_adj)
}

fn gf32_is_zero(value: u32) -> bool {
    value == 0
}

fn gf32_is_inf(value: u32) -> bool {
    let exp = ((value >> 19) & 0xFFF) as i32;
    exp == 4095
}

fn gf32_is_nan(value: u32) -> bool {
    let exp = ((value >> 19) & 0xFFF) as i32;
    let mant = (value & 0x7FFFF) as i32;
    exp == 4095 && mant != 0
}

// ============================================================================
// JavaScript/Wasm Classes
// ============================================================================

/// GF16: 16-bit phi-structured floating-point format
#[wasm_bindgen]
pub struct GF16 {
    value: u16,
}

#[wasm_bindgen]
impl GF16 {
    #[wasm_bindgen(constructor)]
    pub fn new(value: f64) -> GF16 {
        GF16 { value: gf16_encode(value) }
    }

    pub fn to_float(&self) -> f64 {
        gf16_decode(self.value)
    }

    pub fn bits(&self) -> u16 {
        self.value
    }

    pub fn is_zero(&self) -> bool {
        gf16_is_zero(self.value)
    }

    pub fn is_inf(&self) -> bool {
        gf16_is_inf(self.value)
    }

    pub fn is_nan(&self) -> bool {
        gf16_is_nan(self.value)
    }
}

/// GF32: 32-bit phi-structured floating-point format
#[wasm_bindgen]
pub struct GF32 {
    value: u32,
}

#[wasm_bindgen]
impl GF32 {
    #[wasm_bindgen(constructor)]
    pub fn new(value: f64) -> GF32 {
        GF32 { value: gf32_encode(value) }
    }

    pub fn to_float(&self) -> f64 {
        gf32_decode(self.value)
    }

    pub fn bits(&self) -> u32 {
        self.value
    }

    pub fn is_zero(&self) -> bool {
        gf32_is_zero(self.value)
    }

    pub fn is_inf(&self) -> bool {
        gf32_is_inf(self.value)
    }

    pub fn is_nan(&self) -> bool {
        gf32_is_nan(self.value)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the constant phi (golden ratio) as GF16
#[wasm_bindgen]
pub fn phi_gf16() -> u16 {
    gf16_encode(1.618033988749895)
}

/// Get the constant phi (golden ratio) as GF32
#[wasm_bindgen]
pub fn phi_gf32() -> u32 {
    gf32_encode(1.618033988749895)
}

/// Check if the value represents positive zero
#[wasm_bindgen]
pub fn is_positive_zero(value: u16) -> bool {
    value == 0x0000
}

/// Check if the value represents negative zero
#[wasm_bindgen]
pub fn is_negative_zero(value: u16) -> bool {
    value == 0x8000
}
