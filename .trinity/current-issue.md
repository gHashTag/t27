# Wave Loop 410 — real P12 CCLK capture or physical OSCFSEL 6/7 boot + measured-duty formal link

**Issue:** #1325  
**Branch:** `wave-loop-410`  
**Milestone:** W409 delivered the per-OSCFSEL transaction lookup and tighter
duty-cycle validation, but the real P12 CCLK capture remains blocked by missing
wiring. W410 anchors the highest-margin OSCFSEL variants to real hardware and
makes the CCLK validation pipeline formally traceable.

---

## Goal

1. **Variant A** — Real CCLK measurement on pin P12. Once P12 → ADBUS4 (or a
   DSLogic / oscilloscope channel) is wired, capture the canonical cold-POR
   read transaction, commit the CSV, and record measured frequency ± tolerance
   and duty cycle in `fpga/HARDWARE_SSOT.md` §3.6.1.
2. **Variant B** — Fully automated cold-POR flash-boot smoke gate with a relay
   power switch and an isolated / tri-stateable JTAG cable. Deferred to W411
   unless relay hardware is available.
3. **Variant C** — Physically boot `OSCFSEL=6,7` on the Wukong board and add a
   measured-duty formal lemma in Lean 4. The lemma turns a captured
   `(frequency, duty)` pair into a `transaction_satisfies_flash_spec` proof using
   the N25Q128 `t_CL` / `t_CH` limits.

Default recommendation: **Variant A + C bundle**. A real measurement anchors the
model to silicon, while physical verification of OSCFSEL 6/7 closes the lookup
table. If P12 wiring is still unavailable, fall back to **Variant C alone**. If CI
automation is the priority, pick **Variant B**.

---

## Decomposed plan

See `docs/reports/FPGA_LOOP_COOPERATION_W410_2026-07-04.md` for the full
weak-point / competitor scan and detailed decomposition.

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `docs/reports/FPGA_LOOP_COOPERATION_W410_2026-07-04.md` | Cooperation variants |
| 2 | `fpga/HARDWARE_SSOT.md` §3.6.1 (Variant A) | Real measured CCLK frequency/duty cycle |
| 3 | `docs/reports/FPGA_LOOP_EVIDENCE_W410_*.md` | Real capture CSV + command/output log |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant C) | `measured_cclk_satisfies_flash_spec`, measured-duty lemma |
| 5 | `cli/tri/src/fpga.rs` (Variant C) | Export measured-frequency/duty types for the formal link |
| 6 | `build/fpga/boot-log-*.json` (Variant C) | `OSCFSEL=6,7` cold-POR boot logs |
| 7 | `docs/reports/*` | W410 report, evidence, W411 cooperation |
| 8 | `.trinity/experience.md` | W410 learnings |
| 9 | `docs/NOW.md` | W410 entry |
| 10 | git/PR | squash-merge to master, close #1325, open #W411 |

---

## Acceptance criteria

- [ ] AC-A1 (Variant A): a real CCLK capture CSV from P12 exists in
      `docs/reports/` or `build/fpga/`.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.6.1 contains the measured
      frequency and duty cycle with tolerance.
- [ ] AC-A3 (Variant A): `tri fpga measure-cclk --live ... --validate` passes
      on real hardware.
- [ ] AC-B1 (Variant B): deferred to W411 unless relay hardware is available.
- [ ] AC-C1 (Variant C): `OSCFSEL=6` and `OSCFSEL=7` physically booted and logged,
      or failures documented.
- [ ] AC-C2 (Variant C): a `measured_cclk_satisfies_flash_spec` predicate and
      lemma link a captured `(frequency, duty)` pair to the transaction spec.
- [ ] AC-D1: `lake build Trinity.TernaryFPGABoot` passes with the new lemmas.
- [ ] AC-D2: `cargo test -p tri fpga::tests` passes.
- [ ] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] AC-D4: `./scripts/tri test` gen-verilog-yosys-smoke phase is clean, or the
      remaining failures are explicitly tracked and scoped separately.
- [ ] AC-D5: W410 report + evidence + W411 cooperation variants committed.

---

## Default variant

**Variant A + C bundle**. Real silicon measurement (A) plus physical verification
of OSCFSEL 6/7 and a measured-duty formal link (C) is the strongest next move.
Hardware fallback: Variant C alone. Automation priority: Variant B in W411.

---

*phi^2 + phi^-2 = 3 | TRINITY*
