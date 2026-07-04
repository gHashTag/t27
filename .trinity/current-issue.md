# Wave Loop 419 — physical CCLK capture, real relay gate, or instrument-import parity

**Issue:** #1357  
**Branch:** `wave-loop-419`  
**Milestone:** Continue the FPGA boot-evidence line from W418.

---

## Goal

Wave 418 closed the Variant C fallback (formal tooling and instrument import).
Wave 419 re-evaluates the bench state and executes the first available variant.

1. **Variant A (preferred when bench becomes available):**
   - Wire P12 to a logic-analyzer channel and capture real CCLK for
     `OSCFSEL=6` and `OSCFSEL=7`.
   - Program each variant to SPI flash and perform a true cold-POR boot.
   - Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
     --standalone --validate --pvt-context <ctx.json>` and commit the generated
     Lean theorems.
   - Document the measured frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md`.

2. **Variant B (if relay hardware is available, no CCLK probe):**
   - Implement a real `--relay-port` backend for `tri fpga cold-por`
     (e.g. serial or TCP relay controlling board power).
   - Perform an automated cold-POR power-cycle and capture STAT without
     operator intervention.
   - Document relay wiring and port syntax in `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still blocked):**
   - Extend instrument-import parity: additional VCD/CSV formats and hardened
     `$comment` sections.
   - Add PVT envelope monotonicity/antitonicity tests in Rust and Lean.
   - Document the standalone `lake`-package workflow end-to-end in
     `fpga/HARDWARE_SSOT.md`.

---

## Decomposed plan

See `docs/reports/FPGA_LOOP_COOPERATION_W419_2026-07-04.md` and
`.claude/plans/wave-loop-419.md`.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` | Variant A import, B relay backend, or C parity/monotonicity tests |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | PVT monotonicity lemmas or new measured theorems |
| 3 | `fpga/HARDWARE_SSOT.md` | Updated capture / relay / integration protocol |
| 4 | `docs/reports/*` | W419 report, evidence, W420 cooperation |
| 5 | `.trinity/experience.md` | W419 learnings |
| 6 | git/PR | squash-merge to `master`, close #1357, open #? for W420 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: P12 is wired to a logic-analyzer channel and real CCLK capture files exist for `OSCFSEL=6` and `OSCFSEL=7`.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is explicitly explained.

### Bundle B
- [ ] AC-B1: `tri fpga cold-por <bit> --relay-port <real>` performs an automated power-cycle and captures STAT.
- [ ] AC-B2: The resulting log has `relay_mock: false` and a real STAT raw value.
- [ ] AC-B3: `fpga/HARDWARE_SSOT.md` documents relay wiring and port syntax.

### Bundle C
- [x] AC-C1: At least one additional instrument-import unit test lands (VCD `$comment` hardening + CSV `--csv-channel` explicit select).
- [x] AC-C2: Rust and Lean tests verify PVT envelope monotonicity/antitonicity.
- [x] AC-C3: The standalone lake-package workflow is documented end-to-end.

### Invariant checks
- [x] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [x] `lake build Trinity.TernaryFPGABoot` passes.
- [x] `cargo test -p tri fpga::tests` passes.

---

## PR
- Target: `master`
- PR: #1360
- Body: `Closes #1357`
- Report: `docs/reports/WAVE_LOOP_419_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W419_2026-07-05.md`
- Cooperation W420: `docs/reports/FPGA_LOOP_COOPERATION_W420_2026-07-05.md`

---

## Default variant

Execute **Variant A** if the analyzer and DLC10 cable are available. Otherwise
try **Variant B** if a relay and DLC10 cable are available. Otherwise fall back
to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
