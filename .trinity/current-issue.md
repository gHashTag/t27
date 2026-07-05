# Wave Loop 423 — FPGA boot-evidence next variant (physical CCLK / instrument depth / gen-verilog narrowing)

**Issue:** #1368  
**Branch:** `wave-loop-423`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 422.

---

## Goal

Wave 422 re-established live contact with the XC7A200T board (SRAM load OK,
STAT `0x401079FC`, real XADC context) but could not complete the full Variant A
plan because pin P12 is not wired to a logic analyzer. Wave 423 executes the
first available variant from `docs/reports/FPGA_LOOP_COOPERATION_W423_2026-07-06.md`.

1. **Variant A (preferred when bench becomes available):**
   - Confirm P12 is wired to a logic-analyzer channel.
   - Capture real CCLK for `OSCFSEL=6` and `OSCFSEL=7`.
   - Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
     --standalone --validate --pvt-context <ctx.json>` and commit the generated
     Lean theorems.
   - Program each OSCFSEL variant to SPI flash and perform a true cold-POR boot.
   - Document measured frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md`.

2. **Variant B (if an external VCD/CSV capture is available, no on-bench relay):**
   - Add CSV timestamp-column parsing for fractional seconds, milliseconds, and
     sample-number-only exports.
   - Add VCD real-net slope filter: reject transitions where ΔV is below a noise
     window or Δt is below a configurable `t_setup`.
   - Add `tri fpga measured-to-lean --pvt-worstcase` mode using the combined
     monotonicity corner (max temp, min VCCINT, ss corner).
   - Document the multi-format import matrix in `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still blocked):**
   - Extend VCD robustness: detect/report unknown `$timescale` units, handle
     `$dumpoff`/`$dumpon` without a preceding `#timestamp`.
   - Land one safe narrow gen-verilog #1245 sub-fix from the remaining 7 failures,
     if it does not increase the yosys smoke failure count.
   - Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new 2026
     developments.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` or `bootstrap/src/compiler.rs` | Variant A import, B instrument depth, or C parser/gen-verilog hardening |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | New measured theorems or small formal helpers |
| 3 | `fpga/HARDWARE_SSOT.md` / `docs/reports` | Updated protocol or comparison note |
| 4 | `docs/reports/*` | W423 report, evidence, W424 cooperation |
| 5 | `.trinity/experience.md` | W423 learnings |
| 6 | git/PR | squash-merge to `master`, close #1368, open #? for W424 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: Real CCLK capture for `OSCFSEL=6` and `OSCFSEL=7` exists.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is explicitly explained.
- [ ] AC-A4: Cold-POR SPI flash boot for OSCFSEL 6/7 is documented with STAT reads.

### Bundle B
- [ ] AC-B1: CSV fractional-second / millisecond / sample-number timestamp columns are parsed correctly with a regression test.
- [ ] AC-B2: VCD real-net slope filter rejects noisy transitions with a regression test.
- [ ] AC-B3: `--pvt-worstcase` mode validates against the combined-monotonicity corner with a regression test.

### Bundle C
- [ ] AC-C1: VCD parser hardening lands with unit tests (unknown timescale unit handling, or dumpoff without timestamp).
- [ ] AC-C2: One safe gen-verilog #1245 sub-fix lands without increasing the 7-failure yosys smoke count, or is explicitly deferred if unsafe.
- [ ] AC-C3: Competitor snapshot is updated if any new 2026 developments are found.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri fpga::tests` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1368`
- Report: `docs/reports/WAVE_LOOP_423_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W423_YYYY-MM-DD.md`
- Cooperation W424: `docs/reports/FPGA_LOOP_COOPERATION_W424_YYYY-MM-DD.md`

---

## Default variant

Execute **Variant A** if P12 is wired and the analyzer is ready.  
Otherwise try **Variant B** if an external capture file is available.  
Otherwise fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
