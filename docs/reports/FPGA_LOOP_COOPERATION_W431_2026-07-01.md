# Wave Loop 431 Cooperation Variants

**Date:** 2026-07-01  
**For:** issue to be created after W430 lands  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 431 continues the FPGA boot-evidence line. W430 added live XADC
readout and a formal PVT-envelope bridge; W431 should either close a physical
measurement loop (P12 capture or real cold-POR sweep with XADC) or, if the
bench is still blocked, extend the formal bridge and chip away at the
gen-verilog backlog.

**Default selection rule:**

1. Execute **Variant A** if P12 is wired and a relay/remote-power gate is ready.
2. Otherwise execute **Variant B** if the board is reachable for a manual
   cold-POR `cclk-sweep --xadc`, or an external OSCFSEL 6/7 capture can be
   imported.
3. Otherwise fall back to **Variant C**.

---

## Variant A — Physical CCLK capture + cold-POR boot for OSCFSEL 6/7 with live XADC

**Trigger:** P12 is wired to a logic-analyzer channel and a relay/remote-power
gate is available.

### Work

1. Wire P12 to a logic-analyzer channel and verify clean 3.3 V edges at the
   board.
2. Program the XC7A200T SPI flash with OSCFSEL=6 and OSCFSEL=7 variants using
   `tri fpga flash` or `tri fpga cclk-sweep`.
3. Capture the CCLK waveform during cold-POR boot with at least 100× sample rate.
4. Import the capture end-to-end and emit both the Lean snippet and a
   machine-readable JSON summary:
   ```bash
   tri fpga measured-to-lean --csv capture_oscfsel06.csv --raw-ns --standalone \
     --validate --pvt-context pvt_worst_case.json --out oscfsel06.lean --json
   ```
5. Run `tri fpga cclk-sweep ... --xadc` so each log embeds the live operating
   point, and commit the boot logs.
6. Link each captured theorem to the W429 raw-ns OSCFSEL theorems and the W430
   XADC/PVT envelope theorem.

### Acceptance criteria

- AC-A1: Real captures for OSCFSEL=6 and OSCFSEL=7 exist.
- AC-A2: Imported theorems build with `lake build` and the `--json` summaries
  round-trip.
- AC-A3: Each capture satisfies the PVT-aware flash spec, or any exceedance is
  explicitly explained and bounded.
- AC-A4: Cold-POR SPI flash boot for OSCFSEL=6/7 is documented with STAT reads
  and live XADC values.

### Files touched

- `fpga/HARDWARE_SSOT.md` §3.6 / §9.6
- `docs/reports/FPGA_LOOP_EVIDENCE_W431_*.md`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- generated Lean files under `proofs/lean4/Trinity/`

---

## Variant B — Real cold-POR sweep with XADC, or external capture import

**Trigger:** The board is reachable (HS2 cable + openFPGALoader) but P12 is not
wired, or an external OSCFSEL 6/7 capture is available for import.

### Work

1. Run a real `tri fpga cclk-sweep` over OSCFSEL 0..7 with `--xadc` and a
   supplied `--pvt-context`, performing the manual power cycle at each variant.
   Verify the JSON logs contain `xadc.source: "xadc"` and identify the first
   working variant.
2. Alternatively, import one or more external CSV/VCD captures end-to-end using
   the hardened W423–W430 path (`--csv-voltage-unit`, slope filters,
   unknown-timescale fallbacks, `--json` measured-to-lean summaries).
3. Add a Lean theorem or a decidability lemma that checks a concrete XADC JSON
   operating point against `xadc_operating_point_within_envelope`, closing the
   JSON → proof loop.
4. Document the sweep/import recipe and the first-working OSCFSEL in
   `fpga/HARDWARE_SSOT.md`.

### Acceptance criteria

- AC-B1: Real `cclk-sweep --xadc` logs exist for at least OSCFSEL 0..3, OR at
  least one external capture is imported end-to-end with a `--json` summary.
- AC-B2: The recorded operating point is linked to the W430
  `xadc_operating_point_envelope_implies_worst_case_bound` theorem.
- AC-B3: The import/sweep path exposes no unhandled unit, voltage-unit, or
  noise cases.
- AC-B4: `lake build Trinity.TernaryFPGABoot` passes with any new theorem.

### Files touched

- `cli/tri/src/fpga.rs`
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W431_*.md`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`

---

## Variant C — Formal/tooling fallback

**Trigger:** P12 is still unwired and no board access is available.

### Work

1. **XADC theorem library extension.** Build on W430 by adding:
   - a decidability/computability lemma that evaluates a concrete
     `XadcOperatingPoint` against the envelope, or
   - an implication theorem that connects a measured raw-ns capture plus a
     measured XADC operating point to the existing PVT-aware transaction theorem.
2. **Safe gen-verilog #1245 sub-fix.** Re-evaluate the 7 residual yosys smoke
   failures and land exactly one narrow, regression-free fix. If none is safe,
   explicitly defer and update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
3. **`tri fpga measured-to-lean` CLI hardening.** Extend the `--json` summary
   with additional machine-readable fields (e.g.
   `flash_min_half_period_ns`, `margin_ns`, `recommendation`) or add a `--quiet`
   mode suitable for CI consumption.
4. **Competitor watch.** Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any
   new Sparkle/Verilean, Clash, or CIRCT release surfaces during the wave.

### Acceptance criteria

- AC-C1: At least one new XADC/PVT-related theorem is added and builds.
- AC-C2: One safe gen-verilog sub-fix lands without increasing the 7-failure
  yosys smoke count, or is explicitly deferred if unsafe.
- AC-C3: `tri fpga measured-to-lean` JSON output is measurably more actionable
  than in W430.
- AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Files touched

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `cli/tri/src/fpga.rs`
- `bootstrap/src/compiler.rs` (only if a safe fix is feasible)
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

---

## Default selection

**Variant B** is the current best default for W431, because the board is
reachable over HS2 and `--read-xadc` now works, while P12 and the relay gate are
still unavailable. If either hardware blocker clears, switch to **Variant A**;
if the bench becomes completely unreachable, switch to **Variant C**.

---

## Cross-wave themes to keep alive

- **Physical boot evidence** is the headline t27 differentiation; never let a
  wave pass without advancing it (capture, import, theorem, or tooling).
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
