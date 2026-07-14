# Wave Loop 532 — Extend Icarus lowerable subset to signed scalar-array struct fields

**Issue:** #1503 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-532`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 531's extended Icarus simulation gate and advance the
recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W532_2026-07-07.md`.

**Variant A (recommended):**
- Allow scalar-struct fields of the form `[N]i8`, `[N]i16`, `[N]i32` in the
  packed-vector layout already used for `[N]u8/u16/u32` fields.
- Emit signed packed vectors where needed and preserve sign extension on slice
  reads.
- Add positive scratch witnesses for signed array-field read, copy, param, and
  return paths.
- Add negative witnesses for non-lowerable cases (string/enum fields, dynamic
  sizes).
- Reseal affected specs and keep `./scripts/tri test --icarus-simulate` at 0
  simulation failures.
- Maintain the 16 documented yosys smoke baselines flat.

**Variant B:** Harden the lowerability boundary with adversarial non-lowerability
proofs in Lean 4 and a Rust integration test that aligns the classifier with the
formal predicate.

**Variant C:** Add reference-model cosimulation with cocotb by generating
cocotb-compatible testbench wrappers and a Python reference model for the
lowerable subset.

---

## Residual boundaries from W531

- `./scripts/tri test --icarus-simulate --icarus-lowerable` is green on the
  W493–W529 + lowerable W3xx regression suite (24 specs, 0 failures).
- 16 pre-existing yosys smoke failures remain documented.
- Signed scalar-array struct fields and adversarial lowerability proofs are
  deferred.

---

*φ² + φ⁻² = 3 | TRINITY*
