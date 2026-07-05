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
| 1 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Extend per-OSCFSEL PVT theorem library (implication theorems or full table closure) |
| 2 | `cli/tri/src/fpga.rs` | Further `tri fpga` CLI/JSON hardening (e.g. additional sweep-report JSON fields) |
| 3 | `bootstrap/src/compiler.rs` or `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` | One safe gen-verilog #1245 sub-fix, or explicit deferral update |
| 4 | `docs/reports/T27_VS_FORMAL_HDL_2026.md` | Refreshed competitor snapshot if new developments surface |
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
Otherwise execute **Variant B** if an external capture is available or the board
is reachable for a dry-run boot-log.  
Otherwise fall back to **Variant C**.

## Chosen variant

**To be selected at the start of W428.** Current default is **Variant C** because
the hardware blockers that forced W425/W426/W427 Variant C are still present.

---

*φ² + φ⁻² = 3 | TRINITY*
