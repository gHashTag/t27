# FPGA Loop Evidence — Wave Loop 462 (2026-07-07)

**Issue:** #1437  
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
- `./scripts/tri test --fast`: 590/590 non-smoke PASS, 70/70 yosys smoke PASS,
  0 baseline failures, 0 seal mismatches.
- New scratch specs pass `t27c gen-verilog` + yosys `read_verilog -sv -DSIMULATION`:
  - `w462_array_param_literal.t27` emits `_lit_4_u16_1_2_3_4` and
    `_lit_4_u16_10_20_30_40` anonymous ROMs, plus matching
    `sum_pair__lit_4_u16_...` clones.
  - `w462_void_bare_call.t27` emits `task check_call; ... endtask` and
    `always @(*) begin check_call(); end` with no dummy register.
  - `w462_array_param_bench_local.t27` hoists bench-local scalar variables and
    calls the anonymous-ROM clone from the bench block.

## Artifacts

- Report: `docs/reports/WAVE_LOOP_462_REPORT.md`
- Cooperation: `docs/reports/FPGA_LOOP_COOPERATION_W463_2026-07-07.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`

---

*φ² + φ⁻² = 3 | TRINITY*
