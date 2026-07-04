# Wave Loop 415 — FPGA physical capture, real relay gate, or further formal tooling

**Issue:** #1343  
**Branch:** `wave-loop-415`  
**Milestone:** Continue the FPGA boot-evidence line from W414.

---

## Goal

Wave 414 delivered the **Variant C** formal-tooling fallback because the bench
was still blocked. Wave 415 re-evaluates the bench state and executes the first
available variant.

1. **Variant A (preferred when bench becomes available):**
   - Wire P12 to a logic-analyzer channel and capture real CCLK for
     `OSCFSEL=6` and `OSCFSEL=7`.
   - Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
     --standalone --validate` and commit the generated Lean theorems.
   - Document the measured frequencies/duty cycles in `fpga/HARDWARE_SSOT.md`.

2. **Variant B (if relay hardware is available):**
   - Implement a real `--relay-port` backend for `tri fpga cold-por`
     (e.g. serial or TCP relay controlling board power).
   - Perform an automated cold-POR power-cycle and capture STAT without
     operator intervention.
   - Document relay wiring in `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still blocked):**
   - Integrate the PVT envelope into `tri fpga measure-cclk --validate`.
   - Extend VCD parser unit tests for real-world quirks.
   - Build a library of measured-CCLK theorems for every documented Artix-7
     OSCFSEL value (0..7) under nominal and worst-case PVT contexts.

---

## Decomposed plan

See `.claude/plans/wave-loop-415.md` and
`docs/reports/FPGA_LOOP_COOPERATION_W415_2026-07-01.md`.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `.claude/plans/wave-loop-415.md` | Decomposed plan + weak points + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` | Updated capture / relay protocol |
| 3 | `cli/tri/src/fpga.rs` | Variant A import, B relay backend, or C parser/validate extensions |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | PVT-aware theorems or OSCFSEL library |
| 5 | `docs/reports/*` | W415 report, evidence, W416 cooperation |
| 6 | `.trinity/experience.md` | W415 learnings |
| 7 | git/PR | squash-merge to `master`, close #1343, open #1344 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: P12 is wired to a logic-analyzer channel and real CCLK capture files exist for `OSCFSEL=6` and `OSCFSEL=7`.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK is within the N25Q128_3V spec, or any exceedance is explicitly explained.

### Bundle B
- [ ] AC-B1: `tri fpga cold-por <bit> --relay-port <real>` performs an automated power-cycle and captures STAT.
- [ ] AC-B2: The resulting log has `relay_mock: false` and a real STAT raw value.
- [ ] AC-B3: `fpga/HARDWARE_SSOT.md` documents relay wiring and port mapping.

### Bundle C
- [ ] AC-C1: PVT-aware validation is available in `tri fpga measure-cclk --validate`.
- [ ] AC-C2: VCD parser unit tests cover multi-line declarations, bus values, real thresholds, and `$dumpoff`/`$dumpon`.
- [ ] AC-C3: Measured-CCLK theorem library covers OSCFSEL 0..7 under nominal and worst-case PVT contexts.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri fpga::tests` passes.

---

## Default variant

Execute **Variant A** if the analyzer is wired. Otherwise try **Variant B** if a
relay is available. Otherwise fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
