# Wave Loop 109 — Three Cooperation Variants for Wave 110

**Date:** 2026-06-16
**Scope:** IGLA CODER + IGLA RACE
**Precedent:** `docs/reports/WAVE_LOOP_108_COOPERATION.md`

---

## Variant A: OpenRTLSet Dataset Collaboration (Recommended)

### Partner: OpenRTLSet Authors (arXiv:2606.10285v1)

**Goal:** Close the 100× dataset-scale gap by merging Trinity's sacred-constraint spec system with OpenRTLSet's 131K-module dataset.

**What Trinity offers (W109-ready):**
- `export_dataset_to_json` / `export_dataset_to_csv` — dataset exchange format (Track B).
- `sacred_compliance_axis_score` — continuous sacred metric for filtering OpenRTLSet samples (Track D).
- `generate_openrtlset_scale_dataset` — combinatorial expansion showing systematic augmentation methodology.
- R-SI-1 sacred invariant enforcement (zero `*` operators in RTL assignments).

**What partner offers:**
- 131,000 real RTL modules with permissive open-source licensing.
- Existing fine-tuning infrastructure (Qwen2.5-32B → 89.3% Pass@10).
- VerilogEval benchmark integration.

**Mechanism:**
1. Trinity authors write a joint technical paper: "Sacred Constraints in Large-Scale RTL Generation: A Spec-Driven Approach."
2. OpenRTLSet provides dataset; Trinity provides sacred filtering (`sacred_compliance_axis_score`) + formal reward bridge.
3. Joint submission to ICCAD 2026 or DAC 2027.
4. Revenue split: 0% (academic). Value = publication + citation + dataset access.

**Risk:** Medium. OpenRTLSet is fully open-source (GitHub), but no contact information in the arXiv preprint. Requires cold outreach via arXiv author emails.

**Why this matters:** Closes the single largest honest gap (dataset scale) without requiring Trinity to curate 131K modules manually. W109's export utilities make this technically feasible.

---

## Variant B: EDA Vendor Integration

### Partner: Yosys / Verilator / Icarus Maintainers or Commercial FPGA Vendor

**Goal:** Convert conceptual EDA subprocess stubs (`spawn_verilator_process`, `run_icarus_cli`) into real OS subprocess integration.

**What Trinity offers:**
- `verify_rtl_with_full_toolchain` — unified lint → synth → sim pipeline (Track A).
- `spawn_verilator_process`, `spawn_icarus_process` — conceptual subprocess stubs ready for real implementation.
- `synthesis_score` — normalized PPA scoring [0.0, 1.0] for benchmarking.
- Formal verification bridge (Coq/Lean proof-as-reward for generated RTL).

**What partner offers:**
- Existing Yosys/Verilator/Icarus cloud infrastructure.
- Real subprocess spawn APIs (Rust `std::process::Command` wrappers).
- Customer base of FPGA designers seeking automated RTL generation.

**Mechanism:**
1. Joint open-source crate: `trinity-rtl-eda` — Rust bindings for Yosys + Verilator + Icarus subprocess orchestration.
2. Trinity contributes pipeline architecture (`verify_rtl_with_full_toolchain`).
3. Partner contributes OS-level process management + EDA toolchain packaging.
4. Published as open-source crate on crates.io.

**Risk:** Low-Medium. All tools are open-source. Technical risk is moderate (subprocess I/O parsing). No revenue negotiations.

**Why this matters:** Turns the biggest conceptual gap (all EDA functions are stubs) into working infrastructure. This is prerequisite for any empirical Pass@K evaluation.

---

## Variant C: Benchmark Coalition — RTL-BenchLS v2 Axis

### Partner: RTL-BenchLS + VerilogEval + RTLLM Maintainers

**Goal:** Establish Trinity as the **benchmark integrity standard** for RTL generation by contributing evaluation infrastructure.

**What Trinity offers (W109-ready):**
- `trinity_rtl_eval_version()` + `benchmark_axis_supported()` — crate metadata (Track D).
- `sacred_compliance_axis_name()` + `sacred_compliance_axis_score(rtl)` — novel evaluation axis (Track D).
- Pass@K benchmark harness with SacreBLEU n-gram overlap (from W105).
- 564 spec-driven regression suite with deterministic codegen.

**What partners offer:**
- RTL-BenchLS: Novelty tasks (round-trip, repo issues, error localization).
- VerilogEval: Industry-standard machine + human evaluation.
- RTLLM: Synthesis + PPA evaluation framework.

**Mechanism:**
1. Trinity open-sources its benchmark harness as a standalone evaluation framework.
2. Proposes "Sacred Compliance (R-SI-1)" as a **mandatory evaluation axis** in RTL-BenchLS v2.
3. All RTL-BenchLS submissions must report `sacred_compliance_axis_score` alongside Pass@K.
4. Trinity authors get co-authorship on benchmark papers; no revenue, but influence + citations.

**Risk:** Low. All partners are open-source. No revenue negotiations. Benchmark code is already written; just needs packaging.

**Why this matters:** If Trinity defines the evaluation standard, competitors must measure themselves against our sacred constraint. This is **defensive positioning** — VeriAgent and HDLFORGE would need to report R-SI-1 compliance scores.

---

## Comparison Matrix

| Criterion | A — OpenRTLSet | B — EDA Vendor | C — Benchmark Coalition |
|-----------|---------------|----------------|-------------------------|
| **Time to value** | Medium (paper) | Low (crate) | Low (code exists) |
| **Capital required** | None | None | None |
| **Risk** | Medium | Low-Medium | Low |
| **Revenue** | None (citations) | None (open source) | None (influence) |
| **Closes dataset gap?** | ✅ Yes | ❌ No | ❌ No |
| **Closes toolchain gap?** | ❌ No | ✅ Yes | ❌ No |
| **Closes benchmark gap?** | ❌ No | ❌ No | ✅ Yes |
| **Recommended order** | **1st** | 2nd | 3rd |

---

## Decision Recommendation

Execute in parallel:
1. **Now (W110):** Variant B — start `trinity-rtl-eda` crate with real subprocess bindings for Yosys/Verilator/Icarus. This unblocks empirical evaluation.
2. **W110-W111:** Cold-email OpenRTLSet authors for dataset collaboration (Variant A — closes biggest honest gap).
3. **W111+:** Propose Sacred Compliance axis to RTL-BenchLS maintainers (Variant C — defensive positioning).

---

**phi² + 1/φ² = 3 | TRINITY**
