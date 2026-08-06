# Wave Loop 103 Plan
## IGLA CODER x IGLA RACE --- KV-Cache + Dataset Mutation + Hybrid Tokenizer + Yosys Runtime

**Date:** 2026-06-17
**Target:** 562/562 PASS, 0 clippy, L3 clean
**Focus:** Close 4 remaining runtime gaps after W102.

---

## Track A: KV-Cache Incremental Update (arch.t27)

**Problem:** `forward_with_cache_bank` returns cache unchanged. No incremental KV-cache append.
**Solution:**
- Implement `kv_cache_append(cache, key_row, value_row)` to conceptually append rows to 2D cache.
- Add `kv_cache_from_rows(key_rows, value_rows)` helper.
- Update `forward_with_cache_bank` to return updated cache with appended rows.
- Add tests for cache append and cache dimensions.

---

## Track B: Dataset Mutation Engine (dataset.t27)

**Problem:** Dataset augmentation is only comment insertion. No port-name mutation, bit-width swap, or parameter randomization.
**Solution:**
- Add `mutate_port_names(rtl)` --- deterministic port renaming (a->A, b->B, x->X).
- Add `swap_bitwidth_in_rtl(rtl, old_bits, new_bits)` --- replace bit-width declarations.
- Add `generate_mutation_variants(sample)` --- apply all mutations to a single sample.
- Add `generate_full_augmented_dataset(base)` --- comment variants + port mutations + bit-width swaps.
- Add tests for each mutation function.

---

## Track C: Hybrid Prompt Tokenizer (pipeline.t27 + tokenizer.t27)

**Problem:** `tokenize_prompt` uses ASCII character-level encoding. Natural language prompts need keyword-aware tokenization.
**Solution:**
- Add `tokenize_prompt_hybrid(prompt)` to tokenizer.t27:
  - Split prompt on spaces
  - If word matches known Verilog keyword, encode as keyword ID (256--319)
  - Otherwise encode as ASCII character-level
- Wire pipeline.t27 `tokenize_prompt` to use hybrid tokenizer for prompts containing "generate" or "module".
- Add tests for hybrid tokenization.

---

## Track D: Yosys Subprocess Runtime (eval.t27)

**Problem:** `run_yosys_cli` is pure stub. No conceptual subprocess interface.
**Solution:**
- Add `spawn_yosys_process(verilog_file)` --- conceptual subprocess spawn returning process handle.
- Add `wait_for_process(handle)` --- conceptual wait returning exit code.
- Add `run_yosys_subprocess(verilog_file)` --- end-to-end: spawn -> wait -> return report.
- Wire `score_rtl_with_real_yosys` to optionally call subprocess path.
- Add tests for subprocess conceptual interface.

---

## Verification Checklist

- [ ] cargo build --release (0 errors)
- [ ] ./target/release/t27c suite --repo-root . (562/562 PASS)
- [ ] cargo clippy --workspace --all-features (0 warnings)
- [ ] ./target/release/t27c lint --ascii for all modified specs (clean)
- [ ] Regenerate seals for all modified specs
- [ ] Write WAVE_LOOP_103_REPORT.md
- [ ] Write WAVE_LOOP_103_COOPERATION.md
- [ ] Increment .commit_count

---

phi^2 + 1/phi^2 = 3 | TRINITY
