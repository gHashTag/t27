# Wave Loop 493 — Next wave (to be selected from cooperation plan)

**Issue:** #1463 (to create)  
**Branch:** `wave-loop-493`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select one of the three variants in
`docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-12.md` and begin the next
wave:

- **Variant A (default)** — Machine-checked semantic equivalence for the
  Icarus-lowerable scalar subset (extend the W492 soundness proof).
- **Variant B** — Continue gen-verilog struct/call lowering hardening (close the
  documented adversarial baseline witnesses and import-boundary gaps).
- **Variant C** — FPGA live cold-POR / SPI flash boot evidence (contingent on
  DLC10 cable and board availability).

---

## Acceptance

- The selected W493 variant is documented in `.claude/plans/wave-loop-493.md`.
- Any code changes keep the W492 gate green:
  - 693 / 693 non-smoke PASS.
  - 172 / 173 yosys smoke PASS (1 documented baseline failure).
  - 171 / 173 Icarus smoke PASS (2 documented baseline failures).
  - 693 / 693 seal matches.
  - `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
  - `tri verify --lean-lowerable`: green.
- Close-out report and three W494 cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
