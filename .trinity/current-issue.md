# Wave Loop 401 — CCLK measurement & cold-POR hardening

**Issue:** #1303  
**Branch:** `trinity-rust-rings`  
**Milestone:** FPGA boot-from-flash is verified; now lock the working default,
harden the cold-POR protocol, and measure/record CCLK.

---

## Goal

1. Harden the cold-POR protocol with a standalone `tri fpga boot-protocol`
   command and explicit operator checklist.
2. Extend `tri fpga smoke-gate` to enforce the canonical bitstream:
   `IDCODE=0x03636093`, `SPI_BUSWIDTH=x1`, `STARTUPCLK=CCLK`, `OSCFSEL=0`,
   and no CRC register writes.
3. Make `tri fpga measure-cclk --csv` robust to DSView, PulseView, and Saleae
   CSV exports, and add unit tests for the parser.
4. Add a board-less dry-run sweep guard so CI exercises `cclk-sweep` and
   `sweep-report` without hardware.
5. If a physical CCLK capture is available, record the measured frequency in
   `fpga/HARDWARE_SSOT.md`.
6. Land W401 and publish W402 cooperation variants.

---

## Decomposed plan

See `.claude/plans/wave-loop-401.md` for the full work breakdown.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `scripts/dump_bit_config.py` | `--assert-oscfsel N`, `--assert-no-crc-writes` |
| 2 | `cli/tri/src/fpga.rs` | Extended `smoke-gate`, new `boot-protocol`, robust CSV parser + tests |
| 3 | `bootstrap/src/suite.rs` | Board-less dry-run sweep + report guard |
| 4 | `fpga/HARDWARE_SSOT.md`, `docs/reports/*` | SSOT update + W401 report/evidence/cooperation |
| 5 | `.trinity/experience.md` | W401 learnings |
| 6 | git/PR | squash-merge to `trinity-rust-rings`, close #1303, open #W402 |

---

## Acceptance criteria

- [ ] AC1: `tri fpga smoke-gate` asserts `OSCFSEL=0` and no CRC writes.
- [ ] AC2: `tri fpga boot-protocol --checklist` prints the cold-POR steps.
- [ ] AC3: `tri fpga measure-cclk --csv` parses DSView, PulseView, and Saleae CSV
      exports.
- [ ] AC4: Unit tests for CSV parsing pass.
- [ ] AC5: Board-less dry-run sweep + report path is exercised in CI/smoke gate.
- [ ] AC6: `./scripts/tri test` passes (575/575).
- [ ] AC7: W401 report + evidence + W402 cooperation variants committed.
- [ ] AC8: If a physical CCLK capture is available, pin P12 frequency is recorded
      in `fpga/HARDWARE_SSOT.md`.

---

## Default variant

Execute **Variant A** from `docs/reports/FPGA_LOOP_COOPERATION_2026-07-08.md`
when a logic analyser is available; otherwise execute **Variant B**:
board-less hardening and CI guards. This wave implements both so that the
physical capture can be slotted in as soon as hardware is ready.

---

*φ² + φ⁻² = 3 | TRINITY*
