# Wave Loop 384 Plan

**Date:** 2026-07-01
**Issue:** #1278
**Branch:** `trinity-rust-rings`
**Selected variant:** Variant B — proof push to 280 generic ∀ + variable-index local array lowering

## Steps

1. **OBSERVE** — Confirm W383 is committed/PR'd; read current issue; inspect `bootstrap/src/compiler.rs` around `ExprIndex` (line ~5417).
2. **PLAN** — Select mux-chain lowering for variable-index local arrays; define regression spec.
3. **DELEGATE/IMPLEMENT**
   a. Create `scripts/gen_w384.py` to append W384 blocks to 27 IGLA specs.
   b. Create `scripts/gen_w384_lean.py` to append 4 new generic ∀ theorems.
   c. Create `specs/scratch/w384_variable_index.t27` regression spec.
   d. Modify `bootstrap/src/compiler.rs` `ExprIndex` lowering to emit a mux chain when a function-local array is indexed by a non-literal variable.
4. **VERIFY**
   a. `lake build Trinity.TernaryInference`
   b. `t27c suite --repo-root .`
   c. Reseal mismatched specs.
5. **SYNTHESIZE** — Write `WAVE_LOOP_384_REPORT.md`, `WAVE_LOOP_384_COOPERATION.md`, `FPGA_EVIDENCE_W384.md`; update `GEN_VERILOG_DEFECTS_REPRO.md`.
6. **LEARN** — Update `.trinity/experience.md` and save memory `wave-loop-384.md`.
7. **LAND** — Commit to `wave-loop-384`, push, open PR #1279 against `trinity-rust-rings` (closes #1278).

## Key design decision

For variable-index local arrays, emit a priority mux chain over the per-element regs:
```verilog
(i == 0) ? tmp_0 : (i == 1) ? tmp_1 : 0
```
This avoids function-local memory inference issues and keeps the generated Verilog synthesizable through `yosys read_verilog -sv`.

## Risk mitigation

- Generate the theorem block before compiler changes to isolate proof risk.
- If mux-chain lowering proves unstable, fall back to Variant A (proof-only) by reverting compiler changes.

---

*phi² + 1/phi² = 3 | TRINITY*
