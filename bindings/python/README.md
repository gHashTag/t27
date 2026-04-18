# GoldenFloat Python Bindings

Phi-structured floating-point formats for machine learning and scientific computing.

## Installation

```bash
pip install golden-float
```

## Quick Start

```python
import numpy as np
from golden_float import gf16, gf32

# Create GoldenFloat values
phi = gf16(1.618)
pi = gf32(3.14159)

# Convert to float
print(phi.to_float())  # 1.618

# Arithmetic
result = phi + phi

# NumPy array support (with dtype registration)
arr = np.array([1.0, 1.618, 2.718], dtype=gf16)
```

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

## License

MIT
