# ALL FORMULA SEARCH METHODS — FULL REPORT

**Last updated:** 2026-04-10

---

## 1. Template n·φ^a·π^b·e^c (CLASSICAL)

| Version | Range | Targets | Formulas | Time |
|---------|----------|--------|--------|
| v5.1 | 1-100, -15..15 | 25 | 106 | 0.9s |
| v6.2 | 1-1000, -20..20 | 25 | 15,023 | 10.5s |
| v6.3 EXTREME | 1-5000, -20..20 | 25 | 295,564 | 22.1s |
| v6.4 ULTIMATE | 1-10000, -25..25 | 25 | 985,291 | 95.5s |
| v6.5 ABSOLUTE | 1-50000, -30..30 | 25 | 3,382,435 | 201s |

**Command:**
```bash
python3 scripts/ultra_engine_v65_absolute.py
```

---

## 2. GPU acceleration (CUDA)

| Version | Range | Backend | Speedup |
|---------|----------|----------|---------|
| v6.6 GPU | 1-100000, -30..30 | CuPy | ~10-100× |

**Required:** NVIDIA GPU + CUDA
**Installation:**
```bash
pip install cupy-cuda12x
```

**Command:**
```bash
python3 scripts/ultra_engine_v66_gpu.py
```

---

## 3. Rust Chimera Engine

| Parameter | Range | Base expressions |
|-----------|----------|-------------------|
| max_pow=5 | φ^±5·π^±5·e^±5 | 1,331 |
| max_pow=6 | φ^±6·π^±6·e^±6 | 2,197 |
| max_pow=7 | φ^±7·π^±7·e^±7 | 3,375 |
| max_pow=8 | φ^±8·π^±8·e^±8 | 4,913 |
| max_pow=10 | φ^±10·π^±10·e^±10 | 9,261 |

**Operators:** Mul, Div, Add, Sub, Sin, Cos, Log, Exp, Pow (9 total)

**Commands:**
```bash
# Deep search
./target/release/t27c formula chimera-search --max-pow 7 --threshold 0.01

# Deepest search
./target/release/t27c formula chimera-search --max-pow 10 --threshold 0.05
```

---

## 4. Matrix search (v6.7)

**Methods:**
- 2×2 matrices (determinant, trace, Frobenius norm)
- n-ary formula combinations
- Extended φ·π·e search up to max_pow=10

**Command:**
```bash
python3 scripts/ultra_engine_v67_matrix.py
```

---

## 5. NEW STRUCTURES (v6.8) — NEW FRONTIER

**Goes BEYOND classical template n·φ^a·π^b·e^c:**

| Structure | Description | Example |
|----------|-------------|--------|
| sin(n·X) | Sine of n·value | sin(π·φ) |
| cos(n·X) | Cosine of n·value | cos(2·π/φ) |
| ln(X) | Natural logarithm | ln(φ·π) |
| exp(n·X) | Exponential of n·x | exp(φ) |
| sqrt(n·X) | Square root of n·value | √(φ·π) |
| n-root(X) | nth root | ⁿ√(π) |
| Mixed trees | Arbitrary operator trees | (a+b)*(c-d) |

**Command:**
```bash
python3 scripts/ultra_engine_v68_new_structures.py
```

---
