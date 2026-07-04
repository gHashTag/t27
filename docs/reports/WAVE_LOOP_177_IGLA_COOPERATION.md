# Wave Loop 177 — Cooperation Variants for W178

**Date:** 2026-06-16  
**Next Pool:** B — {systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm}  
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — ETH TernaryLLM Reverse-Engineering + Benchmark Integration

**Goal:** Turn the MEDIUM ETH_TernaryLLM entry into a Trinity threat assessment with measurable benchmarks.

**Actions:**
1. Inspect GitHub repository `fpgasystems/ternaryLLM` for exact Alveo U55C utilization, ternary GEMM throughput (TOPS), and sparse-skip efficiency.
2. If open-source RTL is available, synthesize under Vivado or Yosys; measure LUT/FF/DSP counts and compare with Trinity's `rtl.t27` / `gemm.t27` multiplier-free architectures.
3. Add actual `pass_at_k` scores to `benchmark.t27` based on MLPerf or custom inference latency.
4. Produce a comparison table: throughput (TOPS), power (W), area (mm²), sparsity skip rate.

**Deliverables:**
- `docs/competitors/eth_ternaryllm_analysis.md`
- Updated `benchmark.t27` with real scores
- 2 new tests in `gemm.t27` or `backend.t27` inspired by Alveo U55C PPA data

**Effort:** High (requires external repository analysis).
**Risk:** Repository may lack detailed synthesis reports.

---

## Variant 2 — Pool B IGLA CODER+RACE + Property Depth Push

**Goal:** Combine the standard +16 IGLA tests with a property-depth push (hexa-invariants) in one Pool B spec.

**Actions:**
1. Add +2 tests to each Pool B spec (systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm) = +16 tests.
2. Select one Pool B spec with single-inv or double-inv status and add 1 hexa-invariant (6-property chain).
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool B specs
- +1 hexa-invariant in one Pool B spec
- Updated seal files
- W178 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Stable Plateau Intelligence + Competitor Metadata Hardening

**Goal:** No new EXTREME/HIGH competitors in W177 late June sweep. Use the lull to upgrade dormant LOW entries with concrete metadata.

**Actions:**
1. Pick 3 dormant LOW competitors (e.g., ReTern, HGF, Litespark) and upgrade their `benchmark.t27` entries with actual arXiv DOI, GitHub commit hash, or citation count.
2. Add 2–3 new tests in `backend.t27` or `eda.t27` that simulate Alveo U55C vs. Artix-7 PPA trade-offs (power vs. throughput curves).
3. Verify no duplicate competitor names; ensure `name` field is unique.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- 3 upgraded competitor entries with richer metadata
- +2–3 tests in backend/eda
- W178 report with stable-plateau analysis

**Effort:** Low.
**Risk:** Minimal.

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — ETH TernaryLLM Reverse-Engineering | High | Medium | Very High | If repo has RTL |
| 2 — Pool B + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Stable Plateau Hardening | Low | Very Low | Medium | If pressed for time |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
