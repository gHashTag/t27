# Wave Loop 428 Cooperation Variants

**Date:** 2026-07-05  
**For:** issue to be created after W427 lands  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 428 continues the FPGA boot-evidence line. The preferred variant is
always physical bench work; if the bench remains blocked, the fallback is another
round of formal/tooling hardening that keeps the `tri fpga` → Lean 4 PVT theorem
loop moving.

**Default selection rule:**

1. Execute **Variant A** if P12 is wired and the analyzer is ready.
2. Otherwise execute **Variant B** if the board is reachable for real XADC
   readout over HS2, or an external OSCFSEL 6/7 capture can be imported.
3. Otherwise fall back to **Variant C**.

---

## Variant A — Physical CCLK capture + cold-POR boot for OSCFSEL 6/7

**Trigger:** P12 is wired to a logic-analyzer channel and a relay/remote-power
gate is available.

### Work

1. Wire P12 to a logic-analyzer channel and verify clean 3.3 V edges at the
   board.
2. Program the XC7A200T SPI flash with the OSCFSEL=6 variant:
   ```bash
   tri fpga flash fpga/verilog/ternary_mac_demo_top_200t_oscfsel06.bit
   ```
3. Capture the CCLK waveform during cold-POR boot with at least 100× sample rate.
4. Import the capture end-to-end:
   ```bash
   tri fpga measured-to-lean --csv capture_oscfsel06.csv --raw-ns --standalone \
     --validate --pvt-context pvt_worst_case.json
   ```
5. Repeat for OSCFSEL=7.
6. Run `tri fpga cold-por` or `tri fpga smoke-gate` for each working variant and
   commit the STAT logs.
7. Add a Lean theorem per captured variant that applies the finite-grid PVT lemma
   from W426 and the per-OSCFSEL envelope theorems from W427.

### Acceptance criteria

- AC-A1: Real captures for OSCFSEL=6 and OSCFSEL=7 exist.
- AC-A2: Imported theorems build with `lake build`.
- AC-A3: Each capture satisfies the PVT-aware flash spec, or any exceedance is
  explicitly explained and bounded.
- AC-A4: Cold-POR SPI flash boot for OSCFSEL=6/7 is documented with STAT reads.

### Files touched

- `fpga/HARDWARE_SSOT.md` §3.6
- `docs/reports/FPGA_LOOP_EVIDENCE_W428_*.md`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- generated Lean files under `proofs/lean4/Trinity/`

---

## Variant B — Real XADC readout or external capture import

**Trigger:** The board is reachable (HS2 cable + openFPGALoader) but P12 is not
wired, or an external OSCFSEL 6/7 capture is available for import.

### Work

1. Implement real XADC readout over the existing JTAG/FTDI HS2 path so that
   `tri fpga boot-log` / `cclk-sweep` / `cold-por` emit `xadc.source: "xadc"`
   with live `temp_c`, `vccint_mv`, and `vccaux_mv` values. If openFPGALoader
   cannot do this directly, fall back to a small JTAG XADC register access helper
   using the `ftdi` crate or `xc3sprog`-style JTAG bit-banging.
2. Alternatively, import one or more external CSV/VCD captures end-to-end using
   the hardened W423–W427 path (`--csv-voltage-unit`, slope filters,
   unknown-timescale fallbacks, `--json` sweep reports).
3. Run a dry-run or real cold-POR sweep for OSCFSEL 6/7 variants with
   `--pvt-context` and verify the JSON report round-trips.
4. Document the XADC or import recipe in `fpga/HARDWARE_SSOT.md` §3.6.

### Acceptance criteria

- AC-B1: Real XADC readout lands, OR at least one external capture is imported
  end-to-end.
- AC-B2: The import path exposes no unhandled unit, voltage-unit, or noise
  cases.
- AC-B3: Boot-log artifacts for OSCFSEL 6/7 include live or supplied PVT/XADC
  context.
- AC-B4: The captured/recorded operating point is linked to the W426 finite-grid
  PVT lemma and the W427 per-OSCFSEL envelope theorems.

### Files touched

- `cli/tri/src/fpga.rs` (XADC readout or external-import hardening)
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W428_*.md`

---

## Variant C — Formal/tooling fallback

**Trigger:** P12 is still unwired and no board access is available.

### Work

1. **PVT theorem library extension.** Build on W427's per-OSCFSEL envelope theorems
   by adding implication theorems that connect `measured_cclk_ok` for a captured
   variant to `transaction_satisfies_flash_spec`, and by proving the full
   0..7 OSCFSEL table satisfies the PVT-aware flash spec without per-variant
   manual cases.
2. **Safe gen-verilog #1245 sub-fix.** Re-evaluate the 7 residual yosys smoke
   failures and land exactly one narrow, regression-free fix. If none is safe,
   explicitly defer and update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
3. **`tri fpga` CLI hardening.** Extend `sweep-report --json` with additional
   machine-readable fields (e.g. `worst_case_pvt_context`, `cclk_period_ns`,
   `flash_min_half_period_ns`) or add a `--recommendation` flag to the text
   summary.
4. **Competitor watch.** Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any
   new Sparkle/Verilean, Clash, or CIRCT release surfaces during the wave.

### Acceptance criteria

- AC-C1: At least one new PVT-related theorem is added and builds.
- AC-C2: One safe gen-verilog sub-fix lands without increasing the 7-failure
  yosys smoke count, or is explicitly deferred if unsafe.
- AC-C3: `tri fpga` CLI or JSON output is measurably more actionable than in
  W427.
- AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Files touched

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `cli/tri/src/fpga.rs`
- `bootstrap/src/compiler.rs` (only if a safe fix is feasible)
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`

---

## Default selection

**Variant C** is the current best default for W428, because the hardware blockers
that forced W425/W426/W427 Variant C are still present. The moment P12 is wired or
an external capture becomes available, switch to **Variant A** or **Variant B**
respectively.

---

## Cross-wave themes to keep alive

- **Physical boot evidence** is the headline t27 differentiation; never let a wave
  pass without advancing it (capture, import, theorem, or tooling).
- **PVT-aware formal link** must stay falsifiable: every coefficient and margin
  value should be traceable to a datasheet number or marked as a conservative
  placeholder.
- **Gen-verilog safety** is more important than speed: one narrow fix per wave is
  preferable to a broad refactor that risks regressions.
- **Competitor watch** should be a standing task; Sparkle/Verilean is moving
  fastest and is the primary threat.
- **Machine-readable CLI output** is now a maintained contract; any new `tri fpga`
  command or report should consider JSON and a closed recommendation vocabulary
  from the start.

---

*φ² + φ⁻² = 3 | TRINITY*
