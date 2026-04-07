# Language Test Harnesses

Cross-language precision verification for GoldenFloat competitive analysis.

## Purpose

These test harnesses measure decimal place accuracy and floating-point error across different programming languages, demonstrating that t27 ternary computation provides comparable or superior precision to IEEE 754 f64.

## Test Harnesses

| Language | Precision | Script | Output |
|----------|-----------|--------|--------|
| Python | float64 (IEEE 754) | `python_float64.py` | JSON |
| JavaScript | Number (IEEE 754) | `javascript_number.js` | JSON |
| Rust | f64 (IEEE 754) | `rust_f64.rs` | JSON |

## Test Categories

### 1. Golden Ratio Representation
- φ = (1 + √5) / 2
- Measures error against 100-digit GMP reference
- Counts matching decimal places

### 2. φ² = φ + 1 Identity
- Tests algebraic identity precision
- Should be exact in IEEE f64 (within rounding)

### 3. TRINITY Identity: φ² + φ⁻² = 3
- Tests sacred physics constant
- Measures error tolerance

### 4. 1/3 Decimal Places
- Counts significant decimal places
- IEEE f64: ~15-16 places
- t27 ternary: claims 16+ places

### 5. Accumulation Stability
- Σ 1/n for n=1..N
- Measures floating-point error accumulation
- N = 100,000 (JavaScript), 1,000,000 (Python)

## Running Tests

### Python
```bash
python3 python_float64.py > results/python_float64.json
```

### JavaScript
```bash
node javascript_number.js > results/javascript.json
```

### Rust
```bash
cargo run --release --bin rust_f64 > results/rust_f64.json
```

## Running All Tests

```bash
mkdir -p results
python3 python_float64.py > results/python_float64.json
node javascript_number.js > results/javascript.json
cargo run --release --bin rust_f64 > results/rust_f64.json
```

## Output Format

All harnesses produce JSON with the following structure:

```json
{
  "language": "Language Name",
  "precision": "Precision Description",
  "tests": [
    {
      "name": "test_name",
      "passed": true/false,
      "...": "test-specific fields"
    }
  ],
  "all_passed": true,
  "summary": {
    "phi_error": 0.0,
    "phi_decimal_places": 15,
    "one_third_decimal_places": 15
  }
}
```

## Integration with t27

Run the full competitive analysis:

```bash
./scripts/tri math compete --full
```

This will execute all test harnesses and generate a comprehensive markdown report.

## Constitutional Compliance

Per **L2 SSOT-MATH**, these harnesses are **auxiliary verification tools**, not the source of truth. All domain math lives in:
- `specs/math/pellis_precision_verify.t27`
- `specs/numeric/gf_competitive.t27`

The harnesses provide language-specific validation but do not define any constants or formulas.
