# Wave Loop 426 — FPGA boot-evidence next variant (physical CCLK / XADC readout / formal fallback)

**Issue:** #1376  
**Branch:** `wave-loop-426`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 425.

---

## Goal

Wave 425 hardened the FPGA CLI and formal model while the bench was blocked
(P12 unwired, no relay gate). Wave 426 executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W426_2026-07-05.md`.

1. **Variant A (preferred when bench becomes available):**
   - Confirm P12 is wired to a logic-analyzer channel.
   - Program SPI flash with OSCFSEL=6 and OSCFSEL=7 variants.
   - Capture real CCLK during cold-POR boot for both variants.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <ctx.json>` and commit generated Lean theorems.
   - Document frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md` §3.6.21.

2. **Variant B (if board is reachable or external capture available):**
   - Add real XADC readout to `tri fpga boot-log` / `cclk-sweep` / `cold-por`
     over the existing JTAG path, so JSON `xadc` has `source: "xadc"` and live
     temp/vccint/vccaux values.
   - Alternatively, import at least one external CSV/VCD capture end-to-end.
   - Run dry-run cold-POR boot-log for OSCFSEL 6/7 variants with
     `--pvt-context`.
   - Document the recipe in `fpga/HARDWARE_SSOT.md` §3.6.21.

3. **Variant C (fallback if bench still blocked):**
   - Extend the PVT formal model with an operating-rectangle grid theorem or a
     measured-CCLK link theorem for every OSCFSEL 0–7 variant.
   - Land one safe gen-verilog #1245 sub-fix from the remaining 7 failures, if
     narrow and regression-free; otherwise explicitly defer.
   - Harden `tri fpga` JSON schema / decision-tree output.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if new 2026 competitor
     developments surface.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` or `proofs/lean4/Trinity/TernaryFPGABoot.lean` or `bootstrap/src/compiler.rs` | Variant A capture import, B XADC readout/capture import, or C formal/gen-verilog hardening |
| 2 | `fpga/HARDWARE_SSOT.md` / `docs/reports` | Updated protocol or comparison note |
| 3 | `docs/reports/*` | W426 report, evidence, W427 cooperation |
| 4 | `.trinity/experience.md` | W426 learnings |
| 5 | git/PR | squash-merge to `master`, close issue, open #? for W427 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: Real CCLK capture for `OSCFSEL=6` and `OSCFSEL=7` exists.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is explicitly explained.
- [ ] AC-A4: Cold-POR SPI flash boot for OSCFSEL 6/7 is documented with STAT reads.

### Bundle B
- [ ] AC-B1: Real XADC readout lands, OR at least one external capture is imported end-to-end.
- [ ] AC-B2: The import/readout path exposes no unhandled unit, voltage-unit, or noise cases.
- [ ] AC-B3: Dry-run boot-log artifacts exist for OSCFSEL 6/7 and include PVT/XADC context.

### Bundle C
- [ ] AC-C1: At least one new PVT grid / envelope / OSCFSEL-link theorem is added and builds.
- [ ] AC-C2: One safe gen-verilog #1245 sub-fix lands without increasing the 7-failure yosys smoke count, or is explicitly deferred if unsafe.
- [ ] AC-C3: `boot-log` / `cold-por` / `cclk-sweep` tooling is measurably more robust or better documented.
- [ ] AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri fpga::tests` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1376`
- Report: `docs/reports/WAVE_LOOP_426_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W426_YYYY-MM-DD.md`
- Cooperation W427: `docs/reports/FPGA_LOOP_COOPERATION_W427_YYYY-MM-DD.md`

---

## Default variant

Execute **Variant A** if P12 is wired and the analyzer is ready.  
Otherwise execute **Variant B** if the board is reachable for real XADC readout
or an external capture is available.  
Otherwise fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
