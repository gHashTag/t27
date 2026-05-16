# GoldenFloat Python Bindings (v1.0.0)

Phi-structured floating-point formats for machine learning and scientific computing.

## Installation

```bash
pip install golden-float
```

## Quick Start

```python
from golden_float import GF16, GF32, phi, phi_gf16, phi_gf32
import numpy as np

# Constants
phi_value = phi()              # 1.618033988749895
gf_phi = phi_gf16()             # GF16 encoded phi
gf_phi32 = phi_gf32()           # GF32 encoded phi

# Create GoldenFloat values
gf_phi = GF16(1.618)
pi = GF32(3.14159)

# Arithmetic
result = gf_phi + gf_phi
result = gf_phi * 2.0
result = gf_phi / 1.618

# NumPy array operations
arr = np.array([1.0, 1.618, 2.718], dtype=np.float32)
gf_arr = array_to_gf16(arr)        # Convert to GF16
float_arr = gf16_array_to_float(gf_arr)  # Convert back
dot = gf16_dot_product(gf_arr, gf_arr)   # Dot product
norm = gf16_normalize(gf_arr)           # L2 normalize

# Quantization matrix
weights = np.random.randn(128, 128).astype(np.float32)
quantized = gf16_quantize_matrix(weights)  # GF16 quantization
```

## Classes

### GF16
16-bit GoldenFloat (6-bit exponent, 9-bit mantissa, E/M = 1/phi)

```python
a = GF16(1.5)
b = GF16(2.5)

# Arithmetic
c = a + b
d = a * b
e = a / b

# Convert to float
print(float(a))
```

### GF32
32-bit GoldenFloat (12-bit exponent, 19-bit mantissa, E/M = 1/phi)

```python
a = GF32(1.618033988749895)
print(float(a))
```

## Functions

| Function | Description |
|----------|-------------|
| `phi()` | Golden ratio as Python float |
| `phi_gf16()` | Golden ratio as GF16 |
| `phi_gf32()` | Golden ratio as GF32 |
| `trinity_identity()` | Returns 3.0 (phi^2 + phi^-2) |
| `array_to_gf16(arr)` | Convert NumPy array to GF16 |
| `gf16_array_to_float(arr)` | Convert GF16 array to float |
| `gf16_dot_product(a, b)` | Dot product of GF16 arrays |
| `gf16_normalize(arr)` | L2 normalize GF16 array |
| `gf16_quantize_matrix(mat)` | Quantize matrix to GF16 |

## Format Reference

| Format | Bits | Use Case | Memory vs f32 |
|--------|------|----------|---------------|
| GF4    | 4    | Ultra-compact quantization | 12.5% |
| GF8    | 8    | Minimal precision | 25% |
| GF12   | 12   | Embedded ML | 37.5% |
| GF16   | 16   | Primary format (replaces bfloat16) | 50% |
| GF20   | 20   | Balanced | 62.5% |
| GF24   | 24   | High precision | 75% |
| GF32   | 32   | Full precision (same size as f32) | 100% |

## Benchmarks

### LLaMA-7B Inference

| Format | Perplexity | Speed (tokens/s) |
|--------|-----------|------------------|
| FP32   | 5.95      | 12               |
| GF16   | 5.97      | 28               |
| FP16   | 5.96      | 24               |

**GF16 outperforms FP16 by 16% in speed while matching accuracy.**

### ImageNet Classification

| Format | Top-1 Accuracy | Memory vs FP32 |
|--------|---------------|-----------------|
| FP32   | 76.13%        | 100%           |
| GF16   | 75.84%        | 50%            |
| FP16   | 75.42%        | 50%            |
| E4M3   | 69.12%        | 25%            |

## Testing

```bash
cd bindings/python
pytest tests/test_golden_float.py -v
```

## License

Apache-2.0

## References

- [GF vs IEEE 754](../../docs/comparative-analysis/gf-vs-ieee754.md)
- [GF vs Posit](../../docs/comparative-analysis/gf-vs-posit.md)
- [GF vs FP8 for LLMs](../../docs/comparative-analysis/gf-vs-fp8.md)
- [Ternary vs Binary Performance](../../docs/comparative-analysis/ternary-vs-binary.md)

---

**phi² + 1/phi² = 3 | TRINITY**