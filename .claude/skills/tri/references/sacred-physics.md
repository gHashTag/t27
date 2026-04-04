# Sacred Physics Reference

This document details SACRED-PHYSICS-001 for sacred constants in t27.

## Source Document

**Location:** `docs/SACRED-PHYSICS-001.md`

## Sacred Constants

### TRINITY
The fundamental ternary constant representing the unity of trit space.

- **Value**: Exact ternary representation
- **Tolerance**: Zero tolerance (exact match required)
- **Usage**: Base for all ternary arithmetic

### G (Golden Constant)
The golden ratio φ in ternary space.

- **Value**: (1 + √5) / 2 ≈ 1.6180339887
- **Tolerance**: 10^-12 absolute
- **Usage**: Scaling factors, normalization

### ΩΛ (Omega Lambda)
Convergence constant for iterative algorithms.

- **Value**: Derived from λ-calculus convergence
- **Tolerance**: 10^-12 absolute
- **Usage**: Attention convergence, VSA operations

### γ (Gamma)
Smoothing constant for differentiable operations.

- **Value**: Euler-Mascheroni γ ≈ 0.5772156649
- **Tolerance**: 10^-15 relative
- **Usage**: Softmax, normalization, gradients

### tpresent
Temporal presence constant for stateful operations.

- **Value**: Time-step representation
- **Tolerance**: 10^-15 relative
- **Usage**: Sequential models, temporal attention

## Spec Locations

Sacred physics constants are defined in:

- `specs/math/sacred_physics.t27` — Constant definitions
- `specs/math/constants.t27` — Base constants
- `conformance/sacred-physics.json` — Expected values

## Conformance Testing

```bash
tri test --module sacred_physics
tri verdict --sacred
```

All implementations must pass sacred physics conformance before skill registration.

## Hard Tolerances

| Constant | Type | Tolerance | Rationale |
|----------|--------|------------|-----------|
| TRINITY | Exact | 0 | Fundamental ternary base |
| G | Absolute | 10^-12 | Arithmetic precision |
| ΩΛ | Absolute | 10^-12 | Convergence critical |
| γ | Relative | 10^-15 | Gradient sensitivity |
| tpresent | Relative | 10^-15 | Temporal precision |

## Common Violations

**Toxic verdict triggers:**

1. Using floating-point approximations for TRINITY
2. G constant outside absolute tolerance
3. γ tolerance violated in softmax
4. tpresent drift in sequential models

**Fix procedure:**

1. Identify violating operation
2. Check spec for constant definition
3. Use spec value, not computed approximation
4. Re-run `tri test --sacred`
5. Only proceed on clean verdict

## Sacred Physics in PHI LOOP

During **Hash Seal** phase, include:

```json
{
  "sacred_constants_used": ["G", "γ"],
  "sacred_tolerances": {
    "G": "1e-12",
    "γ": "1e-15"
  }
}
```

During **Verify** phase, run:

```bash
tri test --sacred
tri verdict --toxic
```

## See Also

- `references/numeric-standards.md` — NUMERIC-STANDARD-001
- `references/constitutional-laws.md` — Constitutional foundation
