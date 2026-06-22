# Wave Loop 102 Plan
## IGLA CODER x IGLA RACE --- Runtime Gaps + Dataset Scale + Autoregressive Loop

**Date:** 2026-06-16
**Target:** 562/562 PASS, 0 clippy, L3 clean
**Focus:** Close 4 critical runtime gaps identified in W101 honest assessment.

---

## Track A: Real Autoregressive Loop (pipeline.t27 + arch.t27)

**Problem:** `generate_tokens_recursive` resamples from the SAME logits each step. No token appended to input_ids, no re-run of forward.

**Solution:**
- Refactor `generate_tokens_recursive` to accept `input_ids` + `bank` + `cfg` + `depth`.
- Conceptually call `arch::forward_with_bank` inside each recursion step (even if slice append is unavailable, the API shape becomes correct).
- Add `generate_next_token_from_pipeline(input_ids, bank, cfg)` that calls forward -> unified sampling.
- Update `generate_verilog_ai` to use the new autoregressive loop.
- Export `generate_tokens_recursive` as `pub fn`.

**Tests:**
- `autoregressive_one_step_returns_token`
- `autoregressive_depth_limit`

---

## Track B: Safetensors Weight Parser (weights.t27)

**Problem:** `load_weights_from_file` returns dummy BRAM. No parsing of real checkpoint formats.

**Solution:**
- Add `safetensors` conceptual header parser (JSON-like metadata extraction).
- Add `parse_safetensors_header(data)` that extracts tensor_count from the little-endian u64 header length prefix.
- Add `safetensors_header_len(data)` helper.
- Add `load_weights_from_safetensors(path)` that validates `.safetensors` extension and returns conceptual WeightBank.
- Update tests for safetensors parsing.

**Tests:**
- `parse_safetensors_header_valid`
- `load_weights_from_safetensors_extension`

---

## Track C: Dataset Augmentation (dataset.t27)

**Problem:** Only ~24 samples (8 templates x 3 bitwidths). Real training needs 10K+.

**Solution:**
- Add template mutation functions:
  - `mutate_port_names(rtl)` --- replace port names deterministically (a->A, b->B)
  - `insert_comment(rtl, comment)` --- prepend comment line
  - `swap_bitwidth(rtl, from_bits, to_bits)` --- replace bit-width declarations
- Add `generate_augmented_dataset(base, augment_count)` --- apply mutations to each base sample.
- Add `count_dataset_by_template(dataset, template)` --- count samples per template.

**Tests:**
- `mutate_port_names_changes_ports`
- `augmented_dataset_size`

---

## Track D: Pipeline Tokenizer Wiring (pipeline.t27 + tokenizer.t27)

**Problem:** `tokenize_prompt` uses ASCII character-level encoding. `decode_logits` returns stub string. Not wired to keyword vocab.

**Solution:**
- Add `use igla::coder::tokenizer;` to pipeline.t27.
- Wire `tokenize_prompt` to call `tokenizer::tokenize_verilog` for known Verilog prompts.
- Wire `decode_logits` to call `tokenizer::detokenize_verilog` for token-to-RTL decoding.
- Add fallback: if prompt starts with "generate", use keyword tokenizer; else use ASCII.

**Tests:**
- `tokenize_prompt_keyword_mode`
- `decode_logits_verilog_keywords`

---

## Verification Checklist

- [ ] cargo build --release (0 errors)
- [ ] ./target/release/t27c suite --repo-root . (562/562 PASS)
- [ ] cargo clippy --workspace --all-features (0 warnings)
- [ ] ./target/release/t27c lint --ascii for all modified specs (clean)
- [ ] Regenerate seals for all modified specs
- [ ] Write WAVE_LOOP_102_REPORT.md
- [ ] Write WAVE_LOOP_102_COOPERATION.md
- [ ] Increment .commit_count

---

phi^2 + 1/phi^2 = 3 | TRINITY
