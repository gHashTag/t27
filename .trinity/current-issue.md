# Wave Loop 425 — FPGA boot-evidence next variant (physical CCLK / real capture import / formal fallback)

**Issue:** #1374  
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
- [x] AC-C1: At least one new PVT grid / envelope / OSCFSEL-link theorem is added and builds.
- [x] AC-C2: One safe gen-verilog #1245 sub-fix lands without increasing the 7-failure yosys smoke count, or is explicitly deferred if unsafe.
- [x] AC-C3: `boot-log` / `cold-por` / `cclk-sweep` tooling is measurably more robust or better documented.
- [x] AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Invariant checks
- [x] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [x] `lake build Trinity.TernaryFPGABoot` passes.
- [x] `cargo test -p tri fpga::tests` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1376`
- Report: `docs/reports/WAVE_LOOP_426_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W426_2026-07-05.md`
- Cooperation W427: `docs/reports/FPGA_LOOP_COOPERATION_W427_2026-07-05.md`

---

## Default variant

Execute **Variant A** if P12 is wired and the analyzer is ready.  
Otherwise execute **Variant B** if an external capture is available or the board
is reachable for a dry-run boot-log.  
Otherwise fall back to **Variant C**.

## Chosen variant

**Variant C** is selected for W426. The bench probe confirms the XC7A200T board
is reachable via HS2 (`idcode 0x03636093`), but P12 remains unwired, no relay gate
is available, and no external capture was provided. Real XADC readout over the
HS2 path is too large and risky for a single wave.

## Detailed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Add a finite-grid PVT upper-envelope theorem (`pvt_half_ns_operating_rectangle_grid_bounded`) and a corollary for every OSCFSEL 0–7 worst-case PVT flash-spec transaction. |
| 2 | `cli/tri/src/fpga.rs` | Add `pvt_envelope_margin_ns` and `recommendation` fields to `SweepLog`; emit them in `cclk-sweep`, `cold-por`, and `boot-log` JSON; add unit tests. |
| 3 | `docs/reports/T27_VS_FORMAL_HDL_2026.md` | Refresh competitor snapshot with Sparkle July 2026 talk, CIRCT firtool 1.143.0, and Clash 1.8.5 verification fixes. |
| 4 | `docs/reports/W426_WEAK_POINTS_AND_COMPETITORS.md` | Document weak points and competitor scan (already created in PLAN phase). |
| 5 | `docs/reports/WAVE_LOOP_426_REPORT.md`, `FPGA_LOOP_EVIDENCE_W426_2026-07-05.md`, `FPGA_LOOP_COOPERATION_W427_2026-07-05.md` | Close-out artifacts. |
| 6 | `.trinity/experience.md` | W426 learnings. |
| 7 | git/PR | Commit, push, open PR #? closing #1376, create #? for W427. |

## Deferred items

- Real P12 CCLK capture and cold-POR boot for OSCFSEL 6/7 → W427 Variant A if P12
  is wired.
- Real XADC readout → W427 Variant B if a safe JTAG/HS2 path is validated.
- Gen-verilog #1245 sub-fix → deferred; remaining 7 failures are tied to major
  features, not narrow regression-free fixes.

---

*φ² + φ⁻² = 3 | TRINITY*
