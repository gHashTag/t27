# Wave Loop 426 Cooperation Variants

**Date:** 2026-07-05  
**For:** issue #? (to be created after W425 lands)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 426 continues the FPGA boot-evidence line. The preferred variant is
always physical bench work; if the bench remains blocked, the fallback is another
round of formal/tooling hardening that prepares the ground for future captures.

**Default selection rule:**

1. Execute **Variant A** if P12 is wired and the analyzer is ready.
2. Otherwise execute **Variant B** if the board is reachable for a dry-run with
   real XADC readout, or an external OSCFSEL 6/7 capture can be imported.
3. Otherwise fall back to **Variant C**.

---

## Variant A — Physical CCLK capture + cold-POR boot for OSCFSEL 6/7

**Trigger:** P12 is wired to a logic-analyzer channel and the relay/remote-power
gate is available.

### Work

1. Wire P12 to a logic-analyzer channel and verify clean edges at the board.
2. Program the XC7A200T SPI flash with the OSCFSEL=6 variant
   (`tri fpga flash fpga/verilog/ternary_mac_demo_top_200t_oscfsel06.bit`).
3. Capture the CCLK waveform during cold-POR boot.
4. Import the capture:
   ```bash
   tri fpga measured-to-lean --csv capture.csv --raw-ns --standalone \
     --validate --pvt-context pvt_worst_case.json
   ```
5. Repeat for OSCFSEL=7.
6. Commit the generated Lean theorems.

### Acceptance criteria

- AC-A1: Real captures for OSCFSEL=6 and OSCFSEL=7 exist.
- AC-A2: Imported theorems build with `lake build`.
- AC-A3: Each capture satisfies the PVT-aware flash spec, or any exceedance is
  explicitly explained.
- AC-A4: Cold-POR SPI flash boot for OSCFSEL=6/7 is documented with STAT reads.

### Files touched

- `fpga/HARDWARE_SSOT.md` §3.6.21
- `docs/reports/FPGA_LOOP_EVIDENCE_W426_*.md`
- generated Lean files under `proofs/lean4/Trinity/`

---

## Variant B — Real XADC readout or external capture import

**Trigger:** The board is reachable (HS2 cable + openFPGALoader) but P12 is not
wired, or an external OSCFSEL 6/7 capture is available for import.

### Work

1. Add real XADC readout to `tri fpga boot-log` / `cclk-sweep` / `cold-por` over
   the existing JTAG path, so the JSON `xadc` object has
   `source: "xadc"` and live `temp_c`, `vccint_mv`, `vccaux_mv` values.
2. Alternatively, import one or more external CSV/VCD captures end-to-end using
   the W423–W425 unit/voltage-unit/noise handling.
3. Run a dry-run cold-POR boot-log for OSCFSEL 6/7 variants with
   `--pvt-context`.
4. Document the import recipe in `fpga/HARDWARE_SSOT.md` §3.6.21.

### Acceptance criteria

- AC-B1: Real XADC readout lands, OR at least one external capture is imported
  end-to-end.
- AC-B2: The import path exposes no unhandled unit, voltage-unit, or noise
  cases.
- AC-B3: Dry-run boot-log artifacts for OSCFSEL 6/7 include PVT/XADC context.

### Files touched

- `cli/tri/src/fpga.rs` (XADC readout)
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W426_*.md`

---

## Variant C — Formal/tooling fallback

**Trigger:** P12 is still unwired and no board access is available.

### Work

1. Extend the PVT formal model:
   - Add a 2-D/3-D operating-rectangle grid theorem showing that the worst-case
     corner dominates every grid point.
   - Add a theorem linking the PVT-aware half-period bound to the measured CCLK
     predicate for every OSCFSEL 0–7 variant.
2. Land one safe gen-verilog #1245 sub-fix from the remaining 7 yosys smoke
   failures, if any is narrow and regression-free; otherwise explicitly defer.
3. Harden the `tri fpga` JSON schema and decision-tree output (e.g., include
   the recommended next action, PVT envelope margin, andOSC FSEL first-working
   variant in a machine-readable field).
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new 2026 competitor
   developments surface.

### Acceptance criteria

- AC-C1: At least one new PVT grid or envelope theorem is added and builds.
- AC-C2: One safe gen-verilog sub-fix lands without increasing the 7-failure
  yosys smoke count, or is explicitly deferred if unsafe.
- AC-C3: `boot-log` / `cold-por` / `cclk-sweep` JSON or CLI output is
  measurably more robust or better documented.
- AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Files touched

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `cli/tri/src/fpga.rs`
- `bootstrap/src/compiler.rs` (if a safe gen-verilog fix is feasible)
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`

---

## Default selection

**Variant C** is the current best default for W426, because the hardware blockers
that forced W425 Variant C are still present. The moment P12 is wired or an
external capture becomes available, switch to **Variant A** or **Variant B**
respectively.

---

*φ² + φ⁻² = 3 | TRINITY*
