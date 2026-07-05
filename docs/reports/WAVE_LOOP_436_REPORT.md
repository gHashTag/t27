# Wave Loop 436 Report — Live XADC → PVT context pipeline in boot logs and sweep reports

**Issue:** #1402  
**Branch:** `wave-loop-436`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 436 executed **Variant B** from the W436 cooperation plan: extend the
live XADC → PVT context pipeline into cold-POR boot logs and the CCLK sweep
report, add closed-vocabulary `operating_point` source labels, and produce the
quantified combined-check theorem that closes the live-readout → all-OSCFSEL
variants loop.

The physical bench is unchanged: P12 is still unwired, no relay gate exists, and
the in-repo DLC10 driver cannot be used. Because the board is still reachable
over JTAG and live XADC readout succeeds, Variant B remains the highest-leverage
safe choice. Variant A is still the preferred path if the bench unblocks, and
Variant C (master-merge of the gen-verilog fix set) remains a future dedicated
wave.

---

## What was done

### 1. `tri fpga cold-por` / `tri fpga cclk-sweep` PVT context support

- Added `--process-corner` (`ff`/`tt`/`ss`, default `ss`) and `--to-pvt-context`
  to both `tri fpga cold-por` and `tri fpga cclk-sweep`.
- Added `resolve_pvt_context_for_boot` helper so both commands share the same
  priority logic: explicit `--pvt-context` file > live XADC readout > none.
- Live XADC readouts are converted to a rounded `PvtContext` using the supplied
  corner and embedded in every boot log; `--to-pvt-context` persists the same
  context to a file.

### 2. Closed-vocabulary `operating_point` source labels

- Added `operating_point` to the `SweepLog` struct with a backward-compatible
  default.
- Every sweep-report variant now carries `operating_point` with:
  - `source`: `xadc`, `pvt_context_file`, `worstcase`, or `not_read`;
  - `temp_c`, `vccint_mv`, `vccaux_mv`, `process_corner`.
- The cold-POR mock boot log also carries `operating_point`.

### 3. `tri fpga measured-to-lean --pvt-context-source`

- Added `--pvt-context-source <label>` to override the closed-vocabulary `source`
  label emitted in the `--json` summary and in the generated theorem comment.
- Defaults remain `pvt_context_file` for `--pvt-context`, `worstcase` for
  `--pvt-worstcase`, and the measurement source otherwise.

### 4. Quantified combined-check theorem

- Added `xadc_live_w434_all_oscfsel_combined_check_true` in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
- For every documented OSCFSEL selection (0..7), the computable
  `cclk_variant_and_xadc_envelope_check` gate evaluates to `true` under the W434
  live XADC operating point.

### 5. Tests and documentation

- Added `test_measured_to_lean_pvt_context_source_override` and hardened
  `test_sweep_report_json_roundtrip` to assert `operating_point.source`.
- Updated `fpga/HARDWARE_SSOT.md` with §3.6.21 (live XADC → PVT context pipeline).
- Refreshed `docs/reports/T27_VS_FORMAL_HDL_2026.md` with W436 competitive notes.
- Added W436 triage entry to `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

---

## What was not done (and why)

- **Real CCLK capture (Variant A)** — still blocked by P12 wiring and the lack of
  a relay gate.
- **Master-merge of `gen-verilog` fix set (Variant C)** — still too risky for the
  FPGA boot-evidence focus; 7 residual yosys smoke failures remain the documented
  baseline.
- **Physical cold-POR sweep for OSCFSEL=6/7** — requires Variant A hardware or a
  manual power-cycle protocol that is not yet formalized.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | PASS |
| `cargo test -p tri` | **117 passed, 0 failed** |
| `cargo test -p tri fpga::tests` | **84 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS (2967 jobs)** |
| `./scripts/tri test` | 576/576 parse, typecheck, gen-zig, gen-rust, gen-verilog, gen-c, seal; 49/56 yosys smoke pass (7 pre-existing #1245 failures); 0 FPGA smoke fails; 0 fixed-point divergences |

---

## Strategic notes

- The formal boot-evidence line now has a **single quantified theorem** stating
  that every documented Artix-7 CCLK variant is safe under the real captured W434
  silicon operating point, and the dashboard gate behind that theorem is now
  emitted by the CLI.
- Every boot log and sweep-report variant carries a machine-readable provenance
  object so a future physical run can be replayed and audited without guessing
  how the operating point was obtained.
- The remaining vulnerability is the same as W435: the bitstream and theorems
  are ready, but physical capture/sweep automation is blocked by bench wiring.
  The 7 gen-verilog residual failures are a secondary debt.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W437_2026-07-01.md` for three cooperation
variants for Wave Loop 437.

---

*φ² + φ⁻² = 3 | TRINITY*
