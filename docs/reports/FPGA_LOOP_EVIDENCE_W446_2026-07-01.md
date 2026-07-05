# FPGA Loop Evidence — Wave Loop 446 (2026-07-01)

**Issue:** #1420  
**Branch:** `wave-loop-446`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Environment

- Host: macOS 25.5.0, Darwin, shell zsh.
- Target FPGA: QMTech Wukong V1 / XC7A100T-FGG676 (`xc7a100tfgg676-1`),
  IDCODE `0x13631093`.
- In-repo driver: `cli/dlc10` (DLC10 JTAG cable `VID=0x03FD`).
- Toolchain: `yosys` available on `PATH`; no board attached, so all FPGA
  evidence is board-less / synthetic.

---

## 2. Command log

### 2.1 Build

```
cd bootstrap && cargo build --release
```

Result: `t27c` release binary built successfully (`Finished release profile`).

### 2.2 Unit-test gates

```
cargo test -p tri
```

Result: **138 passed, 0 failed, 0 ignored**.

```
cargo test -p t27c --bin t27c suite::tests
```

Result: **PASS**.

```
cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
```

Result: **Build completed successfully (2967 jobs)**.

### 2.3 Full repository sweep

```
./scripts/tri test --json /tmp/suite_report_w446.json
```

Result excerpt:

```
--- Phase 3c: FPGA Board-Less Smoke Gate ---
  FPGA smoke gate: OK (report: /Users/playra/t27/build/fpga/smoke_gate_report.json)
    phases: bit_config=Some("ok") dry_run_sweep=Some("ok")
            verify_lean=Some("ok") yosys_synthesis=Some("ok")
--- Phase 3d: FPGA Board-Less Smoke Gate Replay ---
  FPGA smoke gate: OK (report: /Users/playra/t27/build/fpga/smoke_gate_replay_report.json)
    phases: bit_config=Some("ok") dry_run_sweep=Some("ok")
            verify_lean=Some("ok") yosys_synthesis=Some("ok")
--- Phase 5: Seal Verify ---
Seal Verify: 576 passed, 0 failed

=== SUMMARY ===
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0
Gen Rust failures:        0
Gen Verilog fails:        0
Gen Verilog smoke fails:  7
FPGA smoke fails:         0
Gen C failures:           0
Seal mismatches:          0
FP divergences:           0
TOTAL FAILURES:    7
BASELINE FAILURES: 7
ACCEPTABLE:        yes (known failures match baseline, no other failures)
```

Machine-readable summary (`/tmp/suite_report_w446.json`):

```json
{
  "repo": "/Users/playra/t27",
  "phases": [
    {"name": "parse", "passed": 576, "failed": 0, "skipped": 0},
    {"name": "typecheck", "passed": 576, "failed": 0, "skipped": 0},
    {"name": "gf16_conformance", "passed": 1, "failed": 0, "skipped": 0},
    {"name": "gen-zig", "passed": 576, "failed": 0, "skipped": 0},
    {"name": "gen-rust", "passed": 576, "failed": 0, "skipped": 0},
    {"name": "gen-verilog", "passed": 576, "failed": 0, "skipped": 0},
    {"name": "gen-verilog-yosys-smoke", "passed": 48, "failed": 7, "skipped": 0},
    {"name": "fpga-smoke-gate", "passed": 1, "failed": 0, "skipped": 0},
    {"name": "fpga-smoke-gate-replay", "passed": 1, "failed": 0, "skipped": 0},
    {"name": "gen-c", "passed": 576, "failed": 0, "skipped": 0},
    {"name": "seal-verify", "passed": 576, "failed": 0, "skipped": 0},
    {"name": "fixed-point", "passed": 0, "failed": 0, "skipped": 0}
  ],
  "fpga_smoke_report": "/Users/playra/t27/build/fpga/smoke_gate_report.json",
  "fpga_smoke_passed": true,
  "fpga_smoke_gate_elapsed_ms": 9,
  "fpga_smoke_gate_replay_elapsed_ms": 7,
  "known_failures": [
    "specs/igla/race/cordic.t27",
    "specs/igla/race/cordic_top.t27",
    "specs/scratch/w378_let_destructuring.t27",
    "specs/scratch/w379_let_destructuring_generalized.t27",
    "specs/scratch/w380_tuple_return.t27",
    "specs/scratch/w381_tuple_call_chain.t27",
    "specs/scratch/w383_rom_array.t27"
  ],
  "baseline_failures": 7,
  "total_failures": 7,
  "passed": false,
  "acceptable": true
}
```

`passed: false` is expected because the 7 pre-existing gen-verilog yosys smoke
failures are counted as failures; `acceptable: true` because every failure is
within the documented baseline.

### 2.4 Golden fixture snapshot diff gate

```
cargo test -p tri fpga::tests::test_theorem_matrix_golden_replay_matches_snapshot
```

Result: **PASS**.

The test replays `tests/fixtures/fpga/theorem-matrix/golden/` and compares the
serialized theorem-matrix report block against
`tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`. The actual
report is a strict superset of the snapshot; all 24 variants are present, every
variant has `envelope_check: "ok"`, and every variant carries a `fixtures` block.

---

## 3. Pre-existing baseline

The 7 gen-verilog yosys smoke failures are the documented baseline from
`docs/reports/gen_verilog_smoke_baseline.json` (issue #1245). They are not
regressions introduced by W446:

- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

During W446 a field-access keyword-escape regression was fixed so that
`specs/igla/coder/benchmark.t27` continues to pass yosys; it is therefore **not**
in the baseline.

---

## 4. Hardware blockers

- DLC10 cable not detected on host (`VID=0x03FD`).
- Board P12 power header unwired — no automated cold-POR.
- No relay gate.

All FPGA evidence in W446 is board-less and deterministic.

---

## 5. Seal state

All 576 specs verify against their saved seals after the W446 compiler fix and
seal resync. No seal mismatches remain.

---

*φ² + φ⁻² = 3 | TRINITY*
