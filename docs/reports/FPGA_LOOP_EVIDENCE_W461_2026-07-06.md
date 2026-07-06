# FPGA Loop Evidence — Wave Loop 461 (2026-07-06)

**Issue:** #1435  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Physical bench status

No physical bench run this wave.

- `dlc10 idcode` still reports "DLC10 cable not found (VID=0x03FD)".
- P12 on the QMTech Wukong / XC7A100T-FGG676 board is unwired.
- No automated cold-POR relay gate is deployed.

## Board-less evidence

The board-less smoke gate passed end-to-end via `./scripts/tri test --fast`:

```
FPGA smoke gate: OK (report: /Users/playra/t27/build/fpga/smoke_gate_report.json)
  phases: bit_config=Some("ok")
          dry_run_sweep=Some("ok")
          verify_lean=Some("ok")
          yosys_synthesis=Some("ok")
```

The generated bitstream artifact from prior waves remains available but was not
re-flashed:

- `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB)

## Compiler-backend evidence

- `cargo test -p t27c --bin t27c`: 1524 passed, 0 failed, 2 ignored.
- `./scripts/tri test --fast`: 587/587 non-smoke PASS, 67/67 yosys smoke PASS,
  0 baseline failures, 0 seal mismatches.
- New scratch specs pass `t27c gen-verilog` + yosys `read_verilog -sv -DSIMULATION`:
  - `w461_bare_call_module.t27` emits `_toplevel_0_tmp` dummy reg with
    `always @(*) _toplevel_0_tmp = sum(0, 1);`.
  - `w461_array_param_multi_array.t27` emits `sum_pair_rom_a` and
    `sum_pair_rom_b` clones; each call site is redirected to the matching clone.

## Artifacts

- Report: `docs/reports/WAVE_LOOP_461_REPORT.md`
- Cooperation: `docs/reports/FPGA_LOOP_COOPERATION_W462_2026-07-06.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`

---

*φ² + φ⁻² = 3 | TRINITY*
