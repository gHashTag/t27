# Wave Loop 495 — Next wave (Variant A recommended)

**Issue:** #1465 (to create)  
**Branch:** `wave-loop-495` (to create)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select one of the three variants in
`docs/reports/FPGA_LOOP_COOPERATION_W495_2026-07-13.md` and begin the next
wave:

- **Variant A (default)** — Extend semantic equivalence to function calls and
  the W493 positive witnesses (nested struct-return field access, struct-
  literal fields from scalar-struct identifiers).
- **Variant B** — Continue gen-verilog backend hardening (close the remaining
  local-array-of-struct element boundary and add the next adversarial witness).
- **Variant C** — FPGA live cold-POR / SPI flash boot evidence (contingent on
  DLC10 cable and board availability).

---

## Acceptance

- The selected W495 variant is documented in `.claude/plans/wave-loop-495.md`.
- Any code changes keep the W494 gate green:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS.
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure).
  - 697 / 697 seal matches.
  - `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
  - `tri verify --lean-lowerable`: green.
  - `lake build Trinity.IcarusLowerable.*`: green.
- Close-out report and three W496 cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
