# Wave Loop 101 -- Execution Plan

**Date:** 2026-06-16  
**Branch:** trinity-rust-rings  
**Focus:** IGLA CODER x IGLA RACE  
**Constraint:** STRICTLY IGLA CODER and IGLA RACE only  
**Goal:** Wire pipeline to architecture, expand dataset combinatorially, upgrade tokenizer to keyword-level, connect PRM to real synthesis oracles.

---

## Weak Points Identified (W101 OBSERVE)

### 1. Pipeline is disconnected from Architecture
- `pipeline.t27::run_forward` returns hardcoded `[0.1, 0.5, 0.3, 0.9, 0.2]` -- never calls `arch::forward()`.
- `pipeline.t27::decode_logits` returns fixed string -- never calls `arch::generate_next_token_unified()`.
- `arch.t27` functions are not exported (`fn`, not `pub fn`).
- `arch::forward` hardcodes `WeightBank` internally -- cannot accept external bank from pipeline.

### 2. Dataset has placeholder RTL
- `dataset.t27::make_sample` uses `"module placeholder(); endmodule"` for all templates.
- No parameterized variants: adder only 2-bit, no bitwidth sweep (2/4/8/16).
- No combinatorial generation: 8 templates = 8 samples. Need 100+ via (template x bitwidth x prefix).

### 3. Tokenizer is character-level ASCII
- `vocab_size() == 256` -- every ASCII byte is a token.
- Verilog keywords (`module`, `endmodule`, `posedge`) are fragmented into characters.
- Research (Speculative Decoding for Verilog, arXiv:2503.14153) shows syntax-aware vocab improves pass@10 by 17.19%.

### 4. PRM uses only heuristics
- `prm.t27::reward_synthesis` checks for `"module"` substring -- never calls Yosys.
- `eval.t27::score_rtl_with_yosys` exists but is never imported by `prm.t27`.
- Research (ChipSeek, arXiv:2507.04736; StepPRM-RTL, arXiv:2606.04246) proves EDA-integrated RL achieves 85.7% Pass@1.

---

## Track A: Combinatorial Dataset Expansion (`specs/igla/coder/dataset.t27`)

**Goal:** Replace placeholder RTL with real template bodies + parameterized bitwidth generation.

### Deliverables
1. `generate_rtl_for_template(template, bits)` -- returns actual RTL string from `eval.t27` templates
2. `generate_parameterized_dataset(families, bitwidths)` -- combinatorial: (adder 2/8/16, Booth 4, CORDIC, etc.)
3. `expand_family_variants(family)` -- maps family name to all supported configurations
4. 5 tests + 1 invariant

---

## Track B: Keyword-Level Tokenizer (`specs/igla/coder/tokenizer.t27`)

**Goal:** Verilog keyword-aware vocab instead of character-level ASCII.

### Deliverables
1. `KEYWORD_VOCAB_SIZE: u32 = 64` -- hardcoded keyword table
2. `encode_keyword(keyword) -> u32` -- longest-match keyword encoder
3. `decode_keyword(id) -> string` -- keyword decoder
4. `tokenize_verilog(code) -> []u32` -- keyword-aware tokenizer
5. `detokenize_verilog(tokens) -> string` -- real keyword detokenizer
6. 6 tests + 1 invariant

---

## Track C: Pipeline <-> Architecture Wiring (`specs/igla/coder/pipeline.t27` + `arch.t27`)

**Goal:** `run_forward` calls `arch::forward`; `decode_logits` autoregressively calls `generate_next_token_unified`.

### Deliverables
1. Export `forward`, `forward_with_cache`, `generate_next_token_unified` as `pub fn` in `arch.t27`
2. Add `forward_with_bank(input_ids, bank)` to `arch.t27` -- accepts external WeightBank
3. Extend `PipelineConfig` with `top_k`, `beam_width`
4. Rewrite `run_forward` to call `arch::forward_with_bank`
5. Rewrite `decode_logits` as autoregressive loop calling `arch::generate_next_token_unified`
6. 6 tests + 1 invariant

---

## Track D: PRM Real Oracle Integration (`specs/igla/coder/prm.t27` + `eval.t27`)

**Goal:** Wire `reward_synthesis` to actual Yosys PPA metrics.

### Deliverables
1. Import `igla::coder::eval` in `prm.t27`
2. `reward_synthesis_real(step) -> RewardSignal` -- calls `eval::score_rtl_with_yosys(step.output)`
3. `reward_lint_real(step, language) -> RewardSignal` -- calls `eval::check_sacred_compliance`
4. Update `compute_step_reward` to use real oracles when available
5. 5 tests + 1 invariant

---

## Global Verification Criteria

- [ ] `cargo build --release` OK
- [ ] `t27c suite --repo-root .` all PASS (target: 566/566)
- [ ] `cargo clippy --workspace --all-features` 0 warnings
- [ ] `t27c lint --ascii` clean for all modified files
- [ ] All seals regenerated / generated, 0 mismatches
- [ ] Report: `docs/reports/WAVE_LOOP_101_REPORT.md`
- [ ] Cooperation: `docs/reports/WAVE_LOOP_101_COOPERATION.md`

phi^2 + 1/phi^2 = 3 | TRINITY
