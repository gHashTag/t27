# Wave Loop 100 -- Cooperation Variants for Next Loop

**Date:** 2026-06-16  
**Context:** IGLA CODER x IGLA RACE  
**Purpose:** Three partnership strategies to advance from foundation (W100) to a working coder model

---

## Variant 1: RTL Dataset Curation Partnership (OpenRTLSet / VerilogDB)

**What:** Partner with the authors of OpenRTLSet (arXiv:2606.10285v1, 131K modules) or VerilogDB (arXiv:2507.13369, 20K synthesizable modules) to co-curate a sacred-opcode-labeled RTL dataset.

**Research backing:**
- **OpenRTLSet** uses DeepSeek-R1 70B to generate high-quality NL descriptions for 131K Verilog modules. It is the largest fully open-source RTL dataset.
- **VerilogDB** includes synthesis-aware validation (Yosys + Icarus), ensuring physical realizability.
- **CraftRTL** (ICLR 2025) proves that synthetic data augmentation (Self-Instruct, OSS-Instruct) adapted for Verilog improves pass@k by 3.8-10.9%.

**Value exchange:**
- Trinity provides: R-SI-1 compliance checker (`check_sacred_compliance`), ternary-weight template library (8 verified modules), and phi-based scaling annotations.
- Partner provides: raw Verilog corpus, DeepSeek-R1 prompt generation pipeline, synthesis validation infrastructure.

**Deliverables for W101:**
1. 10,000 (prompt, RTL) pairs labeled with `sacred_compliant: bool` and `template_tag: string`.
2. Synthesis validation report: % of dataset that passes Yosys synthesis with 0 problems.
3. Data augmentation pipeline: template interpolation + OpenROAD mutation + Self-Instruct for Verilog.

**Why this is optimal:** Dataset is the #1 blocker. Without 10K+ samples, training any model is futile. Partnering with existing dataset authors accelerates curation by 10x vs. building from scratch.

---

## Variant 2: Training Infrastructure Partnership (GPU Cloud + SFT/RLHF)

**What:** Partner with a GPU cloud provider (Lambda Labs, CoreWeave, or academic cluster) to train the first IGLA CODER checkpoint on the sacred-opcode dataset.

**Research backing:**
- **Veritas** (Roy et al., NYU, May 2025) fine-tunes Llama-3.2-3B for CNF-to-Verilog with pass@1 = 1 on tested components.
- **StepPRM-RTL** (IBM Research, June 2026) uses Qwen3-8B with stepwise PRM + MCTS for long-horizon RTL reasoning.
- **Finding:** Even "lightweight" RTL models start at 2B-3B parameters for general competence. However, Trinity's sacred-opcode constraint narrows the output vocabulary dramatically, potentially allowing a sub-1B model to outperform a general 3B model on R-SI-1 generation.

**Value exchange:**
- Trinity provides: curated 10K+ sacred-opcode dataset, tokenizer spec, ternary weight encoding, evaluation harness (pass@k on 50-task RTL benchmark).
- Partner provides: 8x A100 cluster for 2-4 weeks, SFT + RLHF expertise, PRM training infrastructure.

**Deliverables for W101:**
1. First 500M-parameter IGLA CODER checkpoint fine-tuned on sacred-opcode corpus.
2. Pass@1 benchmark on 50-task RTL generation suite: target >= 0.40.
3. GGUF export of the checkpoint for Rust runtime loading via `load_weights_from_file`.

**Why this is optimal:** Without a trained checkpoint, IGLA CODER is architecture porn -- beautiful specs with no executable product. A 500M model trained exclusively on R-SI-1 data creates an uncloneable moat.

---

## Variant 3: Rust ML Runtime Integration (Compiler Backend)

**What:** Contract a Rust ML engineer to integrate `ggml-rs`, `candle`, or `llama_gguf` into the t27c runtime, replacing pipeline stubs with real inference.

**Research backing:**
- **ggml-rs** (nktkt): Pure Rust GGUF inference with zero-copy tensor access, supports Q4_0, Q4_1, Q8_0, K-quants.
- **mlmf** (CireSnave): Multi-format model loader (SafeTensors, GGUF, ONNX, PyTorch) with memory-mapped loading.
- **llama_gguf** (llama-rs): Full LLM inference engine with batched inference, built-in tokenizer, and sampling strategies.
- **Ray Data LLM**: Documents production `Prompt --> Tokenize --> Forward --> Detokenize` pipeline with disaggregated CPU/GPU scaling.

**Value exchange:**
- Trinity provides: t27 spec pipeline (`generate_verilog_ai`), `WeightBank` BRAM abstraction, tokenizer spec, sacred-opcode constraints.
- Partner provides: Rust ML runtime integration, GGUF parser, CUDA kernels for attention/feed-forward, batched inference.

**Deliverables for W101:**
1. `run_forward(tokens, bank)` calls actual GGUF model loaded from file (not hardcoded logits).
2. `load_weights_from_file(path)` parses GGUF header and populates `WeightBank` tensors.
3. `tokenize_prompt(prompt)` delegates to SentencePiece/GPT-2 BPE tokenizer loaded from GGUF metadata.
4. End-to-end latency benchmark: prompt --> Verilog module in < 500ms on single A100.

**Why this is optimal:** Even with a trained checkpoint, the model is useless if `generate_verilog_ai` returns stub strings. Runtime integration is the bridge from spec to product. It also unlocks immediate testing of any checkpoint without rewriting specs.

---

## Risk / Mitigation Matrix

| Variant | Biggest risk | Mitigation |
|---------|-------------|------------|
| 1 Dataset | Partner has licensing restrictions | Focus on fully open-source datasets (OpenRTLSet); generate synthetic data as fallback |
| 2 Training | 500M model underfits RTL semantics | Data augmentation x10; evaluate 1B if 500M fails; use Qwen2.5-0.5B as absolute minimum |
| 3 Runtime | ggml-rs API mismatch with t27c types | Scope to `[]f32` logits + `[]u32` tokens only; defer exotic quantization to W102 |

---

## Recommended priority for W101

1. **Immediate:** Variant 1 (Dataset) -- no training without data. Dataset curation runs in parallel with other tracks.
2. **Parallel:** Variant 3 (Runtime) -- integrate GGUF loader into Rust runtime so pipeline spec is ready for real inference.
3. **Stretch:** Variant 2 (Training) -- start SFT once dataset > 10K samples and runtime integration is complete.

phi^2 + 1/phi^2 = 3 | TRINITY
