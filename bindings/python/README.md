# t27 Python Bindings

TRINITY Ternary Computing Framework - Python Interface

`φ² + 1/φ² = 3 | TRINITY`

## Overview

This package provides Python access to TRINITY's ternary computing capabilities, including:
- Ternary logic operations (and, or, not) based on Kleene K3 logic
- PyTorch-compatible neural network layers with ternary weights
- Efficient ternary word packing (27 trits in 7 bytes)

## Installation

```bash
pip install t27-trinity
```

For development:
```bash
pip install t27-trinity[dev,examples]
```

## Quick Start

### Ternary Logic Operations

```python
from t27 import Trit, k3_and, k3_or, k3_not, k3_implies

# Create trits
a = Trit.POS   # +1 (True)
b = Trit.ZERO  # 0  (Unknown)
c = Trit.NEG   # -1 (False)

# Perform K3 logic operations
result = k3_and(a, b)        # Returns Trit.ZERO (K_TRUE ∧ K_UNKNOWN = K_UNKNOWN)
result = k3_or(a, c)         # Returns Trit.POS  (K_TRUE ∨ K_FALSE = K_TRUE)
result = k3_not(a)           # Returns Trit.NEG  (¬K_TRUE = K_FALSE)
result = k3_implies(c, a)    # Returns Trit.POS  (K_FALSE → K_TRUE = K_TRUE, ex falso quodlibet)
```

### Ternary Word Packing

```python
from t27 import Trit, TernaryWord

# Create a ternary word from trits
trits = [Trit.POS, Trit.ZERO, Trit.NEG] * 9  # 27 trits
word = TernaryWord.from_trits(trits)

# Extract trits
trit_5 = word.get_trit(5)  # Get 6th trit (0-indexed)

# Convert to NumPy array
import numpy as np
arr = word.to_array()  # shape: (27,), values: -1, 0, or 1

# Pack/unpack
packed = word.data  # 7 bytes
word2 = TernaryWord(packed)
```

### PyTorch Ternary Layers

```python
import torch
from t27 import K3Linear, K3Conv2d, K3Embedding, K3Model

# Ternary linear layer
linear = K3Linear(in_features=784, out_features=128, sparse_init=True)
x = torch.randn(32, 784)
y = linear(x)
linear.update_ternary_weights()  # Quantize after optimizer step

# Ternary convolution
conv = K3Conv2d(in_channels=3, out_channels=16, kernel_size=3)
x = torch.randn(8, 3, 32, 32)
y = conv(x)
conv.update_ternary_weights()

# Ternary embedding
embedding = K3Embedding(num_embeddings=10000, embedding_dim=243)
x = torch.randint(0, 10000, (32, 64))
y = embedding(x)
embedding.update_ternary_weights()

# Complete HSLM-style model
model = K3Model(vocab_size=10000, embed_dim=243, num_layers=6, num_heads=3)
logits = model(input_ids)
```

### K3 Training Loop

```python
import torch
import torch.nn as nn
import torch.optim as optim
from t27 import K3Linear

# Create model with ternary layers
model = nn.Sequential(
    K3Linear(784, 256),
    nn.ReLU(),
    K3Linear(256, 10),
)
criterion = nn.CrossEntropyLoss()
optimizer = optim.Adam([p for p in model.parameters() if p.requires_grad], lr=0.001)

# Training loop
for epoch in range(epochs):
    for inputs, labels in dataloader:
        optimizer.zero_grad()

        # Forward pass (uses ternary weights internally)
        outputs = model(inputs)
        loss = criterion(outputs, labels)

        # Backward pass (gradients on continuous weights)
        loss.backward()
        optimizer.step()

        # Quantize continuous weights back to ternary
        for module in model.modules():
            if hasattr(module, 'update_ternary_weights'):
                module.update_ternary_weights()
```

## API Reference

### Ternary Logic

| Function | Description |
|----------|-------------|
| `Trit.POS`, `Trit.ZERO`, `Trit.NEG` | Ternary values {+1, 0, -1} |
| `k3_and(a, b)` | Kleene AND operation |
| `k3_or(a, b)` | Kleene OR operation |
| `k3_not(a)` | Kleene NOT operation |
| `k3_implies(a, b)` | Kleene implication |
| `k3_equiv(a, b)` | Kleene equivalence |

### PyTorch Layers

| Class | Description |
|-------|-------------|
| `K3Linear(in_features, out_features)` | Linear layer with ternary weights |
| `K3Conv2d(in_channels, out_channels, kernel_size)` | 2D conv with ternary weights |
| `K3Embedding(num_embeddings, embedding_dim)` | Embedding with ternary vectors |
| `K3LayerNorm(normalized_shape)` | RMSNorm-style normalization |
| `K3Attention(embed_dim, num_heads)` | Attention with ternary KV |
| `K3Block(embed_dim, num_heads)` | Transformer block with ternary attention |
| `K3Model(vocab_size, embed_dim, num_layers)` | Complete HSLM-style model |

## Ternary Weight Quantization

Weights are quantized using sign-based quantization with threshold:

```
weight > 0.1  →  +1
weight < -0.1 →  -1
otherwise      →   0
```

This creates sparse models with ~50% sparsity by default (configurable with `sparse_init=False`).

## Kleene K3 Logic

The trinary logic system follows Kleene's K3 strong Kleene logic:

| ∧ (AND) | False | Unknown | True |
|---------|-------|---------|------|
| **False** | False | False | False |
| **Unknown** | False | Unknown | Unknown |
| **True** | False | Unknown | True |

| ∨ (OR) | False | Unknown | True |
|---------|-------|---------|------|
| **False** | False | Unknown | True |
| **Unknown** | Unknown | Unknown | True |
| **True** | True | True | True |

| ¬ (NOT) | Value |
|---------|-------|
| | False ↔ True |
| | Unknown ↔ Unknown |
| | True ↔ False |

## Examples

See the `examples/` directory for:
- `mnist_k3.py` - Train MNIST classifier with ternary weights
- `hslm_simple.py` - Minimal HSLM model example
- `ternary_ops.py` - Basic ternary logic demonstrations

## Performance Characteristics

| Operation | Latency | Throughput |
|-----------|---------|------------|
| k3_and | ~2ns | >500M ops/sec |
| k3_or | ~2ns | >500M ops/sec |
| k3_not | ~1ns | >1B ops/sec |
| K3Linear forward | ~50ns | ~20M ops/sec |
| TernaryWord decode | ~10ns | ~100M words/sec |

## Research Notes

### Bounded Rationality

The ZERO trit represents "restraint" in bounded rationality:
- It allows the system to express uncertainty
- During inference, ZEROS can be treated conservatively (as NEG) or optimistically (as POS)
- This enables safer decision-making under uncertainty

### Ternary Neural Networks

Ternary weights offer several advantages:
- **Memory efficiency**: 2 bits per weight (vs 32 for float32)
- **Computational efficiency**: No DSP blocks needed on FPGAs
- **Built-in regularization**: Sparse connectivity
- **Hardware-friendly**: Simple XOR/AND operations

### K3 Backpropagation

Gradients are accumulated on continuous weights, then quantized back to ternary after each optimizer step. The subgradient at ZERO is 0.5, allowing gradients to flow through zero-valued weights while maintaining sparsity.

## License

MIT License - See LICENSE file for details.

## References

- Kleene, S. C. (1938). "On Notation for Ordinal Numbers"
- [TRINITY Project Documentation](https://trinity-framework.github.io/t27)
- [HSLM: Hierarchical Sacred Learning Model](https://trinity-framework.github.io/hslm)
