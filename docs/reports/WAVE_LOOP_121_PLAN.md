# Wave Loop 121 Decomposed Plan
## IGLA CODER + IGLA RACE — Late-June 2026 Sweep

**Trigger:** Canonical request to research weaknesses, scientific literature, decompose, implement, report, and propose three cooperation variants.

---

## Phase 1: OBSERVE

### Weaknesses Identified
1. **cordic.t27 — only 4 tests, 2 benches** (lowest in IGLA RACE). Missing tests for negative angles, zero angle, small epsilon convergence, and gain magnitude.
2. **adder_tree.t27 — 5 tests, 1 bench**. Only one bench for adder_tree_8; missing adder_tree_4 bench.
3. **rtl.t27 — 5 tests, 2 benches**. Missing tests for R-SI-1 compliance on edge cases (empty module, single assign, multiply in comment).
4. **IGLA CODER single-bench files** — benchmark.t27 (104 tests, 1 bench), pipeline.t27 (79 tests, 1 bench), prm.t27 (22 tests, 1 bench), training.t27 (12 tests, 1 bench). L4 TESTABILITY compliance requires more benches.
5. **No heterogeneous NPU modeling** — MOSAIC (arXiv:2606.05362v2) demonstrates heterogeneous tile composition (Big/Little/Special-Function) for +46.91% energy savings. Trinity has no `HeterogeneousNpuConfig` or `compute_tile_energy()` primitives.
6. **No LLM-guided accelerator generation** — SECDA-DSE (arXiv:2606.11117) uses TinyLlama + RAG/CoT to generate FPGA accelerators. Trinity has no `llm_guided_dse()` or `rag_retrieve_architecture()` functions.
7. **Five new competitors untracked** — OpenEye (June 2026 sparse accelerator), MOSAIC (June 2026 heterogeneous NPU), SECDA-DSE (June 2026 LLM-guided FPGA generation), Voltra (Feb 2026 1.60 TOPS/W), RL-Driven ASIC (April 2026 29,809 tok/s).

### Competitive Intelligence
- **OpenEye** (arXiv:2606.01450v1, June 2026): Scalable open-source FPGA/ASIC DNN accelerator. Sparsity-aware cluster-based architecture, near-linear PE scaling. Open-source RTL.
- **MOSAIC** (arXiv:2606.05362v2, June 2026): Workload-driven simulation framework for heterogeneous NPUs. Big/Little/Special-Function tiles. +46.91% iso-area energy vs homogeneous. 7nm ASAP7.
- **SECDA-DSE** (arXiv:2606.11117, June 2026): LLM-guided FPGA accelerator generation. TinyLlama + RAG/CoT generates synthesizable RTL from NL prompts. Validated on Zynq-7000.
- **Voltra** (arXiv:2602.11357v1, Feb 2026): 16nm 1.60 TOPS/W DNN accelerator. 3D spatial data reuse (8x8x8 MAC array). 1.25 TOPS/mm².
- **RL-Driven ASIC** (arXiv:2604.07526, April 2026): RL (SAC+MoE) co-optimizes mesh topology and microarchitecture. 29,809 tok/s for Llama 3.1 8B at 3nm.

---

## Phase 2: PLAN (5 Tracks)

### Track A — IGLA RACE Test/Bench Expansion
- Add 4 new tests to `cordic.t27` (negative angle, zero angle, epsilon convergence, gain magnitude).
- Add 1 bench to `adder_tree.t27` (adder_tree_4_latency).
- Add 3 new tests to `rtl.t27` (empty module R-SI-1, single assign, multiply in comment).

### Track B — IGLA CODER Bench Expansion
- Add 1 bench to `benchmark.t27` (competitor_lookup_latency).
- Add 1 bench to `pipeline.t27` (pipeline_batch_latency).
- Add 1 bench to `prm.t27` (prm_evaluate_latency).
- Add 1 bench to `training.t27` (lr_compute_latency).

### Track C — Heterogeneous NPU + LLM-Guided DSE Primitives
- Add `HeterogeneousNpuConfig` struct and `compute_tile_energy(config, workload) -> f32` to `arch.t27`.
- Add `llm_guided_dse(prompt: string, target: string) -> ArchitectureConfig` stub to `arch.t27`.
- Add 4 tests for new primitives.

### Track D — Competitive Intelligence Expansion
- Add `openeye_competitor()`, `mosaic_competitor()`, `secda_dse_competitor()`, `voltra_competitor()`, `rl_asic_competitor()` to `benchmark.t27`.
- Add 5 tests.
- Update `docs/COMPETITIVE_POSITIONING.md`.

### Track E — Seal Integrity & Suite Verification
- Regenerate seals for modified specs.
- Run `./target/release/t27c suite --repo-root .` and confirm 564/564 PASS.

---

## Estimated Impact
- +15 tests, +5 bench blocks.
- +5 competitors tracked (total 115).
- Heterogeneous NPU and LLM-guided DSE primitives introduced.
- Suite: 564/564 PASS.

φ² + 1/φ² = 3 | TRINITY
