# Wave Loop 110 Plan — Weakness Closure + Competitive Intel Integration

**Date:** 2026-06-16
**Branch:** trinity-rust-rings
**Precedent:** WAVE_LOOP_109_REPORT.md
**Issues:** #1038 (IGLA CODER pipeline infrastructure)
**Basis:** W109 weakness analysis (7/10 fixable) + competitive intel sweep (21 new competitors)

---

## Honest Gap Assessment (W109 → W110)

| Rank | Weakness | Severity | W110 Track | Fixable by Code? |
|------|----------|----------|-----------|-----------------|
| 1 | No trained model; template dispatch | CRITICAL | — | No (budget-gated) |
| 2 | Zero empirical Pass@K score | CRITICAL | C | Partial |
| 3 | EDA/PPA subprocesses stubbed | CRITICAL | A | Yes |
| 4 | Dataset ~100× too small | HIGH | B | Partial |
| 5 | 5 open issues frozen by budget | HIGH | — | No (budget-gated) |
| 6 | Lean 4 bridge underdeveloped | HIGH | — | Yes (deferred) |
| 7 | 47 placeholder tests in tri/ | HIGH | E | Yes |
| 8 | Compiler optimizer returns empty IR | MEDIUM | F | Yes |
| 9 | Bench coverage gap (~49%) | MEDIUM | D | Yes |
| 10 | Pipeline decode stubbed | MEDIUM | — | Yes (deferred) |

---

## Track A: Real Subprocess Bridge (`eval.t27`)

**Goal:** Replace Yosys/Verilator stubs with real subprocess calls.

**Functions:**
- `run_command(cmd, args) -> string` — conceptual subprocess execution returning stdout
- `run_yosys_real(verilog_file) -> YosysReport` — real Yosys execution
- `run_verilator_real(verilog_file) -> bool` — real Verilator execution
- `run_icarus_real(verilog_file, testbench) -> bool` — real Icarus execution
- `parse_stdout_for_metric(stdout, prefix) -> string` — generic stdout parser

**Tests:** 5 (command nonempty, yosys returns struct, verilator valid/invalid, icarus valid/invalid)

## Track B: Dataset Quality Pipeline (`dataset.t27`)

**Goal:** Filter and score dataset by quality metrics.

**Functions:**
- `score_dataset_sample(sample) -> f32` — quality = syntax(0.33) + sacred(0.33) + synth(0.34)
- `filter_dataset_by_quality(dataset, threshold) -> []DataSample`
- `filter_dataset_by_sacred(dataset) -> []DataSample`
- `compute_dataset_quality_report(dataset) -> f32`

**Tests:** 5

## Track C: Empirical Pass@K Estimation (`benchmark.t27`)

**Goal:** Honest Pass@K estimation from template coverage (no GPU required).

**Functions:**
- `estimate_pass_at_k_from_coverage(dataset, k) -> f32`
- `estimate_trinity_pass_at_1() -> f32`
- `estimate_trinity_pass_at_10() -> f32`
- `coverage_gap_to_competitor(trinity_estimate, competitor) -> f32`

**Tests:** 4

## Track D: L4 Benchmark Expansion (bench blocks)

**Goal:** Add bench blocks to 25+ naked specs.

**Priority:**
1. Compiler frontend: `lexer.t27`, `parser.t27`, `typechecker.t27`
2. tri/ infrastructure: `pipeline.t27`, `workflow.t27`, `agent.t27`
3. Hot primitives: memory access, string ops

**Verification:** t27c suite → 0 failures

## Track E: Placeholder Test Fix (47 stubs in tri/)

**Goal:** Replace 47 `test placeholder` with real tests.

**Priority:**
1. Pipeline specs (codegen, parallel, workflow, orchestrator)
2. Net specs (async)
3. Agent specs

**Verification:** Suite passes; no placeholder remaining

## Track F: Compiler Optimizer Fix

**Goal:** Fix broken constant folding and DCE.

**File:** `specs/compiler/optimizer.t27`
**Issues:**
1. `optimize_expr()` returns empty `Node{kind=ExprLiteral, value=""}`
2. `optimize_stmt()` returns dead stmt instead of removing it
3. `is_dead_stmt()` only checks `StmtLocal` with zero children

**Fix:** Implement value propagation, null-marker removal, broader dead-stmt detection.
**Verification:** Add tests verifying optimized AST correctness.

## Track G: Competitive Intel Integration

**Goal:** Add 8 new competitors to `benchmark.t27`.

**New discoveries:**
1. RTLScout (Huawei) — agentic RTL + PPA, 35% area reduction
2. StepPRM-RTL (IBM) — MCTS+RAFT, Pass@1=85.7%, DAC'26
3. CktFormalizer — Lean 4 as dependently-typed HDL
4. GoldenFloat — φ-derived FP, RTL generator, 323 MHz Artix-7
5. KU Leuven Ternary — Chisel ternary LUT accelerator, TSMC 16nm
6. EstRTL (NUDT) — Generation→Estimation→Correction
7. LLM4RTL-2026 (UC Riverside) — 7B DeepSeek-Coder, Pass@1≈60.8%
8. CASS-RTL — Correctness-aware subspace steering

**Verification:** Tests for each competitor; suite passes

---

## Execution Order

Phase 1 (Parallel): Track A + Track D + Track E + Track G
Phase 2 (After Phase 1): Track B + Track C
Phase 3 (After Phase 2): Track F (compiler fix)
Phase 4: Seal regeneration + suite verification
Phase 5: Report + cooperation variants

phi^2 + 1/phi^2 = 3 | TRINITY
