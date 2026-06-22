# Wave Loop 118 Decomposed Plan
## IGLA CODER + IGLA RACE — Mid-June 2026 Sweep

**Trigger:** Canonical request to research weaknesses, scientific literature, decompose, implement, report, and propose three cooperation variants.

---

## Phase 1: OBSERVE

### Weaknesses Identified
1. **IGLA RACE `eda.t27` / `yosys.t27` have zero native `test` blocks** — only `#[test]` Rust annotations. L4 TESTABILITY requires `test`/`invariant`/`bench` in t27 syntax.
2. **Hardware Quality Index (HQI) missing** — GLSVLSI 2026 paper shows simulation pass rates can overstate true hardware readiness by ~15 HQI points. Trinity has no HQI metric in IGLA CODER / RACE.
3. **Two new mid-2026 RTL competitors untracked** — AutoVeriFix+ (90.2% pass@10, concolic testing) and Synthesis-in-the-Loop (Hardware Quality Index metric) are absent from `benchmark.t27` and `COMPETITIVE_POSITIONING.md`.
4. **Contrastive learning primitives absent** — VerilogCL (arXiv:2604.18162) is tracked as a competitor, but IGLA CODER has no `contrastive_learning_pair` or `perturb_rtl_minimally` functions in `pipeline.t27`.
5. **Tool-assisted pre-processing absent** — LLM4RTL (June 2026) demonstrates that Python-based truth-table/Karnaugh-map pre-processing lets a 7B model match GPT-4O. IGLA CODER lacks `tool_assisted_preprocess` primitives.
6. **arXiv preprint competitive landscape stale** — §4 Competitive Landscape does not include AutoVeriFix+ or Synthesis-in-the-Loop.
7. **Coq: 0 real Admitted remain** (verified — only 2 comment occurrences of the word "Admitted"), but we keep monitoring.

### Competitive Intelligence
- **AutoVeriFix+** (arXiv:2603.11489): Three-stage framework (Python golden model → RTL refinement → concolic testing). Pass@10 = 90.2% on VerilogEval-machine.
- **Synthesis-in-the-Loop Evaluation** (GLSVLSI 2026): Evaluates 32 models with Hardware Quality Index (HQI) weighing post-synthesis area, delay, and warnings. Exposes ~15-point simulation-vs-synthesis gap.
- Stable landscape otherwise: 99 tracked competitors, no new EXTREME threats in RTL generation since Goedel-Architect (W117).

---

## Phase 2: PLAN (5 Tracks)

### Track A — IGLA RACE L4 Recovery + HQI Metric
- Add `HardwareQualityIndex` struct and `compute_hqi(area, delay, warnings, luts)` to `eval.t27`.
- Add native t27 `test` blocks to `eda.t27` (toolchain detection, script generation, PPA parsing).
- Add native t27 `test` blocks to `yosys.t27` (SVA emission, equivalence script, coverage aggregation).
- Add `bench` blocks for OpenROAD and Yosys script latency.

### Track B — Competitive Intelligence Expansion
- Add `autoverifix_plus_competitor()` to `benchmark.t27` (90.2% pass@10, concolic testing).
- Add `synthesis_in_the_loop_competitor()` to `benchmark.t27` (HQI metric, 32-model evaluation).
- Add 4 tests (2 name + 2 score/benchmark tests).

### Track C — IGLA CODER Contrastive & Tool-Assisted Primitives
- Add `contrastive_learning_pair()`, `perturb_rtl_minimally()`, `contrastive_loss()` to `pipeline.t27` (VerilogCL-style).
- Add `tool_assisted_preprocess()`, `karnaugh_map_minimize()`, `truth_table_to_sop()` to `eval.t27` (LLM4RTL-style).
- Add 8 tests and regenerate seal.

### Track D — arXiv Preprint §4 Update
- Insert AutoVeriFix+ and Synthesis-in-the-Loop into `docs/arXiv/TRINITY_ARXIV_DRAFT.md` and regenerate PDF.

### Track E — Seal Integrity & Suite Verification
- Regenerate seals for modified specs.
- Run `./target/release/t27c suite --repo-root .` and confirm 564/564 PASS.

---

## Phase 3: DELEGATE
- Creator Agent (C): Implement Tracks A, B, C spec changes.
- Verifier Agent (V): Run suite, check seal integrity, verify L1-L7 compliance.
- Learner Agent (L): Extract patterns for future wave loops.

---

## Phase 4: VERIFY
- Suite: 564/564 PASS, 0 seal mismatches.
- L4 TESTABILITY: All modified specs have `test`/`invariant`/`bench`.
- L3 PURITY: ASCII-only, English identifiers.

---

## Phase 5: SYNTHESIZE
- Combine all tracks into single commit with `Closes #1042`.

---

## Phase 6: LEARN
- Save wave-loop-118.md memory.
- Update MEMORY.md index.

---

## Estimated Impact
- +12 tests, +3 bench blocks across IGLA RACE.
- +2 competitors tracked.
- +3 new primitive functions in IGLA CODER.
- arXiv preprint competitive landscape current through June 2026.

φ² + 1/φ² = 3 | TRINITY
