# Wave Loop 109 Plan — IGLA CODER / IGLA RACE

**Date:** 2026-06-16
**Branch:** trinity-rust-rings
**Precedent:** WAVE_LOOP_108_REPORT.md, WAVE_LOOP_108_COOPERATION.md
**Issue:** #1038 (IGLA CODER pipeline infrastructure)

---

## Honest Gap Assessment (W108 → W109)

| Gap | Severity | W109 Track |
|-----|----------|-----------|
| No real subprocess for Verilator/Icarus | **HIGH** | A |
| Dataset 100× smaller than OpenRTLSet | **HIGH** | B |
| No PRM training loop (only scoring) | **HIGH** | C |
| No benchmark coalition crate packaging | **MEDIUM** | D |
| No empirical Pass@K score | **CRITICAL** | Deferred (requires GPU/API) |
| 90+ point gap to VeriAgent | **CRITICAL** | Deferred (requires trained model) |

---

## Track A: EDA Subprocess Hardening (`eval.t27`)

**Goal:** Make conceptual EDA subprocess stubs more complete by adding Verilator and Icarus equivalents.

**Functions to add:**
- `spawn_verilator_process(verilog_file) -> ProcessHandle` — conceptual Verilator subprocess spawn
- `spawn_icarus_process(verilog_file, testbench_file) -> ProcessHandle` — conceptual Icarus subprocess spawn
- `run_verilator_cli(verilog_file) -> bool` — end-to-end Verilator lint CLI runner
- `run_icarus_cli(verilog_file, testbench_file) -> bool` — end-to-end Icarus simulation CLI runner
- `verify_rtl_with_full_toolchain(rtl, template_name) -> bool` — unified pipeline: lint → synth → sim

**Tests:** 5 (verilator spawn valid/invalid, icarus spawn valid/invalid, full toolchain pass/fail)

---

## Track B: Dataset Scale Expansion (`dataset.t27`)

**Goal:** Close dataset gap by adding export utilities and a concrete 10K-scale generation function.

**Functions to add:**
- `export_dataset_to_json(dataset) -> string` — conceptual JSON serialization for dataset exchange
- `export_dataset_to_csv(dataset) -> string` — conceptual CSV serialization
- `generate_openrtlset_scale_dataset() -> []DataSample` — uses all expansion techniques (parametric × permutation × mutation × composition) to reach ~10K samples
- `count_unique_templates(dataset) -> u32` — diversity metric: how many distinct templates are represented
- `dataset_diversity_score(dataset) -> f32` — ratio of unique templates to total samples

**Tests:** 5 (json nonempty, csv nonempty, scale dataset size > 1000, unique templates count, diversity score range)

---

## Track C: PRM Training Loop (`prm.t27`)

**Goal:** Close gap with StepPRM-RTL by adding conceptual training primitives.

**Functions to add:**
- `train_prm_step(chosen_batch, rejected_batch, language) -> f32` — conceptual batch training step returning average loss
- `compute_prm_loss(chosen, rejected, language) -> f32` — wrapper around preference_loss for single pair
- `batch_score_with_prm(steps, language) -> []f32` — score multiple steps with PRM
- `update_prm_weights(current_loss, learning_rate) -> f32` — conceptual weight update (returns estimated new loss)
- `evaluate_prm_on_validation(validation_steps, language) -> f32` — validation accuracy proxy

**Tests:** 5 (train step returns loss, batch score count matches steps, weight update decreases loss, validation score bounded)

---

## Track D: Benchmark Coalition Crate (`benchmark.t27`)

**Goal:** Package evaluation harness for benchmark coalition (Variant C from W108 cooperation).

**Functions to add:**
- `trinity_rtl_eval_version() -> string` — crate version string "0.1.0-sacred"
- `sacred_compliance_axis_name() -> string` — "Sacred Compliance (R-SI-1)"
- `sacred_compliance_axis_score(rtl) -> f32` — 1.0 - sacred_constraint_penalty, continuous score for benchmark axis
- `benchmark_axis_supported() -> []string` — list of supported evaluation axes

**Tests:** 4 (version format, axis name, score compliant = 1.0, score violation < 1.0)

---

## Execution Order

1. Track A → Track B → Track C → Track D (parallel-safe; different files)
2. Suite run: `./scripts/tri test`
3. Fix seal mismatches
4. Write report + cooperation + memory
5. Git commit with L1 TRACEABILITY

---

**phi² + 1/φ² = 3 | TRINITY**
