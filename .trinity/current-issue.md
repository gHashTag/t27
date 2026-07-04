# Wave Loop 422 — formal-only guarding and safe gen-verilog narrowing (Variant C fallback)

**Issue:** #1365  
**Branch:** `wave-loop-422`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 421.

---

## Goal

Wave 421 closed the Variant C fallback with VCD `$timescale` exact-terminator,
combined PVT monotonicity, and a competitor snapshot while the physical bench
remains blocked (`openFPGALoader --detect` reports 0 devices). Wave 422
re-evaluates the bench state and executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W422_2026-07-06.md`.

1. **Variant A (preferred when bench becomes available):**
   - Confirm the board responds on JTAG (`openFPGALoader --detect`).
   - Wire P12 to a logic-analyzer channel and capture real CCLK for
     `OSCFSEL=6` and `OSCFSEL=7`.
   - Program each variant to SPI flash and perform a true cold-POR boot.
   - Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
     --standalone --validate --pvt-context <ctx.json>` and commit the generated
     Lean theorems.
   - Document the measured frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md`.

2. **Variant B (if an external VCD/CSV capture is available, no on-bench relay):**
   - Add CSV timestamp-column parsing for fractional seconds, milliseconds, and
     sample-number-only exports.
   - Add VCD real-net slope filter: reject transitions where ΔV is below a noise
     window or Δt is below a configurable `t_setup`.
   - Add `tri fpga measured-to-lean --pvt-worstcase` mode using the combined
     monotonicity corner (max temp, min VCCINT, ss corner).
   - Document the multi-format import matrix in `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still blocked):**
   - Extend VCD robustness: detect/report unknown `$timescale` units, handle
     `$dumpoff`/`$dumpon` without a preceding `#timestamp`.
   - Add remaining PVT envelope shape lemmas: separate combined monotonicity for
     `n25q128_min_sck_low_ns_pvt` / `n25q128_min_sck_high_ns_pvt`, and a
     worst-case operating-point search theorem.
   - Investigate the 16 pre-existing yosys smoke failures from weak point #1245
     and land one safe narrow sub-fix that does not increase the failure count.
   - Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new 2026
     developments.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `cli/tri/src/fpga.rs` or `cli/dlc10/src/main.rs` | Variant A import, B instrument depth, or C parser/formal hardening |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | New measured theorems or PVT shape lemma |
| 3 | `fpga/HARDWARE_SSOT.md` / `docs/reports` | Updated protocol or comparison note |
| 4 | `docs/reports/*` | W422 report, evidence, W423 cooperation |
| 5 | `.trinity/experience.md` | W422 learnings |
| 6 | git/PR | squash-merge to `master`, close #1365, open #? for W423 |

---

## Acceptance criteria

### Bundle A
- [ ] AC-A1: `openFPGALoader --detect` finds the XC7A200T and real CCLK capture files exist for `OSCFSEL=6` and `OSCFSEL=7`.
- [ ] AC-A2: `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone` generated Lean files build with `lake build`.
- [ ] AC-A3: Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is explicitly explained.
- [ ] AC-A4: Cold-POR SPI flash boot for OSCFSEL 6/7 is documented with STAT reads.

### Bundle B
- [ ] AC-B1: CSV fractional-second / millisecond / sample-number timestamp columns are parsed correctly with a regression test.
- [ ] AC-B2: VCD real-net slope filter rejects noisy transitions with a regression test.
- [ ] AC-B3: `--pvt-worstcase` mode validates against the combined-monotonicity corner with a regression test.

### Bundle C
- [ ] AC-C1: VCD parser hardening lands with unit tests (unknown timescale unit handling, or dumpoff without timestamp).
- [ ] AC-C2: New PVT envelope shape lemma/test lands (low/high combined monotonicity, or worst-case search).
- [ ] AC-C3: One safe gen-verilog #1245 sub-fix lands without increasing the 16-failure yosys smoke count (optional; defer if unsafe).
- [ ] AC-C4: Competitor snapshot is updated if any new 2026 developments are found.

### Invariant checks
- [ ] `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri fpga::tests` passes.

---

## PR

- Target: `master`
- PR: to open after work
- Body: `Closes #1365`
- Report: `docs/reports/WAVE_LOOP_422_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W422_YYYY-MM-DD.md`
- Cooperation W423: `docs/reports/FPGA_LOOP_COOPERATION_W423_YYYY-MM-DD.md`

---

## Default variant

Execute **Variant A** if the board responds on JTAG and the analyzer is wired.
Otherwise try **Variant B** if an external capture file is available. Otherwise
fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
