# Wave Loop 531 — Extend Icarus simulation regression suite

**Issue:** #1502 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-531`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 530's executable Icarus simulation gate and advance the
recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W531_2026-07-07.md`.

**Variant A (recommended):**
- Add any new W531 lowerable scratch specs to the Icarus simulation regression
  suite in `./scripts/tri test --icarus-simulate`.
- Record JSON baselines under `.trinity/icarus-baselines/` for the new
  witnesses.
- Refine the classifier so only lowerable specs enter the gate.
- Maintain 0 Icarus simulation failures and keep the 16 documented yosys smoke
  baselines flat.

**Variant B:** Support signed scalar-array fields in packed scalar structs.

**Variant C:** Add adversarial lowerability proofs and document the boundary.

---

## Residual boundaries from W530

- `./scripts/tri test --icarus-simulate --icarus-lowerable` is green on the
  W493–W529 regression suite (10 specs, 0 failures).
- 16 pre-existing yosys smoke failures remain documented.
- Signed scalar-array struct fields and adversarial lowerability proofs are
  deferred.

---

*φ² + φ⁻² = 3 | TRINITY*
