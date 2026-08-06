# Wave Loop 120 Decomposed Plan
## IGLA CODER + IGLA RACE — Late-June 2026 Sweep

**Trigger:** Canonical request to research weaknesses, scientific literature, decompose, implement, report, and propose three cooperation variants.

---

## Phase 1: OBSERVE

### Weaknesses Identified
1. **cordic_top.t27 — only 3 tests** (lowest in IGLA RACE). Missing AXI-stream handshaking tests (ready/valid toggle, reset behavior, backpressure), and CORDIC gain accuracy.
2. **backend.t27 — only 4 tests** (low in RACE). Missing tests for R-SI-1 multiply detection in comments, power-of-two replacement, and Booth encoding path.
3. **systolic_array.t27 — only 4 tests, 1 bench**. Missing throughput benchmark for 2x2 GEMM and accumulator overflow guard tests.
4. **Single-bench files** — dataset.t27, pipeline.t27, prm.t27, training.t27, weights.t27, adder_tree.t27, bram_weights.t27, cordic.t27, cordic_fixed.t27 all have exactly 1 bench. L4 TESTABILITY compliance push requires at least 2 benches per critical spec.
5. **Backend realizability metric absent** — CktFormalizer (arXiv:2605.07782v2) demonstrates 95–100% backend realizability (synthesis → P&R → DRC → LVS). Trinity has no `BackendRealizabilityScore` struct or `compute_backend_realizability()` function.
6. **Integration gap: dataset quality → hardware synthesis** — `dataset.t27` scores text quality but never checks if generated RTL synthesizes or passes formal verification.
7. **Three new competitors untracked** — TeLLMe (ternary edge FPGA), TernaryCore (open-source BitNet accelerator), CORDIC-Is-All-You-Need (systolic CORDIC engine).

### Competitive Intelligence
- **TeLLMe** (arXiv:2504.16266): First end-to-end ternary LLM accelerator for edge FPGAs (AMD KV260). Table-lookup ternary MAC, ~9.5 tok/s under 7W. **HIGH threat** — directly overlaps Trinity ternary+FPGA positioning.
- **TernaryCore** (GitHub shepherdscientific/ternarycore): Open-source FPGA accelerator for BitNet b1.58. Native `{-1,0,+1}` add/sub/mux logic, no multipliers. Verified in simulation, Artix-7 target. **MEDIUM-HIGH threat** — open-source ternary RTL generator without sacred constraints.
- **CORDIC-Is-All-You-Need** (arXiv:2503.11685): SYCore — systolic CORDIC engine with reconfigurable PEs. 5-stage pipelined CORDIC MAC for DNNs/RNNs/Transformers. **MEDIUM threat** — CORDIC+systolic but no ternary or physics connection.

---

## Phase 2: PLAN (5 Tracks)

### Track A — IGLA RACE Test/Bench Expansion
- Add 6 new tests to `cordic_top.t27` (reset behavior, valid handshaking, backpressure, gain accuracy, invalid input, boundary angles).
- Add 4 new tests to `backend.t27` (multiply in comment ignored, power-of-two replacement, Booth path, empty expression).
- Add 2 new tests + 1 bench to `systolic_array.t27` (overflow guard, 2x2 throughput bench).

### Track B — Backend Realizability Metric
- Add `BackendRealizabilityScore` struct (synthesis_ok, par_ok, drc_ok, lvs_ok) to `eda.t27`.
- Add `compute_backend_realizability(score) -> f32` to `eda.t27`.
- Add `dataset_synthesis_score(dataset: Dataset) -> f32` to `dataset.t27` — calls `generate_verilog` + `compute_backend_realizability` on sample.
- Add 4 tests and 1 bench.

### Track C — Bench Expansion for Single-Bench Files
- Add 1 bench to `dataset.t27` (dataset pipeline latency).
- Add 1 bench to `cordic.t27` (CORDIC iteration latency).
- Add 1 bench to `cordic_fixed.t27` (fixed-point convergence latency).
- Add 1 bench to `weights.t27` (weight bank tensor conversion latency).

### Track D — Competitive Intelligence Expansion
- Add `tellme_competitor()`, `ternarycore_competitor()`, `cordic_is_all_you_need_competitor()` to `benchmark.t27`.
- Add 3 tests.
- Update `docs/COMPETITIVE_POSITIONING.md`.

### Track E — Seal Integrity & Suite Verification
- Regenerate seals for modified specs.
- Run `./target/release/t27c suite --repo-root .` and confirm 564/564 PASS.

---

## Estimated Impact
- +16 tests, +5 bench blocks.
- +3 competitors tracked (total 110).
- Backend realizability metric introduced.
- Suite: 564/564 PASS.

φ² + 1/φ² = 3 | TRINITY
