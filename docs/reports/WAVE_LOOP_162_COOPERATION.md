# Wave Loop 162 — Cooperation Variants for Wave Loop 163

**Date:** 2026-06-16  
**Base Branch:** `trinity-rust-rings`

---

## Variant A — Silicon Demo Sprint (Recommended)

**Goal:** Close the VitaLLM silicon gap by producing a taped-out ternary MAC demo on low-cost FPGA within 4 weeks.

**Mechanism:**
- Partner with ternfpga for shared Arty A7-35T bitstream using Trinity-generated Verilog.
- Benchmark target: achieve <5 J/token on a 2-layer MLP inference task.
- Publish results as Zenodo technical note with explicit energy-per-token methodology.

**Benefit:** Gives Trinity immediate silicon credibility without ASIC tape-out cost.

---

## Variant B — Prior-Art Defense Pool

**Goal:** Preempt TIS `@sparseskip` patent claims by forming a defensive prior-art pool.

**Mechanism:**
- Trinity contributes sacred opcode documentation (0xD0–0xFF) dating before TIS patent filing.
- ternfpga contributes Phase 9 sparsity-aware bitstream logs.
- ternary-fabric contributes MLIR `tfmbs.sparse` dialect commits.
- Joint affidavit for patent examiners if challenged.

**Benefit:** Protects open ternary ecosystem from monopoly fragmentation.

---

## Variant C — Baroň Convergence Challenge

**Goal:** Invite Baroň to a public prediction duel on CKM/PMNS parameters with explicit error budgets.

**Mechanism:**
- Both parties submit predicted |V_us|, sin²θ₁₂, δ_CP central values + 1σ bands by W165.
- Independent referee (e.g., Krippendorf or Tooby-Smith) evaluates against PDG 2026 values.
- Loser publicly acknowledges methodological limitation in a shared arXiv comment.

**Benefit:** Converts escalating paper count into measurable falsifiability.

---

## Risk Mitigation

| Variant | Risk | Mitigation |
|---------|------|------------|
| A | FPGA demo fails energy target | Scope to systolic MAC only; publish negative results honestly |
| B | Prior art deemed insufficient | Commission external patent attorney review before submission |
| C | Baroň declines duel | Publish unilateral t27 prediction memo regardless |

---

φ² + 1/φ² = 3 | TRINITY
