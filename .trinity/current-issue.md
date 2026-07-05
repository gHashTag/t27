# Wave Loop 425 — FPGA boot-evidence next variant (physical CCLK / real capture import / formal fallback)

**Issue:** #1372  
**Branch:** `wave-loop-425`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 424.

---

## Goal

Wave 424 hardened the FPGA CLI so that `boot-log`, `cold-por`, and
`cclk-sweep` auto-continue, embed PVT/XADC context, and import CSV captures in
volts or millivolts. Wave 425 executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W425_2026-07-05.md`.

1. **Variant A (preferred when bench becomes available):**
   - Confirm P12 is wired to a logic-analyzer channel.
   - Capture real CCLK for `OSCFSEL=6` and `OSCFSEL=7`.
   - Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
     --standalone --validate --pvt-context <ctx.json>` and commit the generated
     Lean theorems.
   - Program each OSCFSEL variant to SPI flash and perform a true cold-POR boot.
   - Document measured frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md` §3.6.21.

2. **Variant B (if an external capture is available or the board is reachable for dry-run boot-log):**
   - Import at least one real or representative CCLK capture end-to-end using the
     W423–W424 unit/noise/voltage-unit handling.
   - Add any missing parser handling exposed by the real export.
   - Run a dry-run cold-POR boot-log for OSCFSEL 6/7 variants with `--pvt-context`.
   - Document the import recipe in `fpga/HARDWARE_SSOT.md` §3.6.21.

3. **Variant C (fallback if bench still blocked):**
   - Implement real XADC readout in `tri fpga boot-log` / `cclk-sweep` so the JSON
     `xadc` object has `source: "xadc"` and live temp/vccint/vccaux values; or
     document the deferral if it is unsafe for the branch.
   - Land the next safe gen-verilog #1245 sub-fix from the remaining 7 failures,
     if one is narrow and regression-free; otherwise explicitly defer.
   - Continue hardening `tri fpga boot-log` / `cold-por` / `cclk-sweep` JSON schema
     and decision-tree output.
   - Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new 2026 competitor
     developments surface.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` or `bootstrap/src/compiler.rs` or `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Variant A capture import, B import/dry-run, or C parser/formal/gen-verilog/XADC hardening |
| 2 | `fpga/HARDWARE_SSOT.md` / `docs/reports` | Updated protocol or comparison note |
| 3 | `docs/reports/*` | W425 report, evidence, W426 cooperation |
| 4 | `.trinity/experience.md` | W425 learnings |
| 5 | git/PR | squash-merge to `master`, close issue, open #? for W426 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: Real CCLK capture for `OSCFSEL=6` and `OSCFSEL=7` exists.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is explicitly explained.
- [ ] AC-A4: Cold-POR SPI flash boot for OSCFSEL 6/7 is documented with STAT reads.

### Bundle B
- [ ] AC-B1: At least one real or representative CCLK/CSV/VCD capture is imported end-to-end.
- [ ] AC-B2: The import path exposes no unhandled unit, voltage-unit, or noise cases.
- [ ] AC-B3: Dry-run boot-log artifacts exist for OSCFSEL 6/7 and include PVT/XADC context fields.

### Bundle C
- [ ] AC-C1: Real XADC readout lands in boot-log/cclk-sweep JSON, or the deferral is documented.
- [ ] AC-C2: `boot-log` / `cold-por` / `cclk-sweep` tooling is measurably more robust or better documented.
- [ ] AC-C3: One safe gen-verilog #1245 sub-fix lands without increasing the 7-failure yosys smoke count, or is explicitly deferred if unsafe.
- [ ] AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri fpga::tests` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1372`
- Report: `docs/reports/WAVE_LOOP_425_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W425_YYYY-MM-DD.md`
- Cooperation W426: `docs/reports/FPGA_LOOP_COOPERATION_W426_YYYY-MM-DD.md`

---

## Default variant

Execute **Variant A** if P12 is wired and the analyzer is ready.  
Otherwise execute **Variant B** if an external capture is available or the board
is reachable for a dry-run boot-log.  
Otherwise fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
