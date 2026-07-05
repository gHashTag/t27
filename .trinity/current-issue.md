# Wave Loop 432 — FPGA boot-evidence next variant (real CCLK capture, live XADC validation, or master-merge)

**Issue:** #1391  
**Branch:** `wave-loop-432`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 431.

---

## Goal

Wave Loop 431 hardened the XADC → PVT context bridge, added a computable Lean 4
envelope check, and improved the machine-readable `measured-to-lean --json`
summary while the bench remained partially blocked (P12 unwired, no relay gate,
no DLC10 cable). Wave Loop 432 executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W432_2026-07-01.md`.

1. **Variant A (preferred when bench becomes fully available):**
   - Confirm P12 is wired to a logic-analyzer channel and a relay/remote-power
     gate is available.
   - Program SPI flash with OSCFSEL=6 (and OSCFSEL=7 if time permits).
   - Capture real CCLK during cold-POR boot.
   - Run `tri fpga cclk-sweep ... --xadc` so boot logs record live operating points.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <xadc.json> --out <theorem.lean> --json` and commit
     generated Lean theorems plus JSON summaries.
   - Update `fpga/HARDWARE_SSOT.md` §3.6 with measured frequency/duty/margin.

2. **Variant B (if board is reachable but P12 / relay are still blocked):**
   - Capture a real `tri fpga read-xadc --json` from the live board.
   - Verify the JSON converts to a valid `PvtContext` via `tri fpga pvt-envelope`.
   - Generate at least one `measured-to-lean` theorem using the real XADC context
     (synthetic CCLK fixture is acceptable for proof-of-pipeline).
   - Alternatively, run `tri fpga cclk-sweep` over OSCFSEL 0..5 with `--xadc` and
     manual power cycles.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` and re-evaluate
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

3. **Variant C (fallback if bench still blocked):**
   - Execute the deferred master-merge/rebase wave to bring the `master` fix set
     (`701d79b3b`) into `wave-loop-432` and clear the 7 residual yosys smoke
     failures (#1245).
   - If the merge is too risky for one wave, land a formal/tooling sub-task:
     per-OSCFSEL PVT-corner theorems, machine-readable `sweep-report --json`, or
     a deeper competitor refresh.
   - Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the new baseline.

---

## Definition of done

- [ ] The chosen variant is executed and its acceptance criteria are met.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the new documented baseline (ideally 0 new
      gen-verilog failures; if Variant C succeeds, the 7 #1245 failures are
      cleared).
- [ ] `cargo test --bin tri fpga::` passes.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 433 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
