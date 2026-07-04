# Wave Loop 401 — CCLK measurement & cold-POR hardening

**Issue:** #1301  
**Branch:** `trinity-rust-rings`  
**Milestone:** FPGA boot-from-flash is verified; now lock the working default and measure CCLK.

---

## Goal

Close the remaining W400 follow-ups:

1. Measure actual CCLK frequency on pin P12 for the working default bitstream
   (`OSCFSEL=0`).
2. Harden the cold-POR protocol in the CLI and documentation.
3. Update CI smoke gate to enforce the canonical bitstream configuration.
4. Land W401 and publish W402 cooperation variants.

---

## Acceptance criteria

- [ ] AC1: CCLK frequency measured and recorded in `fpga/HARDWARE_SSOT.md`.
- [ ] AC2: `tri fpga smoke-gate` asserts `OSCFSEL=0` and no CRC/ID errors.
- [ ] AC3: `tri fpga boot-protocol --checklist` (or equivalent) documents/validates the cold-POR steps.
- [ ] AC4: `./scripts/tri test` passes (575/575).
- [ ] AC5: W401 report + evidence + W402 cooperation variants committed.

---

## Default variant

Execute **Variant A** from `docs/reports/FPGA_LOOP_COOPERATION_2026-07-08.md`:
- capture P12 with a logic analyser;
- parse with `tri fpga measure-cclk --csv <trace>`;
- update SSOT and land.

If no logic analyser is available, fall back to **Variant B**: board-less
hardening and CI guards.

---

*φ² + φ⁻² = 3 | TRINITY*
