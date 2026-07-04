# Wave Loop 421 — formal-only guarding and instrument-import depth (Variant C fallback)

**Issue:** #1363  
**Branch:** `wave-loop-421`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 420.

---

## Goal

Wave 420 closed the Variant C fallback with VCD exact-terminator / real-net
auto-threshold and PVT process-corner monotonicity while the physical bench
remains blocked (P12 unwired, DLC10 cable missing, no relay). Wave 421
re-evaluates the bench state and executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W421_2026-07-06.md`.

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

2. **Variant B (if an external VCD/CSV capture is available, no on-bench relay):**
   - Add CSV timestamp-column formats (fractional seconds / milliseconds).
   - Add VCD real-net slope filters (reject transitions with Δt < t_setup or
     ΔV < threshold_window).
   - Add a `dlc10 capture --stub` dry-run path for later replay.
   - Extend the PVT envelope with `OSCFSEL` derating coefficients.

3. **Variant C (fallback if bench still blocked):**
   - Extend VCD robustness: `$timescale` parsing, `$dumpoff`/`$dumpon`
     completeness, real-net slope/rise-time rejection.
   - Add remaining PVT envelope shape lemmas (max half-period antitonicity,
     combined temp+voltage+corner monotonicity, worst-case operating-point
     search).
   - Write a public comparison note: t27 vs Sparkle/Verilean vs Clash/Chisel.
   - Land one safe gen-verilog #1245 sub-fix that does not increase the
     16-failure yosys smoke baseline, if a narrow regression-free fix is
     available.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` or `cli/dlc10/src/main.rs` | Variant A import, B instrument depth, or C parser/formal hardening |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | New measured theorems or PVT shape lemma |
| 3 | `fpga/HARDWARE_SSOT.md` / `docs/reports` | Updated protocol or comparison note |
| 4 | `docs/reports/*` | W421 report, evidence, W422 cooperation |
| 5 | `.trinity/experience.md` | W421 learnings |
| 6 | git/PR | squash-merge to `master`, close #1363, open #? for W422 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: Real CCLK capture for `OSCFSEL=6` and `OSCFSEL=7` exists.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is explicitly explained.
- [ ] AC-A4: Cold-POR SPI flash boot for OSCFSEL 6/7 is documented with STAT reads.

### Bundle B
- [ ] AC-B1: CSV fractional-second / millisecond timestamp columns are parsed correctly with a regression test.
- [ ] AC-B2: VCD real-net slope filter rejects spurious transitions with a regression test.
- [ ] AC-B3: `dlc10 capture --stub` writes a dry-run command log with a regression test.

### Bundle C
- [ ] AC-C1: VCD parser hardening lands with unit tests (`$timescale`, slope filter, or dumpoff completeness).
- [ ] AC-C2: New PVT envelope shape lemma/test lands (max half-period, combined monotonicity, or worst-case search).
- [ ] AC-C3: `docs/reports/T27_VS_FORMAL_HDL_2026.md` comparison note is published.
- [ ] AC-C4: One safe gen-verilog #1245 sub-fix lands without increasing the 16-failure yosys smoke count (optional; defer if unsafe).

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri fpga::tests` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1363`
- Report: `docs/reports/WAVE_LOOP_421_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W421_YYYY-MM-DD.md`
- Cooperation W422: `docs/reports/FPGA_LOOP_COOPERATION_W422_YYYY-MM-DD.md`

---

## Default variant

Execute **Variant A** if the analyzer and DLC10 cable are available. Otherwise
try **Variant B** if an external capture file is available. Otherwise fall back
to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
