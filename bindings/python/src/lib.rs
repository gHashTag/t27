/* phi^2 + phi^-2 = 3 | TRINITY */
use pyo3::prelude::*;
use golden_float_ffi::{
    gf16_from_f64, gf16_to_f64,
    gf16_add, gf16_sub, gf16_mul, gf16_div,
    gf16_is_zero, gf16_is_nan, gf16_is_inf,
    gf16_extract_sign, gf16_extract_exponent, gf16_extract_mantissa,
    gf16_eq, gf16_lt,
    gf32_from_f64, gf32_to_f64,
    gf32_add, gf32_sub, gf32_mul, gf32_div,
    gf32_is_zero, gf32_is_nan, gf32_is_inf,
    gf32_extract_sign, gf32_extract_exponent, gf32_extract_mantissa,
    gf32_eq, gf32_lt,
};

#[pyclass(name = "GF16", module = "golden_float")]
#[derive(Clone, Copy, Debug)]
pub struct PyGF16 {
    pub inner: u16,
}

#[pymethods]
impl PyGF16 {
    #[new]
    fn new(value: &Bound<PyAny>) -> PyResult<Self> {
        let float_val = value.extract::<f64>()?;
        Ok(PyGF16 { inner: gf16_from_f64(float_val) })
    }
    fn bits(&self) -> u16 { self.inner }
    fn to_float(&self) -> f64 { gf16_to_f64(self.inner) }
    fn is_zero(&self) -> bool { gf16_is_zero(self.inner) }
    fn is_inf(&self) -> bool { gf16_is_inf(self.inner) }
    fn is_nan(&self) -> bool { gf16_is_nan(self.inner) }
    fn sign(&self) -> u8 { gf16_extract_sign(self.inner) }
    fn exponent(&self) -> u8 { gf16_extract_exponent(self.inner) }
    fn mantissa(&self) -> i16 { gf16_extract_mantissa(self.inner) }
    fn __repr__(&self) -> String {
        format!("GF16({:.6}, bits=0x{:04X})", self.to_float(), self.inner)
    }
    fn __float__(&self) -> f64 { self.to_float() }
    fn __int__(&self) -> u64 { self.inner as u64 }
    fn __add__(&self, other: &Self) -> PyGF16 {
        PyGF16 { inner: gf16_add(self.inner, other.inner) }
    }
    fn __sub__(&self, other: &Self) -> PyGF16 {
        PyGF16 { inner: gf16_sub(self.inner, other.inner) }
    }
    fn __mul__(&self, other: &Self) -> PyGF16 {
        PyGF16 { inner: gf16_mul(self.inner, other.inner) }
    }
    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF16 { inner: gf16_div(self.inner, other.inner) })
    }
    fn __eq__(&self, other: &Self) -> bool {
        gf16_eq(self.inner, other.inner)
    }
    fn __lt__(&self, other: &Self) -> bool {
        gf16_lt(self.inner, other.inner)
    }
    fn __hash__(&self) -> u64 { self.inner as u64 }
}

#[pyclass(name = "GF32", module = "golden_float")]
#[derive(Clone, Copy, Debug)]
pub struct PyGF32 {
    pub inner: u32,
}

#[pymethods]
impl PyGF32 {
    #[new]
    fn new(value: &Bound<PyAny>) -> PyResult<Self> {
        let float_val = value.extract::<f64>()?;
        Ok(PyGF32 { inner: gf32_from_f64(float_val) })
    }
    fn bits(&self) -> u32 { self.inner }
    fn to_float(&self) -> f64 { gf32_to_f64(self.inner) }
    fn is_zero(&self) -> bool { gf32_is_zero(self.inner) }
    fn is_inf(&self) -> bool { gf32_is_inf(self.inner) }
    fn is_nan(&self) -> bool { gf32_is_nan(self.inner) }
    fn sign(&self) -> u8 { gf32_extract_sign(self.inner) }
    fn exponent(&self) -> u8 { gf32_extract_exponent(self.inner) }
    fn mantissa(&self) -> i32 { gf32_extract_mantissa(self.inner) }
    fn __repr__(&self) -> String {
        format!("GF32({:.8}, bits=0x{:08X})", self.to_float(), self.inner)
    }
    fn __float__(&self) -> f64 { self.to_float() }
    fn __int__(&self) -> u64 { self.inner as u64 }
    fn __add__(&self, other: &Self) -> PyGF32 {
        PyGF32 { inner: gf32_add(self.inner, other.inner) }
    }
    fn __sub__(&self, other: &Self) -> PyGF32 {
        PyGF32 { inner: gf32_sub(self.inner, other.inner) }
    }
    fn __mul__(&self, other: &Self) -> PyGF32 {
        PyGF32 { inner: gf32_mul(self.inner, other.inner) }
    }
    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF32 { inner: gf32_div(self.inner, other.inner) })
    }
    fn __eq__(&self, other: &Self) -> bool {
        gf32_eq(self.inner, other.inner)
    }
    fn __lt__(&self, other: &Self) -> bool {
        gf32_lt(self.inner, other.inner)
    }
    fn __hash__(&self) -> u64 { self.inner as u64 }
}

#[pymodule]
fn golden_float(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyGF16>()?;
    m.add_class::<PyGF32>()?;
    Ok(())
}
