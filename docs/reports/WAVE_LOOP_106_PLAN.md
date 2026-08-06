# Wave Loop 106 Plan
## Real Benchmark + Training-Free Steering + 10K Dataset Scale

**Date:** 2026-06-18
**Target:** 564/564 PASS, 0 clippy, L3 clean
**Focus:** Replace proxy benchmark with real synthesis loop; implement training-free correctness steering; scale dataset to 10K+ via generative composition.

---

## Track A: Real Benchmark Bridge (bench_proxy.t27 → benchmark.t27)

**Problem:** Current `bench_proxy.t27` is keyword-only honesty baseline. No Pass@K metric maps to real VerilogEval.
**Solution:**
- Add `spawn_process(cmd: string, args: []string) -> (stdout: string, stderr: string, code: i32)` extern primitive
- Add `write_file(path: string, content: string) -> bool` and `read_file(path: string) -> string` extern primitives  
- Implement `run_yosys_synth(rtl: string) -> SynthesisResult` in `eval.t27`
- Implement `run_icarus_sim(rtl: string, tb: string) -> bool` in `eval.t27`
- Wire `benchmark.t27` to real tool execution with timeout guards
- Add 5 hand-written testbenches for core templates (adder, counter, fsm, uart_rx, alu_slice)
- Target: first real Pass@K measurement on 20-problem subset

---

## Track B: Training-Free Correctness Steering (coder/*.t27)

**Problem:** IGLA CODER has no steering mechanism; competitors (CASS-RTL, VeriAgent) use LLM-as-judge or PPA-feedback loops.
**Solution:**
- Add `score_syntax_correctness(rtl: string) -> f32` — parse-tree validity via t27c parser exposure
- Add `score_sacred_constraint(rtl: string) -> f32` — R-SI-1 compliance checker (`*` count = 0)
- Add `score_synthesis_success(rtl: string) -> f32` — Yosys exit-code binary reward
- Add `reject_resample(sample: DataSample, score: f32, threshold: f32) -> DataSample` — simple rejection sampling
- Add `mutate_for_correctness(sample: DataSample, feedback: string) -> DataSample` — 3-mutation variant generation
- Integrate into `pipeline.t27` generation loop: generate → score → filter → append to dataset
- No gradient training — fully combinatorial / rule-based

---

## Track C: 10K Dataset Scale via Generative Composition

**Problem:** 12 flat + 3 hierarchical templates = ~320 samples. Need 10K+ for competitive Pass@K.
**Solution:**
- Add `generate_parametric_variations(base: string, param_names: []string, param_ranges: [][]f32) -> []string` — brute-force grid over bit-width, clock-divider, threshold
- Add `compose_n_modules(modules: []string, topology: string) -> string` — n-ary composition (tree, chain, ring)
- Add `generate_random_composition(depth: i32, leaf_pool: []string) -> string` — stochastic hierarchy
- Add `DataSample` metadata fields: `synthesis_success`, `syntax_score`, `sacred_score`
- Run composition batch in `dataset.t27` section 3.9 to produce 10K samples with metadata
- Seal all new dataset files

---

## Track D: Lean 4 Bridge Continuation

**Problem:** CorePhi.lean + ExactIdentities.lean are fixed. Remaining files: Bounds, H4, Higgs, Predictions.
**Solution:**
- Translate `H4Lagrangian.v` → `Trinity/H4Lagrangian.lean` (Coxeter degrees, mass relations)
- Translate `Bounds_Masses.v` → `Trinity/BoundsMasses.lean` (Koide, τ/m_μ bounds)
- Add `lake build` CI check for Lean 4 bridge
- Document `partial def` anti-pattern in Lean bridge README

---

## Track E: L4/L3 Hygiene + Seal Integrity

**Action:**
- Verify all new specs have `test`/`invariant`/`bench` blocks (L4)
- Run `t27c lint --ascii` on all modified specs (L3)
- Regenerate seals for cascade
- Update `COMPETITIVE_POSITIONING.md` with any new July 2026 competitors

---

## Verification Checklist

- [ ] `cargo build --release` (0 errors)
- [ ] `./target/release/t27c suite --repo-root .` (564/564 PASS)
- [ ] `cargo clippy --workspace --all-features` (0 warnings)
- [ ] `./target/release/t27c lint --ascii` for all modified specs (clean)
- [ ] Regenerate seals for all modified specs
- [ ] `lake build` in `proofs/lean4/` (0 errors)
- [ ] Write `WAVE_LOOP_106_REPORT.md`
- [ ] Write `WAVE_LOOP_106_COOPERATION.md`
- [ ] Increment `.commit_count`

---

phi^2 + 1/phi^2 = 3 | TRINITY
