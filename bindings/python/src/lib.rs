/* phi^2 + phi^-2 = 3 | TRINITY */

use pyo3::prelude::*;

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
    let ieee_exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let ieee_mant = bits & 0x7FFFFF;

    let mut gf16_exp = ieee_exp + 31; // GF16_BIAS = 31
    if gf16_exp < 0 { gf16_exp = 0; }
    if gf16_exp > 62 { gf16_exp = 62; }

    let mut gf16_mant = (ieee_mant >> 14) as u16;

    // Round to nearest
    let discarded = ieee_mant & 0x3FFF;
    if discarded & 0x2000 != 0 {
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
    let ieee_exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let ieee_mant = bits & 0x7FFFFF;

    let mut gf32_exp = ieee_exp + 2047; // GF32_BIAS = 2047
    if gf32_exp < 0 { gf32_exp = 0; }
    if gf32_exp > 4094 { gf32_exp = 4094; }

    // Scale mantissa for GF32 (19 bits vs 23 bits)
    let gf32_mant = (ieee_mant >> 4) & 0x7FFFF;

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

fn gf16_add(a: u16, b: u16) -> u16 {
    let a_f32 = gf16_decode(a);
    let b_f32 = gf16_decode(b);
    let sum = a_f32 + b_f32;
    gf16_encode(sum)
}

fn gf16_sub(a: u16, b: u16) -> u16 {
    let a_f32 = gf16_decode(a);
    let b_f32 = gf16_decode(b);
    let diff = a_f32 - b_f32;
    gf16_encode(diff)
}

fn gf16_mul(a: u16, b: u16) -> u16 {
    let a_f32 = gf16_decode(a);
    let b_f32 = gf16_decode(b);
    let prod = a_f32 * b_f32;
    gf16_encode(prod)
}

fn gf16_div(a: u16, b: u16) -> u16 {
    let a_f32 = gf16_decode(a);
    let b_f32 = gf16_decode(b);
    if b_f32 == 0.0f64 {
        return if a_f32 < 0.0f64 { 0xFE00u16 } else { 0x7E00u16 };
    }
    gf16_encode(a_f32 / b_f32)
}

fn gf16_eq(a: u16, b: u16) -> bool {
    if gf16_is_nan(a) || gf16_is_nan(b) {
        return false;
    }
    if gf16_is_zero(a) && gf16_is_zero(b) {
        return true;
    }
    a == b
}

fn gf16_lt(a: u16, b: u16) -> bool {
    if gf16_is_nan(a) || gf16_is_nan(b) {
        return false;
    }
    gf16_decode(a) < gf16_decode(b)
}

fn gf32_add(a: u32, b: u32) -> u32 {
    let a_f32 = gf32_decode(a);
    let b_f32 = gf32_decode(b);
    let sum = a_f32 + b_f32;
    gf32_encode(sum)
}

fn gf32_sub(a: u32, b: u32) -> u32 {
    let a_f32 = gf32_decode(a);
    let b_f32 = gf32_decode(b);
    let diff = a_f32 - b_f32;
    gf32_encode(diff)
}

fn gf32_mul(a: u32, b: u32) -> u32 {
    let a_f32 = gf32_decode(a);
    let b_f32 = gf32_decode(b);
    let prod = a_f32 * b_f32;
    gf32_encode(prod)
}

fn gf32_div(a: u32, b: u32) -> u32 {
    let a_f32 = gf32_decode(a);
    let b_f32 = gf32_decode(b);
    if b_f32 == 0.0f64 {
        return if a_f32 < 0.0f64 { 0xFFF00000u32 } else { 0x7FF00000u32 };
    }
    gf32_encode(a_f32 / b_f32)
}

fn gf32_eq(a: u32, b: u32) -> bool {
    if gf32_is_nan(a) || gf32_is_nan(b) {
        return false;
    }
    a == b
}

fn gf32_lt(a: u32, b: u32) -> bool {
    if gf32_is_nan(a) || gf32_is_nan(b) {
        return false;
    }
    gf32_decode(a) < gf32_decode(b)
}

// ============================================================================
// Python Classes
// ============================================================================

/// GF16: 16-bit phi-structured floating-point format
#[pyclass(name = "GF16", module = "golden_float")]
#[derive(Clone, Debug)]
pub struct PyGF16 {
    value: u16,
}

#[pymethods]
impl PyGF16 {
    #[new]
    #[pyo3(text_signature = "value")]
    fn new(value: &Bound<PyAny>) -> PyResult<Self> {
        let float_val = value.extract::<f64>()?;
        Ok(PyGF16 { value: gf16_encode(float_val) })
    }

    fn to_float(&self) -> f64 {
        gf16_decode(self.value)
    }

    fn bits(&self) -> u16 {
        self.value
    }

    fn is_zero(&self) -> bool {
        gf16_is_zero(self.value)
    }

    fn is_inf(&self) -> bool {
        gf16_is_inf(self.value)
    }

    fn is_nan(&self) -> bool {
        gf16_is_nan(self.value)
    }

    fn __repr__(&self) -> String {
        let fval = gf16_decode(self.value);
        format!("GF16({})", fval)
    }

    fn __float__(&self) -> f64 {
        gf16_decode(self.value)
    }

    fn __str__(&self) -> String {
        let fval = gf16_decode(self.value);
        format!("{}", fval)
    }

    fn __add__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF16 {
            value: gf16_add(self.value, other.value)
        })
    }

    fn __sub__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF16 {
            value: gf16_sub(self.value, other.value)
        })
    }

    fn __mul__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF16 {
            value: gf16_mul(self.value, other.value)
        })
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF16 {
            value: gf16_div(self.value, other.value)
        })
    }

    fn __eq__(&self, other: &Self) -> bool {
        gf16_eq(self.value, other.value)
    }

    fn __lt__(&self, other: &Self) -> bool {
        gf16_lt(self.value, other.value)
    }
}

/// GF32: 32-bit phi-structured floating-point format
#[pyclass(name = "GF32", module = "golden_float")]
#[derive(Clone, Debug)]
pub struct PyGF32 {
    value: u32,
}

#[pymethods]
impl PyGF32 {
    #[new]
    #[pyo3(text_signature = "value")]
    fn new(value: &Bound<PyAny>) -> PyResult<Self> {
        let float_val = value.extract::<f64>()?;
        Ok(PyGF32 { value: gf32_encode(float_val) })
    }

    fn to_float(&self) -> f64 {
        gf32_decode(self.value)
    }

    fn bits(&self) -> u32 {
        self.value
    }

    fn is_zero(&self) -> bool {
        gf32_is_zero(self.value)
    }

    fn is_inf(&self) -> bool {
        gf32_is_inf(self.value)
    }

    fn is_nan(&self) -> bool {
        gf32_is_nan(self.value)
    }

    fn __repr__(&self) -> String {
        let fval = gf32_decode(self.value);
        format!("GF32({})", fval)
    }

    fn __float__(&self) -> f64 {
        gf32_decode(self.value)
    }

    fn __str__(&self) -> String {
        let fval = gf32_decode(self.value);
        format!("{}", fval)
    }

    fn __add__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF32 {
            value: gf32_add(self.value, other.value)
        })
    }

    fn __sub__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF32 {
            value: gf32_sub(self.value, other.value)
        })
    }

    fn __mul__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF32 {
            value: gf32_mul(self.value, other.value)
        })
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF32 {
            value: gf32_div(self.value, other.value)
        })
    }

    fn __eq__(&self, other: &Self) -> bool {
        gf32_eq(self.value, other.value)
    }

    fn __lt__(&self, other: &Self) -> bool {
        gf32_lt(self.value, other.value)
    }
}

// ============================================================================
// Module Initialization
// ============================================================================

#[pymodule]
fn golden_float(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyGF16>()?;
    m.add_class::<PyGF32>()?;
    Ok(())
}
