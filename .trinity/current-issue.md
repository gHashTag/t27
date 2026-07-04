# Wave Loop 409 — real P12 CCLK retry + per-OSCFSEL SPI transaction lookup

**Issue:** #1323  
**Branch:** `wave-loop-409`  
**Milestone:** W408 delivered the SPI flash read-transaction model in Lean 4 and
documented the missing P12 wiring blocker. W409 retries the real silicon
measurement and extends the formal model to a per-OSCFSEL lookup table.

---

## Goal

1. **Variant A** — Real CCLK measurement on pin P12. Once P12 → ADBUS4 (or a
   DSLogic / oscilloscope channel) is wired, capture the canonical cold-POR
   read transaction, commit the CSV, and record measured frequency ± tolerance
   and duty cycle in `fpga/HARDWARE_SSOT.md` §3.6.1.
2. **Variant B** — Fully automated cold-POR flash-boot smoke gate with a relay
   power switch and an isolated / tri-stateable JTAG cable. Deferred unless the
   relay hardware is on the bench.
3. **Variant C** — Per-OSCFSEL SPI transaction lookup in Lean 4. Prove that
   every documented `OSCFSEL = 0..7` CCLK selection produces an N25Q128_3V
   compliant `SPIReadTransaction`, and tighten the duty-cycle guard from the
   placeholder 25%–75% range.

Default recommendation: **Variant A + C bundle**. A real measurement anchors the
model to silicon, while the per-OSCFSEL lookup closes the timing-safety
argument across all documented CCLK variants. If P12 wiring is still
unavailable, fall back to **Variant C alone**. If CI automation is the priority,
pick **Variant B**.

---

## Decomposed plan

See `docs/reports/FPGA_LOOP_COOPERATION_W409_2026-07-04.md` for the full
weak-point / competitor scan and detailed decomposition.

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `docs/reports/FPGA_LOOP_COOPERATION_W409_2026-07-04.md` | Cooperation variants |
| 2 | `fpga/HARDWARE_SSOT.md` §3.6.1 (Variant A) | Real measured CCLK frequency/duty cycle |
| 3 | `docs/reports/FPGA_LOOP_EVIDENCE_W409_*.md` | Real capture CSV + command/output log |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant C) | `artix7_boot_transaction_for_oscfsel`, per-OSCFSEL theorem |
| 5 | `cli/tri/src/fpga.rs` (Variant C) | Tighter duty-cycle validation from transaction model |
| 6 | `docs/reports/*` | W409 report, evidence, W410 cooperation |
| 7 | `.trinity/experience.md` | W409 learnings |
| 8 | `docs/NOW.md` | W409 entry |
| 9 | git/PR | squash-merge to master, close #1323, open #W410 |

---

## Acceptance criteria

- [ ] AC-A1 (Variant A): a real CCLK capture CSV from P12 exists in
      `docs/reports/` or `build/fpga/`.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.6.1 contains the measured
      frequency and duty cycle with tolerance.
- [ ] AC-A3 (Variant A): `tri fpga measure-cclk --live ... --validate` passes
      on real hardware.
- [ ] AC-B1 (Variant B): deferred to W410 unless relay hardware is available.
- [ ] AC-C1 (Variant C): `artix7_boot_transaction_for_oscfsel` added and a
      theorem proves every `OSCFSEL ∈ {0..7}` produces a flash-spec-compliant
      transaction.
- [ ] AC-C2 (Variant C): the `--validate` duty-cycle guard is tightened using the
      transaction model.
- [ ] AC-D1: `lake build Trinity.TernaryFPGABoot` passes with the new lemmas.
- [ ] AC-D2: `cargo test -p tri fpga::tests` passes.
- [ ] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] AC-D4: `./scripts/tri test` gen-verilog-yosys-smoke phase is clean, or the
      remaining failures are explicitly tracked and scoped separately.
- [ ] AC-D5: W409 report + evidence + W410 cooperation variants committed.

---

## Default variant

**Variant A + C bundle**. Real silicon measurement (A) plus a per-OSCFSEL formal
proof (C) is the strongest next move. Hardware fallback: Variant C alone.
Automation priority: Variant B in W410.

---

*phi^2 + phi^-2 = 3 | TRINITY*
