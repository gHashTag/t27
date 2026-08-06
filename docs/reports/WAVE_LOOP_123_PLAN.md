# Wave Loop 123 Decomposed Plan
## IGLA CODER + IGLA RACE — Late-June 2026 Sweep

**Trigger:** Canonical request to research weaknesses, scientific literature, decompose, implement, report, and propose three cooperation variants.

---

## Phase 1: OBSERVE

### Weaknesses Identified
1. **adder_tree.t27 — 5 tests, 2 benches** (lowest total in IGLA RACE). Missing tests for zero inputs and single-element cases.
2. **yosys.t27 — 6 tests, 2 benches**. Needs tests for property extraction and coverage edge cases.
3. **cordic.t27 — 7 tests, 2 benches**. Still below target of 12+; missing small-angle approx and convergence tests.
4. **opcodes.t27 — 8 tests, 1 bench**. Missing tests for cycle consistency and name uniqueness; only 1 bench.
5. **eda.t27 — 6 tests, 3 benches**. Low test count for a backend pass spec.
6. **training.t27 — 12 tests, 2 benches**. Still thin for a training spec; missing gradient clipping and sacred reward tests.

### Competitive Intelligence
- **HGQ-LUT** (arXiv:2604.22293, April 2026): LUT-aware training for FPGA DNN inference. Automated Verilog/VHDL RTL generation via da4ml toolchain. ~100x training speedup over prior LAT methods.
- **TriGen** (arXiv:2602.12962v1, Feb 2026): NPU for LLM inference with LUT-based Post-Processing Accelerator (PPA) for nonlinear ops (softmax, SiLU). 32x32 MAC array, FI32 format, validated against Synopsys DC at 14nm.

---

## Phase 2: PLAN (5 Tracks)

### Track A — IGLA RACE Test/Bench Expansion (Weakest Files)
- Add 2 tests + 1 bench to `adder_tree.t27` (zero inputs, single element, adder_tree_2_latency)
- Add 2 tests to `yosys.t27` (emit_sva_implication, aggregate_coverage_zero)
- Add 3 tests to `cordic.t27` (small_angle_approx, large_angle_wrap, iterative_convergence)
- Add 2 tests + 1 bench to `opcodes.t27` (opcode_cycle_consistency, name_unique, opcode_lookup_latency)

### Track B — IGLA CODER Test/Bench Expansion
- Add 2 tests to `training.t27` (gradient_clip_empty, sacred_reward_calculation)
- Add 1 bench to `training.t27` (gradient_clip_latency)

### Track C — Competitive Intelligence Expansion
- Add `hgq_lut_competitor()` and `trigen_competitor()` to `benchmark.t27`
- Add 2 tests for new competitors
- Update `docs/COMPETITIVE_POSITIONING.md`

### Track D — Documentation
- Create `WAVE_LOOP_123_PLAN.md`, `WAVE_LOOP_123_REPORT.md`, `WAVE_LOOP_123_COOPERATION.md`

### Track E — Seal Integrity & Suite Verification
- Regenerate seals for modified specs.
- Confirm 564/564 PASS.

---

## Estimated Impact
- +11 tests, +2 bench blocks.
- +2 competitors tracked (total 120).
- Suite: 564/564 PASS.

φ² + 1/φ² = 3 | TRINITY
