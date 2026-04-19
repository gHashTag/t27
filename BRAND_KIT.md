# TRIOS Project — Master Plan & Brand Kit
### Hybrid Quantization (GF16 + Ternary) · Repository: [gHashTag/trios](https://github.com/gHashTag/trios)

---

## 1. Project Overview

**TRIOS** is a research Rust project implementing a hybrid neural network quantization architecture where:

- **GoldenFloat16 (GF16)** is used for critical layers (embedding, attention, output) — the only 16-bit format achieving parity-level accuracy with f32 (97.67% on MNIST MLP, 0.00% gap).
- **Ternary {-1, 0, +1}** is used for high-workload bulk layers (mass-quantized) — under condition of specialized kernel path + QAT.
- **Hybrid policy engine** automatically selects per-layer format based on sensitivity profiling, hardware budget (DSP/LUT), and error budget.

**Architectural principle:** Every step = `tri CLI` + `experience/<Φn>` as source of truth in `.trinity` SSOT.

**Repository:** https://github.com/gHashTag/trios
**Technology stack:** Rust (core), TypeScript (UI), `.t27` custom language (policy/config), GitHub Actions (CI/CD)
**Repository rules:** No `.sh` scripts, only Rust/TypeScript/.t27; each PR must have linked issue + status checks; merge only after `tri verify experience/Φn = green`.

---

## 2. Success Metrics (Final Release)

Final metrics from `.trinity` release:

```
trinity_release:
  accuracy_mnist_mlp:       ">= 97.67%"    # priority with f32 / GF16 whitepaper
  accuracy_gap_vs_f32:      "<= 0.10%"
  memory_total_mb:          "<= 16.00"     # Parameter Golf
  dsp_utilization_xc7a100t: ">= 0.95"     # 15x16 = 240/240 DSP
  throughput_gops:          ">= 1000"      # HPPE reference
  jepa_collapse_detected:   false          # LeJEPA-style check
  coq_proofs_admitted:      0
  merkle_root:              "<blake3>"
  reproducibility:          "tri replay --from=experience/Φ0 == bit-exact"
```

---

## 3. Project Phases (Φ0 — Φ8)

### Φ-0 · Foundation & SSOT Schema
**Agent:** `trinity-architect` · **Branch:** `feat/foundation` · **CI:** `schema-check`

**Task:** Establish `.trinity` as the single source of truth for the entire project. Define base data types, configuration schema, structure of `experience/` directory.

**Deliverables:**
- `experience/Φ0/schema.trinity` — validated schema
- `crates/trios-core/src/types.rs` — base types: `LayerSpec`, `MemoryBudget`, `DspBudget`, `PrecisionFormat`
- `tri init` CLI command with `.trinity` scaffold generation
- GitHub Issue: `#Φ0-foundation`

**Readiness criteria:** `schema-check` CI green, `tri verify experience/Φ0 = green`

---

### Φ-1 · Precision Router (Mixed-Precision Policy Engine)
**Agent:** `precision-router` · **Branch:** `feat/precision-map` · **CI:** `budget-lint`

**Task:** Implement per-layer format distribution based on sensitivity, memory budget, and DSP budget.

**Deliverables:**
- `crates/precision-router/src/lib.rs`:
```rust
pub fn plan(layer: &LayerSpec, budget: MemoryBudget, dsp: DspBudget)
    -> Result<PrecisionFormat /* GF16 | Ternary158 */>
```
- `precision_policy.t27` — declarative policy on custom language (not Python)
- CLI command: `tri route plan --budget <mb> --dsp <n> --input <model.trinity> --out <plan.trinity>`
- Policy table (encoded in `.t27`):

| Layer | Sensitivity | DSP Cost | Assigned Format |
|--------|-------------|-------------------|
| Embedding | HIGH | 16 DSP/unit | GF16 |
| Attention | HIGH | 16 DSP/unit | GF16 |
| Output | HIGH | 16 DSP/unit | GF16 |
| FC/Linear (bulk) | LOW | 0 DSP (ternary) | Ternary158 |
| Activations | MEDIUM | — | per-layer policy |

**Readiness criteria:** `budget-lint` green; policy correctly routes all 8 layer types

---

### Φ-2 · GF16 Kernel (GoldenFloat16 Implementation)
**Agent:** `zig-gf-eng` · **Branch:** `feat/gf16-kernel` · **CI:** `bit-exact`, `bench >= 97.67%`

**Task:** Implement complete GF16 arithmetic stack — encoding/decoding, MAC-16 operations, FPGA mapping on DSP48.

**Key facts from whitepaper:**
- GF16 is the only 16-bit format with 0.00% gap vs f32 on MNIST MLP
- BF16 and naive ternary degrade to 9.80% — catastrophic failure
- Hardware trade-off: 118 LUT/unit (vs 2 LUT ternary) → ratio 59×; but on MAC-16 level: 71 LUT vs 52 LUT → ratio 1.37× (DSP dominates)
- DSP bottleneck XC7A100T (240 DSP): GF16 fits ~15 parallel MAC-16 (16 DSP each) vs ~1219 for ternary (0 DSP)

**Deliverables:**
- `crates/trios-golden-float/` — full Rust GF16 implementation
  - `encode(f32) -> GF16`
  - `decode(GF16) -> f32`
  - `mac16(GF16, GF16) -> GF16` (fused multiply-accumulate)
- Benchmark suite: accuracy ≥ 97.67% on MNIST MLP, bit-exact reproducibility
- FPGA utilization report for XC7A100T
- CLI: `tri bench gf16 --dataset mnist --model mlp`

**Hardware table:**

| Metric | Ternary | GF16 | Ratio |
|---------|---------|------|-------|
| Unit LUT | 2 LUT | 118 LUT | 59× |
| MAC-16 LUT | 52 LUT | 71 LUT | 1.37× |
| DSP/unit | 0 | 16 | — |
| Max parallel MAC (XC7A100T) | ~1219 | ~15 | — |

**Readiness criteria:** `bit-exact` CI + `bench >= 97.67%`

---

### Φ-3 · Ternary BitLinear Engine
**Agent:** `ternary-eng` · **Branch:** `feat/ternary-bitlinear` · **CI:** `QAT gap <= 2%`

**Task:** Implement ternary quantization with QAT (Quantization-Aware Training), specialized ternary matmul kernel, and routing policy. Naive ternary (without QAT/co-design) is unacceptable — see 9.80% failure.

**Deliverables:**
- `crates/trios-ternary/` — Ternary BitLinear implementation
  - Ternary-specific matmul kernel (compute-memory alignment)
  - Sensitivity-aware projection scheme
  - QAT training loop integration
  - `ternary_routing_policy.t27` — rules: when layer gets ternary vs GF16
  - Mixed-precision GEMM (mpGEMM) — key bottleneck for LLM inference
  - Benchmark: training-aware + inference benchmark (not just inference!)
  - CLI: `tri bench ternary --qat --dataset mnist`

**Requirements from literature:**
- TerEffic-style: compression + custom ternary matmul unit + compute-memory alignment
- BitNet.cpp pattern: specialized mpGEMM library for lossless sub-2-bit inference
- Per-layer/per-tensor bit allocation better than uniform quantization (MixQuant)

**Readiness criteria:** QAT gap ≤ 2% from f32 baseline

---

### Φ-4 · Hardware Scheduler (DSP/FPGA Resource Planner)
**Agent:** `hw-scheduler` · **Branch:** `feat/dsp-plan` · **CI:** `roofline-report`

**Task:** Resource-aware planner for FPGA deployment. Not just accuracy-aware, but also utilization prediction until full build.

**Deliverables:**
- `crates/trios-hw-scheduler/` — FPGA resource estimator
  - Roofline model for XC7A100T and other targets
  - DSP/LUT utilization predictor
  - Hessian-driven / row-wise mixed-precision bit allocation for transformer acceleration
  - Compiler-level optimization: IR selection + operator fusion (minimizing quantize/dequantize overhead — like QuantuneV2)
  - Format transition optimizer: GF16 ↔ Ternary transitions optimized (~50% penalty reduction)
  - CLI: `tri hw plan --target xc7a100t --model <plan.trinity> --out <hw_plan.trinity>`

**Readiness criteria:** `roofline-report` CI — report with predicted vs actual utilization

---

### Φ-5 · Phi-Attention (φ-based Sparse Attention)
**Agent:** `attn-eng` · **Branch:** `feat/phi-attn` · **CI:** `sparsity = 2.15 ± 0.05%`

**Task:** Implement φ (golden ratio) based sparse attention mechanism for transformer architectures in trios.

**Deliverables:**
- `crates/trios-phi-attn/` — φ-attention implementation
  - Sparse attention with golden-ratio sparsity pattern
  - Integration with precision router (GF16 for attention layers)
  - Sparsity validation: 2.15 ± 0.05%
  - Benchmark vs dense attention: throughput, memory, accuracy
  - CLI: `tri attn benchmark --sparsity phi`

**Readiness criteria:** `sparsity = 2.15 ± 0.05%` CI check

---

### Φ-6 · JEPA Trainer (Joint-Embedding Predictive Architecture)
**Agent:** `jepa-trainer` · **Branch:** `feat/jepa-t-loop` · **CI:** `no-collapse-check`

**Task:** Implement JEPA (LeJEPA-style) training loop with anti-collapse mechanism for quantized models. QAT is critical here — without it, ternary is fundamentally weak in production pipeline.

**Deliverables:**
- `crates/trios-jepa/` — JEPA training loop
  - Online + target encoder architecture
  - Anti-collapse regularization (LeJEPA-style)
  - QAT integration for ternary layers
  - Training-aware benchmarking
  - `experience/Φ6/jepa_config.trinity` — training configuration
  - CLI: `tri train jepa --config experience/Φ6/jepa_config.trinity`

**Readiness criteria:** `no-collapse-check` CI — encoder doesn't collapse

---

### Φ-7 · Formal Proofs (Coq Verification)
**Agent:** `coq-prover` · **Branch:** `feat/formal-proofs` · **CI:** `Qed, no Admitted`

**Task:** Formal verification of key algorithms in Coq. Null admission on `Admitted` is required.

**Deliverables:**
- `proofs/` — Coq file directory
  - `GF16Correctness.v` — proof of bit-exact encoding
  - `TernaryBoundedness.v` — proof of bounded quantization error
  - `PolicyMonotonicity.v` — monotonicity of precision router policy
  - `MixedPrecisionSoundness.v` — correctness of hybrid format switching
  - Integration with `tri verify` CLI
  - Merkle tree hash of all provers for reproducibility bundle

**Readiness criteria:** All `Qed`, 0 `Admitted`, CI green

---

### Φ-8 · Publication Artifact (Zenodo + NeurIPS/DARPA CLARA)
**Agent:** `paper-builder` · **Branch:** `release/igla-v1` · **CI:** `zenodo-dry-run`

**Task:** Prepare complete publication bundle for Zenodo + academic conference submissions.

**Due date:** Day 10

**Deliverables:**
- Zenodo deposit: `tri pack --zenodo` — collects all `.trinity` into merkle-root bundle
  - Reproducibility bundle: bit-exact reproducibility through `tri replay --from=experience/Φ0`
  - Article NeurIPS 2026 format: `tri paper build --venue=neurips26`
  - Article DARPA CLARA format: `tri paper build --venue=clara-darpa`
  - `experience/Φ8/release.trinity` + DOI
  - DOI registration via Zenodo

**Readiness criteria:** `zenodo-dry-run` CI green; all metrics from §2 confirmed

---

## 4. Timeline & Priorities

| Priority | Phase | Dependencies | Criticality |
|-----------|-------|-------------|-------------|
| **P0** | Φ0 Foundation | — | Blocks everything |
| **P1** | Φ1 Precision Router | Φ0 | Blocks Φ2, Φ3, Φ4 |
| **P0** | Φ2 GF16 Kernel | Φ1 | Core innovation |
| **P1** | Φ3 Ternary Engine | Φ1 | Hybrid stack |
| **P1** | Φ4 HW Scheduler | Φ2, Φ3 | FPGA deployment |
| **P2** | Φ5 Phi-Attention | Φ2 | Transformer support |
| **P2** | Φ6 JEPA Trainer | Φ3 | QAT validation |
| **P3** | Φ7 Formal Proofs | Φ2, Φ3 | Academic credibility |
| **P3** | Φ8 Publication | all | Output |

**Note on priorities:**
- After Φ3+Φ4, trios becomes a **proper orchestration engine**, not just a collection of experiments (PRIORITY 2 — Phase 3+4)
- Φ6 (QAT) resolves the fundamental question: can ternary survive in production pipeline (PRIORITY 3 — Phase 6)

---

## 5. RACI Matrix (Agents)

| Agent | Role | Phases | Branch | CI Check |
|-------|------|-------|--------|----------|
| `trinity-architect` | R | Φ0 | `feat/foundation` | `schema-check` |
| `precision-router` | R | Φ1 | `feat/precision-map` | `budget-lint` |
| `zig-gf-eng` | R | Φ2 | `feat/gf16-kernel` | `bit-exact, bench≥97.67%` |
| `ternary-eng` | R | Φ3 | `feat/ternary-bitlinear` | `QAT gap ≤ 2%` |
| `hw-scheduler` | R | Φ4 | `feat/dsp-plan` | `roofline-report` |
| `attn-eng` | R | Φ5 | `feat/phi-attn` | `sparsity=2.15±0.05%` |
| `jepa-trainer` | R | Φ6 | `feat/jepa-t-loop` | `no-collapse-check` |
| `coq-prover` | R | Φ7 | `feat/formal-proofs` | `Qed, no Admitted` |
| `paper-builder` | R | Φ8 | `release/igla-v1` | `zenodo-dry-run` |
| **General** (you) | **A** | all | `main` | branch protection |

**Agent rules:**
- Every PR → linked issue → required status checks → merge only after `tri verify experience/Φn = green`
- No `.sh` files — only Rust / TypeScript / `.t27`
- No `.sh` — this is non-negotiable

---

## 6. Architecture CLI (`tri`)

```
tri init                              # Scaffold .trinity SSOT
tri route plan --budget --dsp --input --out
tri bench gf16 --dataset mnist --model mlp
tri bench ternary --qat --dataset mnist
tri hw plan --target xc7a100t --model --out
tri attn benchmark --sparsity phi
tri train jepa --config experience/Φ6/jepa_config.trinity
tri verify experience/Φn             # → green/red
tri pack --zenodo                     # Merkle-root reproducibility bundle
tri replay --from=experience/Φ0      # Bit-exact replay
tri paper build --venue=neurips26
tri paper build --venue=clara-darpa
```

---

## 7. Repository Structure

```
trios/
├── crates/
│   ├── trios-core/            # Base types (Φ0)
│   ├── precision-router/      # Policy engine (Φ1)
│   ├── trios-golden-float/    # GF16 kernel (Φ2) ← active development
│   ├── trios-ternary/         # Ternary BitLinear (Φ3)
│   ├── trios-hw-scheduler/    # FPGA planner (Φ4)
│   ├── trios-phi-attn/        # φ-attention (Φ5)
│   ├── trios-jepa/            # JEPA trainer (Φ6)
│   └── trios-kg/              # Knowledge graph (WIP)
├── proofs/                    # Coq formal proofs (Φ7)
├── experience/
│   ├── Φ0/ → Φ8/             # Per-phase .trinity artifacts
│   └── release.trinity        # Final release artifact
├── policies/
│   ├── precision_policy.t27   # Declarative precision routing
│   └── ternary_routing_policy.t27
├── tri/                       # CLI binary (Rust)
└── .trinity                   # Root SSOT config
```

**Current status crates (at last review):**
- 5 TRIOS crates → **GREEN** ✅
- `trios-golden-float` → **active development** 🔄
- `trios-kg` → WIP / untracked 📋

---

## 8. Scientific Context & Justification

### GF16 vs BF16 vs Naive Ternary

| Format | MNIST MLP Accuracy | Gap vs f32 | Status |
|---------|-------------------|------------|--------|
| f32 | 97.67% | 0.00% | Reference |
| **GF16** | **97.67%** | **0.00%** | **✅ Priority** |
| BF16 | 9.80% | 87.87% | ❌ Catastrophic |
| Naive Ternary | 9.80% | 87.87% | ❌ Catastrophic |

### Why Hybrid (not just GF16 or Ternary)?

1. **GF16 — DSP-bottleneck:** 240 DSP on XC7A100T → only ~15 parallel MAC-16 (vs ~1219 for ternary). Cannot scale to all layers.
2. **Naive ternary fails without QAT:** Pure PTQ at low bitwidth insufficient — need QAT + sensitivity-aware projection.
3. **Mixed precision = standard:** activation layers / projection layers — most sensitive to quantization; bulk computation offloaded to low-bit.
4. **Format hopping overhead:** GF16 ↔ Ternary transitions cost ~50% loss → need compiler-level IR fusion (QuantuneV2 pattern).

### Related Research
- **TerEffic** — ternary as target mode for on-chip LLM inference on FPGA (needs specialized architecture)
- **BitNet.cpp** — mixed-precision GEMM as key bottleneck for lossless sub-2-bit inference
- **MixQuant** — per-layer/per-tensor bit allocation > uniform quantization (MixQuant)
- **LeJEPA** — anti-collapse mechanism for joint-embedding predictive architectures
- **Hessian-driven MPQ** — resource-aware quantization for transformer on FPGA

---

## 9. First Step for Agent `precision-router`

**Issue:** `#Φ1-precision-map`
**Branch:** `feat/precision-map`

**Next deliverables:**

1. Create file `crates/precision-router/src/lib.rs` with function:
```rust
pub fn plan(layer: &LayerSpec, budget: MemoryBudget, dsp: DspBudget)
    -> Result<PrecisionFormat /* GF16 | Ternary158 */>
```

2. Policy from Φ1 table, declaratively encoded in `precision_policy.t27` (custom language — not Python).

3. CLI command `tri route plan --budget --dsp --input --out`.

---

*Document generated: 2026-04-19 · Repository: https://github.com/gHashTag/trios*
