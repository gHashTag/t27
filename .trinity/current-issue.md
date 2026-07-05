# Wave Loop 435 — FPGA boot-evidence next variant (real capture, live XADC pipeline hardening, or master-merge retry)

**Issue:** (to create)  
**Branch:** `wave-loop-435` (to create)  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 434.

---

## Goal

Wave Loop 434 validated the live XADC → PVT context pipeline on a real FPGA
readout (temp≈41 °C, VCCINT≈1.00 V, VCCAUX≈1.81 V), generated a
`measured-to-lean` theorem from the live context using a synthetic CCLK fixture,
and added `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`. The bench remains blocked (P12
unwired, no relay gate, no DLC10 cable) and the master-merge path for the
`gen-verilog` fix set (`701d79b3b`) is still not safely reachable. Wave Loop 435
executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W435_2026-07-01.md`.

1. **Variant A (preferred when bench becomes fully available):**
   - Confirm P12 is wired to a logic-analyzer channel and a relay/remote-power
     gate is available.
   - Program SPI flash with OSCFSEL=6 (and OSCFSEL=7 if time permits).
   - Capture real CCLK during cold-POR boot.
   - Run `tri fpga cclk-sweep ... --xadc` so boot logs record live operating points.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <xadc.json> --out <theorem.lean> --json` and commit
     generated Lean theorems plus JSON summaries.
   - Reference `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` or the generic
     W433 theorem in the generated proof.
   - Update `fpga/HARDWARE_SSOT.md` §3.6 with measured frequency/duty/margin.

2. **Variant B (default if board reachable but P12/relay still blocked):**
   - Harden `tri fpga read-xadc` to export a rounded `PvtContext` JSON directly
     (e.g. `--to-pvt-context <file>` or `--process-corner <corner>`).
   - Add unit/integration tests for the full
     `read-xadc → pvt-envelope → measured-to-lean` pipeline.
   - Extend `measured-to-lean --json` summary with the source operating point
     (`temp_c`, `vccint_mv`, `vccaux_mv`, `process_corner`).
   - Generate `measured-to-lean` theorems for OSCFSEL 0..7 using the live XADC
     context and synthetic CCLK fixtures, producing a coverage matrix.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` and re-evaluate
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

3. **Variant C (fallback if bench still blocked and Variant B is too small):**
   - Re-attempt the `master` merge/rebase wave from a fresh topic branch to bring
     the `gen-verilog` fix set (`701d79b3b`) into the wave-loop line and clear the
     7 residual yosys smoke failures (#1245).
   - If the merge is still too risky, land another formal/tooling sub-task:
     add a computable combined OSCFSEL+XADC envelope check, or refresh the
     competitor report.
   - Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the new baseline.

---

## Definition of done

- [ ] The chosen variant is executed and its acceptance criteria are met.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the new documented baseline (ideally 0 new
      gen-verilog failures; if Variant C1 succeeds, the 7 #1245 failures are
      cleared).
- [ ] `cargo test -p tri --bin tri fpga::` passes.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 436 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
