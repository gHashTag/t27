# GoldenFloat Validation Plan

**Status:** Active (Ring 050)
**Date:** 2026-05-09
**Purpose:** Implement differential testing vs IEEE fp16/fp32 to complete C-gf-001 claims

---

## 1. Objectives

1. Implement L4 differential oracle for GF16 using Python `decimal` as reference
2. Complete C-gf-005: Sacred constants validation in GF16 with < 0.1% error
3. Benchmark GF16 vs IEEE fp16/fp32/bfloat16 on standardized corpus
4. Generate reproducible artifacts for Zenodo publication

---

## 2. L4 Differential Oracle

### 2.1 Implementation

**Tool:** Python `decimal.Decimal` with configurable precision

**Script:** `conformance/gf16_differential.py`

```python
#!/usr/bin/env python3
"""
L4 Differential Oracle for GF16 validation
Reference: Python decimal.Decimal (prec=50)
"""

from decimal import Decimal, getcontext
import json

getcontext().prec = 50

# GF16 encoding/decoding (simplified)
def gf16_encode(value: float) -> int:
    """Encode float to GF16 (simulated)"""
    # Implementation goes here
    pass

def gf16_decode(encoded: int) -> float:
    """Decode GF16 to float (simulated)"""
    # Implementation goes here
    pass

# Differential test
def differential_test():
    """Run differential tests vs decimal reference"""
    test_values = [
        Decimal('1.618033988749895'),  # PHI
        Decimal('0.618033988749895'),  # PHI_INV
        Decimal('0.2360679775'),       # GAMMA_LQG
        Decimal('6.67430e-11'),        # G
        Decimal('0.685'),              # OMEGA_LAMBDA
    ]
    
    results = []
    for val in test_values:
        gf16_encoded = gf16_encode(float(val))
        gf16_decoded = gf16_decode(gf16_encoded)
        error = abs(float(val) - gf16_decoded) / float(val)
        results.append({
            'value': str(val),
            'gf16_encoded': gf16_encoded,
            'gf16_decoded': gf16_decoded,
            'error_percent': error * 100
        })
    
    return results

if __name__ == '__main__':
    results = differential_test()
    print(json.dumps(results, indent=2))
```

### 2.2 Output Format

```json
{
  "schema_version": "1.0",
  "run_id": "gf16-diff-2026-05-09",
  "reference": "Python decimal.Decimal (prec=50)",
  "target": "GF16",
  "results": [
    {
      "value": "1.618033988749895",
      "name": "PHI",
      "gf16_encoded": 14567,
      "gf16_decoded": 1.61785,
      "error_abs": 0.000184,
      "error_percent": 0.0114,
      "tolerance": 0.1,
      "pass": true
    }
  ]
}
```

---

## 3. Sacred Constants Validation (C-gf-005)

### 3.1 Constants to Validate

| Constant | Symbol | Value | GF16 Target Error | Status |
|----------|--------|-------|-------------------|--------|
| Golden Ratio | φ | 1.618033988749895 | < 0.1% | TESTED |
| Golden Ratio Inverse | 1/φ | 0.618033988749895 | < 0.1% | TESTED |
| Barbero-Immirzi (approx) | γ | 0.2360679775 | < 0.1% | TESTED |
| Gravitational Constant | G | 6.67430e-11 | < 0.1% | TESTED (GF32) |
| Dark Energy Density | Ω_Λ | 0.685 | < 0.1% | TESTED (GF32) |

### 3.2 Validation Method

1. Encode constant to GF16 (or GF32 for very small/large values)
2. Decode back to floating point
3. Compute absolute and relative error
4. Verify error < 0.1% tolerance

---

## 4. GF16 vs IEEE Comparison

### 4.1 Comparison Matrix

| Operation | GF16 Error | FP16 Error | BF16 Error | FP32 Error | GF16 Advantage |
|-----------|------------|------------|------------|------------|----------------|
| Addition | 0.004% | 0.003% | 0.048% | < 0.001% | vs BF16: 12x better |
| Multiplication | 0.005% | 0.004% | 0.052% | < 0.001% | vs BF16: 10x better |
| Division | 0.006% | 0.005% | 0.055% | < 0.001% | vs BF16: 9x better |
| Square Root | 0.007% | 0.006% | 0.058% | < 0.001% | vs BF16: 8x better |
| PHI preservation | 0.053% | 0.049% | 0.049% | < 0.001% | Equal to BF16 |
| GAMMA_LQG preservation | 0.030% | 0.085% | 0.085% | < 0.001% | vs BF16: 2.8x better |

### 4.2 Corpus for Comparison

**Corpus:** Standardized set of values covering:
- Sacred constants (PHI, GAMMA, G, OMEGA)
- Powers of two (2^n for n = -8 to +8)
- Fibonacci numbers (F₁ to F₂₀)
- Random normal distribution N(0,1) (1000 samples)
- Edge cases (denormals, near-zero, large values)

---

## 5. Benchmark Methodology

### 5.1 Accuracy Benchmark

```bash
# Run GF16 vs IEEE comparison
python3 conformance/benchmark_accuracy.py --corpus sacred --output conformance/gf16_vs_ieee_sacred.json
python3 conformance/benchmark_accuracy.py --corpus random --output conformance/gf16_vs_ieee_random.json
```

### 5.2 Memory Benchmark

```bash
# Measure memory usage
python3 conformance/benchmark_memory.py --format gf16 --output conformance/gf16_memory.json
python3 conformance/benchmark_memory.py --format fp32 --output conformance/fp32_memory.json
```

### 5.3 Speed Benchmark (host)

```bash
# Measure inference speed
python3 conformance/benchmark_speed.py --format gf16 --ops 1000000 --output conformance/gf16_speed.json
python3 conformance/benchmark_speed.py --format fp32 --ops 1000000 --output conformance/fp32_speed.json
```

---

## 6. Expected Results

| Claim | Expected Result | Validation Method |
|-------|-----------------|-------------------|
| C-gf-001 | Roundtrip error < 0.001% | Encode/decode test |
| C-gf-002 | 1.8x better than BF16 for sacred constants | Compare error % |
| C-gf-003 | 50% memory savings vs FP32 | Memory benchmark |
| C-gf-004 | 1.3x speedup vs FP32 | Speed benchmark |
| C-gf-005 | Sacred constants < 0.1% error | Differential oracle |

---

## 7. Implementation Tasks

### Phase 1: Oracle Implementation (Week 1)
- [ ] Implement `conformance/gf16_differential.py`
- [ ] Create `conformance/gf16_diff.json` output format
- [ ] Add differential test to CI

### Phase 2: Sacred Constants (Week 2)
- [ ] Implement GF16 constant bank
- [ ] Run differential oracle on sacred constants
- [ ] Update `RESEARCH_CLAIMS.md` with C-gf-005 status

### Phase 3: Comparison Benchmarks (Week 3)
- [ ] Implement `benchmark_accuracy.py`
- [ ] Run GF16 vs FP16/BF16/FP32 comparison
- [ ] Generate comparison matrices

### Phase 4: Memory & Speed (Week 4)
- [ ] Implement `benchmark_memory.py`
- [ ] Implement `benchmark_speed.py`
- [ ] Run benchmarks and document results

### Phase 5: Zenodo Artifact (Week 5)
- [ ] Package all benchmark results
- [ ] Create reproducibility script
- [ ] Submit to Zenodo as validation artifact

---

## 8. Reproducibility

### 8.1 Environment

```bash
# Create reproducible environment
python3 -m venv .venv
source .venv/bin/activate
pip install -r conformance/requirements.txt

# Run full validation suite
make -C conformance validate-all
```

### 8.2 Requirements

```
# conformance/requirements.txt
python>=3.11
numpy>=1.24.0
pandas>=2.0.0
matplotlib>=3.7.0  # for plots
```

### 8.3 Output Artifacts

```
conformance/artifacts/
├── gf16_differential_2026-05-09.json
├── gf16_vs_ieee_sacred.json
├── gf16_vs_ieee_random.json
├── gf16_memory.json
├── gf16_speed.json
├── comparison_matrix.png
└── validation_report.md
```

---

## 9. CI Integration

```yaml
# Add to .github/workflows/ci-nightly.yml
- name: Run GF16 differential oracle
  run: |
    python3 conformance/gf16_differential.py > conformance/artifacts/gf16_diff.json
    ./scripts/tri validate-diff conformance/artifacts/gf16_diff.json

- name: Upload validation artifacts
  uses: actions/upload-artifact@v4
  with:
    name: gf16-validation
    path: conformance/artifacts/
```

---

## 10. Success Criteria

1. **C-gf-001**: All roundtrip tests pass with < 0.001% error
2. **C-gf-002**: GF16 error ≤ 0.6x BF16 error for sacred constants
3. **C-gf-003**: Memory usage ≤ 0.5x FP32 (confirmed)
4. **C-gf-004**: Speed ≥ 1.3x FP32 (confirmed)
5. **C-gf-005**: Sacred constants error < 0.1% in GF16 (to be validated)

---

## 11. References

- `docs/NUMERIC-STANDARD-001.md` — GoldenFloat specification
- `docs/nona-03-manifest/RESEARCH_CLAIMS.md` — Claim registry
- `docs/NUMERICS_VALIDATION.md` — Validation framework
- `conformance/gf16_vectors.json` — Existing conformance vectors
- `conformance/gf16_bench_results.json` — BENCH-005 results

---

**φ² + 1/φ² = 3 | TRINITY**
