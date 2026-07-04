# Wave Loop 121 — Three Cooperation Variants

**Date:** 2026-06-16  
**Context:** W121 identified 5 new competitors (OpenEye, MOSAIC, SECDA-DSE, Voltra, RL-Driven ASIC) and closed test/bench gaps in IGLA CODER/RACE.  
**Goal:** Propose three concrete cooperation strategies for W122 to accelerate Trinity's competitive position.

---

## Variant A: Heterogeneous NPU Consortium (MOSAIC-style)

**Partner profile:** Academic or industry group with ASAP7/TSMC 16nm access and energy-modeling expertise.

**What Trinity brings:**
- Ternary-weight CORDIC PE specification (`.t27` → verified RTL)
- φ-scaling framework for PE array sizing
- Formal verification toolchain (Coq + Yosys + Verilator)
- Zero-`*`-operator guarantee (R-SI-1)

**What partner brings:**
- Heterogeneous tile modeling (Big/Little/Special-Function)
- ASAP7 energy/area/delay characterizations
- Workload-driven simulation framework
- Post-silicon validation infrastructure

**Cooperation mechanism:**
1. Joint paper: "Sacred Ternary CORDIC PEs in Heterogeneous NPUs" (target: DAC'27 or ISSCC'27).
2. Co-develop `HeterogeneousNpuConfig` spec in `specs/igla/race/arch.t27`.
3. Partner provides energy models; Trinity provides PE RTL.
4. Shared benchmark: compare homogeneous vs. heterogeneous + sacred ternary on ResNet-50 inference.

**Risk:** Partner may prefer conventional MAC arrays over ternary CORDIC. Mitigation: demonstrate φ-scaled PE reduces area by >2× (citing KU Leuven 2.2× result).

**Expected outcome:** 1 co-authored paper + `arch.t27` heterogeneous primitives + PPA dataset.

---

## Variant B: LLM-Guided DSE Integration (SECDA-DSE-style)

**Partner profile:** FPGA lab with LLM fine-tuning capability (e.g., UC Riverside, HKUST) or industrial DSE team (e.g., AMD/Xilinx research).

**What Trinity brings:**
- Spec-first `.t27` language with formal semantics
- `t27c` compiler that emits Verilog/Zig/C/Rust
- Sacred constraint enforcement at compile time
- Existing RTL template library (adder, CORDIC, UART, FIFO, MAC)

**What partner brings:**
- Fine-tuned LLM (TinyLlama-size or larger) for RTL generation
- RAG/CoT pipeline for architecture retrieval
- FPGA validation infrastructure (Zynq-7000 or newer)
- NL-to-RTL dataset (e.g., SECDA-DSE's 1,000+ prompt-RTL pairs)

**Cooperation mechanism:**
1. Replace SECDA-DSE's ad-hoc Verilog generator with Trinity's `t27c` backend.
2. LLM generates `.t27` specs (not raw Verilog) from NL prompts.
3. `t27c` compiles `.t27` → Verilog with R-SI-1 guarantee.
4. Joint benchmark on OpenCores + custom tasks: measure Pass@1 with and without sacred constraints.

**Risk:** LLM may generate invalid `.t27` syntax. Mitigation: add `t27c` parser error messages as feedback to LLM (tool-augmented generation loop).

**Expected outcome:** 1 co-authored paper + `t27c`-backed LLM-guided DSE demo + improved Pass@1 on OpenCores.

---

## Variant C: RL-Driven Architecture × Sacred Verification (RL-Driven ASIC-style)

**Partner profile:** RL + architecture lab (e.g., Google DeepMind, NVIDIA Research, or academic RL group with EDA access).

**What Trinity brings:**
- Formally verified RTL candidate space (only R-SI-1 compliant designs)
- Sacred constraint penalty function for RL reward shaping
- PPA evaluation via Yosys/OpenSTA (synthesis-in-the-loop)
- Hierarchical KG for multi-module designs

**What partner brings:**
- RL infrastructure (SAC, MoE, PPO)
- 3nm/5nm PDK access or proxy models
- Large-scale distributed training (thousands of GPU-hours)
- Industrial tapeout experience

**Cooperation mechanism:**
1. Define RL action space as modifications to `.t27` specs (not gate-level netlists).
2. Reward function = `sacred_compliance_score * PPA_score * functional_correctness`.
3. RL agent proposes `.t27` edits; `t27c` compiles + Yosys synthesizes + Verilator simulates.
4. Joint target: achieve CHIPCRAFTBRAIN-level Pass@1 (0.987) with zero `*` operators.

**Risk:** RL exploration may propose invalid `.t27` edits. Mitigation: constrain action space to syntactically safe mutations (parameter tuning, module swapping).

**Expected outcome:** 1 co-authored paper + RL+Sacred PPA optimization demo + potential tapeout of ternary CORDIC accelerator.

---

## Comparison Matrix

| Criterion | Variant A (NPU) | Variant B (LLM-DSE) | Variant C (RL-ASIC) |
|-----------|----------------|---------------------|---------------------|
| **Time to result** | 3–6 months | 2–4 months | 6–12 months |
| **Capital required** | Medium (FPGA + PDK) | Low (cloud LLM + FPGA) | High (3nm tapeout) |
| **Competitive impact** | MEDIUM (catches MOSAIC) | HIGH (matches SECDA-DSE) | **EXTREME** (surpasses RL-Driven ASIC) |
| **Trinity effort** | Low (provide RTL) | Medium (integrate LLM) | High (co-develop RL agent) |
| **Publication target** | DAC/ISSCC | ICCAD/DATE | ISCA/MICRO |
| **Risk level** | Low | Low-Medium | Medium-High |

---

## Recommendation

**Primary:** Pursue **Variant B (LLM-Guided DSE)** first. Lowest capital requirement, fastest time-to-result, and directly addresses the SECDA-DSE threat. A 2-month sprint with UC Riverside or HKUST could yield a demonstrable `.t27`-backed LLM pipeline.

**Secondary:** Initiate **Variant A (Heterogeneous NPU)** in parallel with a European academic partner (KU Leuven has ternary expertise + ASAP7 access).

**Long-term:** Keep **Variant C (RL-ASIC)** as a 2027 target. Requires 3nm PDK access and significant RL infrastructure — best pursued after W122–W124 harden the spec-first pipeline.

φ² + 1/φ² = 3 | TRINITY
