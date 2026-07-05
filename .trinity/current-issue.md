# Wave Loop 425 — FPGA board evidence / relay gate / PVT falsification

**Issue:** #1374  
**Branch:** `wave-loop-425`  
**Milestone:** Continue the FPGA boot-evidence line from W424.

---

## Goal

Wave 424 hardened the FPGA tooling around instrument import, PVT context, CSV
voltage units, and non-blocking auto-continue. Wave 425 must produce real
boot evidence by executing the first available variant:

1. **Variant A (preferred when bench becomes available):**
   - Confirm P12 is wired to a logic-analyzer channel.
   - Program SPI flash with OSCFSEL=6 and OSCFSEL=7 variants.
   - Capture real CCLK during cold-POR boot for both variants.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <ctx.json>` and commit generated Lean theorems.
   - Document frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md` §3.6.21.

2. **Variant B (if hardware is partial):**
   - Import a real or representative CCLK/CSV/VCD capture with
     `--pvt-worstcase` validation.
   - Run `tri fpga boot-log --dry-run` or `cclk-sweep --dry-run` for OSCFSEL
     6/7 with `--pvt-context`.
   - Document the import recipe and PVT-context checklist in
     `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still fully blocked):**
   - Implement or defer real XADC readout in `tri fpga boot-log` / `cclk-sweep`.
   - Land the next safe gen-verilog #1245 sub-fix if one is narrow and
     regression-free; otherwise explicitly defer.
   - Harden boot-log / cold-por / cclk-sweep JSON schema and update the
     competitor snapshot.

---

## Decomposed plan

See `docs/reports/FPGA_LOOP_COOPERATION_W425_2026-07-05.md`.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` | Variant A capture import, B dry-run + PVT context, or C XADC/schema hardening |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | New measured theorems or PVT helpers |
| 3 | `fpga/HARDWARE_SSOT.md` | Updated capture / import / PVT protocol |
| 4 | `docs/reports/*` | W425 report, evidence, W426 cooperation |
| 5 | `.trinity/experience.md` | W425 learnings |
| 6 | git/PR | squash-merge to `master`, close #1374, open next issue for W426 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: Real CCLK capture for `OSCFSEL=6` and `OSCFSEL=7` exists.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is explicitly explained.
- [ ] AC-A4: Cold-POR SPI flash boot for OSCFSEL 6/7 is documented with STAT reads.

### Bundle B
- [ ] AC-B1: At least one real or representative capture is imported end-to-end and passes `--validate`.
- [ ] AC-B2: Dry-run boot-log / cclk-sweep artifacts include PVT/XADC context fields for OSCFSEL 6/7.
- [ ] AC-B3: `fpga/HARDWARE_SSOT.md` documents the import recipe and PVT-context checklist.

### Bundle C
- [ ] AC-C1: Real XADC readout is implemented or a documented deferral explains why it remains placeholder.
- [ ] AC-C2: `gen-verilog-yosys-smoke` failure count does not increase; any deferred #1245 sub-fix is explained.
- [ ] AC-C3: Boot-log / cold-por / cclk-sweep JSON schema is measurably more robust or better documented.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri fpga::tests` passes.

---

## PR

- Target: `master`
- Body: `Closes #1374`
- Report: `docs/reports/WAVE_LOOP_425_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W425_2026-07-05.md`
- Cooperation W426: `docs/reports/FPGA_LOOP_COOPERATION_W426_2026-07-05.md`

---

## Default variant

Execute **Variant A** if P12 is wired and a logic analyzer is available.  
Otherwise try **Variant B** if an external capture or partial board access is
available.  
Otherwise fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
