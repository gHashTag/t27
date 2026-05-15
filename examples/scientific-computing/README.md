# Scientific Computing Example

This example demonstrates numerical methods and φ-optimized algorithms using t27.

## Background

Scientific computing requires high precision for mathematical constants and numerical methods. t27's GF format provides φ-optimal bit allocation that balances dynamic range (exponent) with numerical precision (mantissa).

## The Specification

`numerical-methods.t27` includes:

1. **Golden ratio constants** — φ, φ², φ⁻¹, φ⁻²
2. **Numerical integration** — Simpson's rule, Monte Carlo
3. **Root finding** — Newton-Raphson method
4. **Linear algebra** — Matrix operations, eigenvalues
5. **Special functions** — Sine, cosine approximations
6. **Trinity identity verification** — φ² + φ⁻² = 3

## Running

```bash
# Parse
tri parse numerical-methods.t27

# Generate C code for scientific libraries
tri gen-c numerical-methods.t27 > gen/numerical.c

# Generate WASM for browser-based calculators
tri gen-wasm numerical-methods.t27 > gen/numerical.wat

# Run tests
tri test numerical-methods.t27
```

## Mathematical Results

- **Trinity Identity**: φ² + φ⁻² = 3 verified to 1e-15 precision
- **Integration Error**: Simpson's rule error < 1e-6 for smooth functions
- **Root Finding**: Newton-Raphson converges in < 10 iterations for φ

## Applications

- Physics simulations (particle systems)
- Financial modeling (option pricing)
- Engineering analysis (FEA, CFD precomputation)

---

**φ² + 1/φ² = 3 | TRINITY**