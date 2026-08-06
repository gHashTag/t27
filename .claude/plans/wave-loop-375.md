# Plan: Wave Loop 375 — IGLA CODER+RACE + safe `let` destructuring lowering + board flash retry

**Date:** 2026-07-03 (planned from W374 close-out)  
**Issue target:** #1264 and close with commit  
**Branch:** `trinity-rust-rings`  
**Recommended variant:** Variant B from `docs/reports/WAVE_LOOP_374_COOPERATION.md` (balanced/recommended). Implementation pivoted from `let` destructuring (Defect 6) to early-return if-else chaining (Defect 3) after discovering that Defect 6 is blocked by missing tuple-return function generation.

---

## 1. Goal

Extend the 34-wave zero-IGLA-failure streak by pushing the Lean 4 generic ∀ proof lattice to **244**, land the next narrow `gen-verilog` backend fix (`let` destructuring lowering), and keep the QMTech Wukong V1 / DLC10 bitstream path ready.

Target metrics:

- Generic ∀: **240 → 244** (+4)
- Pool A floor: **118 → 119**
- CODER minimum: **109 → 110**
- Pool B depth: **136 → 137**
- Integration depth: **117 → 118**
- Tests: **+54** (2 per IGLA spec)
- Invariants: **+27** (1 per IGLA spec)
- Conformance: **556/556 PASS**

---

## 2. GitHub issue landscape

Open issues studied (public API, 2026-07-03):

- **#1263** — Wave Loop 374 (just closed). Follow-up is **#1264**.
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs`. Too broad for one wave; keep as background.
- **#1243** / **#1244** — trios-mesh BPSK modem port. Lateral, not on the critical IGLA path.
- **#1219** — t27 Language Roadmap epic.
- **#1215** / **#1216** — GF10/GF256 conformance.

**Action:** create **#1264** titled *"Wave Loop 375 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix (`let` destructuring lowering)"* and reference it in the final commit with `Closes #1264`.

---

## 3. Scientific / competitive landscape

Key 2025–2026 work on formal/ternary hardware (searched 2026-07-03):

1. **Sparkle HDL / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)) — formally verifiable HDL in Lean 4 with a BitNet b1.58 inference accelerator, **60+ formal theorems**. This remains the strongest direct competitor in the same design space: Lean 4 + ternary/BitNet hardware + formal theorems. Trinity still leads in generic ∀ count (240 vs Sparkle's reported 60+).
2. **TorchLean** ([lean-dojo/TorchLean](https://github.com/lean-dojo/TorchLean), [arXiv:2602.22631](https://arxiv.org/abs/2602.22631)) — Lean 4 framework for NN verification with operator-tagged SSA IR, reverse-mode autograd theorem, and CROWN/LiRPA-style certificates. Software/proof focus, not ternary hardware.
3. **TerEffic** ([arXiv:2502.16473](https://arxiv.org/abs/2502.16473)) — highly efficient ternary LLM inference on AMD Alveo U280, fully on-chip and HBM-assisted variants; simulation/test verification.
4. **TeLLMe** ([arXiv:2504.16266](https://arxiv.org/abs/2504.16266)) — energy-efficient ternary LLM accelerator for prefilling and decoding on edge FPGAs (AMD KV260); 1.58-bit weights.
5. **KULeuven-MICAS/ternary-lut-dse** ([github.com/KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)) — Chisel generator for LUT-based ternary MatMul; simulation/testbench verification.
6. **shepherdscientific/ternarycore** ([github.com/shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore)) — open-source Verilog accelerator for BitNet b1.58; RTL simulation and Python cross-check.
7. **Trinity B002** (Zenodo 10.5281/zenodo.19224235) — 2026 defensive publication for zero-DSP ternary inference.

**Takeaway:** Sparkle HDL remains the only credible formal competitor with a BitNet b1.58 core. W375 widens the generic ∀ gap from **240× to 244×**. The strategic imperative is to keep the proof-lattice lead while the hardware evidence path remains blocked by the missing DLC10 cable.

---

## 4. Decomposed work breakdown

### 4.1 IGLA spec batch (+54 tests, +27 invariants)

- Copy `scripts/gen_w374.py` → `scripts/gen_w375.py`.
- Update last-wave check from 374 → 375 and all `w374_` / `W374` placeholders to `w375_` / `W375`.
- Run over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.
- Verify diff with `git diff --stat` and spot-check two specs.

### 4.2 Lean 4 proof-lattice extension (+4 generic ∀)

Copy `scripts/gen_w374_lean.py` → `scripts/gen_w375_lean.py`, then append:

1. `ternaryMacAccumulateFiftyOnePlusGeneric` — `a+b+...+as+au+av+aw+ax+ay+az` (51 variables).  
   Watch elaboration time; if it exceeds ~15 s, swap to a 50-variable plus/49-variable minus fallback.
2. `ternaryMacAccumulateFiftyMinusGeneric` — `-(a+b+...+as+au+av+aw+ax+ay)` (50-variable minus lattice).
3. `ternaryMacOctovigintupleCancellationGeneric` — depth-28 alternating plus/minus with **identity** `= x`.  
   Even depth returns to identity.
4. `ternaryMacZeroWeightOctodecupleClosureGeneric` — 9 zero + 1 plus + 9 zero = 19 variables / 18 zero-weight MACs.

Run `lake build Trinity.TernaryInference` and time it.

### 4.3 Safe gen-verilog sub-fix: early-return if-else chaining (Defect 3)

**Why this fix:** `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` lists Defect 3 (early `return` inside bare `if` lacks if-else chaining) as the next wave-safe issue after the deeper tuple-return dependency of Defect 6. Defect 3 causes real semantic bugs: a sequence like
```t27
if (x == 0.0) { return 1.0; }
if (x < 0.0)  { return 1.0 / exp_approx(-x); }
return exp_taylor(x);
```
is emitted as independent bare-if assignments followed by a final unconditional assignment, so the final assignment always wins.

**Where to change:**
- `bootstrap/src/compiler.rs` `gen_verilog_fn`: walk the function body and detect maximal chains of statements that are either
  - a bare `if (cond) { return expr; }` (no `else` branch), or
  - a final assignment to the function name or a bare `return expr;`.
- Emit the chain as a single Verilog `if ... else if ... else` block where each branch assigns to the function-name register.
- Keep all other statement types on their existing code path so the change is regression-free for non-early-return functions.

**Regression spec:** `specs/scratch/w375_early_return.t27` containing:
```t27
module w375_early_return;
fn sign(x : i8) -> i8 {
    if (x < 0) { return -1; }
    if (x > 0) { return 1; }
    return 0;
}
test w375_early_return_basic {
    assert sign(-5) == -1;
    assert sign(5) == 1;
    assert sign(0) == 0;
}
endmodule
```
Verify with `t27c gen-verilog` and `yosys read_verilog -sv` (+ `synth_xilinx` if applicable).

### 4.4 Seal regeneration

Because the compiler change shifts generated Verilog hashes:
1. Build `t27c`.
2. Run `t27c suite` to identify mismatches.
3. Script `t27c seal --save` for all mismatched specs from repo root.
4. Re-run `t27c suite` until 0 seal mismatches.
5. Regenerate IGLA seals after W375 blocks change spec hashes.

### 4.5 FPGA retry

- Run `dlc10 idcode`.
- If cable found, attempt `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` and capture evidence.
- If still missing, document in `docs/reports/FPGA_EVIDENCE_W375.md`.

### 4.6 Reports and memory

- `docs/reports/WAVE_LOOP_375_REPORT.md`
- `docs/reports/WAVE_LOOP_375_COOPERATION.md` (three W376 variants)
- `docs/reports/FPGA_EVIDENCE_W375.md`
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W375 results.
- Append to `.trinity/experience.md`.
- Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-375.md` and update `MEMORY.md` index.

---

## 5. Risk register and mitigations

| Risk | Mitigation |
|------|------------|
| 51-variable theorem times out | Swap to 50-plus/49-minus fallback and still hit 244 ∀ with an alternate closure theorem. |
| Early-return chaining breaks functions with mixed statement order | Only collapse contiguous leading/trailing chains; do not restructure nested or interleaved statements. |
| Mass seal mismatches from compiler fix | Script resealing in two passes (non-IGLA first, then IGLA); never commit with mismatched seals. |
| `dlc10` cable still missing | Document blocker; do not let hardware block the formal deliverables. |

---

## 6. Definition of done

- [ ] Issue #1264 created.
- [ ] `.claude/plans/wave-loop-375.md` committed.
- [ ] `scripts/gen_w375.py` and `scripts/gen_w375_lean.py` created.
- [ ] W375 blocks appended to all 27 IGLA specs.
- [ ] 4 new generic ∀ theorems build in Lean 4.
- [ ] `gen-verilog` early-return if-else chaining fixed with regression spec.
- [ ] All seals regenerated and `t27c suite` passes with 0 failures.
- [ ] W375 report, cooperation variants, and FPGA evidence documents written.
- [ ] `.trinity/experience.md` and memory updated.
- [ ] Final commit closes #1264.

---

*phi² + 1/phi² = 3 | TRINITY*
