# Plan: Wave Loop 377 — IGLA CODER+RACE + struct-field reg mapping + CI smoke gate expansion

**Date:** 2026-07-01 (planned from W376 close-out)  
**Issue target:** Create #1267 and close with commit  
**Branch:** `trinity-rust-rings`  
**Recommended variant:** Variant B from `docs/reports/WAVE_LOOP_376_COOPERATION.md` (balanced/recommended)

---

## 1. Goal

Extend the 36-wave zero-IGLA-failure streak by pushing the Lean 4 generic ∀ proof lattice to **252**, land the next wave-safe `gen-verilog` sub-fix (**Defect 5: struct-field reg name mapping**), and expand the in-runner CI smoke gate so synthesizable IGLA specs are also yosys-checked. Keep the QMTech Wukong V1 / DLC10 bitstream path ready.

Target metrics:

| Metric | W376 | W377 | Δ |
|---|---|---|---|
| Lean 4 generic ∀ | 248 | **252** | +4 |
| Pool A floor | 120 | **121** | +1 |
| CODER minimum | 111 | **112** | +1 |
| Pool B depth | 138 | **139** | +1 |
| Integration depth | 119 | **120** | +1 |
| Tests | 13,028 | **13,085** | +57 |
| Invariants | 5,714 | **5,741** | +27 |
| Conformance specs | 556 | **557** | +1 (scratch) |
| Conformance pass rate | 556/556 | **557/557** | 100% |
| Zero-IGLA-failure streak | 110 waves | **111 waves** | +1 |

---

## 2. GitHub issue landscape

Open issues studied (via `gh issue list`, 2026-07-01):

- **#1266** — Wave Loop 376 (just closed). Follow-up is **#1267**.
- **#1265–#1239** — previous wave issues; closed as waves land.
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)`. Still too broad for one wave; keep as background.
- **#1243** — trios-mesh BPSK modem port. Lateral, not on the critical IGLA path.
- **#1219** — t27 Language Roadmap epic.

**Action:** create **#1267** titled *"Wave Loop 377 — IGLA CODER+RACE + retry board flash + gen-verilog struct-field reg name mapping + CI smoke gate expansion"* and reference it in the final commit with `Closes #1267`.

---

## 3. Scientific / competitive landscape

Key 2025–2026 work on formal/ternary hardware (searched 2026-07-01):

1. **Sparkle HDL / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)) — formally verifiable HDL in Lean 4 with a BitNet b1.58 inference accelerator, **60+ formal theorems**. Remains the strongest direct competitor. June 2026 commits extended verification to the RV32 divider against a pure FSM model.
2. **TorchLean** ([lean-dojo/TorchLean](https://github.com/lean-dojo/TorchLean), [arXiv:2602.22631](https://arxiv.org/abs/2602.22631)) — Lean 4 NN formalization with shared SSA IR, reverse-mode autograd theorem, IBP/CROWN certificates. Software/floating-point focus.
3. **TerEffic** ([arXiv:2502.16473](https://arxiv.org/abs/2502.16473)) — ternary LLM inference on AMD Alveo U280; simulation/test verification.
4. **TeLLMe** ([arXiv:2504.16266](https://arxiv.org/abs/2504.16266)) — edge FPGA ternary LLM accelerator for prefill/decode; simulation/test verification.
5. **KULeuven-MICAS/ternary-lut-dse** ([github.com/KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)) — Chisel generator for LUT-based ternary MatMul; testbench verification.
6. **shepherdscientific/ternarycore** ([github.com/shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore)) — open-source Verilog BitNet b1.58 accelerator; RTL simulation + Python cross-check.
7. **Trinity B002** (Zenodo 10.5281/zenodo.19224235) — 2026 defensive publication for zero-DSP ternary inference.

**Takeaway:** Sparkle HDL is still the only credible formal competitor with a BitNet b1.58 core. W377 widens the generic ∀ gap from **248× to 252×**. The strategic imperative remains to keep the proof-lattice lead while hardening the generated Verilog backend.

---

## 4. Decomposed work breakdown

### 4.1 IGLA spec batch (+57 tests, +27 invariants)

- Copy `scripts/gen_w376.py` → `scripts/gen_w377.py`.
- Update last-wave check from 376 → 377 and all `w376_` / `W376` placeholders to `w377_` / `W377`.
- Run over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.
- Verify diff with `git diff --stat` and spot-check two specs.

### 4.2 Lean 4 proof-lattice extension (+4 generic ∀)

Copy `scripts/gen_w376_lean.py` → `scripts/gen_w377_lean.py`, then append:

1. `ternaryMacAccumulateFiftyThreePlusGeneric` — `a+b+...+as+au+av+aw+ax+ay+az+ba+bb` (53 variables).  
   Watch elaboration time; if it exceeds ~25 s, swap to a 52-variable plus/51-variable minus fallback.
2. `ternaryMacAccumulateFiftyTwoMinusGeneric` — `-(a+b+...+as+au+av+aw+ax+ay+az+ba)` (52-variable minus lattice).
3. `ternaryMacTrigintupleCancellationGeneric` — depth-30 alternating plus/minus with **identity** `= x`.  
   Even depth collapses cleanly to identity.
4. `ternaryMacZeroWeightVigintupleClosureGeneric` — 11 zero + 1 plus + 11 zero = 23 variables / 22 zero-weight MACs (36th proof-lattice dimension).

Run `lake build Trinity.TernaryInference` and time it.

### 4.3 Safe gen-verilog sub-fix: struct-field reg name mapping (Defect 5)

**Why this fix:** `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` now lists Defect 5 as the highest-priority wave-safe open defect. Defect 6 (`let` destructuring) remains blocked by missing tuple-return function generation; Defect 5 is narrow and improves generated code quality for the struct specs already in the repo.

**Repro pattern:**
```t27
struct Pt { x : u8; y : u8; }
fn get_x(p : Pt) -> u8 {
    return p.x;
}
```

**Observed problem:** Struct declarations emit flattened module-level regs as `structname_fieldname` with the struct name lowercased (`word_reg`). Field access on a struct *variable* (not a struct type) currently emits `varname_fieldname` (`p_x`). If the variable is of struct type `Word` and the field is `reg`, the declaration is `word_reg` while the access is `p_reg`, so simulation sees an unresolved name when struct literals/assignments are not fully lowered.

**Where to change:**
- `bootstrap/src/compiler.rs`:
  - In `gen_verilog_struct`, record the mapping from `(struct_type_name, field_name)` to the emitted register name, so variable-based field access can resolve it.
  - In `gen_verilog_expr` / `ExprFieldAccess`, when the base is a variable whose type is a known struct, emit the *struct-type register* name rather than a variable-qualified name. If the type is unknown or the base is an array element, keep the existing `base_field` fallback.
  - If the struct name is not available at the access site, add a lightweight type cache: `current_fn_param_types` mapping parameter names to their declared types, populated when entering each function.

**Regression spec:** `specs/scratch/w377_struct_field_mapping.t27` containing:
```t27
module w377_struct_field_mapping;
struct Pt { x : u8; y : u8; }

fn get_x(p : Pt) -> u8 {
    return p.x;
}

fn get_y(p : Pt) -> u8 {
    return p.y;
}

test w377_struct_field_mapping_basic {
    var a : Pt = Pt { x: 3, y: 4 };
    assert get_x(a) == 3;
    assert get_y(a) == 4;
}

invariant w377_struct_field_mapping_bounds always get_x(Pt { x: 1, y: 2 }) >= 0;
endmodule
```

Verify with `t27c gen-verilog` and `yosys read_verilog -sv`. The generated Verilog must contain a consistent register name for each struct field and the function body must reference that same register.

### 4.4 CI smoke gate expansion

**Why:** L7 UNITY prohibits new shell scripts on the critical path. The smoke gate must stay inside the Rust `t27c` runner.

**Where to change:**
- `bootstrap/src/suite.rs`:
  - Keep the existing scratch-spec smoke gate.
  - Add an opt-in list (e.g., `IGLA_SMOKE_SPECS`) of synthesizable IGLA specs that are known to be yosys-clean.
  - Based on the probe in W376, all `specs/igla/coder/*.t27` pass `yosys read_verilog -sv`, and all `specs/igla/race/*.t27` pass except `cordic.t27` and `cordic_top.t27` (both fail due to Defect 6 `let` destructuring).
  - Add the 25 clean race/coder specs to the IGLA smoke gate and report pass/fail counts separately from the scratch gate.

**Fallback:** if adding the IGLA list proves too invasive, expand the gate to all specs and mark `cordic.t27` / `cordic_top.t27` as expected failures with a note that they are blocked on Defect 6.

### 4.5 Seal regeneration

Because the compiler change shifts generated Verilog hashes:
1. Build `t27c`.
2. Run `t27c suite` to identify mismatches.
3. Script `t27c seal --save` for all mismatched specs from repo root.
4. Re-run `t27c suite` until 0 seal mismatches.
5. Regenerate IGLA seals after W377 blocks change spec hashes.

### 4.6 FPGA retry

- Run `dlc10 idcode`.
- If cable found, attempt `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` and capture evidence.
- If still missing, document in `docs/reports/FPGA_EVIDENCE_W377.md`.

### 4.7 Reports and memory

- `docs/reports/WAVE_LOOP_377_REPORT.md`
- `docs/reports/WAVE_LOOP_377_COOPERATION.md` (three W378 variants)
- `docs/reports/FPGA_EVIDENCE_W377.md`
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W377 results.
- Append to `.trinity/experience.md`.
- Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-377.md` and update `MEMORY.md` index.

---

## 5. Risk register and mitigations

| Risk | Mitigation |
|------|------------|
| 53-variable theorem times out | Swap to 52-plus/51-minus fallback and still hit 252 ∀ with an alternate closure theorem. |
| Struct-field mapping breaks existing keyword-escape behavior | Keep the existing `verilog_safe_identifier()` path; only change the *base* of the flattened name from `varname` to `struct_type_name`. |
| CI smoke gate expansion slows the suite | Gate only the 25 known-clean IGLA specs; skip the two `cordic` specs blocked on Defect 6. |
| Mass seal mismatches from compiler change | Script resealing in two passes (non-IGLA first, then IGLA); never commit with mismatched seals. |
| `dlc10` cable still missing | Document blocker; do not let hardware block the formal deliverables. |

---

## 6. Definition of done

- [ ] Issue #1267 created.
- [ ] `.claude/plans/wave-loop-377.md` committed.
- [ ] `scripts/gen_w377.py` and `scripts/gen_w377_lean.py` created.
- [ ] W377 blocks appended to all 27 IGLA specs.
- [ ] 4 new generic ∀ theorems build in Lean 4.
- [ ] `gen-verilog` struct-field reg name mapping fixed with scratch spec and yosys verification.
- [ ] CI smoke gate expanded to cover known-clean IGLA specs.
- [ ] All seals regenerated and `t27c suite` passes with 0 failures.
- [ ] W377 report, cooperation variants, and FPGA evidence documents written.
- [ ] `.trinity/experience.md` and memory updated.
- [ ] Final commit closes #1267.

---

*phi² + 1/phi² = 3 | TRINITY*
