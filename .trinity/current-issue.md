# Wave Loop 494 — Next wave (Variant A recommended)

**Issue:** #1464 (to create)  
**Branch:** `wave-loop-494` (to create)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select one of the three variants in
`docs/reports/FPGA_LOOP_COOPERATION_W494_2026-07-13.md` and begin the next
wave:

- **Variant A (default)** — Machine-checked semantic equivalence for the
  Icarus-lowerable scalar subset in Lean 4 (extend the W492/W493 soundness work).
- **Variant B** — Continue gen-verilog struct/call lowering hardening (close
  the remaining local-array-of-struct element boundary and add the next
  adversarial witness).
- **Variant C** — FPGA live cold-POR / SPI flash boot evidence (contingent on
  DLC10 cable and board availability).

---

## Acceptance

- The selected W494 variant is documented in `.claude/plans/wave-loop-494.md`.
- Any code changes keep the W493 gate green:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS.
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure).
  - 697 / 697 seal matches.
  - `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
  - `tri verify --lean-lowerable`: green.
- Close-out report and three W495 cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
