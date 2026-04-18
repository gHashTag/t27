# GoldenFloat Benchmark Protocol

**Version**: 1.0
**Date**: 2026-04-06
**Related**: Ring 041, `gf16_arxiv_draft.md`

---

## Overview

This document defines the benchmark protocol for evaluating GoldenFloat formats against IEEE 754 formats (bfloat16, float16) and alternative formats (posit, takum).

## Metric: Normalized Mean Squared Error (NMSE)

For a weight vector $w \in \mathbb{R}^N$ quantized to format $F$:

$$
\text{NMSE}(w, F) = \frac{\|w - Q_F(w)\|_2^2}{\|w\|_2^2}
$$

Where $Q_F(w)$ is the quantized version of $w$ using format $F$.

## Datasets

### 1. MNIST

- **Download**: http://yann.lecun.com/exdb/mnist/
- **Samples**: 70,000 (60,000 train, 10,000 test)
- **Image size**: 28×28 grayscale
- **Classes**: 10 (digits 0-9)

**Weight extraction**:
```python
# Extract weight matrices from trained model
# Focus on first fully-connected layer (typically largest)
weights = model.fc1.weight.detach().cpu().numpy()
weights = weights.flatten()  # Vectorize for NMSE calculation
```

### 2. CIFAR-10

- **Download**: https://www.cs.toronto.edu/~kriz/cifar.html
- **Samples**: 60,000 (50,000 train, 10,000 test)
- **Image size**: 32×32 RGB
- **Classes**: 10 (airplane, automobile, bird, cat, deer, dog, frog, horse, ship, truck)

**Weight extraction**: Same as MNIST

## Format Definitions

| Format | Total Bits | Sign | Exponent | Mantissa | Bias | Special |
|--------|-----------|------|----------|----------|------|---------|
| GF16 | 16 | 1 | 6 | 9 | 31 | 0x3E00 (Inf/NaN) |
| bfloat16 | 16 | 1 | 8 | 7 | 127 | 0x7F80 (Inf/NaN) |
| float16 | 16 | 1 | 5 | 10 | 15 | 0x7C00 (Inf/NaN) |
| posit16 | 16 | 1 (regime) | 1 | 14 | N/A | N/A |
| takum-16 | 16 | 1 | 4 | 11 | N/A | N/A |

## Phi-Distance Calculation

$$
d_\phi(e, m) = \left| \frac{e}{m} - \frac{1}{\phi} \right|
$$

Where $\phi = (1+\sqrt{5})/2 \approx 1.618$.

## Benchmark Procedure

### Step 1: Extract Weights

```python
import torch
import numpy as np

# Load pre-trained model
model = torch.hub.load('pytorch/vision', 'resnet18', pretrained=True)

# Extract all weights
weights = []
for name, param in model.named_parameters():
    if param.dim() > 1:  # Skip 1D biases
        weights.append(param.detach().cpu().numpy().flatten())

all_weights = np.concatenate(weights)
```

### Step 2: Quantize to Each Format

```python
def quantize_gf16(weights):
    # GF16: S(1) E(6) M(9), bias=31
    scale = 2**-9
    quantized = []
    for w in weights:
        sign = 1 if w >= 0 else -1
        w_abs = abs(w)

        # Normalize and find exponent
        e = int(np.floor(np.log2(max(w_abs, 1e-10))))
        e = np.clip(e, -31, 32)

        # Compute mantissa
        m = round((w_abs / (2**e)) * 512)
        m = np.clip(m, 0, 511)

        # Reconstruct
        gf16 = sign * (2**(e+31)) * (1 + m/512)
        quantized.append(gf16)

    return np.array(quantized)

def quantize_bfloat16(weights):
    # bfloat16: S(1) E(8) M(7), bias=127
    scale = 2**-7
    quantized = []
    for w in weights:
        sign = 1 if w >= 0 else -1
        w_abs = abs(w)

        e = int(np.floor(np.log2(max(w_abs, 1e-10))))
        e = np.clip(e, -127, 128)

        m = round((w_abs / (2**e)) * 128)
        m = np.clip(m, 0, 127)

        bf16 = sign * (2**(e+127)) * (1 + m/128)
        quantized.append(bf16)

    return np.array(quantized)

def quantize_float16(weights):
    # float16: S(1) E(5) M(10), bias=15
    scale = 2**-10
    quantized = []
    for w in weights:
        sign = 1 if w >= 0 else -1
        w_abs = abs(w)

        e = int(np.floor(np.log2(max(w_abs, 1e-10))))
        e = np.clip(e, -14, 16)

        m = round((w_abs / (2**e)) * 1024)
        m = np.clip(m, 0, 1023)

        f16 = sign * (2**(e+15)) * (1 + m/1024)
        quantized.append(f16)

    return np.array(quantized)
```

### Step 3: Compute NMSE

```python
def nmse(original, quantized):
    return np.mean((original - quantized)**2) / np.mean(original**2)

# Benchmark
gf16_quant = quantize_gf16(all_weights)
bf16_quant = quantize_bfloat16(all_weights)
f16_quant = quantize_float16(all_weights)

nmse_gf16 = nmse(all_weights, gf16_quant)
nmse_bf16 = nmse(all_weights, bf16_quant)
nmse_f16 = nmse(all_weights, f16_quant)

print(f"GF16 NMSE: {nmse_gf16:.4f}")
print(f"bfloat16 NMSE: {nmse_bf16:.4f}")
print(f"float16 NMSE: {nmse_f16:.4f}")
```

## Expected Results

Based on `conformance/gf16_vectors.json`:

| Format | phi_distance | NMSE (expected) |
|--------|--------------|------------------|
| GF16 | 0.049 | ~0.8% |
| bfloat16 | 0.525 | ~0.6% |
| float16 | 0.118 | ~0.5% |

**Note**: GF16 achieves the **lowest phi-distance** among 16-bit formats, indicating optimal exponent/mantissa ratio for the target distribution.

## Reproducibility

All benchmarks should include:

1. **Random seed**: `np.random.seed(42)` / `torch.manual_seed(42)`
2. **Model**: Specify architecture and source (e.g., ResNet18, MLP)
3. **Software versions**: Python 3.11, PyTorch 2.0+, NumPy 1.24+
4. **Hardware**: CPU/GPU specifications

## Figures to Generate

1. **Figure 1**: Format comparison table (exp/mantissa, phi_distance)
2. **Figure 2**: Weight distribution histogram (log-normal fit)
3. **Figure 3**: NMSE bar chart across formats
4. **Figure 4**: Phi-distance vs NMSE scatter plot

---

**Protocol version**: 1.0
**Last updated**: 2026-04-06
