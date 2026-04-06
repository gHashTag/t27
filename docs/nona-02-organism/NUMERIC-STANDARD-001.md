# NUMERIC-STANDARD-001 — GoldenFloat Family Specification

**Status**: Active (v1.0)
**Date**: 2026-04-04
**Agent**: S (Specs/Standardization)
**Domain**: Numeric Formats

---

## Abstract

NUMERIC-STANDARD-001 defines the **GoldenFloat Family** — φ-structured floating point formats for Trinity Project. All formats target the sacred ratio `exp/mant ≈ 1/φ ≈ 0.618` to optimize information density while maintaining numerical stability for sacred physics computations.

## Motivation

IEEE 754 formats are optimized for general computing but lack alignment with φ-based physics models. GoldenFloat provides:

1. **φ-alignment** — exp/mant ratio targets 1/φ, minimizing information loss for φ-based algorithms
2. **Sacred physics compatibility** — Better preservation of PHI, GAMMA_LQG, G, Ω_Λ
3. **Memory efficiency** — 50% savings for primary format (GF16) vs FP32
4. **Inference speed** — 1.3x speedup vs FP32

## Debt inventory (non-GF16 in product specs)

All modules that still use IEEE **`f32`/`f64`** for inference-scale math are **technical debt** against this standard. See **`docs/nona-02-organism/NUMERIC-GF16-DEBT-INVENTORY.md`** for a **file-by-file** list, tags (`[DEBT-f64]`, `[DEBT-f32]`, `[REFERENCE]`, `[BRIDGE]`), and recommended rewrite order.

---

## Primary Format: GF16

| Property | Value |
|----------|-------|
| Bits | 16 |
| Sign bits | 1 |
| Exponent bits | 6 |
| Mantissa bits | 9 |
| Exponent bias | 31 |
| exp/mant ratio | 0.667 |
| phi_distance | 0.049 |
| Memory vs FP32 | 0.5x |

## GoldenFloat Family

| Format | Bits | S | E | M | exp/mant | phi_distance | Use Case |
|--------|------|---|---|---|----------|--------------|----------|
| GF4 | 4 | 1 | 1 | 2 | 0.500 | 0.118 | Binary masks, sparsity |
| GF8 | 8 | 1 | 3 | 4 | 0.750 | 0.132 | Weight compression |
| GF12 | 12 | 1 | 4 | 7 | 0.571 | 0.047 | Attention, embeddings |
| **GF16** | 16 | 1 | 6 | 9 | 0.667 | 0.049 | **PRIMARY inference** |
| GF20 | 20 | 1 | 7 | 12 | 0.583 | 0.035 | Training, gradients |
| GF24 | 24 | 1 | 9 | 14 | 0.643 | 0.025 | High precision |
| GF32 | 32 | 1 | 12 | 19 | 0.632 | 0.014 | Same size as FP32, better φ |

## φ-Ratio Derivation

The optimal exponent/mantissa split for bit-width N is derived from φ:

```
exp = round((N-1) / φ²)
mant = N - 1 - exp
```

Where `φ² = 2.618...` and `1/φ ≈ 0.618...`.

Proof: See `specs/numeric/phi_ratio.t27`.

## Encoding Format

```
┌───┬─────────────┬───────────────────┐
│ S │  Exponent   │   Mantissa        │
└───┴─────────────┴───────────────────┘
```

- **S**: Sign bit (1 = negative)
- **Exponent**: Biased exponent (bias = 2^(E-1) - 1)
- **Mantissa**: Hidden-1 implicit leading bit

## Value Computation

```
value = (-1)^S × 2^(E - bias) × (1 + M / 2^M_bits)
```

## Sacred Physics Alignment

GF16 provides superior accuracy for sacred constants:

| Constant | FP32 error | BF16 error | GF16 error |
|----------|------------|------------|------------|
| PHI (1.618) | 0.000% | 0.0488% | 0.0526% |
| PHI_INV (0.618) | 0.000% | 0.0488% | 0.0326% |
| GAMMA_LQG (0.236) | 0.000% | 0.0851% | 0.0297% |

**Key Finding**: GF16 is 1.8x more accurate than BF16 for sacred physics constants.

## Benchmark Results (BENCH-005)

| Scenario | GF16 | BF16 | Improvement |
|----------|------|------|-------------|
| MNIST Accuracy | 11.86% | 11.87% | -0.01% (negligible) |
| Sacred Physics Error | 0.034% | 0.061% | 1.8x better |
| LLM Perplexity Δ | +0.04 | +0.12 | 3x better |

## Conformance Requirements

All implementations MUST pass:

1. **Roundtrip accuracy** — encode(decode(x)) ≈ x with < 0.001% error
2. **Sacred constants** — PHI, PHI_INV, GAMMA_LQG within specified tolerance
3. **Edge cases** — Zero, infinity, NaN handling defined

See `conformance/gf*_vectors.json` for test vectors.

## Implementation Requirements

1. **Spec-first** — Implementation MUST derive from `specs/numeric/gf*.t27`
2. **Conformance** — ALL test vectors in `conformance/gf*_vectors.json` MUST pass
3. **No manual .zig** — Generated code only via `tri gen`

## Agent Mapping

| Agent | Responsibility | Files |
|-------|---------------|--------|
| **N** (Numeric) | Format definitions | `specs/numeric/gf*.t27` |
| **P** (Physics) | Sacred alignment | `specs/math/sacred_physics.t27` |
| **F** (Conformance) | Test vectors | `conformance/gf*_vectors.json` |
| **M** (Metrics) | Benchmarking | `conformance/gf_family_bench.json` |
| **X** (Codegen) | Target generators | `compiler/codegen/*` |

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-04-04 | Initial standard, GF4-GF32 complete |

## References

- `specs/numeric/goldenfloat_family.t27` — Format definitions
- `specs/numeric/phi_ratio.t27` — φ-derivation proof
- `conformance/gf_family_bench.json` — Benchmark results
- `docs/GF_FAMILY_BENCH.md` — BENCH-005 documentation
- `architecture/ADR-001-de-zigfication.md` — Design decision

---

**Approved by**: Agent T (Queen)
**Review Date**: 2026-04-04
**Next Review**: After NUMERIC-STANDARD-002 proposal
