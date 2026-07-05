# Wave Loop 429 — FPGA boot-evidence next variant (physical CCLK / XADC readout / formal fallback)

**Issue:** #1385  
**Branch:** `wave-loop-429`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 428.

---

## Goal

Wave Loop 428 hardened the FPGA CLI and formal model while the bench was blocked
(P12 unwired, no relay gate, no DLC10 cable, no OSCFSEL 6/7 bitstreams, no
external captures). Wave Loop 429 executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W429_2026-07-05.md`.

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
   - Extend the unified OSCFSEL PVT theorem library from W428 (e.g. a theorem
     linking a measured raw-ns capture to the unified
     `cclk_variant_implies_transaction_ok` family when the period matches a
     nominal variant within tolerance).
   - Land one safe gen-verilog #1245 sub-fix if narrow and regression-free;
     otherwise explicitly defer and update
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
   - Further harden `tri fpga` CLI or JSON output (e.g. add `--json` to
     `measured-to-lean` summary or enrich `pvt-envelope --json`).
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if new 2026 competitor
     developments surface.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Extend unified OSCFSEL theorem family with measured-to-unified link or whole-table worst-case theorem |
| 2 | `cli/tri/src/fpga.rs` or `bootstrap/src/compiler.rs` | Land one safe gen-verilog #1245 sub-fix, or update deferral note |
| 3 | `cli/tri/src/fpga.rs` | Extend machine-readable `tri fpga` output (e.g. `--json` for `measured-to-lean` summary) |
| 4 | `docs/reports/T27_VS_FORMAL_HDL_2026.md` | Refresh competitor snapshot if new signals appear |
| 5 | `docs/reports/*` | W429 report, evidence, W430 cooperation |
| 6 | `.trinity/experience.md` | W429 learnings |
| 7 | git/PR | squash-merge to `master`, close issue, open #? for W430 |

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
- [ ] AC-B4: The captured/recorded operating point is linked to the W428 unified OSCFSEL theorems.

### Bundle C
- [ ] AC-C1: At least one new PVT-related theorem is added and builds.
- [ ] AC-C2: One safe gen-verilog sub-fix lands without increasing the 7-failure yosys smoke count, or is explicitly deferred if unsafe.
- [ ] AC-C3: `tri fpga` CLI or JSON output is measurably more actionable than in W428.
- [ ] AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1385`
- Report: `docs/reports/WAVE_LOOP_429_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W429_YYYY-MM-DD.md`
- Cooperation W430: `docs/reports/FPGA_LOOP_COOPERATION_W430_YYYY-MM-DD.md`

---

## Default variant

Execute **Variant A** if P12 is wired and the analyzer is ready.  
Otherwise execute **Variant B** if an external capture is available or the board
is reachable for a dry-run boot-log.  
Otherwise fall back to **Variant C**.

## Chosen variant

**Variant C** is the current best default for W429.

Rationale from the W428 close-out:
- Bench is reachable via Digilent HS2 (`idcode 0x03636093`), but P12 is still
  unwired, no relay/remote-power gate is available, the DLC10 cable is still
  missing, and only OSCFSEL 0–5 bitstreams exist in `build/fpga/cclk_variants`
  (no 6/7 variants).
- No external CSV/VCD captures for OSCFSEL 6/7 were provided.
- `./scripts/tri test` baseline shows the same 7 pre-existing
  gen-verilog-yosys-smoke failures from weak point #1245; no new regressions.
- Therefore Variant A and Variant B are blocked until hardware state changes.

---

*φ² + φ⁻² = 3 | TRINITY*
