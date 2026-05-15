# ML Quantization Example

This example demonstrates layer-wise neural network quantization using GoldenFloat GF16 format.

## Background

Neural network quantization reduces model size and improves inference speed by using lower-precision floating-point formats. GoldenFloat GF16 provides φ-optimal bit allocation (6-bit exponent, 9-bit mantissa) achieving better accuracy than standard FP16 for the same memory footprint.

## The Specification

`model-quant.t27` defines:

1. **GF16 encoding/decoding** — φ-optimal 16-bit floating point
2. **Layer statistics** — Min/max range, mean, variance
3. **Scale calculation** — φ-guided scale factor
4. **Quantization** — Convert f32 weights to GF16
5. **Dequantization** — Convert GF16 back to f32
6. **Accuracy metrics** — MSE, MAE between original and quantized

## Running

```bash
# Parse
tri parse model-quant.t27

# Generate Rust code (for PyTorch integration)
tri gen-rust model-quant.t27 > gen/model_quant.rs

# Run tests
tri test model-quant.t27

# Generate C code (for TensorFlow Lite)
tri gen-c model-quant.t27 > gen/model_quant.c
```

## Expected Output

- Quantized weights use 50% less memory than f32
- MSE < 0.01 for ImageNet-like models
- MAE < 0.005 for typical weight distributions

## References

- [GF Format Specification](https://trinity-s3ai.org/docs/formats/gf.md)
- [Ternary Computing Paper](https://trinity-s3ai.org/papers/ternary-ml.pdf)

---

**φ² + 1/φ² = 3 | TRINITY**