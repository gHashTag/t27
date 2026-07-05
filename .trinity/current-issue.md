# Wave Loop 430 — FPGA boot-evidence next variant (physical CCLK / XADC readout / formal fallback)

**Issue:** #1388  
**Branch:** `wave-loop-430`  
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 429.

---

## Goal

Wave Loop 429 hardened the FPGA CLI and formal model while the bench was blocked
(P12 unwired, no relay gate, no DLC10 cable, no OSCFSEL 6/7 bitstreams, no
external captures). Wave Loop 430 executes the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W430_2026-07-01.md`.

1. **Variant A (preferred when bench becomes available):**
   - Confirm P12 is wired to a logic-analyzer channel.
   - Program SPI flash with OSCFSEL=6 and OSCFSEL=7 variants.
   - Capture real CCLK during cold-POR boot for both variants.
   - Import with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone
     --validate --pvt-context <ctx.json> --out <theorem.lean> --json` and commit
     generated Lean theorems plus JSON summaries.
   - Document frequencies/duty cycles and PVT context in
     `fpga/HARDWARE_SSOT.md` §3.6.

2. **Variant B (if board is reachable or external capture available):**
   - Add real XADC readout to `tri fpga boot-log` / `cclk-sweep` / `cold-por`
     over the existing JTAG path, so JSON `xadc` has `source: "xadc"` and live
     temp/vccint/vccaux values.
   - Alternatively, import at least one external CSV/VCD capture end-to-end.
   - Run dry-run or real cold-POR sweep for OSCFSEL 6/7 variants with
     `--pvt-context` and verify the JSON report round-trips.
   - Document the recipe in `fpga/HARDWARE_SSOT.md` §3.6.

3. **Variant C (fallback if bench still blocked):**
   - Extend the raw-ns OSCFSEL theorem library from W429 (e.g. a theorem linking
     an arbitrary raw-ns capture to the unified OSCFSEL result when the period
     matches a documented variant within tolerance).
   - Land one safe gen-verilog #1245 sub-fix if narrow and regression-free;
     otherwise explicitly defer and update
     `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
   - Harden `tri fpga measured-to-lean` JSON output (e.g. add
     `flash_min_half_period_ns`, `margin_ns`, or a closed `recommendation`
     vocabulary).
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
- [ ] Issue/branch for Wave Loop 431 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
