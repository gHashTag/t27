# Wave Loop 428 — FPGA boot-evidence next variant (physical CCLK / XADC readout / formal fallback)

**Issue:** #1383  
**Branch:** `wave-loop-428`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 427.

---

## Goal

Wave Loop 427 hardened the FPGA CLI and formal model while the bench was blocked
(P12 unwired, no relay gate). Wave Loop 428 executes the first available variant
from `docs/reports/FPGA_LOOP_COOPERATION_W428_2026-07-05.md`.

1. **Variant A (preferred when bench becomes available):**
   - Confirm P12 is wired to a logic-analyzer channel.
   - Program SPI flash with OSCFSEL=6 and OSCFSEL=7 variants.
   - Capture real CCLK during cold-POR boot for both variants.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <ctx.json>` and commit generated Lean theorems.
   - Document frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md` §3.6.

2. **Variant B (if board is reachable or external capture available):**
   - Add real XADC readout to `tri fpga boot-log` / `cclk-sweep` / `cold-por`
     over the existing JTAG path, so JSON `xadc` has `source: "xadc"` and live
     temp/vccint/vccaux values.
   - Alternatively, import at least one external CSV/VCD capture end-to-end.
   - Run dry-run or real cold-POR sweep for OSCFSEL 6/7 variants with
     `--pvt-context`.
   - Document the recipe in `fpga/HARDWARE_SSOT.md` §3.6.

3. **Variant C (fallback if bench still blocked):**
   - Extend the per-OSCFSEL PVT theorem library from W427 (e.g. implication
     theorems linking measured CCLK to transaction safety for any variant).
   - Land one safe gen-verilog #1245 sub-fix if narrow and regression-free;
     otherwise explicitly defer and update
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
   - Further harden `tri fpga` CLI or JSON output (e.g. extend
     `sweep-report --json` with additional machine-readable fields).
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if new 2026 competitor
     developments surface.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Add unified PVT envelope theorem `all_oscfsel_cclk_within_pvt_envelope` and implication theorem `measured_cclk_variant_implies_transaction_ok` |
| 2 | `cli/tri/src/fpga.rs` | Add `--json` output to `tri fpga pvt-envelope` with `cclk_period_ns`, `flash_min_half_period_ns`, `worst_case_pvt_context`, `margin_ns`, plus unit tests |
| 3 | `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` | Confirm the 7 residual #1245 failures are still tied to major features; explicitly defer any sub-fix for W428 |
| 4 | `docs/reports/T27_VS_FORMAL_HDL_2026.md` | Refresh competitor snapshot with Sparkle/Hesper, Clash 1.11.0 candidate, Chisel 7.13.0, Bluespec 2026.01, firtool 1.152.0, and emerging signals |
| 5 | `docs/reports/*` | W428 report, evidence, W429 cooperation |
| 6 | `.trinity/experience.md` | W428 learnings |
| 7 | git/PR | squash-merge to `master`, close issue, open #? for W429 |

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
- [ ] AC-B3: Boot-log artifacts for OSCFSEL 6/7 include PVT/XADC context.
- [ ] AC-B4: The captured/recorded operating point is linked to the W427 per-OSCFSEL envelope theorems.

### Bundle C
- [ ] AC-C1: At least one new PVT-related theorem is added and builds.
- [ ] AC-C2: One safe gen-verilog sub-fix lands without increasing the 7-failure yosys smoke count, or is explicitly deferred if unsafe.
- [ ] AC-C3: `tri fpga` CLI or JSON output is measurably more actionable than in W427.
- [ ] AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1383`
- Report: `docs/reports/WAVE_LOOP_428_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W428_YYYY-MM-DD.md`
- Cooperation W429: `docs/reports/FPGA_LOOP_COOPERATION_W429_YYYY-MM-DD.md`

---

## Default variant

Execute **Variant A** if P12 is wired and the analyzer is ready.  
Otherwise execute **Variant B** if the board is reachable for real XADC readout
or an external capture is available.  
Otherwise fall back to **Variant C**.

## Chosen variant

**Variant C** is selected for W428.

Rationale from the W428 start-of-wave probe:
- Bench is reachable via Digilent HS2 (`idcode 0x03636093`), but P12 is still
  unwired, no relay/remote-power gate is available, the DLC10 cable is still
  missing, and only OSCFSEL 0–5 bitstreams exist in `build/fpga/cclk_variants`
  (no 6/7 variants).
- No external CSV/VCD captures for OSCFSEL 6/7 were provided.
- `./scripts/tri test` baseline shows the same 7 pre-existing
  gen-verilog-yosys-smoke failures from weak point #1245; no new regressions.
- Therefore Variant A and Variant B are blocked, and Variant C is the only
  shippable path for W428.

---

*φ² + φ⁻² = 3 | TRINITY*
