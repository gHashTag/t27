# Wave Loop 101 Report
## IGLA CODER x IGLA RACE --- Architecture-to-Pipeline Wiring + Tokenizer Upgrade

**Date:** 2026-06-16
**Branch:** trinity-rust-rings
**Suite:** 562/562 PASS (0 failures)
**Clippy:** 0 warnings (--workspace --all-features)
**L3 ASCII:** Clean for all modified specs

---

## 1. Executive Summary

Wave Loop 101 closed four critical integration gaps between IGLA CODER specification and runtime-ready inference:

1. **Combinatorial Dataset Expansion** (dataset.t27) --- real RTL bodies inlined from eval.t27 templates; parameterized bitwidth generation (2/8/16-bit adders); expand_family_variants + generate_parameterized_dataset for family x bitwidth combinatorics.
2. **Keyword-Level Tokenizer** (tokenizer.t27) --- replaced ASCII character-level stub with 64-keyword Verilog vocabulary (IDs 256--319); tokenize_verilog / detokenize_verilog round-trip; vocab_size_total() == 320.
3. **Pipeline - Architecture Wiring** (pipeline.t27 + arch.t27) --- exported forward, forward_with_cache, generate_next_token_unified, generate_next_token_stochastic as pub fn; added forward_with_bank accepting external WeightBank; run_forward now calls arch::forward_with_bank; decode_logits wired to autoregressive generate_tokens_recursive calling generate_next_token_unified.
4. **PRM Real Oracle Integration** (prm.t27 + eval.t27) --- reward_synthesis now invokes eval::score_rtl_with_real_yosys; exported score_rtl_with_yosys, score_rtl_with_real_yosys, run_yosys_cli, parse_yosys_log, write_verilog_file as pub fn.

---

## 2. Track A: Combinatorial Dataset Expansion

**File:** specs/igla/coder/dataset.t27

### Changes
- Added generate_rtl_for_template(template_name, bits) --- inline copies of all 8 eval.t27 RTL templates (cordic, booth, adder 2/8/16, tree, systolic, ternary_gemm, ternary_cordic, systolic_streaming).
- Added generate_prompt_with_bits(template_name, bits) --- parameterized natural language prompts including bit width.
- Added expand_family_variants(family) --- maps family name to template variants.
- Added generate_parameterized_dataset(families, bitwidths) --- combinatorial cross-product of families x bitwidths.
- Updated generate_dataset to use real RTL bodies instead of "module placeholder(); endmodule".

### Verification
- Suite PASS, seal regenerated.

---

## 3. Track B: Keyword-Level Tokenizer

**File:** specs/igla/coder/tokenizer.t27

### Changes
- Added encode_keyword(keyword) --- 64-keyword Verilog vocabulary lookup (module, endmodule, input, output, wire, reg, assign, always, begin, end, if, else, case, endcase, posedge, negedge, or, and, not, nand, nor, xor, xnor, buf, integer, parameter, localparam, defparam, generate, endgenerate, function, endfunction, task, endtask, signed, unsigned, clk, rst_n, rst, inout, tri, supply0, supply1, ground, highz, pullup, pulldown, specify, endspecify, initial, forever, repeat, while, for, switch, return, break, continue, typedef, struct, union, enum, packed, unpacked).
- Added decode_keyword(id) --- reverse lookup from keyword ID to string.
- Added tokenize_verilog(code) --- space-split keyword tokenizer.
- Added detokenize_verilog(tokens) --- join decoded keywords with spaces.
- Added vocab_size_total() == 320 (256 ASCII + 64 keywords).
- Preserved backward-compatible encode_char / decode_char / tokenize / detokenize.

### Verification
- Suite PASS, seal regenerated.

---

## 4. Track C: Pipeline - Architecture Wiring

**Files:** specs/igla/coder/arch.t27, specs/igla/coder/pipeline.t27

### Changes in arch.t27
- Exported as pub fn: forward, forward_with_cache, generate_next_token, generate_next_token_temp, generate_next_token_unified, generate_next_token_stochastic.
- Added forward_with_bank(input_ids, bank, past_kv) --- accepts external WeightBank; core inference primitive.
- Added forward_with_cache_bank(input_ids, cache, bank) --- KV-cache + external bank variant.
- Updated forward to wrap forward_with_bank with hardcoded weights (legacy compatibility).
- Updated forward_with_cache to wrap forward_with_cache_bank with hardcoded weights.
- Added tests: forward_with_bank_returns_output, forward_with_cache_bank_returns_output.

### Changes in pipeline.t27
- Added use igla::coder::arch;.
- Rewrote run_forward(tokens, bank) to call arch::forward_with_bank(tokens, bank, []) and return out.logits (was hardcoded [0.1, 0.5, 0.3, 0.9, 0.2]).
- Added generate_next_token_from_logits(logits, cfg) --- calls arch::generate_next_token_unified with PipelineConfig hyperparameters.
- Added generate_tokens_recursive(logits, cfg, depth) --- autoregressive token generation up to max_tokens.
- Kept decode_logits as conceptual stub (real detokenize requires runtime vocab).
- Added tests: generate_next_token_from_logits_valid, generate_tokens_recursive_depth_zero, generate_tokens_recursive_one_step.
- Updated run_forward_returns_logits test: now expects len(logits) == 32000 (VOCAB_SIZE) instead of hardcoded 5.

### Verification
- Suite PASS, seals regenerated.

---

## 5. Track D: PRM Real Oracle Integration

**Files:** specs/igla/coder/prm.t27, specs/igla/coder/eval.t27

### Changes in eval.t27
- Exported as pub fn: score_rtl_with_yosys, score_rtl_with_real_yosys, run_yosys_cli, write_verilog_file, parse_yosys_log.

### Changes in prm.t27
- Rewrote reward_synthesis(step) to call eval::score_rtl_with_real_yosys(step.output) and return score 1.0 if synth_ok, else 0.0.
- Removed heuristic score based on output length / "module" substring.
- Added tests: reward_synthesis_oracle_ok, reward_synthesis_oracle_fail.

### Verification
- Suite PASS, seals regenerated.

---

## 6. Metrics

| Metric | Before W101 | After W101 |
|--------|-------------|------------|
| Total specs | 562 | 562 |
| Suite pass | 562/562 | 562/562 |
| Clippy warnings | 0 | 0 |
| Seal mismatches | 0 | 0 |
| Exported arch fns | 0 pub fn | 6 pub fn |
| Keyword vocab | 0 | 64 |
| Dataset RTL bodies | placeholder | 8 real templates |
| PRM synthesis oracle | heuristic | Yosys CLI wired |
| Pipeline forward | hardcoded | calls arch::forward_with_bank |

---

## 7. Remaining Gaps (Honest Assessment)

1. **Tokenizer runtime** --- tokenize_verilog is space-split only; real Verilog lexer needs comment/string/identifier handling.
2. **Weight loading runtime** --- tensor_to_weight_bank is conceptual; needs real .safetensors / .gguf parser.
3. **Autoregressive loop** --- generate_tokens_recursive resamples from same logits each step; real autoregression requires appending token to input_ids and re-running forward.
4. **Yosys CLI runtime** --- run_yosys_cli is stub; needs actual subprocess invocation in t27c runtime or external runner.
5. **Vocabulary detokenize** --- decode_logits still returns stub string; needs keyword-to-Verilog code emission.
6. **Training data scale** --- dataset.t27 generates ~24 samples (8 templates x 3 bitwidths); real training needs 10K+ samples with diversity augmentation.

---

## 8. Competitive Landscape

Stable at 96 competitors tracked. No new June 2026 entries discovered during W101 intel sweep. Key threats unchanged:
- #96 Baez & Schwahn (arXiv:2606.15235, exceptional Jordan algebra -> SM, EXTREME)
- #84 Douglas et al. (Lean 4 QFT, arXiv:2506.10301)
- #85 Washburn (arXiv:2506.12859v3, Lean 4, phi-based fermion masses, 0 sorry)

---

## 9. Next Wave Priorities (W102)

1. **Autoregressive forward loop** --- rewrite generate_tokens_recursive to append sampled token to input_ids and re-invoke forward_with_bank (requires slice append in t27).
2. **Training data augmentation** --- expand dataset.t27 to 100+ samples via template mutation (bit-width permutation, port renaming, comment insertion).
3. **Weight runtime loader** --- implement load_weights_from_file for .safetensors header parsing in spec.
4. **Yosys real CLI runner** --- add run_yosys_subprocess in t27c Zig backend that spawns yosys binary.

---

phi^2 + 1/phi^2 = 3 | TRINITY
