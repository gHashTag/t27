# Wave Loop 182 — Cooperation Variants for W183

**Date:** 2026-06-18
**Next Pool:** A — {rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm}
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — ETH TernaryLLM Deep-Dive + RTL Benchmarking

**Goal:** Investigate the ETH Zurich `fpgasystems/ternaryLLM` repository as the most credible MEDIUM-HIGH hardware competitor. Extract actual Verilog modules and compare against Trinity's `systolic_array.t27` / `ternary_gemm.t27` multiplier-free architectures.

**Actions:**
1. Clone or inspect `fpgasystems/ternaryLLM` for ternary GEMM Verilog RTL, HLS C++ kernels, and Xilinx Alveo U55C resource utilization reports.
2. Measure LUTs, DSPs, BRAMs for their ternary MAC vs. Trinity's `ternary_mac.t27` / `adder_tree.t27`.
3. Add an `eth_ternaryllm_v2_competitor` entry to `benchmark.t27` with actual resource numbers if available.
4. Add 2 new tests in `ternary_gemm.t27` or `systolic_array.t27` inspired by their dataflow (e.g., pipelined accumulation, sparse weight skipping).

**Deliverables:**
- `docs/competitors/eth_ternaryllm_deepdive.md`
- Updated `benchmark.t27` with v2 competitor entry
- +2 Pool A tests inspired by ETH TernaryLLM
- W183 IGLA report

**Effort:** Medium.
**Risk:** Repository may be code-only without synthesis reports.

---

## Variant 2 — Pool A IGLA CODER+RACE + Invariant Depth Push Hybrid

**Goal:** Combine the standard +16 IGLA tests with a property-depth push in one Pool A spec.

**Actions:**
1. Add +2 tests to each Pool A spec (rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm) = +16 tests.
2. Select one Pool A spec with single-inv or double-inv status and add 1 hepta-invariant (7-property chain) or upgrade a double-inv to triple-inv.
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool A specs
- +1 invariant depth upgrade in one Pool A spec
- Updated seal files
- W183 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Stable Plateau Intelligence + Coq Axiom Roadmap

**Goal:** The competitive maturation plateau has lasted 6 consecutive IGLA waves (W175–W182). Use this stability window to advance the Coq axiom closure roadmap.

**Actions:**
1. Audit `proofs/trinity/` for the 5 remaining Coq Axioms: Koide (1), NeutrinoMasses (4). Identify which can be closed with existing PDG bounds or interval arithmetic.
2. Attempt to close **1 NeutrinoMasses axiom** using the updated PDG 2026 neutrino mass bounds (< 0.052 eV sum) and `interval` tactic.
3. Add 2 new tests in `formal.t27` or `cordic_top.t27` that simulate formal-equivalence checking between a ternary and binary MAC unit.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- 1 closed Coq Axiom (or documented proof attempt)
- +2 tests in formal/cordic_top
- W183 report with plateau analysis and axiom roadmap update

**Effort:** Medium-High.
**Risk:** Axiom closure may require PhD-level spectral-action work; honest `Admitted` with roadmap is acceptable.

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — ETH TernaryLLM Deep-Dive | Medium | Medium | High | If repo has synthesis reports |
| 2 — Pool A + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Coq Axiom Roadmap | Medium-High | Medium | Very High | If spectral-action expert available |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
