# Wave Loop 179 — Cooperation Variants for W180

**Date:** 2026-06-16  
**Next Pool:** B — {systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm}  
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — TernaryCore Shepherd Scientific Reverse-Engineering

**Goal:** Investigate the shepherdscientific/ternarycore GitHub repository (April 2026) as a potential MEDIUM competitor.

**Actions:**
1. Inspect `shepherdscientific/ternarycore` for Verilog RTL quality, ternary MAC unit design, and simulation passing status.
2. If open-source, measure Xilinx Artix-7 resource utilization and compare with Trinity's `rtl.t27` / `gemm.t27` multiplier-free architectures.
3. Add a `ternarycore_competitor` entry to `benchmark.t27` with actual metadata.
4. Add 2 new tests in `systolic_ternary.t27` or `ternary_mac.t27` inspired by their PE design.

**Deliverables:**
- `docs/competitors/ternarycore_analysis.md`
- Updated `benchmark.t27` with new competitor
- +2 Pool B tests inspired by ternarycore
- W180 IGLA report

**Effort:** Medium.
**Risk:** Repository may lack detailed documentation.

---

## Variant 2 — Pool B IGLA CODER+RACE + IGLA Depth Push Hybrid

**Goal:** Combine the standard +16 IGLA tests with a property-depth push in one Pool B spec.

**Actions:**
1. Add +2 tests to each Pool B spec (systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm) = +16 tests.
2. Select one Pool B spec with single-inv or double-inv status and add 1 hexa-invariant (6-property chain) or upgrade a double-inv to triple-inv.
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool B specs
- +1 invariant depth upgrade in one Pool B spec
- Updated seal files
- W180 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Stable Plateau Intelligence + Competitor Consolidation

**Goal:** The competitive maturation plateau has lasted 5 consecutive IGLA waves (W175–W179). Use this stability window to consolidate and prune the competitor registry.

**Actions:**
1. Audit `benchmark.t27` for dormant LOW competitors (no activity in 6+ months) and downgrade or remove 3 entries.
2. Verify all remaining entries have unique `name` fields and consistent `benchmark` strings.
3. Add 2–3 new tests in `backend.t27` or `opcodes.t27` that simulate competitive PPA scoring (power vs. area trade-off curves for ternary vs. binary MAC units).
4. Run full suite; 570/570 PASS.

**Deliverables:**
- 3 pruned/updated competitor entries
- +2–3 tests in backend/opcodes
- W180 report with plateau analysis and consolidation summary

**Effort:** Low.
**Risk:** Minimal.

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — TernaryCore Investigation | Medium | Medium | High | If repo has RTL |
| 2 — Pool B + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Plateau Consolidation | Low | Very Low | Medium | If maintenance mode |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
