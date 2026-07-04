# Wave Loop 120 — Three Cooperation Variants
## IGLA CODER + IGLA RACE — Late-June 2026

---

## Executive Summary

Wave Loop 120 closed the backend realizability gap, expanded the weakest-tested IGLA RACE files, and tracked three new ternary/CORDIC competitors. The following three cooperation variants are proposed for Wave Loop 121, ordered by strategic priority.

---

## Variant 1: Backend Realizability Consortium (RECOMMENDED)

**Partner:** OpenROAD / Yosys maintainers + ChipBench authors
**Goal:** Integrate `compute_backend_realizability` into actual EDA toolchain wrappers and establish a joint synthesis→P&R→DRC→LVS evaluation pipeline.

**Why this fits Trinity:**
- Trinity now has `BackendRealizabilityScore` and `compute_backend_realizability()` but no runtime bridge to OpenROAD/Yosys.
- ChipBench requires industrial-grade backend verification (synthesis + P&R + DRC + LVS) for its 44 real-world IP modules.
- A unified consortium would give Trinity early access to ChipBench backend data and tooling.

**Trinity contribution:**
- `t27c` compiler infrastructure to emit synthesizable Verilog from `.t27` spec.
- Sacred-constraint hardwiring (R-SI-1) ensuring zero multipliers at source level — simplifies synthesis.
- `BackendRealizabilityScore` struct and scoring primitives as shared data model.

**Partner contribution:**
- OpenROAD/Yosys synthesis and P&R scripts.
- DRC/LVS checkers and pass/fail reporting.
- ChipBench module golden references for regression testing.

**Risk:** Low-Medium — both parties benefit from standardization; OpenROAD is already open-source. DRC/LVS may require proprietary PDK access (TSMC, SkyWater).

---

## Variant 2: Ternary Accelerator Benchmark Cooperative

**Partner:** TeLLMe authors (arXiv:2504.16266) + TernaryCore maintainers (GitHub shepherdscientific/ternarycore)
**Goal:** Establish a unified benchmark comparing ROM-based (TeLLMe), LUT-based (TernaryCore), and systolic/CORDIC-based (Trinity) ternary accelerators.

**Why this fits Trinity:**
- TeLLMe achieves 9.5 tok/s on AMD KV260; TernaryCore targets Artix-7; Trinity has ternary systolic array + CORDIC specs.
- No unified benchmark exists for ternary hardware accelerators — each paper uses different models (BitNet-2B, BitNet b1.58) and different metrics (tok/s, area, energy).
- Trinity's HQI metric (area + delay + warnings + LUTs) could be extended to include token throughput and energy per token.

**Trinity contribution:**
- `ternary_gemm.t27`, `cordic_fixed.t27`, `systolic_ternary.t27` specs as benchmark inputs.
- Yosys synthesis pipeline (`yosys_area_delay_product`, `ppa_score_from_report`).
- `compute_backend_realizability` for synthesis→layout pass rate.

**Partner contribution:**
- TeLLMe RTL and KV260 deployment data.
- TernaryCore Verilog and Artix-7 synthesis scripts.
- Standardized token-throughput and energy-efficiency measurement protocol.

**Risk:** Medium — competitive tension (all three are hardware generators); but benchmark standardization benefits the entire ternary ML community. Need to agree on fair comparison (same model size, same quantization scheme).

---

## Variant 3: CORDIC + Ternary Research Alliance

**Partner:** CORDIC-Is-All-You-Need authors (arXiv:2503.11685) + CARMEN authors (arXiv:2605.06878)
**Goal:** Explore hybrid CORDIC + ternary architectures combining SYCore's systolic CORDIC with Trinity's ternary-weight systolic array.

**Why this fits Trinity:**
- SYCore uses CORDIC PEs for DNNs/RNNs/Transformers but without ternary weights.
- CARMEN uses CORDIC for multi-precision inference but without systolic arrays.
- Trinity has both ternary weights and CORDIC — a natural bridge between the two.
- A research alliance could produce the first ternary-CORDIC-systolic paper, filling a gap in the literature.

**Trinity contribution:**
- Ternary weight encoding (`{-1, 0, +1}`) and Booth multiplier-free GEMM specs.
- CORDIC Q15 fixed-point sin/cos primitives (`cordic_fixed.t27`).
- φ-scaling mathematical framework for architecture optimization.

**Partner contribution:**
- SYCore's 5-stage pipelined CORDIC PE RTL.
- CARMEN's runtime-adaptive precision CORDIC MAC design.
- FPGA deployment expertise (Pynq-Z2, KV260, Artix-7).

**Risk:** High — research alliances require sustained collaboration; publication timelines are slow. However, the output (a joint paper on ternary-CORDIC-systolic architecture) would be a unique contribution with no direct competitors.

---

## Recommendation

**Pursue Variant 1 first.** Backend realizability is the highest-leverage, lowest-risk step: it addresses Trinity's biggest credibility gap (synthesis→layout pass rate), leverages existing OpenROAD/Yosys integration, and provides immediate value for ChipBench evaluation. Variants 2 and 3 should be explored as medium-term research collaborations.

φ² + 1/φ² = 3 | TRINITY
