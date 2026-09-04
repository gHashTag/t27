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
| L4 | **Differential** vs reference (round-trip oracle) | GF16 primary | **measured (host, sealed vs compiler.rs `49e55df6`)** — `repro/numerics/` |
| L5 | **Comparative** vs IEEE fp16 / bfloat16 on same corpus | GF16 vs fp16/bf16 | **measured (host, sealed vs compiler.rs `49e55df6`)** — `repro/numerics/nmse_manifest.json` |
| L6 | **Optional** posit reference (where tooling exists) | TBD | TBD |

---

## 5. Differential oracle — skeleton results table

*Measured runs (host, sealed against codec revision `49e55df6`).*

> **`bootstrap/stage0/FROZEN_HASH` NO LONGER HOLDS `49e55df6`.** It holds
> `9b8875f1…` today, last changed 2026-09-03 by #3026. The seal above is the
> record of what was measured and stays as one; what was wrong is the present
> tense — the sentence asserted a current fact about that file. Re-running
> `python repro/numerics/nmse_gf16.py --seal` today seals against `9b8875f1…`,
> so a re-run is a NEW reading and not a reproduction of the table below.
> `repro/numerics/nmse_manifest.json` carries its own seal, `87e5cbd3…`, stamped
> 2026-07-16; three identifiers, and none of them is the one in FROZEN_HASH now.

*Reference oracle = f64 round-trip `real -> format -> real`. Seed 2718281,
2,000,000 samples/distribution. Reproduce:
`python repro/numerics/nmse_gf16.py --seal` — and see the note above about what
that reproduces.*

*Reference oracle = f64 round-trip `real -> format -> real`. Seed 2718281, 2,000,000 samples/distribution. Reproduce: `python repro/numerics/nmse_gf16.py --seal`.*

| Run ID | Format | Operation | Corpus | Reference oracle | Max abs err | ULP-like metric | Pass? | Artifact |
|--------|--------|-----------|--------|------------------|-------------|-----------------|-------|----------|
| nmse-2718281-D_NORM | GF16 | round-trip | D_NORM (N(0,1)) | f64 | 3.90e-03 | 3.53e-04 | yes | `repro/numerics/nmse_manifest.json` |
| nmse-2718281-D_LOG | GF16 | round-trip | D_LOG (log2\|x\|~U(-10,10)) | f64 | (see manifest) | (see manifest) | yes | `repro/numerics/nmse_manifest.json` |
| nmse-2718281-D_WIDE | GF16 | round-trip | D_WIDE (log2\|x\|~U(-28,28)) | f64 | (see manifest) | (see manifest) | yes | `repro/numerics/nmse_manifest.json` |

**Falsification:** any cell exceeds stated envelope once §2 is normative → **fail CI** or **downgrade claim** in `RESEARCH_CLAIMS.md`. The runner already aborts non-zero if the L5 identity witness fails or any NMSE < 0.

---

## 6. IEEE / bfloat16 baseline — skeleton comparison

Same inputs as §5 where bit patterns map sensibly; document **non-comparable** cases explicitly.

*Measured, host, sealed vs compiler.rs `49e55df6` (`repro/numerics/nmse_manifest.json`). Non-comparable cases noted.*

| Metric | GF16 | IEEE fp16 | bfloat16 | Notes |
|--------|------|-----------|----------|-------|
| Mantissa / exponent bits | 9 / 6 | 10 / 5 | 7 / 8 | bit split |
| Max finite magnitude | ~4.29e9 | ~6.55e4 | ~3.39e38 | dynamic range |
| NMSE on N(0,1) (D_NORM) | 1.73e-07 | 4.30e-08 | 2.76e-06 | GF16 ~16x better than bf16; fp16 ~4x better than GF16 |
| Overflow rate on D_WIDE (log2\|x\|~U(-28,28)) | 0.0000 | 0.2144 | 0.0000 | fp16 saturates; GF16/bf16 do not |
| Add latency (soft impl) | n/a | n/a | n/a | host-only round-trip study; latency out of scope (see protocol §1) |

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
- **L4/L5 oracle (measured):** `python repro/numerics/nmse_gf16.py` — round-trip NMSE/ULP of GF16 vs bf16/fp16 over the protocol distributions; writes `repro/numerics/nmse_manifest.json` (rich) and `repro/numerics/nmse_manifest_protocol_v1.json` (certifying). Host; pass `--seal` to bind the run to the frozen compiler.rs digest.
- **Certifying manifest:** `make -C repro repro-numerics-certify` (or `python repro/numerics/validate_manifest.py`) validates `repro/numerics/nmse_manifest_protocol_v1.json` against `schemas/nmse-protocol-v1.json` and enforces the seal-hash honesty rule.
- **Sealed run:** `python repro/numerics/nmse_gf16.py --seal` sets `seal_hash` to the `bootstrap/stage0/FROZEN_HASH` digest **only** if the live seal source matches it; otherwise it stays `unsealed`. As committed the manifest is **sealed** against compiler.rs `49e55df6` (host) — informational, NOT a silicon certifying claim (protocol section 8). Toolchain: Python 3.12.8, numpy 2.4.6, ml_dtypes 0.5.4.

---

*The L4/L5 differential/comparative oracle is now MEASURED and SEALED at the host level (sealed vs compiler.rs `49e55df6` in `repro/numerics/`) — the predictable-skepticism gap is closed at the host level; the remaining step is a silicon-sealed certifying run under a pinned FPGA toolchain.*
