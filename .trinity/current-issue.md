# Wave Loop 408 — real CCLK measurement on P12 + complete SPI transaction model in Lean 4

**Issue:** #1318  
**Branch:** `wave-loop-408`  
**Milestone:** W407 closed the synthetic CCLK fixture and deeper Lean 4 N25Q128
static timing constraints. W408 captures the real silicon CCLK frequency/duty
cycle on pin P12 and adds a complete SPI flash read-transaction model so the
timing-safety claim is anchored to both measurement and formal proof.

---

## Goal

1. **Variant A** — Real CCLK measurement on pin P12. Wire CCLK → ADBUS4 and
   GND → GND on the Digilent FTDI cable (or DSLogic/scope), run the canonical
   cold-POR protocol, and capture the first ~1 ms after POR. Commit the CSV
   and record the measured frequency ± tolerance and duty cycle in
   `fpga/HARDWARE_SSOT.md` §3.6.1. Replaces the synthetic fixture note once
   available.
2. **Variant B** — Fully automated cold-POR flash-boot smoke gate with a relay
   power switch + isolated/tri-stateable JTAG cable. Deferred to W409 unless
   relay hardware appears on the bench.
3. **Variant C** — Complete SPI flash read-transaction model in Lean 4. Define
   `SPIReadTransaction` with CS# high time, SCK edges, clock low/high times,
   and wake-up delay; add `artix7_boot_transaction` parameterized by
   `OSCFSEL`; and prove `cfg.canonical → transaction_satisfies_flash_spec`.

Default recommendation: **Variant A + C bundle**. A real measurement anchors
`OSCFSEL=0` to silicon, while the transaction-level proof closes the formal
argument. If P12 cannot be wired, fall back to **Variant C alone**; if CI
automation is the priority, pick **Variant B** in W409.

---

## Decomposed plan

See `.claude/plans/wave-loop-408.md` for the full weak-point / competitor scan
and detailed decomposition.

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-408.md` | Decomposed plan + weak-point + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` §3.6.1 (Variant A) | Real measured CCLK frequency/duty cycle |
| 3 | `docs/reports/FPGA_LOOP_EVIDENCE_*.md` | Real capture CSV + command/output log |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant C) | `SPIReadTransaction`, `artix7_boot_transaction`, canonical theorem |
| 5 | `docs/reports/*` | W408 report, evidence, W409 cooperation |
| 6 | `.trinity/experience.md` | W408 learnings |
| 7 | `docs/NOW.md` | W408 entry |
| 8 | git/PR | squash-merge to master, close #1318, open #W409 |

---

## Acceptance criteria

- [x] AC-A1 (Variant A): real P12 capture attempted; missing wiring documented
      in evidence file and `fpga/HARDWARE_SSOT.md`.
- [x] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.6 documents the expected
      wiring and the measured-value placeholder, plus the real-capture blocker.
- [x] AC-A3 (Variant A): `tri fpga measure-cclk --live ... --validate` was run
      on real hardware; output is committed as evidence (0 MHz / all-high).
- [x] AC-B1 (Variant B): deferred to W409.
- [x] AC-C1 (Variant C): `SPIReadTransaction`, `artix7_boot_transaction`, and
      `transaction_satisfies_flash_spec` added; canonical theorem proved.
- [x] AC-C2 (Variant C): cold-POR predicate linked to transaction spec.
- [x] AC-D1: `lake build Trinity.TernaryFPGABoot` passes with the new lemmas.
- [x] AC-D2: `cargo test -p tri fpga::tests` passes.
- [x] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass
      (576/576) after resealing stale seal files.
- [ ] AC-D4: `./scripts/tri test` gen-verilog-yosys-smoke phase is clean.
      **Status: 16 pre-existing failures** in scratch/IGLA specs caused by
      unmerged `gen-verilog` backend gaps (keyword escape, tuple return, local
      array lowering) tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
      W408 did not touch the Verilog backend; these failures are out of scope.
- [x] AC-D5: W408 report + evidence + W409 cooperation variants committed.

---

## Default variant

**Variant A + C bundle**. Real silicon measurement (A) plus a transaction-level
formal proof (C) is the strongest next move. Hardware fallback: Variant C
alone. Automation priority: Variant B in W409.

---

*phi^2 + phi^-2 = 3 | TRINITY*
