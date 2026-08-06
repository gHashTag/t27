# FPGA Loop Evidence — Wave Loop 436 (2026-07-01)

**Issue:** #1402  
**Branch:** `wave-loop-436`  
**Target board:** QMTech Wukong V1 / XC7A100T-FGG676, IDCODE `0x13631093`  
**JTAG cable:** Xilinx Platform Cable USB II (DLC10, VID `0x03FD`)  
**Host driver:** `cli/dlc10` (`dlc10 idcode|sram|flash|reload`)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What this evidence file records

This file records the formal, tooling, and hardware state at the end of Wave
Loop 436. W436 extended the live XADC → PVT context pipeline into cold-POR boot
logs and the CCLK sweep report, added closed-vocabulary source labels, and
proved the quantified combined-check theorem for all documented Artix-7 OSCFSEL
variants under the W434 live XADC operating point.

No new physical bitstream was generated this wave; the W436 artifacts extend the
same W434/435 evidence trail.

---

## 2. Environment and toolchain

| Component | Version / Commit |
|---|---|
| t27 branch | `wave-loop-436` |
| t27 commit | (to be filled after land) |
| `cli/dlc10` | in-repo Rust driver |
| Vivado | not used on macOS host; OpenXC7 / Vivado-in-Docker per `fpga/HARDWARE_SSOT.md` |
| Yosys | `0.51+` (used for gen-verilog smoke gate) |
| Lean 4 toolchain | `leanprover/lean4:v4.18.0` (lake) |
| Rust toolchain | `rustc 1.86.0`, `cargo 1.86.0` |

---

## 3. Physical bench state

| Item | State | Evidence |
|---|---|---|
| DLC10 JTAG cable | **Not connected** | `dlc10 idcode` fails with `DLC10 cable not found (VID=0x03FD)` |
| Board P12 power header | **Unwired** | No relay/automated power-cycle gate possible |
| Wukong V1 on lab desk | Reachable via JTAG when cable present; no board power telemetry on host |
| Bitstream | W436 did not regenerate the bitstream; W434/435 bitstream remains ready |

---

## 4. Test and build evidence

### Rust CLI (`cargo test -p tri`)

```text
cargo test -p tri
  running 117 tests
test result: ok. 117 passed; 0 failed; 0 ignored; 0 measured
```

Relevant FPGA tests:

```text
cargo test -p tri fpga::tests
  running 84 tests
test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured
```

### Lean 4 boot evidence (`lake build`)

```text
lake build Trinity.TernaryFPGABoot
# build: ... 2967 jobs / 2967 done
```

Key theorems materialized:

- `xadc_live_w434_operating_point_within_envelope`
- `xadc_live_w434_all_oscfsel_combined_check_true` — quantified over
  `oscfsel : Nat` with `h : oscfsel ≤ 7`, proving the computable
  `cclk_variant_and_xadc_envelope_check` gate returns `true` for every
  documented Artix-7 CCLK variant under the W434 live XADC operating point.

### Full repo sweep (`./scripts/tri test`)

```text
TOTAL FAILURES: 7
  Gen Verilog Yosys Smoke: 49 passed, 7 failed
```

All other phases pass:

| Phase | Result |
|---|---|
| parse | all pass |
| typecheck | all pass |
| gen-zig | all pass |
| gen-rust | all pass |
| gen-verilog emit | all pass |
| gen-verilog yosys smoke | 49 pass / 7 fail (#1245 baseline) |
| gen-c | all pass |
| seal verify | all pass |
| fixed-point divergences | 0 |

---

## 5. New CLI behavior

### `tri fpga cold-por` now embeds `operating_point`

```bash
tri fpga cold-por \
  --process-corner ss \
  --to-pvt-context out/w436_pvt.json \
  --json out/w436_boot.json
```

`out/w436_boot.json` contains:

```json
{
  "operating_point": {
    "source": "xadc",
    "temp_c": 42.0,
    "vccint_mv": 997.0,
    "vccaux_mv": 1801.0,
    "process_corner": "ss"
  },
  ...
}
```

### `tri fpga cclk-sweep` now stores `operating_point` per variant

```bash
tri fpga cclk-sweep \
  --process-corner ss \
  --to-pvt-context out/w436_pvt.json \
  --json out/w436_sweep.json
```

`out/w436_sweep.json` contains a `log` array. Each element carries:

```json
{
  "oscfsel": 1,
  "cclk_hz": 3300000,
  "pass": true,
  "operating_point": {
    "source": "xadc",
    "temp_c": 42.0,
    "vccint_mv": 997.0,
    "vccaux_mv": 1801.0,
    "process_corner": "ss"
  }
}
```

### `tri fpga sweep-report --json` propagates `operating_point`

```bash
tri fpga sweep-report --input out/w436_sweep.json --json out/w436_report.json
```

`out/w436_report.json` contains, per variant:

```json
{
  "variant": 1,
  "status": "PASS",
  "operating_point": { ... }
}
```

### `tri fpga measured-to-lean --pvt-context-source`

```bash
tri fpga measured-to-lean \
  --input out/w436_sweep.json \
  --pvt-context out/w436_pvt.json \
  --pvt-context-source xadc \
  --json out/w436_lean.json \
  > out/w436_lean.lean
```

The generated `.lean` theorem comment includes the provenance label:

```lean
-- operating_point source: xadc
```

---

## 6. Source label vocabulary (closed set)

| Label | Meaning |
|---|---|
| `xadc` | Live on-die XADC readout, converted to PVT context |
| `pvt_context_file` | Loaded from `--pvt-context` JSON file |
| `worstcase` | Worst-case envelope selected by `--pvt-worstcase` |
| `not_read` | Default / no PVT context available |

All labels are checked at CLI parse time.

---

## 7. Known residual issues

1. **DLC10 cable not found** — physical bench still cannot be driven by the
   in-repo driver.
2. **P12 unwired** — no automated cold-POR power-cycle.
3. **7 gen-verilog yosys smoke failures** — documented in
   `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`; full fix set exists on `master`
   but was not merged to keep W436 focused.

---

## 8. Conclusion

W436 successfully closed the live XADC → PVT context → all-OSCFSEL combined-check
loop without touching the hardware. The bitstream from W434/435 remains the
latest physical artifact; the next wave that unblocks the bench can replay the
same pipeline end-to-end with real capture data.

---

*φ² + φ⁻² = 3 | TRINITY*
