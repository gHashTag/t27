# Numerics validation — GoldenFloat and related formats

**Status:** Program document — **commit-friendly skeleton**; fill cells as tests and Zenodo bundles land.  
**Companion:** `docs/NUMERIC-STANDARD-001.md`, `docs/NUMERIC-GF16-DEBT-INVENTORY.md`, `docs/RESEARCH_CLAIMS.md` (**C-gf-001**, **C-gf-002**).

---

## 1. Goals

- Make GoldenFloat **falsifiable** for numerics reviewers.  
- Separate **specification** from **benchmark narrative**.  
- Produce **machine-checkable** outputs (CSV / JSON) suitable for CI and Zenodo reproduction.

---

## 2. Required definitions (normative targets)

| Topic | Question | Spec / doc target | Status |
|-------|----------|-------------------|--------|
| Rounding | Per-operation rule (nearest, toward zero, …) | `specs/numeric/*.t27` + this doc | TBD |
| Overflow / underflow | Saturation, ±Inf, or trap | Same | TBD |
| NaN / Inf | Allowed or excluded | Same | TBD |
| Subnormals | Flush to zero vs gradual | Same | TBD |
| Transcendentals | Forbidden, lib-mapped, or range-limited | Same | TBD |
| Error envelopes | ULP-like or max-abs error per op per format | Same | TBD |

Until filled, treat numeric behavior as **implementation-defined** outside conformance vectors.

---

## 3. Claim traceability (`docs/RESEARCH_CLAIMS.md`)

| ID | Claim (short) | This doc § |
|----|---------------|------------|
| C-gf-001 | GF16/GF32 effective accuracy vs width | §5–7 |
| C-gf-002 | Accuracy–energy vs IEEE fp32 on FPGA | §8 |

---

## 4. Testing ladder (execution order)

| Stage | Method | Formats | Status |
|-------|--------|---------|--------|
| L1 | **Exhaustive** encode/decode + op table | GF4 (and GF8 if feasible) | TBD |
| L2 | **Conformance JSON** — existing `conformance/gf*_vectors.json` | GF4–GF32 as covered | partial |
| L3 | **Property-based / randomized** boundaries | GF16+ | TBD |
| L4 | **Differential** vs reference (Python `decimal`, or MPFR) | GF16 primary | TBD — P1 |
| L5 | **Comparative** vs IEEE fp16 / fp32 / bfloat16 on same corpus | GF16 vs fp16/bf16 | TBD |
| L6 | **Optional** posit reference (where tooling exists) | TBD | TBD |

---

## 5. Differential oracle — skeleton results table

*Replace `TBD` with versioned runs; one row per (format, operation, corpus slice).*

| Run ID | Format | Operation | Corpus | Reference oracle | Max abs err | ULP-like metric | Pass? | Artifact |
|--------|--------|-----------|--------|------------------|-------------|-----------------|-------|----------|
| TBD | GF16 | add | conformance subset | Python `decimal` | TBD | TBD | TBD | `repro/numerics/` (future) |
| TBD | GF16 | mul | … | … | TBD | TBD | TBD | … |
| TBD | GF32 | add | … | … | TBD | TBD | TBD | … |

**Falsification:** any cell exceeds stated envelope once §2 is normative → **fail CI** or **downgrade claim** in `RESEARCH_CLAIMS.md`.

---

## 6. IEEE / bfloat16 baseline — skeleton comparison

Same inputs as §5 where bit patterns map sensibly; document **non-comparable** cases explicitly.

| Metric | GF16 | IEEE fp16 | bfloat16 | IEEE fp32 | Notes |
|--------|------|-----------|----------|-----------|-------|
| Dynamic range (stated) | TBD | TBD | TBD | TBD | From spec / measured |
| MSE on N(0,1) sample | TBD | TBD | TBD | TBD | Trinity Phase-1 style table may be ported |
| Add latency (soft impl) | TBD | TBD | — | TBD | Host-only; not FPGA |

---

## 7. Conformance vectors ↔ validation map

| Conformance file (pattern) | Spec module (typical) | Ladder stage |
|----------------------------|------------------------|--------------|
| `conformance/gf*_vectors.json` | `specs/numeric/` | L2 |
| (future) `conformance/gf16_diff.json` | numeric + testgen | L4 |

Extend `docs/RINGS.md` TASK-5.x when a traceability graph is automated.

---

## 8. FPGA / energy — skeleton (C-gf-002)

| Benchmark | Platform | Metric | GF vs fp32 | Method | Status |
|-----------|----------|--------|------------|--------|--------|
| TBD | e.g. XC7A100T | J/inference | TBD | Measured wall + power meter / board telemetry | CONJECTURAL until filled |

---

## 9. Phi as engineering hypothesis

Document **why** phi-scaled exponent/mantissa ratios are **useful** (dynamic range, bit budget, stability of integer-backed paths) as **falsifiable engineering** claims — tie metrics to columns in §6–8 and to new rows in `docs/RESEARCH_CLAIMS.md` if needed.

---

## 10. CODATA / NIST

Constant comparisons (if any) must cite **year and revision** and uncertainty; do not mix CODATA epochs in one table without conversion notes.

---

## 11. Reproduction

- **Smoke:** `make -C repro repro-numerics` (JSON validity).  
- **Future:** `make repro-numerics-diff` (pinned Python + lockfile) — add in `repro/Makefile` when L4 exists.

---

*Without differential oracles, GoldenFloat will face predictable skepticism — this file is the contract to close that gap.*
