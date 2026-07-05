# Wave Loop 434 — FPGA boot-evidence next variant (real capture, live XADC validation, or master-merge retry)

**Issue:** #1395  
**Branch:** `wave-loop-434`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 433.

---

## Goal

Wave Loop 433 composed the W431 live-XADC envelope bound with the W432
per-process-corner raw-ns OSCFSEL theorem, producing a single theorem that covers
any in-envelope XADC operating point and any documented OSCFSEL selection. The
bench remains blocked (P12 unwired, no relay gate, no DLC10 cable) and the
master-merge path for the `gen-verilog` fix set (`701d79b3b`) is still not safely
reachable from `wave-loop-433`. Wave Loop 434 executes the first available variant
from `docs/reports/FPGA_LOOP_COOPERATION_W434_2026-07-01.md`.

1. **Variant A (preferred when bench becomes fully available):**
   - Confirm P12 is wired to a logic-analyzer channel and a relay/remote-power
     gate is available.
   - Program SPI flash with OSCFSEL=6 (and OSCFSEL=7 if time permits).
   - Capture real CCLK during cold-POR boot.
   - Run `tri fpga cclk-sweep ... --xadc` so boot logs record live operating points.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <xadc.json> --out <theorem.lean> --json` and commit
     generated Lean theorems plus JSON summaries.
   - Reference the W433 quantified theorem
     (`xadc_envelope_justifies_cclk_variant_raw_ns_pvt`) in the generated proof.
   - Update `fpga/HARDWARE_SSOT.md` §3.6 with measured frequency/duty/margin.

2. **Variant B (if board is reachable but P12 / relay are still blocked):**
   - Capture a real `tri fpga read-xadc --json` from the live board.
   - Verify the JSON converts to a valid `PvtContext` via `tri fpga pvt-envelope`.
   - Generate at least one `measured-to-lean` theorem using the real XADC context
     (synthetic CCLK fixture is acceptable for proof-of-pipeline).
   - Reference the W433 quantified theorem in the generated proof.
   - Alternatively, run `tri fpga cclk-sweep` over OSCFSEL 0..5 with `--xadc` and
     manual power cycles.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` and re-evaluate
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

3. **Variant C (fallback if bench still blocked):**
   - Re-attempt the `master` merge/rebase wave from a fresh topic branch to bring
     the `gen-verilog` fix set (`701d79b3b`) into the wave-loop line and clear the
     7 residual yosys smoke failures (#1245).
   - If the merge is still too risky, land another formal/tooling sub-task:
     harden `measured-to-lean` to emit the W433 theorem name, add a computable
     combined OSCFSEL+XADC envelope check, or refresh the competitor report.
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
- [ ] Issue/branch for Wave Loop 435 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
