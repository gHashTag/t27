# Plan: Wave Loop 379 — IGLA CODER+RACE + semantically-aware `let` destructuring

**Date:** 2026-07-03  
**Issue:** #1269  
**Branch:** `trinity-rust-rings`  
**Basis:** W378 close-out report and W378 cooperation Variant B

---

## 1. Goal

Extend the IGLA CODER+RACE zero-failure streak to **113 waves**, push the Lean 4 generic ∀ lattice to **260**, and harden the W378 `let` destructuring workaround so it is **semantically aware** (binding count and slot width inferred from the LHS pattern, not hardcoded 3×32-bit). Keep the QMTech Wukong V1 / DLC10 bitstream path ready.

## 2. Target metrics

| Metric | W378 | W379 | Δ |
|---|---|---|---|
| Lean 4 generic ∀ | 256 | **260** | +4 |
| Pool A floor | 122 | **123** | +1 |
| CODER minimum | 112 | **113** | +1 |
| Pool B depth (`systolic_ternary`) | 140 | **141** | +1 |
| Integration depth (`ternary_inference`) | 121 | **122** | +1 |
| Full-repo tests | 13,138 | **13,193** | +55 |
| Full-repo invariants | 5,769 | **5,796** | +27 |
| Conformance specs | 558 | **559** | +1 (scratch) |
| Conformance pass rate | 558/558 | **559/559** | 100% |
| Gen-verilog yosys smoke targets | 38 | **38** | stable (full IGLA already) |
| Zero-IGLA-failure streak | 112 waves | **113 waves** | +1 |

## 3. Issue landscape

- **#1268** — W378 (closed).
- **#1269** — W379 (created for this work).
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)`. Too broad for one wave; remains tracked but not primary.
- Older wave issues remain open only as tracking records until the `trinity-rust-rings` branch lands in `master`.

## 4. Scientific / competitive landscape

Key 2025–2026 work:

1. **Sparkle HDL / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)) — type-safe, formally verifiable HDL in Lean 4. Verified BitNet b1.58 accelerator (60+ theorems), RV32 SoC boots Linux 6.6.0 (102 proofs), JIT simulation faster than Verilator. Talk at **Functional Festival 2026 (July 11, 2026)** by Junji Hashimoto. Remains the only credible formal competitor; still **0 generic ∀** in the public repository.
2. **KU Leuven / MICAS ternary-lut-dse** ([arXiv:2604.25183](https://arxiv.org/abs/2604.25183), ISPASS 2026) — open-source Chisel generator for LUT-based 1.58-bit LLM inference. ASIC-oriented; **no formal verification**.
3. **shepherdscientific/ternarycore** ([GitHub](https://github.com/shepherdscientific/ternarycore)) — Verilog FPGA accelerator for BitNet b1.58, targeting Artix-7; simulation-only verification.
4. **Neumann-Labs/ternfpga** ([GitHub](https://github.com/Neumann-Labs/ternfpga)) — $130 Arty A7-35T ternary LLM decode engine; testbench verification.
5. **TerEffic / TeLLMe / TENET** — 2025 ternary LLM FPGA accelerators; simulation/benchmark verification.
6. **VitaLLM** ([arXiv:2605.00320v1](https://arxiv.org/abs/2605.00320v1)) — mixed-precision BitNet b1.58 ASIC, no formal proofs.

**Takeaway:** Sparkle HDL is the only active formal-verification competitor. W379 keeps the generic ∀ pressure at **260×** while closing the next backend semantic gap.

## 5. Decomposed work breakdown

### 5.1 IGLA spec batch (+55 tests, +27 invariants)

- Copy `scripts/gen_w378.py` → `scripts/gen_w379.py`.
- Update last-wave check from 378 → 379 and all placeholders.
- Run over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.
- Spot-check diff and run `t27c suite`.

### 5.2 Lean 4 proof-lattice extension (+4 generic ∀)

Copy `scripts/gen_w378_lean.py` → `scripts/gen_w379_lean.py`, then append:

1. `ternaryMacAccumulateFiftyFivePlusGeneric` — 55-variable plus accumulation. Watch elaboration time; fallback to 54-plus/53-minus if timeout.
2. `ternaryMacAccumulateFiftyFourMinusGeneric` — 54-variable minus lattice.
3. `ternaryMacDuotrigintupleCancellationGeneric` — depth-32 alternating plus/minus with residual `= x`.
4. `ternaryMacZeroWeightTrevigintupleClosureGeneric` — 13 zero + 1 plus + 13 zero (38th proof-lattice dimension).

### 5.3 gen-verilog `let` destructuring — semantically aware

**Finding from W378:** the syntax-level fix hardcodes a 96-bit packed temporary and 32-bit slots. This works for `cordic_top` (3 × i32) but is not correct for arbitrary tuple sizes or bit widths.

**W379 improvement:**
- Detect the number of bindings `N` from the `let(...)` LHS.
- Detect the bit width of each binding from its declared type if available, otherwise default to 32.
- Emit a packed temporary whose total width is `sum(slot_widths)`.
- Declare scalar regs and slice assignments matching each binding's actual width.
- Add scratch specs for `let(a, b) = ...` (2 bindings) and `let(a, b, c, d) = ...` (4 bindings) to prove the generalization.
- Keep the smoke gate green for all 27 IGLA specs.

This is still a syntax-level workaround in the sense that the backend does not yet implement first-class tuple-return types, but it is now **semantically correct for any tuple shape** that the parser exposes through the `let(...)` call pattern.

### 5.4 CI smoke gate

- Keep the 38-target gate unchanged (full IGLA already achieved in W378).
- Optionally add a synthetic test that exercises the generalized `let` destructuring paths via the new scratch specs.

### 5.5 Seal regeneration and verification

- Build `t27c` release after compiler change.
- Run `t27c suite --repo-root .`; expect seal mismatches.
- Capture mismatch list and batch reseal.
- Run suite again until 0 failures.

### 5.6 Documentation and learnings

- Write `docs/reports/WAVE_LOOP_379_REPORT.md`.
- Write `docs/reports/WAVE_LOOP_379_COOPERATION.md` (three variants for W380).
- Write `docs/reports/FPGA_EVIDENCE_W379.md`.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` if the let-destructuring status changes.
- Update `.trinity/experience.md`.
- Save memory file and update `MEMORY.md`.

## 6. Risk and fallback

- **55-variable theorem** may push Lean elaboration >30 s. Fallback: 54-plus/53-minus, accepting **259 generic ∀**.
- **Semantically-aware `let` destructuring** may require type information that is not yet attached to `StmtAssign` LHS identifiers. Fallback: keep the current hardcoded 3×32-bit fix for the IGLA path and generalize only for the new scratch specs.

## 7. Variant rationale

Selected **Variant B** from W378 cooperation: keep the proof-lattice pressure on Sparkle HDL while hardening the most recent backend fix. This avoids the strategic stall of Variant C and the one-dimensional risk of Variant A.

---

*phi² + 1/phi² = 3 | TRINITY*
