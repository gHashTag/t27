# Wave Loop 176 — Cooperation Variants for W177

**Date:** 2026-06-16  
**Next Pool:** A — {rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm}  
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — Deep Ternary Ibex Reverse-Engineering

**Goal:** Turn the MEDIUM-HIGH TernaryIbex entry into a Trinity threat assessment with measurable benchmarks.

**Actions:**
1. Fork or inspect GitHub repository `TheusHen/ternary-ibex` for exact ternary ALU instructions (tADD, tMUL, tMAC) and NPU systolic array size.
2. Build under Verilator; measure Ibex vs. TernaryIbex on MLPerfTiny KWS inference latency.
3. If code is open-source, add a `ternary_ibex_competitor` test to `benchmark.t27` with actual `pass_at_k` scores from CI.
4. Produce a comparison table: power (mW), area (mm²), accuracy (%), latency (ms).

**Deliverables:**
- `docs/competitors/ternary_ibex_analysis.md`
- Updated `benchmark.t27` with real scores
- 2 new tests in `rtl.t27` or `gemm.t27` inspired by Ibex ALU instructions

**Effort:** High (requires external repository build).
**Risk:** Repository may be private or poorly documented.

---

## Variant 2 — Pool A IGLA CODER+RACE + Depth Push Hybrid

**Goal:** Combine the standard +16 IGLA tests with a property-depth push (hexa-invariants) in `gemm.t27` or `cordic_top.t27`.

**Actions:**
1. Add +2 tests to each Pool A spec (rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm) = +16 tests.
2. Select one Pool A spec with single-inv or double-inv status (check via `tri inspect --depth`) and add 1 hexa-invariant (6-property chain).
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool A specs
- +1 hexa-invariant in one Pool A spec
- Updated seal files
- W177 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Stable Plateau Intelligence + Seal Hardening

**Goal:** No new competitors found in W175/W176 late June sweep. Use the lull to harden existing LOW/MEDIUM entries into Trinity-integrated benchmarks.

**Actions:**
1. Pick 3 dormant LOW competitors (e.g., ReTern, HGF, Litespark) and upgrade their `benchmark.t27` entries with actual arXiv/GitHub metadata (DOI, commit hash, citation count).
2. Add 2–3 new tests in `backend.t27` or `eda.t27` that simulate PPA scoring for these competing architectures (area vs. power trade-off curves).
3. Verify no duplicate competitor names; ensure `name` field is unique.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- 3 upgraded competitor entries with richer metadata
- +2–3 tests in backend/eda
- W177 report with stable-plateau analysis

**Effort:** Low.
**Risk:** Minimal.

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — Ibex Reverse-Engineering | High | Medium | Very High | If repo is public |
| 2 — Pool A + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Stable Plateau Hardening | Low | Very Low | Medium | If pressed for time |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
