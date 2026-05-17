# t27 Python Bindings

Python interface to the t27 toolchain for format conversions, quantization, and simulation.

## Installation

```bash
pip install numpy
# Add t27/python to PYTHONPATH or install in development mode
cd t27/python
pip install -e .
```

## Quick Start

```python
from t27 import GF16, FP8_E4M3, Int8, convert

# Create values
one_gf16 = GF16.from_float(1.0)
print(one_gf16)  # GF16(0x7C00 ~ 1.000)

# Convert between formats
fp8_val = convert(one_gf16, FP8_E4M3)
print(fp8_val)  # 1.0

# Quantize
from t27.quantizers import Int8Quantizer
quantizer = Int8Quantizer(symmetric=True)
quantizer.calibrate([1.5, 2.5, 3.5])
quantized = quantizer.quantize(2.7)
print(quantized)  # 2
```

## Formats

- **GF16, GF32, GF64, GF128, GF256**: GoldenFloat (φ-optimized)
- **FP8_E4M3, FP8_E5M2**: OCP FP8 variants
- **Int4, Int8**: Signed integers
- **NF4**: NormalFloat4 (QLoRA)
- **Posit16**: Posit type 16

## Quantization

```python
from t27.quantizers import (
    Int8Quantizer, Int4Quantizer,
    NF4Quantizer, FP8Quantizer,
    quantize_tensor, quantization_error
)

# Tensor quantization
import numpy as np
tensor = np.random.randn(100, 100).astype(np.float32)

quantized, quantizer = quantize_tensor(tensor, Int8Quantizer())
dequantized = dequantize_tensor(quantized, quantizer)

# Calculate error
error = quantization_error(tensor, dequantized, metric='sqnr')
print(f"SQNR: {error:.2f} dB")
```

## φ-Optimization

```python
from t27.phi import (
    phi_distance, optimal_phi_distance,
    get_phi_optimal_format, print_phi_table
)

# Analyze a format
dist = phi_distance(6, 9)  # GF16
print(f"φ-distance: {dist:.6f}")

# Find optimal allocation
exp, mant, dist = optimal_phi_distance(16)
print(f"Optimal: {exp} exp, {mant} mant, φ-dist={dist:.6f}")

# Print full table
print_phi_table()
```

## Cross-Format Conversion

```python
from t27.conversions import (
    convert, convert_batch, conversion_matrix,
    gf16_to_fp8_e4m3, f32_to_nf4
)

# Batch conversion
values = [1.0, 2.0, 3.14, 5.5]
gf16_vals = convert_batch(values, GF16)

# Optimized paths
fp8_val = gf16_to_fp8_e4m3(3.14)
nf4_bits = f32_to_nf4(5.5, scale=1.0)

# Conversion accuracy matrix
formats = [GF16, FP8_E4M3, Int8]
test_vals = [0.5, 1.5, 10.0, 100.0]
errors = conversion_matrix(formats, formats, test_vals)
print(errors)
```

## License

Apache-2.0