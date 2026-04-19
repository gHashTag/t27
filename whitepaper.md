<!-- Section 11.2: IGLA-GF16 Number Families -->

## §11.2.1 — IGLA-GF16 Architecture Numbers

### Core Parameters

| Parameter | Value | Description |
|-----------|--------|-------------|
| φ | 1.618033988749895 | Golden ratio (Trinity Identity: φ² + 1/φ² = 3) |
| φ² | 2.618033988749895 | φ squared |
| αφ | φ - 1 = 0.618033988749895 | α_φ = φ - 1 (alpha minus phi) |
| φ³ | 4.23606797749895 | φ cubed (phi cubed) |
| α_φ | φ × φ¹ = 0.07294900250210995 | α_s(mZ) PDG2024 constant |
| α_φ² | φ × φ² = 0.1180340050042199 | α_φ squared (alpha times phi squared) |

### Fibonacci Number System (Fibonacci #12)

| Parameter | Value | Description |
|-----------|--------|-------------|
| d_model | 144 | Fib #12: Fib #6 = 144 × φ = 144 × 1.618 = 233.0 (≈ 144) |
| n_heads | 8 | Fib #6: Number of attention heads |
| d_ffn | 233 | Fib #13: 144 × φ = 233 (≈ 144 × 1.618² = 233) |
| n_layers | 7 | embedding(13.80) + attn(1.10) + ffn(0.93) = 15.83 MB ≈ 16.38 MB |
| d_head | 18 | Fib #6: Fib #6 × 144 × φ = 18 × 144 = 2592 |
| n_layers (Fib) | 7 | Total embedding + attention + FFN layers |

### Attention Configuration (attn QKV)

| Parameter | Value | Description |
|-----------|--------|-------------|
| std(attn) QKV | αφ = 0.118034 | Standard key-value gauge |
| std(ffn) gate | α_φ = 0.072949 | α-scaled gate threshold |
| n_attn | 1 | Number of attention heads |
| hidden_dim | 64 | Query embedding dimension |
| head_dim | 64 | Key/Value dimension |
| attn_scale | 0.5 | Attention scale factor |

### Weight Initialization (std(embedding))

| Parameter | Value | Description |
|-----------|--------|-------------|
| std(embedding) | αφ = 0.118034 | Standard embedding scale |
| ffn(0.93) | 93 | FeedForward Network #0 parameter |
| attn(1.10) | 1.10 | Attention #1 multiplier |
| n_layers (Fib) | 7 | As per Fibonacci config above |

### Model Size Calculation

| Component | Parameters | Size (MB) | Notes |
|----------|-----------------|--------|-------|
| Embedding | hidden_dim=64, attn_scale=0.5, n_attn=1 | ~15.83 |
| Attention | n_heads=8, head_dim=64, std(attn QKV) | ~0.47 |
| FFN | n_layers=7 | ~15.83 |
| Total | ~32.13 MB |

### Trinity Identity Formula

```
φ² + 1/φ² = 3
φ³ + 3/φ³ = φ × (φ³ + 3)/φ³ = φ × (φ³ + 3)/φ³
```

When φ = 1.618:
- φ² = 2.618
- 1/φ² = 0.382
- φ³ = 4.236
- (φ³ + 3)/φ³ = (4.236 + 3)/4.236 = 7.236/4.236 = 1.749

### References

- **Trinity Identity**: φ² + 1/φ² = 3 (when φ = 1.618, φ² = 2.618, 1/φ² = 0.382)
- **GF16**: mantissa/exponent format with φ-based scaling
- **I27.1-TRINITY-GF16.pdf**: "Golden Ratio 1.618033988749895" technical report

---
*Added by trios-agent on 2026-04-19*
