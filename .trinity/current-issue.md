# Wave Loop 420 — physical CCLK capture, real relay gate, or instrument-import depth

**Issue:** #1361  
**Branch:** `wave-loop-420`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 419.

---

## Goal

Wave 419 closed the Variant C fallback (instrument-import parity, PVT
monotonicity, standalone lake workflow). Wave 420 re-evaluates the bench state
and executes the first available variant.

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
   - Extend instrument-import depth: VCD auto-threshold, CSV sample-rate
     auto-detection, or additional vendor header aliases.
   - Refine the PVT envelope if real N25Q128_3V timing curves become available,
     otherwise add another shape-preservation lemma.
   - Land one safe gen-verilog #1245 sub-fix that does not destabilize the
     existing 16-failure yosys smoke baseline.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` | Variant A import, B relay backend, or C instrument-import depth / gen-verilog sub-fix |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | New measured theorems or PVT shape lemma |
| 3 | `fpga/HARDWARE_SSOT.md` | Updated capture / relay / integration protocol |
| 4 | `docs/reports/*` | W420 report, evidence, W421 cooperation |
| 5 | `.trinity/experience.md` | W420 learnings |
| 6 | git/PR | squash-merge to `master`, close #1361, open #? for W421 |

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
- [x] AC-C1: VCD instrument-import unit tests land: exact `$end` token terminator regression and real-valued net auto-threshold.
- [x] AC-C2: New PVT envelope shape lemma/test lands: process-corner monotonicity (`ff ≤ tt ≤ ss`).
- [ ] AC-C3: One safe gen-verilog #1245 sub-fix lands without increasing the 16-failure yosys smoke count. (Deferred; the remaining tracked gap is RAM style inference, which is not a safe narrow sub-fix for a Variant C wave.)

### Invariant checks
- [x] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [x] `lake build Trinity.TernaryFPGABoot` passes.
- [x] `cargo test -p tri fpga::tests` passes.

---

## PR
- Target: `master`
- PR: to open after work
- Body: `Closes #1361`
- Report: `docs/reports/WAVE_LOOP_420_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W420_2026-07-06.md`
- Cooperation W421: `docs/reports/FPGA_LOOP_COOPERATION_W421_2026-07-06.md`

---

## Default variant

Execute **Variant A** if the analyzer and DLC10 cable are available. Otherwise
try **Variant B** if a relay and DLC10 cable are available. Otherwise fall back
to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
