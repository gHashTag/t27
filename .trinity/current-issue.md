# Wave Loop 414 — FPGA physical capture, real relay gate, or further formal tooling

**Issue:** #1342  
**Branch:** `wave-loop-414`  
**Milestone:** Continue the FPGA boot-evidence line from W413.

---

## Goal

Wave 413 delivered the **Variant C** formal-tooling fallback because the bench
was blocked. Wave 414 re-evaluates the bench state and executes the first
available variant.

1. **Variant A (preferred when bench becomes available):**
   - Wire P12 to a logic-analyzer channel and capture real CCLK for
     `OSCFSEL=6` and `OSCFSEL=7`.
   - Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
     --standalone` and commit the generated Lean theorems.
   - Document the measured frequencies/duty cycles in `fpga/HARDWARE_SSOT.md`.

2. **Variant B (if relay hardware is available):**
   - Implement a real `--relay-port` backend for `tri fpga cold-por`
     (e.g. serial or TCP relay controlling board power).
   - Perform an automated cold-POR power-cycle and capture STAT without
     operator intervention.
   - Document relay wiring in `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still blocked):**
   - Replace the single-constant 2× PVT placeholder with a temperature /
     voltage-aware uncertainty envelope.
   - Extend the VCD parser to multi-bit buses and analog real-valued traces.
   - Add `--validate` to `measured-to-lean --raw-ns` to reject instrument
     exports that would produce false theorems.

---

## Decomposed plan

See `.claude/plans/wave-loop-414.md` and
`docs/reports/FPGA_LOOP_COOPERATION_W414_2026-07-04.md`.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `.claude/plans/wave-loop-414.md` | Decomposed plan + weak points + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` | Updated capture / relay protocol |
| 3 | `cli/tri/src/fpga.rs` | Variant A CSV/VCD capture import, B relay backend, or C parser/validate extensions |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | PVT envelope or new example theorems |
| 5 | `docs/reports/*` | W414 report, evidence, W415 cooperation |
| 6 | `.trinity/experience.md` | W414 learnings |
| 7 | git/PR | squash-merge to `master`, close #1342, open #1343 |

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
- [ ] AC-C1: PVT model depends on at least temperature and voltage bounds, not a single constant.
- [ ] AC-C2: VCD parser handles scalar and multi-bit logic traces.
- [ ] AC-C3: `measured-to-lean --raw-ns --validate` rejects captures that violate the flash spec.

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
