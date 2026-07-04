# Wave Loop 416 — PVT-envelope CLI, VCD parser coverage, OSCFSEL transaction theorems

**Issue:** #1347  
**Branch:** `wave-loop-416`  
**Milestone:** Continue the FPGA boot-evidence line from W415.

---

## Goal

Wave 415 delivered the **Variant C** formal-tooling fallback because the bench
was still blocked. Wave 416 re-evaluates the bench state and executes the first
available variant.

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
   - Add a PVT-envelope CLI helper (`tri fpga pvt-envelope`).
   - Prove monotonicity of the PVT derating functions in Lean 4.
   - Extend VCD parser coverage for escaped identifiers, scalar x/z transitions,
     and hex bus literals.
   - Link the OSCFSEL 0..7 nominal theorems to
     `transaction_satisfies_flash_spec` proofs.

---

## Decomposed plan

See `.claude/plans/wave-loop-416.md` and
`docs/reports/FPGA_LOOP_COOPERATION_W417_2026-07-01.md`.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `.claude/plans/wave-loop-416.md` | Decomposed plan + weak points + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` | Updated PVT-envelope / VCD protocol |
| 3 | `cli/tri/src/fpga.rs` | `pvt-envelope` subcommand + VCD parser hardening |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | PVT monotonicity + OSCFSEL transaction theorems |
| 5 | `docs/reports/*` | W416 report, evidence, W417 cooperation |
| 6 | `.trinity/experience.md` | W416 learnings |
| 7 | git/PR | squash-merge to `master`, close #1347, open #1348 |

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
- [ ] AC-C1: `tri fpga pvt-envelope` works with and without `--pvt-context`.
- [ ] AC-C2: VCD parser unit tests cover escaped identifiers, scalar x/z, and hex bus literals.
- [ ] AC-C3: OSCFSEL 0..7 nominal rates are linked to `transaction_satisfies_flash_spec` proofs.
- [ ] AC-C4: PVT derating monotonicity lemmas build in Lean 4.

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
