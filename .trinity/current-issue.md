# Wave Loop 411 — real P12 + OSCFSEL 6/7 retry, relay CI gate, or auto-proof tooling

**Issue:** #1329  
**Branch:** `wave-loop-411`  
**Milestone:** W410 completed the measured-duty formal link but both physical
paths (real P12 capture and DLC10-based `OSCFSEL=6,7` boot) remain blocked. W411
either finally connects the bench or uses the extra cycle to build automation /
formal tooling.

---

## Goal

1. **Variant A** — Real P12 CCLK capture + physical `OSCFSEL=6,7` cold-POR
   boot. Wire P12 to ADBUS4 (or a DSLogic/scope channel), connect the DLC10
   cable, capture the real CCLK, run the 6/7 sweep, and turn the measured
   values into a `measured_cclk_satisfies_flash_spec` proof.
2. **Variant B** — Relay-controlled cold-POR hardware CI gate. Build the
   relay + tri-stateable JTAG automation infrastructure so the flash-boot gate
   can run unattended.
3. **Variant C** — Auto-proof tooling + PVT margins. Add a subcommand that
   generates a Lean theorem from a `--json` measurement, and extend the formal
   model with conservative process/voltage/temperature margins.

Default recommendation: **Variant A + C bundle**. If the bench is still
unavailable, pick **Variant B** or **Variant C alone**.

---

## Decomposed plan

See `docs/reports/FPGA_LOOP_COOPERATION_W411_2026-07-04.md` for the full
weak-point / competitor scan and detailed decomposition.

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `docs/reports/FPGA_LOOP_COOPERATION_W411_2026-07-04.md` | Cooperation variants |
| 2 | `fpga/HARDWARE_SSOT.md` §3.6.1/§3.6.9 (Variant A) | Real measured CCLK and 6/7 boot status |
| 3 | `build/fpga/` + `docs/reports/` (Variant A) | Real capture CSV + boot-log JSON |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant A/C) | `measured_*_satisfies_flash_spec` instance from real data |
| 5 | `cli/tri/src/fpga.rs` (Variant B/C) | Relay trait + auto-power-cycle mode, or `measured-to-lean` subcommand |
| 6 | `docs/reports/*` | W411 report, evidence, W412 cooperation |
| 7 | `.trinity/experience.md` | W411 learnings |
| 8 | `docs/NOW.md` | W411 entry |
| 9 | git/PR | squash-merge to master, close #1329, open #W412 |

---

## Acceptance criteria

- [ ] AC-A1: real P12 CCLK capture CSV exists (or blocker documented). **Blocked — P12 not wired.**
- [ ] AC-A2: `OSCFSEL=6,7` boot logs exist (PASS or documented failure). **Blocked — DLC10 cable missing.**
- [ ] AC-A3: a Lean theorem links the captured `(frequency, duty)` pair to
      `transaction_satisfies_flash_spec`. **Delivered via `measured-to-lean` generated theorems on synthetic data.**
- [ ] AC-B1: relay auto-power-cycle mode exists behind a trait with a board-less
      mock path, or explicitly deferred. **Deferred to W412.**
- [x] AC-C1: `measured-to-lean` subcommand generates a type-correct theorem
      skeleton from `--json`.
- [x] AC-C2: PVT margin predicate exists in Lean 4 and implies
      `measured_cclk_satisfies_flash_spec`.
- [x] AC-C3: A Lean theorem proves that the PVT margin predicate implies
      `transaction_satisfies_flash_spec`.
- [x] AC-D1: `lake build Trinity.TernaryFPGABoot` passes.
- [x] AC-D2: `cargo test -p tri fpga::tests` passes.
- [x] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [x] AC-D4: gen-verilog-yosys-smoke failures remain tracked separately.
- [x] AC-D5: W411 report + evidence + W412 cooperation variants committed.

---

## Close-out note

W411 implemented **Variant C alone** because the bench blockers persisted. The
`tri fpga measured-to-lean` subcommand and the PVT-margin predicate close two
real weak points in the measured-to-formal pipeline. Variant A and the physical
half of any bundle remain blocked until P12 is wired and the DLC10 cable is
connected.

---

## Default variant

**Variant A + C bundle**: physical measurement and 6/7 boot, with auto-proof
tooling. If the bench is still unavailable, fall back to **Variant B** (relay
CI gate) or **Variant C alone** (formal-tooling improvements).

---

*phi^2 + phi^-2 = 3 | TRINITY*
