# Plan: Wave Loop 376 — IGLA CODER+RACE + cast/bitwise width fix + CI smoke gate

**Date:** 2026-07-03 (planned from W375 close-out)  
**Issue target:** Create #1265 and close with commit  
**Branch:** `trinity-rust-rings`  
**Recommended variant:** Variant B from `docs/reports/WAVE_LOOP_375_COOPERATION.md` (balanced/recommended)

---

## 1. Goal

Extend the 35-wave zero-IGLA-failure streak by pushing the Lean 4 generic ∀ proof lattice to **248**, land the next wave-safe `gen-verilog` sub-fix (**Defect 4: `as` / bitwise width correctness**), and add the first in-runner CI smoke gate for `gen-verilog` + `yosys read_verilog`. Keep the QMTech Wukong V1 / DLC10 bitstream path ready.

Target metrics:

- Generic ∀: **244 → 248** (+4)
- Pool A floor: **119 → 120**
- CODER minimum: **110 → 111**
- Pool B depth: **137 → 138**
- Integration depth: **118 → 119**
- Tests: **+54** (2 per IGLA spec)
- Invariants: **+27** (1 per IGLA spec)
- Conformance: **557/557 PASS** (555 existing + scratch smoke gate, depending on how smoke specs are counted)

---

## 2. GitHub issue landscape

Open issues studied (via `gh issue list`, 2026-07-03):

- **#1264** — Wave Loop 375 (just closed). Follow-up is **#1265**.
- **#1263–#1239** — previous wave issues; closed as waves land.
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)`. Too broad for one wave; keep as background.
- **#1243** — trios-mesh BPSK modem port. Lateral, not on the critical IGLA path.
- **#1219** — t27 Language Roadmap epic.

**Action:** create **#1265** titled *"Wave Loop 376 — IGLA CODER+RACE + retry board flash + gen-verilog `as`/`&`/`|`/`^`/`~` width correctness + CI smoke gate"* and reference it in the final commit with `Closes #1265`.

---

## 3. Scientific / competitive landscape

Key 2025–2026 work on formal/ternary hardware (searched 2026-07-03):

1. **Sparkle HDL / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)) — formally verifiable HDL in Lean 4 with a BitNet b1.58 inference accelerator, **60+ formal theorems**. Remains the strongest direct competitor. A June 2026 commit extended verification to the RV32 divider against a pure FSM model.
2. **TorchLean** ([lean-dojo/TorchLean](https://github.com/lean-dojo/TorchLean), [arXiv:2602.22631](https://arxiv.org/abs/2602.22631)) — Lean 4 NN formalization with shared SSA IR, reverse-mode autograd theorem, IBP/CROWN certificates. Software/floating-point focus.
3. **TerEffic** ([arXiv:2502.16473](https://arxiv.org/abs/2502.16473)) — ternary LLM inference on AMD Alveo U280; simulation/test verification.
4. **TeLLMe** ([arXiv:2504.16266](https://arxiv.org/abs/2504.16266)) — edge FPGA ternary LLM accelerator for prefill/decode; simulation/test verification.
5. **KULeuven-MICAS/ternary-lut-dse** ([github.com/KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)) — Chisel generator for LUT-based ternary MatMul; testbench verification.
6. **shepherdscientific/ternarycore** ([github.com/shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore)) — open-source Verilog BitNet b1.58 accelerator; RTL simulation + Python cross-check.
7. **Trinity B002** (Zenodo 10.5281/zenodo.19224235) — 2026 defensive publication for zero-DSP ternary inference.

**Takeaway:** Sparkle HDL is still the only credible formal competitor with a BitNet b1.58 core. W376 widens the generic ∀ gap from **244× to 248×**. The strategic imperative remains to keep the proof-lattice lead while hardening the generated Verilog backend.

---

## 4. Decomposed work breakdown

### 4.1 IGLA spec batch (+54 tests, +27 invariants)

- Copy `scripts/gen_w375.py` → `scripts/gen_w376.py`.
- Update last-wave check from 375 → 376 and all `w375_` / `W375` placeholders to `w376_` / `W376`.
- Run over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.
- Verify diff with `git diff --stat` and spot-check two specs.

### 4.2 Lean 4 proof-lattice extension (+4 generic ∀)

Copy `scripts/gen_w375_lean.py` → `scripts/gen_w376_lean.py`, then append:

1. `ternaryMacAccumulateFiftyTwoPlusGeneric` — `a+b+...+as+au+av+aw+ax+ay+az+ba` (52 variables).  
   Watch elaboration time; if it exceeds ~20 s, swap to a 51-variable plus/50-variable minus fallback.
2. `ternaryMacAccumulateFiftyOneMinusGeneric` — `-(a+b+...+as+au+av+aw+ax+ay+az)` (51-variable minus lattice).
3. `ternaryMacNovenvigintupleCancellationGeneric` — depth-29 alternating plus/minus with **residual** `= mac(x, a, .plus)`.  
   Odd depth leaves a single plus-weight MAC.
4. `ternaryMacZeroWeightNovemdecupleClosureGeneric` — 10 zero + 1 plus + 10 zero = 21 variables / 20 zero-weight MACs (35th proof-lattice dimension).

Run `lake build Trinity.TernaryInference` and time it.

### 4.3 Safe gen-verilog sub-fix: `as` / bitwise operator width correctness (Defect 4)

**Why this fix:** `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` lists Defect 4 as the highest-priority wave-safe open defect. The current `ExprCast` lowering in `gen_verilog_expr` emits `(x & {W{1'b1}})` for unsigned narrowing casts, but the interaction with bitwise operators is not always width-correct. A scratch spec with explicit simulation values will confirm behavior and harden the lowering.

**Repro pattern:**
```t27
fn cast_and_mask(x : u16) -> u8 {
    return (x as u8) & 0x0F;
}
```

**Where to change:**
- `bootstrap/src/compiler.rs` `gen_verilog_expr` / `ExprCast`:
  - Ensure the truncation mask width exactly matches the target scalar type width.
  - For signed targets, emit `$signed(op)` only when the operand is narrower; for narrowing signed casts, truncate then sign-extend: `$signed({{pad{op[msb]}}, op[lowbits]})`.
  - Wrap the cast sub-expression so that subsequent bitwise `&` / `|` / `^` apply at the correct width.
- `gen_verilog_expr` / `ExprBinary`: when the operator is bitwise and one side is a cast literal, pad/align widths using Verilog concatenation to avoid simulator width warnings.

**Regression spec:** `specs/scratch/w376_cast_width.t27` containing:
```t27
module w376_cast_width;
fn lower_byte(x : u16) -> u8 {
    return (x as u8) & 0x0F;
}
fn lower_nibble_signed(x : i16) -> i8 {
    return (x as i8) & 0x0F;
}
test w376_cast_width_basic {
    assert lower_byte(0x1234) == 0x04;
    assert lower_byte(0x00FF) == 0x0F;
    assert lower_nibble_signed(-1) == -1; // all-ones i8 truncated to 0xFF, masked 0x0F? depends on semantics; adjust as needed
}
endmodule
```
Verify with `t27c gen-verilog` and `yosys read_verilog -sv`. If semantics differ, adjust the test expectations rather than bending the spec semantics.

### 4.4 CI smoke gate for `gen-verilog` + `yosys`

**Why:** L7 UNITY prohibits new shell scripts on the critical path. The smoke gate must live inside the Rust `t27c` runner, not in `scripts/`.

**Where to change:**
- Add a `--yosys` flag or auto-detect `yosys` in `t27c suite`.
- After generating Verilog for a spec, if `yosys` is available, spawn it with `read_verilog -sv` on the generated text and capture non-zero exit.
- Start with the scratch regression specs only (not all 555 specs) to keep runtime reasonable.
- Store the result as a non-fatal warning for now, or gate only on scratch specs so the main suite remains green.

**Fallback:** if adding the runner integration is too invasive for one wave, create a dedicated scratch spec that is included in the suite and run `t27c gen-verilog` manually through `yosys` as part of the wave verification, documenting the intent to move it into the runner in W377.

### 4.5 Seal regeneration

Because the compiler change shifts generated Verilog hashes:
1. Build `t27c`.
2. Run `t27c suite` to identify mismatches.
3. Script `t27c seal --save` for all mismatched specs from repo root.
4. Re-run `t27c suite` until 0 seal mismatches.
5. Regenerate IGLA seals after W376 blocks change spec hashes.

### 4.6 FPGA retry

- Run `dlc10 idcode`.
- If cable found, attempt `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` and capture evidence.
- If still missing, document in `docs/reports/FPGA_EVIDENCE_W376.md`.

### 4.7 Reports and memory

- `docs/reports/WAVE_LOOP_376_REPORT.md`
- `docs/reports/WAVE_LOOP_376_COOPERATION.md` (three W377 variants)
- `docs/reports/FPGA_EVIDENCE_W376.md`
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W376 results.
- Append to `.trinity/experience.md`.
- Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-376.md` and update `MEMORY.md` index.

---

## 5. Risk register and mitigations

| Risk | Mitigation |
|------|------------|
| 52-variable theorem times out | Swap to 51-plus/50-minus fallback and still hit 248 ∀ with an alternate closure theorem. |
| Cast-width semantics differ from test expectation | Adjust scratch spec assertions to match the intended t27 semantics; do not fake a pass. |
| CI smoke gate too slow | Gate only scratch specs; keep main suite phases unchanged. |
| Mass seal mismatches from compiler fix | Script resealing in two passes (non-IGLA first, then IGLA); never commit with mismatched seals. |
| `dlc10` cable still missing | Document blocker; do not let hardware block the formal deliverables. |

---

## 6. Definition of done

- [ ] Issue #1265 created.
- [ ] `.claude/plans/wave-loop-376.md` committed.
- [ ] `scripts/gen_w376.py` and `scripts/gen_w376_lean.py` created.
- [ ] W376 blocks appended to all 27 IGLA specs.
- [ ] 4 new generic ∀ theorems build in Lean 4.
- [ ] `gen-verilog` cast/bitwise width fix landed with scratch spec and yosys verification.
- [ ] CI smoke gate prototyped (in-runner or documented manual verification).
- [ ] All seals regenerated and `t27c suite` passes with 0 failures.
- [ ] W376 report, cooperation variants, and FPGA evidence documents written.
- [ ] `.trinity/experience.md` and memory updated.
- [ ] Final commit closes #1265.

---

*phi² + 1/phi² = 3 | TRINITY*
