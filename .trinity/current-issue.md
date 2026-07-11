# Wave Loop 496 — Next wave (Variant A recommended)

**Issue:** #1466 (to create)  
**Branch:** `wave-loop-496` (to create)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select one of the three variants in
`docs/reports/FPGA_LOOP_COOPERATION_W496_2026-07-13.md` and begin the next
wave:

- **Variant A (default)** — Prove the generic structural equivalence theorem
  for the Icarus-lowerable scalar subset, removing the `sorry` in
  `module_value_equiv_statement`.
- **Variant B** — Close the remaining Icarus baseline:
  `w493_local_aos_element_field_not_lowerable.t27`.
- **Variant C** — FPGA live cold-POR / SPI flash boot evidence (contingent on
  DLC10 cable and board availability).

---

## Acceptance

- The selected W496 variant is documented in `.claude/plans/wave-loop-496.md`.
- Any code changes keep the W495 gate green:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS.
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure).
  - 697 / 697 seal matches.
  - `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
  - `lake build` of the IcarusLowerable modules: green.
- Close-out report and three W497 cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
