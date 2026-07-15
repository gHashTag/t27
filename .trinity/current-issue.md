# Wave Loop 540 — Multi-signal VCD probes for wide packed structs and arrays

**Issue:** #1511 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-540`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 539's typed VCD probe + full Python expression evaluator
and implement the recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W540_2026-07-08.md`.

**Variant A (recommended):**
- Extend `VerilogCodegen` to emit multiple 64-bit (or smaller) VCD probes for
  `assert_eq` actual expressions wider than 64 bits.
- Record slice offset/width metadata for each probe.
- Extend `scripts/cocotb_ref_model.py` to reconstruct the full packed value from
  slices and compare it against the independently evaluated expected value.
- Add a scratch witness with a wide packed-struct-array assertion.
- Keep `./scripts/tri test --icarus-lowerable --cocotb --fast` at 0 cocotb
  failures and 0 seal mismatches.
- Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Variant B:** Support scalar function-call arguments in the Python reference
model so assertions like `assert_eq(add(-3, 4), 1)` get an independent VCD
cross-check.

**Variant C:** Formalize VCD-time expression equivalence in Lean, connecting the
cocotb reference model to `module_value_equiv`.

---

## Residual boundaries from W539

- `./scripts/tri test --icarus-lowerable --cocotb --fast` is green:
  35 Icarus simulations passed, 0 failed; 35 cocotb reference-model checks
  passed, 0 failed; 0 seal mismatches.
- 24 pre-existing yosys smoke baseline failures remain documented and unchanged.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`,
  `cargo test -p tri`, and `cargo test -p t27c --test icarus_lowerable` are
  green.
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`: 8572 jobs,
  0 `sorry`.
- Wide packed-struct/array assertions (width > 64 bits) still skip the
  independent VCD check and rely on the log-based self-check.

---

*φ² + φ⁻² = 3 | TRINITY*
