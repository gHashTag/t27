# Comparative Analysis — t27 Format Research

This directory contains comparative analysis papers documenting t27's GoldenFloat formats against alternatives.

## Papers

| Paper | Description | Key Finding |
|-------|-------------|-------------|
| [GF vs IEEE 754](gf-vs-ieee754.md) | GF against standard floating-point | GF16 outperforms FP16 in ML quantization |
| [GF vs Posit](gf-vs-posit.md) | GF against Posit type III unum | GF16 is simpler, faster, matches Posit accuracy |
| [GF vs FP8](gf-vs-fp8.md) | GF against FP8 for LLM quantization | GF16 matches FP16 accuracy, E4M3 loses 31% |
| [Ternary vs Binary](ternary-vs-binary.md) | Ternary computing performance | Ternary offers 58.5% density gain, 30% speedup |

## Executive Summary

| Format | Best For | Why |
|--------|----------|-----|
| GF16 | ML Inference, Gradients | Best tail coverage, simple hardware |
| GF32 | ML Training, Scientific | φ-alignment, better error propagation |
| GF12 | Embedded DSP | Balanced 12-bit format |
| GF8 | Ultra-compact | Research format for extreme memory constraints |

## Recommendations by Application

| Application | Primary Format | Alternative |
|-------------|-----------------|------------|
| LLM Inference | GF16 | FP16 (if GF not available) |
| LLM Training | GF32 (weights), GF16 (activations) | FP32 |
| Scientific Computing | GF64 | FP64 |
| Embedded DSP | GF12 | Posit16 |
| Ultra-Low Power | GF8 (research) | E4M3 (accept loss) |

## Key Takeaways

1. **GF16 ≈ FP16 in accuracy** but with φ-aligned hardware benefits
2. **GF outperforms FP8** despite using 2× memory
3. **Ternary advantage is real** — 30-50% gains, waiting for hardware
4. **t27 bridges the gap** — Spec-first, binary-ready now, ternary-ready later

---

**φ² + 1/φ² = 3 | TRINITY**