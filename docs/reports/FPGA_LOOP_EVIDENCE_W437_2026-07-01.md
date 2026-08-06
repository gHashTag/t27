# FPGA Loop Evidence — Wave Loop 437 (2026-07-01)

**Issue:** #1405  
**Branch:** `wave-loop-437`  
**Target board:** QMTech Wukong V1 / XC7A100T-FGG676, IDCODE `0x13631093`  
**JTAG cable:** Xilinx Platform Cable USB II (DLC10, VID `0x03FD`)  
**Host driver:** `cli/dlc10` (`dlc10 idcode|sram|flash|reload`)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What this evidence file records

This file records the tooling and formal state at the end of Wave Loop 437. W437
was a software-only wave: it added deterministic `--synthetic-operating-point`
modes and a `verify-lean` gate so the boot-evidence pipeline can be exercised in
CI without the physical bench. No new physical bitstream or silicon capture was
produced.

---

## 2. Environment and toolchain

| Component | Version / Commit |
|---|---|
| t27 branch | `wave-loop-437` |
| t27 commit | (to be filled after land) |
| `cli/dlc10` | in-repo Rust driver |
| Yosys | `0.51+` |
| Lean 4 toolchain | `leanprover/lean4:v4.18.0` |
| Rust toolchain | `rustc 1.86.0`, `cargo 1.86.0` |

---

## 3. Physical bench state

| Item | State | Evidence |
|---|---|---|
| DLC10 JTAG cable | **Not connected** | `dlc10 idcode` fails with `DLC10 cable not found (VID=0x03FD)` |
| Board P12 power header | **Unwired** | No automated cold-POR gate possible |
| Latest physical bitstream | W434/435/436 artifacts remain ready; no new bitstream this wave |

---

## 4. Test and build evidence

### Rust CLI (`cargo test -p tri`)

```text
cargo test -p tri
  running 123 tests
test result: ok. 123 passed; 0 failed; 0 ignored; 0 measured
```

Relevant FPGA tests:

```text
cargo test -p tri fpga::tests
  running 90 tests
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured
```

### Lean 4 boot evidence (`lake build`)

```text
lake build Trinity.TernaryFPGABoot
# build: ... 2967 jobs / 2967 done
```

The W434 live-point theorem lattice remains the most recent formal anchor; no
new silicon-derived theorem was minted this wave.

### Full repo sweep (`./scripts/tri test`)

```text
TOTAL FAILURES: 7
  Gen Verilog Yosys Smoke: 49 passed, 7 failed
```

All other phases pass.

---

## 5. New CLI behavior

### Synthetic cold-POR boot log

```bash
tri fpga cold-por \
  fpga/verilog/ternary_mac_demo_top_200t.bit \
  --relay-port MOCK \
  --synthetic-operating-point \
  --process-corner ss \
  --log-dir out/w437
```

Produces `out/w437/boot-log-cold-por-mock-*.json` containing:

```json
{
  "operating_point": {
    "source": "synthetic",
    "temp_c": 42,
    "vccint_mv": 1000,
    "vccaux_mv": 1800,
    "process_corner": "ss"
  },
  "xadc": {
    "source": "synthetic",
    "temp_c": 42,
    "vccint_mv": 1000,
    "vccaux_mv": 1800
  },
  ...
}
```

### Synthetic CCLK sweep

```bash
tri fpga cclk-sweep \
  fpga/verilog/ternary_mac_demo_top_200t.bit \
  --dry-run \
  --synthetic-operating-point \
  --process-corner ss \
  --log-dir out/w437_sweep
```

Every `boot-log-*.json` entry carries `operating_point.source: "synthetic"`.

### `tri fpga verify-lean`

```bash
tri fpga verify-lean \
  out/w437_theorem.lean \
  --summary out/w437_theorem.json \
  --expected-source synthetic \
  --json
```

Sample output:

```json
{
  "lean_file": "out/w437_theorem.lean",
  "summary_file": "out/w437_theorem.json",
  "operating_point_source": "synthetic",
  "theorem_count": 2,
  "theorems": [
    "synthetic_cclk_40_20_20_satisfies_flash_spec",
    "synthetic_cclk_40_20_20_transaction_ok"
  ],
  "expected_source": "synthetic",
  "passed": true
}
```

---

## 6. Source label vocabulary (closed set)

| Label | Meaning |
|---|---|
| `xadc` | Live on-die XADC readout |
| `pvt_context_file` | Loaded from `--pvt-context` JSON file |
| `worstcase` | Worst-case envelope from `--pvt-worstcase` |
| `synthetic` | Deterministic CI operating point from `--synthetic-operating-point` |
| `not_read` | Default / no PVT context |

---

## 7. Known residual issues

1. DLC10 cable not found / P12 unwired — physical capture still blocked.
2. 7 gen-verilog yosys smoke failures — documented baseline, fix set on `master`.

---

## 8. Conclusion

W437 closed the software-only validation loop. The next wave that unblocks the
bench can now run a real capture and use the same `verify-lean` / source-label
pipeline to mint auditable silicon-backed theorems.

---

*φ² + φ⁻² = 3 | TRINITY*
