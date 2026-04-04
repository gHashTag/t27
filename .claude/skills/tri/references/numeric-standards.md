# Numeric Standards Reference

This document details NUMERIC-STANDARD-001 for GoldenFloat and numeric formats in t27.

## Source Document

**Location:** `docs/NUMERIC-STANDARD-001.md`

## GoldenFloat Family

GoldenFloat is the canonical ternary floating-point format family:

- **GF4**: 4-bit format (1 exponent, 1 mantissa)
- **GF8**: 8-bit format (2 exponent, 2 mantissa)
- **GF12**: 12-bit format (3 exponent, 3 mantissa)
- **GF20**: 20-bit format (5 exponent, 5 mantissa)
- **GF24**: 24-bit format (6 exponent, 6 mantissa)
- **GF32**: 32-bit format (8 exponent, 8 mantissa)

## Format Structure

```
┌─────────┬──────────┬───────────────┐
│  Sign   │ Exponent │   Mantissa     │
│  (1 bit)│  (E bits) │  (M bits)     │
└─────────┴──────────┴───────────────┘
```

- **Sign**: Ternary trit (-1, 0, +1)
- **Exponent**: Biased representation
- **Mantissa**: Hidden-implicit 1 normalization

## Ternary Formats

### TF3 (Ternary Fixed-Point)
- 3 trits per value
- Fixed-point arithmetic
- Used for VSA accumulation

### IPS (Integer Packed Storage)
- Compact integer representation
- Packed trit encoding
- Memory-efficient storage

## Conformance Requirements

All numeric implementations must pass:

1. **Spec conformance**: Generated code matches spec behavior
2. **Test vector verification**: Matches reference outputs
3. **Sacred physics compliance**: Respects G, ΩΛ constants
4. **GoldenFloat family contracts**: Follows format specification

## Tolerances

Per SACRED-PHYSICS-001:

- **TRINITY constant**: Exact match required
- **Gamma (γ)**: 10^-15 relative tolerance
- **G constant**: 10^-12 absolute tolerance
- **ΩΛ**: 10^-12 absolute tolerance

## Test Vectors

Stored in `conformance/` directory:

- `gf4-conformance.json`
- `gf8-conformance.json`
- `gf12-conformance.json`
- `gf20-conformance.json`
- `gf24-conformance.json`
- `gf32-conformance.json`

Each contains:
```json
{
  "test_cases": [
    {
      "input": "trit_pattern",
      "expected": "ternary_result",
      "tolerance": "absolute_or_relative"
    }
  ]
}
```

## Verification Commands

```bash
tri gen               # Generate backends
tri test --format gf8   # Test specific format
tri verdict --toxic      # Check for regressions
tri bench --format gf12  # Benchmark performance
```

## See Also

- `references/sacred-physics.md` — SACRED-PHYSICS-001 constants
- `references/constitutional-laws.md` — Constitutional foundation
