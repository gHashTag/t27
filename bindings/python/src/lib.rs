/* phi^2 + phi^-2 = 3 | TRINITY */
use pyo3::prelude::*;
use pyo3::types::PyDict;
use numpy::{prelude::*, PyArray1, PyArray2, PyArrayDyn, PyReadonlyArrayDyn};
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

// ============================================================================
// GF16 Class
// ============================================================================

#[pyclass(name = "GF16", module = "golden_float")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PyGF16 {
    pub inner: u16,
}

#[pymethods]
impl PyGF16 {
    #[new]
    fn new(value: f64) -> Self {
        PyGF16 { inner: gf16_from_f64(value) }
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

    fn __str__(&self) -> String {
        format!("{:.6}", self.to_float())
    }

    fn __float__(&self) -> f64 { self.to_float() }

    fn __int__(&self) -> u64 { self.inner as u64 }

    fn __add__(&self, other: &Self) -> Self {
        PyGF16 { inner: gf16_add(self.inner, other.inner) }
    }

    fn __sub__(&self, other: &Self) -> Self {
        PyGF16 { inner: gf16_sub(self.inner, other.inner) }
    }

    fn __mul__(&self, other: &Self) -> Self {
        PyGF16 { inner: gf16_mul(self.inner, other.inner) }
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF16 { inner: gf16_div(self.inner, other.inner) })
    }

    fn __neg__(&self) -> Self {
        PyGF16 { inner: self.inner ^ 0x8000 }
    }

    fn __eq__(&self, other: &Self) -> bool {
        gf16_eq(self.inner, other.inner)
    }

    fn __lt__(&self, other: &Self) -> bool {
        gf16_lt(self.inner, other.inner)
    }

    fn __le__(&self, other: &Self) -> bool {
        self.__lt__(other) || self.__eq__(other)
    }

    fn __gt__(&self, other: &Self) -> bool {
        !self.__le__(other)
    }

    fn __ge__(&self, other: &Self) -> bool {
        !self.__lt__(other)
    }

    fn __hash__(&self) -> u64 { self.inner as u64 }

    /// Convert to/from bytes
    #[staticmethod]
    fn from_bytes(data: [u8; 2]) -> Self {
        PyGF16 { inner: u16::from_be_bytes(data) }
    }

    fn to_bytes(&self) -> [u8; 2] {
        self.inner.to_be_bytes()
    }

    /// Class method from bits
    #[staticmethod]
    fn from_bits(bits: u16) -> Self {
        PyGF16 { inner: bits }
    }
}

// ============================================================================
// GF32 Class
// ============================================================================

#[pyclass(name = "GF32", module = "golden_float")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PyGF32 {
    pub inner: u32,
}

#[pymethods]
impl PyGF32 {
    #[new]
    fn new(value: f64) -> Self {
        PyGF32 { inner: gf32_from_f64(value) }
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

    fn __str__(&self) -> String {
        format!("{:.8}", self.to_float())
    }

    fn __float__(&self) -> f64 { self.to_float() }

    fn __int__(&self) -> u64 { self.inner as u64 }

    fn __add__(&self, other: &Self) -> Self {
        PyGF32 { inner: gf32_add(self.inner, other.inner) }
    }

    fn __sub__(&self, other: &Self) -> Self {
        PyGF32 { inner: gf32_sub(self.inner, other.inner) }
    }

    fn __mul__(&self, other: &Self) -> Self {
        PyGF32 { inner: gf32_mul(self.inner, other.inner) }
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        Ok(PyGF32 { inner: gf32_div(self.inner, other.inner) })
    }

    fn __neg__(&self) -> Self {
        PyGF32 { inner: self.inner ^ 0x80000000 }
    }

    fn __eq__(&self, other: &Self) -> bool {
        gf32_eq(self.inner, other.inner)
    }

    fn __lt__(&self, other: &Self) -> bool {
        gf32_lt(self.inner, other.inner)
    }

    fn __le__(&self, other: &Self) -> bool {
        self.__lt__(other) || self.__eq__(other)
    }

    fn __gt__(&self, other: &Self) -> bool {
        !self.__le__(other)
    }

    fn __ge__(&self, other: &Self) -> bool {
        !self.__lt__(other)
    }

    fn __hash__(&self) -> u64 { self.inner as u64 }

    /// Convert to/from bytes
    #[staticmethod]
    fn from_bytes(data: [u8; 4]) -> Self {
        PyGF32 { inner: u32::from_be_bytes(data) }
    }

    fn to_bytes(&self) -> [u8; 4] {
        self.inner.to_be_bytes()
    }

    /// Class method from bits
    #[staticmethod]
    fn from_bits(bits: u32) -> Self {
        PyGF32 { inner: bits }
    }
}

// ============================================================================
// NumPy Array Operations
// ============================================================================

#[pyfunction]
fn array_to_gf16<'py>(
    py: Python<'py>,
    arr: PyReadonlyArrayDyn<f64>,
) -> PyResult<&'py PyArray1<u16>> {
    let arr = arr.as_array();
    let result: Vec<u16> = arr.iter().map(|&x| gf16_from_f64(x)).collect();
    Ok(PyArray1::from_vec(py, result))
}

#[pyfunction]
fn gf16_array_to_float<'py>(
    py: Python<'py>,
    arr: PyReadonlyArrayDyn<u16>,
) -> PyResult<&'py PyArray1<f64>> {
    let arr = arr.as_array();
    let result: Vec<f64> = arr.iter().map(|&x| gf16_to_f64(x)).collect();
    Ok(PyArray1::from_vec(py, result))
}

#[pyfunction]
fn gf16_dot_product(
    a: PyReadonlyArrayDyn<u16>,
    b: PyReadonlyArrayDyn<u16>,
) -> PyResult<u16> {
    let a = a.as_array();
    let b = b.as_array();

    if a.shape() != b.shape() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Arrays must have the same shape",
        ));
    }

    let mut acc: u16 = 0;
    for (a_val, b_val) in a.iter().zip(b.iter()) {
        let prod = gf16_mul(*a_val, *b_val);
        acc = gf16_add(acc, prod);
    }
    Ok(acc)
}

#[pyfunction]
fn gf16_normalize<'py>(
    py: Python<'py>,
    arr: PyReadonlyArrayDyn<u16>,
) -> PyResult<&'py PyArray1<u16>> {
    let arr = arr.as_array();

    // Compute L2 norm in f64, then normalize
    let mut sum: f64 = 0.0;
    for &val in arr.iter() {
        let f = gf16_to_f64(val);
        sum += f * f;
    }
    let norm = sum.sqrt();

    if norm == 0.0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Cannot normalize zero vector",
        ));
    }

    let result: Vec<u16> = arr.iter()
        .map(|&val| gf16_from_f64(gf16_to_f64(val) / norm))
        .collect();
    Ok(PyArray1::from_vec(py, result))
}

#[pyfunction]
fn gf16_quantize_matrix<'py>(
    py: Python<'py>,
    mat: PyReadonlyArrayDyn<f64>,
) -> PyResult<&'py PyArray2<u16>> {
    let mat = mat.as_array();
    let mut result = Vec::with_capacity(mat.len());

    if mat.ndim() == 2 {
        let shape = (mat.shape()[0], mat.shape()[1]);
        for &val in mat.iter() {
            result.push(gf16_from_f64(val));
        }
        let array = PyArray2::from_vec_shape(py, shape, result)?;
        Ok(array)
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Input must be a 2D matrix",
        ))
    }
}

// ============================================================================
// Constants and Utilities
// ============================================================================

#[pyfunction]
fn phi() -> f64 {
    1.618033988749895_f64
}

#[pyfunction]
fn phi_gf16() -> PyGF16 {
    PyGF16 { inner: gf16_from_f64(1.618033988749895) }
}

#[pyfunction]
fn phi_gf32() -> PyGF32 {
    PyGF32 { inner: gf32_from_f64(1.618033988749895) }
}

#[pyfunction]
fn trinity_identity() -> bool {
    // φ² + φ⁻² = 3 (with tolerance for floating point)
    let phi = 1.618033988749895_f64;
    let phi_sq = phi * phi;
    let phi_neg_sq = 1.0 / (phi * phi);
    let sum = phi_sq + phi_neg_sq;
    (sum - 3.0).abs() < 1e-10
}

#[pyfunction]
fn gf16_bias() -> i32 {
    31
}

#[pyfunction]
fn gf32_bias() -> i32 {
    127
}

#[pyfunction]
fn gf16_exp_bits() -> u32 {
    6
}

#[pyfunction]
fn gf16_mant_bits() -> u32 {
    9
}

#[pymodule]
fn golden_float(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // Classes
    m.add_class::<PyGF16>()?;
    m.add_class::<PyGF32>()?;

    // Functions
    m.add_function(wrap_pyfunction!(array_to_gf16, m)?)?;
    m.add_function(wrap_pyfunction!(gf16_array_to_float, m)?)?;
    m.add_function(wrap_pyfunction!(gf16_dot_product, m)?)?;
    m.add_function(wrap_pyfunction!(gf16_normalize, m)?)?;
    m.add_function(wrap_pyfunction!(gf16_quantize_matrix, m)?)?;

    // Constants
    m.add_function(wrap_pyfunction!(phi, m)?)?;
    m.add_function(wrap_pyfunction!(phi_gf16, m)?)?;
    m.add_function(wrap_pyfunction!(phi_gf32, m)?)?;
    m.add_function(wrap_pyfunction!(trinity_identity, m)?)?;

    // Format info
    m.add_function(wrap_pyfunction!(gf16_bias, m)?)?;
    m.add_function(wrap_pyfunction!(gf32_bias, m)?)?;
    m.add_function(wrap_pyfunction!(gf16_exp_bits, m)?)?;
    m.add_function(wrap_pyfunction!(gf16_mant_bits, m)?)?;

    // Version
    m.add("__version__", "1.0.0")?;

    Ok(())
}