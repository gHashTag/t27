# Wave Loop 436 — FPGA boot-evidence next variant (real capture, live XADC pipeline extension, or master-merge retry)

**Issue:** #1402  
**Branch:** `wave-loop-436`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 435.

---

## Goal

Wave Loop 435 hardened the live XADC → PVT context pipeline, added an end-to-end integration test, extended the `measured-to-lean --json` summary with source operating points, and generated a synthetic OSCFSEL 0..7 theorem matrix under the real W434 silicon operating point in `proofs/lean4/Trinity/TernaryFPGABoot.lean`. The bench remains blocked (P12 unwired, no relay gate, no DLC10 cable) and the master-merge path for the `gen-verilog` fix set (`701d79b3b`) is still not safely reachable. Wave Loop 436 executes the first available variant from `docs/reports/FPGA_LOOP_COOPERATION_W436_2026-07-01.md`.

1. **Variant A (preferred when bench becomes fully available):**
   - Confirm P12 is wired to a logic-analyzer channel and a relay/remote-power gate is available.
   - Program SPI flash with OSCFSEL=6 (and OSCFSEL=7 if time permits).
   - Capture real CCLK during cold-POR boot.
   - Run `tri fpga cclk-sweep ... --xadc --to-pvt-context` so boot logs record live operating points.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <xadc_pvt.json> --out <theorem.lean> --json` and commit
     generated Lean theorems plus JSON summaries.
   - Reference `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` or the W435
     `cclk_variant_and_xadc_envelope_check` gate in the generated proof.
   - Update `fpga/HARDWARE_SSOT.md` §3.6 with measured frequency/duty/margin.

2. **Variant B (default if board reachable but P12/relay still blocked):**
   - Extend `tri fpga cold-por` / `tri fpga cclk-sweep` to support `--to-pvt-context` so
     every boot log JSON contains the rounded PVT context recorded at boot time.
   - Add `operating_point` to the sweep/boot log JSON schema, mirroring the
     `measured-to-lean --json` summary.
   - Add a `tri fpga sweep-report --pvt-context` path producing a machine-readable
     JSON report correlating OSCFSEL variant, live XADC point, PVT margin, and
     recommendation.
   - Teach `measured-to-lean` to accept an `operating_point` source label `"xadc"`
     when the PVT context is derived from a live `read-xadc` export.
   - Add a Lean example theorem evaluating `cclk_variant_and_xadc_envelope_check`
     over OSCFSEL 0..7 at the W434 live point.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` and re-evaluate
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

3. **Variant C (fallback if bench still blocked and Variant B is too small):**
   - Re-attempt the `master` merge/rebase wave from a fresh topic branch to bring
     the `gen-verilog` fix set (`701d79b3b`) into the wave-loop line and clear the
     7 residual yosys smoke failures (#1245).
   - If the merge is still too risky, land another formal/tooling sub-task:
     extend PVT bounds to additional process corners/flash parts, or refresh the
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
- [ ] Issue/branch for Wave Loop 437 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
