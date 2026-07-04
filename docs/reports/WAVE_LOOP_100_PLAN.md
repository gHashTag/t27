# Wave Loop 100 -- Execution Plan

**Date:** 2026-06-16  
**Branch:** trinity-rust-rings  
**Focus:** IGLA CODER x IGLA RACE  
**Constraint:** STRICTLY IGLA CODER and IGLA RACE only  
**Goal:** Build the foundation blocks (dataset, tokenizer, weights, pipeline) without which a working coder model is impossible.

---

## Weak Points Identified (W100 OBSERVE)

### Critical Missing Blocks (no code exists at all)
1. **Dataset builder** -- no spec generates `(prompt, RTL)` pairs from templates or external corpora.
2. **Tokenizer** -- no `tokenize()`, `detokenize()`, vocab file loader, or BPE/WordPiece anywhere in `igla/coder/`.
3. **Checkpoint / weight file loader** -- no `.safetensors`, `.gguf`, or custom parser. `forward()` hardcodes a 4-element dummy array.
4. **End-to-end inference loop** -- `generate_verilog` is keyword dispatch. It never calls `forward()`, `generate_next_token_unified`, or any model inference.

### Research Context
- **OpenRTLSet** (arXiv:2606.10285v1): 131K Verilog modules, DeepSeek-R1 generated prompts, open-source. Standard for RTL dataset scale.
- **VerilogDB** (arXiv:2507.13369): 20K synthesizable modules, Yosys + Icarus validation pipeline. Proves synthesis-aware curation is mandatory.
- **CraftRTL** (ICLR 2025): 80K synthetic examples + 28.5K correct-by-construction (Karnaugh maps, FSMs, waveforms). Self-Instruct + OSS-Instruct adapted for Verilog.
- **Tokenizer insight**: Standard BPE fragments Verilog keywords (`endmodule`, `posedge`) into subwords, destroying syntax. Need AST-aware or keyword-preserved vocab.
- **Weight loading**: `ggml-rs` (pure Rust GGUF), `mlmf` (multi-format), `llama_gguf` (full engine) are mature Rust crates. Trinity can target GGUF or a custom minimal format.
- **E2E pipeline**: Ray Data LLM documents `Prompt --> Tokenize --> Forward --> Detokenize --> Postprocess`. IGLA CODER needs the same 5-stage flow.

---

## Track A: Dataset Builder (`specs/igla/coder/dataset.t27`) [DONE]

**Goal:** Generate `(prompt, RTL)` pairs from Trinity templates + augmentation.

### Deliverables
1. `DataSample` type -- `prompt: string`, `rtl: string`, `template: string`
2. `generate_prompt(template_name: string) --> string` -- returns natural language prompt for a known template
3. `generate_dataset(templates: []string) --> []DataSample` -- creates pairs for each template
4. `augment_prompt(prompt: string) --> string` -- adds prefixes like "generate a", "build", "design"
5. `count_dataset_samples(dataset: []DataSample) --> u32` -- recursive count
6. 9 tests + 1 invariant

### Verification
- [x] `t27c suite` passes
- [x] Seal generated
- [x] L3 ASCII clean

---

## Track B: Tokenizer (`specs/igla/coder/tokenizer.t27`) [DONE]

**Goal:** Vocab + encode/decode stubs for Verilog/t27 syntax.

### Deliverables
1. `VocabEntry` struct -- `token: string`, `id: u32`
2. `tokenize(text: string) --> []u32` -- conceptual character-level encoder (deterministic stub)
3. `detokenize(tokens: []u32) --> string` -- conceptual decoder stub
4. `vocab_size() --> u32` -- returns hardcoded vocab size
5. `encode_char(c: u8) --> u32` -- maps ASCII to ID
6. `decode_char(id: u32) --> u8` -- maps ID back to ASCII
7. 6 tests + 1 invariant

### Verification
- [x] `t27c suite` passes
- [x] Seal generated
- [x] L3 ASCII clean

---

## Track C: Weight Loading (`specs/igla/coder/weights.t27`) [DONE]

**Goal:** Checkpoint format stub + tensor-to-WeightBank loader.

### Deliverables
1. `CheckpointHeader` struct -- `magic: u32`, `tensor_count: u32`, `version: u32`
2. `load_weights_from_file(path: string) --> WeightBank` -- conceptual file loader stub
3. `parse_checkpoint_header(data: []u8) --> CheckpointHeader` -- conceptual parser
4. `tensor_to_weight_bank(data: []f32, rows: u32, cols: u32) --> WeightBank` -- flat f32 array --> BRAM format
5. `is_valid_checkpoint(path: string) --> bool` -- heuristic: non-empty path + has ".ckpt" or ".gguf"
6. 7 tests + 1 invariant

### Verification
- [x] `t27c suite` passes
- [x] Seal generated
- [x] L3 ASCII clean

---

## Track D: End-to-End Pipeline (`specs/igla/coder/pipeline.t27`) [DONE]

**Goal:** `generate_verilog_ai` that conceptually runs `tokenize --> forward --> detokenize`.

### Deliverables
1. `PipelineConfig` struct -- `max_tokens: u32`, `temperature: f32`, `top_p: f32`
2. `generate_verilog_ai(prompt: string, bank: WeightBank, cfg: PipelineConfig) --> string` -- 5-stage pipeline stub
3. `tokenize_prompt(prompt: string) --> []u32` -- calls tokenizer
4. `run_forward(tokens: []u32, bank: WeightBank) --> []f32` -- conceptual forward pass
5. `decode_logits(logits: []f32) --> string` -- conceptual detokenize
6. `fallback_to_template(prompt: string) --> string` -- fallback to existing keyword dispatch
7. 8 tests + 1 invariant

### Verification
- [x] `t27c suite` passes
- [x] Seal generated
- [x] L3 ASCII clean

---

## Global Verification Results

- [x] `cargo build --release` OK
- [x] `t27c suite --repo-root .` 562/562 PASS (+4 new specs)
- [x] `cargo clippy --workspace --all-features` 0 warnings
- [x] `t27c lint --ascii` clean for all new files
- [x] All seals generated, 0 mismatches
- [x] Report: `docs/reports/WAVE_LOOP_100_REPORT.md`
- [x] Cooperation: `docs/reports/WAVE_LOOP_100_COOPERATION.md`

phi^2 + 1/phi^2 = 3 | TRINITY
