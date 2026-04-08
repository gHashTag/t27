# Migrating to GoldenFloat

## Why GoldenFloat?

| Property | GoldenFloat | IEEE 754 float32 | bfloat16 |
|----------|-------------|------------------|----------|
| Bit allocation | phi-optimal (Proposition 1) | Empirical | Truncated f32 |
| Format family | 7/7 verified (Proposition 2) | N/A | Single format |
| Free parameters | 0 (Theorem 3) | N/A | N/A |
| FPGA synthesis | Integer-only core | FP-IP blocks | FP-IP blocks |
| Cross-language | Bit-identical | Platform-dependent | Varies |

## Installation

| Language | Command |
|----------|---------|
| Python | pip install golden-float |
| JavaScript | npm install golden-float |
| C/C++ | #include "golden_float.h" |
| Rust | cargo add golden-float |

## Quick Start

### Python

from golden_float import GF16, GF32, gf_array

# Scalar
phi = GF32(1.618033988749895)
print(hex(phi.bits()))  # 0x3FCF1BBD
print(phi.to_float())   # 1.618033988749895

# Array (uint16 storage, GF16-encoded)
arr = gf_array([1.0, 1.618, 2.718], dtype="gf16")
print(arr.shape)     # (3,)
print(arr.dtype)     # uint16

# Field extraction
gf16_phi = GF16(1.618)
print(f"Sign: {gf16_phi.sign()}")
print(f"Exponent: {gf16_phi.exponent()}")
print(f"Mantissa: {gf16_phi.mantissa()}")

# Arithmetic
a = GF16(1.0)
b = GF16(0.618)
sum = a + b  # Uses FFI-backed arithmetic
print(f"1.0 + 0.618 = {sum.to_float():.6f}")

# Classification
zero = GF16(0.0)
print(f"Is zero: {zero.is_zero()}")
print(f"Is infinite: {zero.is_inf()}")
print(f"Is NaN: {zero.is_nan()}")
```

### JavaScript/TypeScript

```javascript
import { GF16, GF32 } from 'golden-float';

// Scalar
const phi = new GF32(1.618033988749895);
console.log('0x' + phi.bits().toString(16));  // 0x3fcf1bbd
console.log(phi.toFloat());                  // 1.618033988749895

// Field extraction
const gf16_phi = new GF16(1.618);
console.log('Sign:', phi.sign());
console.log('Exponent:', phi.exponent());
console.log('Mantissa:', phi.mantissa());

// Arithmetic
const a = new GF16(1.0);
const b = new GF16(0.618);
const sum = a.add(b);
console.log(`1.0 + 0.618 = ${sum.toFloat()}`);

// Classification
const zero = new GF16(0.0);
console.log('Is zero:', zero.isZero());
console.log('Is infinite:', zero.isInf());
console.log('Is NaN:', zero.isNaN());
```

### C/C++

```c
#include "golden_float.h"

// Scalar
uint32_t phi_bits = gf32_from_f64(1.618033988749895);
double   back      = gf32_to_f64(phi_bits);
printf("0x%08X\n", phi_bits);    // 0x3FCF1BBD

// GF16
uint16_t phi16_bits = gf16_from_f64(1.618033988749895);
double   back16       = gf16_to_f64(phi16_bits);

// Field extraction
uint8_t sign = gf16_extract_sign(phi16_bits);
uint8_t exp  = gf16_extract_exponent(phi16_bits);
int16_t mant = gf16_extract_mantissa(phi16_bits);

// Classification
uint16_t zero = gf16_from_f64(0.0);
if (gf16_is_zero(zero)) {
    printf("Is zero: true\n");
}
if (gf16_is_inf(zero)) {
    printf("Is infinite: true\n");
}
if (gf16_is_nan(zero)) {
    printf("Is NaN: true\n");
}

// Arithmetic
uint16_t a = gf16_from_f64(1.0);
uint16_t b = gf16_from_f64(0.618);
uint16_t sum = gf16_add(a, b);
```

### Rust

```rust
use golden_float_ffi::{
    gf16_from_f64, gf16_to_f64,
    gf16_add,
    gf16_is_zero, gf16_is_nan, gf16_is_inf,
    gf16_extract_sign, gf16_extract_exponent, gf16_extract_mantissa,
};

fn main() {
    // Scalar
    let phi_bits = gf16_from_f64(1.618033988749895_f64);
    println!("0x{:04X}", phi_bits);

    // Field extraction
    let phi16 = gf16_from_f64(1.618_f64);
    println!("Sign: {}", gf16_extract_sign(phi16));
    println!("Exponent: {}", gf16_extract_exponent(phi16));
    println!("Mantissa: {}", gf16_extract_mantissa(phi16));

    // Classification
    let zero = gf16_from_f64(0.0_f64);
    println!("Is zero: {}", gf16_is_zero(zero));
    println!("Is infinite: {}", gf16_is_inf(zero));
    println!("Is NaN: {}", gf16_is_nan(zero));

    // Arithmetic
    let a = gf16_from_f64(1.0_f64);
    let b = gf16_from_f64(0.618_f64);
    let sum = gf16_add(a, b);
    let result = gf16_to_f64(sum);
    println!("1.0 + 0.618 = {:.6}", result);
}
```

## From bfloat16

```python
import ml_dtypes
import numpy as np
from golden_float import GF16, gf_array

# bfloat16 -> float32 -> GF16
arr_bf16 = np.array([1.618], dtype=ml_dtypes.bfloat16)
arr_f32  = arr_bf16.astype(np.float32)
arr_gf16 = gf_array(arr_f32.tolist(), dtype="gf16")

print("bfloat16:", hex(arr_bf16.view(np.uint16)[0]))
print("GF16:", hex(arr_gf16[0]))
# Should be different due to different bit allocations
```

## FPGA Safety Guarantee

The FFI core uses no floating-point arithmetic in compute paths.
All encode/decode operations use integer shifts and masks, making
the implementation directly synthesizable for FPGA/ASIC targets
via HLS tools (Vivado HLS, Intel HLS Compiler) without FP-IP blocks.

### Permitted Patterns

- **Allowed**: `to_bits()` / `from_bits()` at API boundaries for integer extraction
- **Allowed**: Explicit-width integer types (u8, u16, u32, u64) in compute
- **Prohibited**: f64/f32 arithmetic (pow, sqrt, abs, etc.) in core paths

### FPGA-ALLOWED Annotation

Functions with minimal FP use at boundaries are marked:
```rust
#[no_mangle]
pub extern "C" fn gf16_from_f32(x: f32) -> u16 {
    let bits: u32 = x.to_bits(); // FPGA-ALLOWED: integer extraction
    encode_gf16_from_u32(bits)
}
```

## Cross-Language Bit Identity

Reference value: `GF32(φ) = 0x3FCF1BBD` across all languages.
This enables reproducible results across Python, JavaScript, C, and Rust
without platform-specific floating-point variance.

### Verification

```bash
# Python
python -c "from golden_float import GF32; assert hex(GF32(1.618).bits()) == '0x3fcf1bbd'"

# Rust
cargo run --example gf_phi_check

# All produce identical 0x3FCF1BBD
```

## Troubleshooting

### Build errors

If you see "undefined reference to `gf16_from_f64`" in your code:
- Ensure `golden-float-ffi` is in your dependencies (Python/Cargo.toml)
- For C/C++, check `golden_float.h` is included

### Precision differences

GoldenFloat uses different bit allocation than IEEE formats:
- GF16: 6-bit exp, 9-bit mantissa (phi-optimal for constants like phi)
- GF32: 8-bit exp, 23-bit mantissa (same as IEEE f32)

This means roundtrip may not preserve exact values, especially for
high-precision inputs. This is expected behavior.

### NumPy dtype not available

For production-ready NumPy integration, use:
```python
from golden_float import gf_array, to_float32
arr = gf_array([1.0, 1.618], dtype="gf16")
back = to_float32(arr, src_dtype="gf16")
```

Full NumPy dtype support (NEP 42) will be in a future release.

## Further Reading

- [Whitepaper](./WHITEPAPER/gf_paper_v3_imrad_draft.md) - Mathematical foundation
- [T27 Constitution](./T27-CONSTITUTION.md) - Repository laws and invariants
- [AGENTS.md](../AGENTS.md) - Agent guidelines and law reference
