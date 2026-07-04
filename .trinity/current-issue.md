# Wave Loop 412 — FPGA physical capture + relay CI gate or PVT refinement

**Issue:** #1332  
**Branch:** `wave-loop-412`  
**Milestone:** Continue the FPGA boot-evidence line from W411.

---

## Goal

1. **Variant A + B (preferred when bench is available):**
   - Wire P12 to a logic-analyzer channel and capture real CCLK for
     `OSCFSEL=6` and `OSCFSEL=7`.
   - Verify the Digilent DLC10 cable is detected with `dlc10 idcode`.
   - Boot from flash with both oscillator settings and collect boot logs.
   - Feed the measured `(frequency, duty)` into
     `tri fpga measured-to-lean --json` and commit the generated Lean theorems.
   - Add relay-controlled cold-POR automation (`tri fpga cold-por`) with a
     mock CI path.

2. **Variant C (fallback if bench still blocked):**
   - Replace the placeholder 2× PVT constants with real N25Q128 PVT derating
     data or document the assumption more precisely.
   - Extend `tri fpga measured-to-lean` to emit a self-contained `.lean` file.
   - Add raw-ns input mode (`period_ns`, `low_ns`, `high_ns`) and a
     corresponding Lean predicate.

---

## Decomposed plan

See `.claude/plans/wave-loop-412.md` and
`docs/reports/FPGA_LOOP_COOPERATION_W412_2026-07-04.md`.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `.claude/plans/wave-loop-412.md` | Decomposed plan + weak points + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` §3.6.12 | Relay wiring and CI cold-POR protocol (A+B) |
| 3 | `cli/tri/src/fpga.rs` + new `fpga/src/relay.rs` | `tri fpga cold-por` with mock + hardware backends (A+B) |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Real-measurement theorems or PVT refinement (C) |
| 5 | `docs/reports/*` | W412 report, evidence, W413 cooperation |
| 6 | `.trinity/experience.md` | W412 learnings |
| 7 | git/PR | squash-merge to `master`, close #1332, open #1333 |

---

## Acceptance criteria

### Bundle A + B (deferred — bench unavailable)
- [ ] AC-A1: P12 is wired to a logic-analyzer channel and a real CCLK capture CSV exists for `OSCFSEL=6` and `OSCFSEL=7`.
- [ ] AC-A2: `dlc10 idcode` returns the expected Artix-7 IDCODE.
- [ ] AC-A3: Cold-POR boot logs exist for `OSCFSEL=6` and `OSCFSEL=7`.
- [ ] AC-A4: Measured `(frequency, duty)` theorems are generated and `lake build` green.
- [ ] AC-B1: `tri fpga cold-por --relay-port MOCK` runs in CI and returns a JSON boot log.
- [ ] AC-B2: `fpga/HARDWARE_SSOT.md` documents the relay wiring.

### Bundle C (delivered)
- [x] AC-C1: PVT placeholder model is documented in `fpga/HARDWARE_SSOT.md` §3.6.12 with falsification plan.
- [x] AC-C2: `tri fpga measured-to-lean --standalone --out File.lean` emits a self-contained file.
- [x] AC-C3: Raw-ns input predicate `measured_cclk_from_raw_ns_satisfies_flash_spec` is defined and implies `transaction_satisfies_flash_spec`.

### Invariant checks
- [x] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass (16 pre-existing yosys-smoke failures remain).
- [x] `lake build Trinity.TernaryFPGABoot` passes.
- [x] `cargo test -p tri fpga::tests` passes (16/16).

---

## Default variant

Execute **Variant A + B** when the bench becomes available. Otherwise fall back
 to **Variant C** to keep the formal tooling useful while the physical blockers
persist.

---

*φ² + φ⁻² = 3 | TRINITY*
