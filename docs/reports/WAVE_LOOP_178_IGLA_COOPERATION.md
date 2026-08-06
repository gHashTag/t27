# Wave Loop 178 — Cooperation Variants for W179

**Date:** 2026-06-16  
**Next Pool:** A — {rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm}  
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — Baez & Schwahn EXTREME Threat Assessment + Response Paper

**Goal:** Respond to Baez & Schwahn's June 2026 arXiv:2606.15235 with a Trinity differentiation document.

**Actions:**
1. Deep-read arXiv:2606.15235 and compare with Trinity's E₈→H₄→SM approach.
2. Identify gaps: Baez & Schwahn's paper is algebraic (Jordan algebra) but lacks machine proofs, hardware instantiation, and testable predictions.
3. Write `docs/competitors/baez_schwahn_2606_15235_response.md` highlighting Trinity's advantages: 23 observables vs. their gauge-group derivation, FPGA sacred opcodes, 4 testable predictions.
4. Add 2 new tests in `formal.t27` that verify equivalence-checking capabilities relevant to Jordan-algebra subalgebra preservation.

**Deliverables:**
- Response document vs. Baez & Schwahn June 2026
- +2 formal.t27 tests
- W179 IGLA report

**Effort:** High (requires paper analysis).
**Risk:** Medium (paper may be dense).

---

## Variant 2 — Pool A IGLA CODER+RACE + Ternary Ibex Deep Dive

**Goal:** Combine standard +16 IGLA tests with reverse-engineering of TernaryIbex (W176 competitor) if repository is now more mature.

**Actions:**
1. Add +2 tests to each Pool A spec (rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm) = +16 tests.
2. Re-inspect GitHub `TheusHen/ternary-ibex` for new commits since Jan 2026. If RTL is now available, synthesize under Yosys and measure area/power.
3. If data is available, update `benchmark.t27` TernaryIbex entry with actual `pass_at_k` scores.
4. Verify 570/570 PASS.

**Deliverables:**
- +16 tests across 8 Pool A specs
- TernaryIbex benchmark update (if repo matured)
- W179 IGLA report

**Effort:** Medium-High.
**Risk:** Repo may still be research-grade without RTL.

---

## Variant 3 — Stable Plateau + Competitor Metadata Backfill

**Goal:** No new competitors for 3 consecutive IGLA waves (W175–W178). Use the stable period to upgrade dormant entries.

**Actions:**
1. Select 5 LOW/MEDIUM dormant competitors and add concrete metadata (arXiv DOI, GitHub stars, last commit date, license).
2. Add +2 tests in `benchmark.t27` per upgraded competitor (name + benchmark string tests) = +10 tests. Wait, benchmark already has 2 tests per competitor. Instead, add PPA-simulation tests in `backend.t27` or `eda.t27`.
3. Verify no duplicate names; run full suite.

**Deliverables:**
- 5 upgraded competitor entries with richer metadata
- +2–4 tests in backend/eda
- W179 report with extended plateau analysis

**Effort:** Low.
**Risk:** Minimal.

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — Baez & Schwahn Response | High | Medium | Very High | If physics track priority |
| 2 — Pool A + TernaryIbex Dive | Medium-High | Medium | High | If repo updated |
| 3 — Stable Plateau Backfill | Low | Very Low | Medium | **(Recommended default)** |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
