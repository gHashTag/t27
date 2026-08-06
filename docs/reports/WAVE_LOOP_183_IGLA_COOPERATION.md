# Wave Loop 183 — Cooperation Variants for W184

**Date:** 2026-06-18
**Next Pool:** B — {systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm}
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — Neumann-Labs ternfpga Deep-Dive + Energy Benchmarking

**Goal:** Investigate the Neumann-Labs `ternfpga` GitHub repository (HIGH, Jun 2026) as the most credible open-source hardware threat. Extract actual Verilog RTL and compare energy-per-token against Trinity's `ternary_mac.t27` / `systolic_ternary.t27`.

**Actions:**
1. Inspect `Neumann-Labs/ternfpga` for Verilog RTL quality, ternary PE design, and sparsity-skipping logic.
2. Measure LUTs, BRAMs, power consumption on Arty A7-35T and compare with Trinity's multiplier-free architectures.
3. Add a `neumann_labs_ternfpga_v2_competitor` entry to `benchmark.t27` with actual resource numbers.
4. Add 2 new tests in `systolic_ternary.t27` or `ternary_mac.t27` inspired by their sparsity-skipping design (e.g., zero-weight skip, activation sparsity gating).

**Deliverables:**
- `docs/competitors/neumann_labs_ternfpga_deepdive.md`
- Updated `benchmark.t27` with v2 competitor entry
- +2 Pool B tests inspired by ternfpga
- W184 IGLA report

**Effort:** Medium.
**Risk:** Repository may lack detailed documentation or synthesis reports.

---

## Variant 2 — Pool B IGLA CODER+RACE + Invariant Depth Push Hybrid

**Goal:** Combine the standard +16 IGLA tests with a property-depth push in one Pool B spec.

**Actions:**
1. Add +2 tests to each Pool B spec (systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm) = +16 tests.
2. Select one Pool B spec with single-inv or double-inv status and add 1 hepta-invariant (7-property chain) or upgrade a double-inv to triple-inv.
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool B specs
- +1 invariant depth upgrade in one Pool B spec
- Updated seal files
- W184 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Stable Plateau Intelligence + L3 Hygiene Sweep

**Goal:** The competitive maturation plateau has lasted 6+ consecutive IGLA waves (W175–W183). Use this stability window for L3 PURITY hygiene and seal integrity audit.

**Actions:**
1. Audit all `specs/igla/` files for any remaining Unicode violations (em-dashes, math symbols, arrows).
2. Fix any L3 violations found (replace with ASCII equivalents).
3. Run `./target/release/t27c seal --verify` across all 570 specs to detect stale or missing seals.
4. Regenerate any stale seals.
5. Add +2 tests in `backend.t27` or `opcodes.t27` that simulate competitive PPA scoring (power vs. area trade-off curves for ternary vs. binary MAC units).
6. Run full suite; 570/570 PASS.

**Deliverables:**
- L3 hygiene report (violations found and fixed)
- Seal integrity audit report
- +2 tests in backend/opcodes
- W184 report with plateau analysis and hygiene summary

**Effort:** Low.
**Risk:** Minimal.

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — Neumann-Labs ternfpga Deep-Dive | Medium | Medium | High | If repo has RTL |
| 2 — Pool B + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — L3 Hygiene + Seal Audit | Low | Very Low | Medium | If maintenance mode |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
