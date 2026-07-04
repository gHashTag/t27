# Wave Loop 418 — FPGA physical capture, real relay gate, or further formal tooling

**Issue:** #1353  
**Branch:** `wave-loop-418`  
**Milestone:** Continue the FPGA boot-evidence line from W417.

---

## Goal

Wave 417 closed the W415/W416 hygiene loop. Wave 418 re-evaluates the bench
state and executes the first available variant.

1. **Variant A (preferred when bench becomes available):**
   - Wire P12 to a logic-analyzer channel and capture real CCLK for
     `OSCFSEL=6` and `OSCFSEL=7`.
   - Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
     --standalone --validate --pvt-context <ctx.json>` and commit the generated
     Lean theorems.
   - Document the measured frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md`.

2. **Variant B (if relay hardware is available):**
   - Implement a real `--relay-port` backend for `tri fpga cold-por`
     (e.g. serial or TCP relay controlling board power).
   - Perform an automated cold-POR power-cycle and capture STAT without
     operator intervention.
   - Document relay wiring in `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still blocked):**
   - Add a regression test that the PVT envelope stays ≥ the nominal bound
     across the full operating rectangle.
   - Extend instrument import for VCD `$date`/`$version`/`$comment` headers and
     analog CSV voltage columns.
   - Build a standalone Lean proof integration test from a synthetic CSV.
   - Document the first-real-capture checklist in `fpga/HARDWARE_SSOT.md`.

---

## Decomposed plan

See `docs/reports/FPGA_LOOP_COOPERATION_W418_2026-07-04.md`.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` | Variant A import, B relay backend, or C regression/integration tests |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | PVT envelope regression lemma or new measured theorems |
| 3 | `fpga/HARDWARE_SSOT.md` | Updated capture / relay / integration protocol |
| 4 | `docs/reports/*` | W418 report, evidence, W419 cooperation |
| 5 | `.trinity/experience.md` | W418 learnings |
| 6 | git/PR | squash-merge to `master`, close #1353, open #? for W419 |

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
- [ ] AC-C1: A regression test verifies the PVT envelope lower bound across the operating rectangle.
- [ ] AC-C2: Instrument import handles VCD `$date`/`$version`/`$comment` headers or analog CSV voltage columns.
- [ ] AC-C3: A standalone `.lean` file generated from the CLI type-checks in a temporary `lake` package.

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
