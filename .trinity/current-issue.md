# Wave Loop 431 — FPGA boot-evidence next variant (physical CCLK / XADC sweep / formal fallback)

**Issue:** #1389  
**Branch:** `wave-loop-431`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 430.

---

## Goal

Wave Loop 430 added live XADC readout and a formal PVT-envelope bridge while
the bench remained partially blocked (P12 unwired, no relay gate, no DLC10
cable). Wave Loop 431 executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W431_2026-07-01.md`.

1. **Variant A (preferred when bench becomes fully available):**
   - Confirm P12 is wired to a logic-analyzer channel and a relay/remote-power
     gate is available.
   - Program SPI flash with OSCFSEL=6 and OSCFSEL=7 variants.
   - Capture real CCLK during cold-POR boot for both variants.
   - Run `tri fpga cclk-sweep ... --xadc` so boot logs record live operating
     points.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <ctx.json> --out <theorem.lean> --json` and commit
     generated Lean theorems plus JSON summaries.
   - Document frequencies, duty cycles, and XADC/PVT context in
     `fpga/HARDWARE_SSOT.md` §3.6 / §9.6.

2. **Variant B (if board is reachable but P12 / relay are still blocked):**
   - Run a real `tri fpga cclk-sweep` over OSCFSEL 0..7 with `--xadc` and a
     supplied `--pvt-context`, performing manual power cycles.
   - Alternatively, import at least one external CSV/VCD capture end-to-end.
   - Add a Lean theorem or decidability lemma connecting a concrete XADC JSON
     operating point to `xadc_operating_point_envelope_implies_worst_case_bound`.
   - Document the sweep/import recipe in `fpga/HARDWARE_SSOT.md`.

3. **Variant C (fallback if bench still blocked):**
   - Extend the XADC/PVT theorem library (e.g. computable envelope check or an
     implication theorem linking measured raw-ns + XADC to transaction OK).
   - Land one safe gen-verilog #1245 sub-fix if narrow and regression-free;
     otherwise explicitly defer and update
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
   - Harden `tri fpga measured-to-lean` JSON output further.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if new competitor signals
     appear.

---

## Definition of done

- [ ] The chosen variant is executed and its acceptance criteria are met.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with no new failures beyond the documented
      7 gen-verilog #1245 failures.
- [ ] `cargo test --bin tri fpga::` passes.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 432 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
