# FPGA Wave Loop 449 Evidence

**Wave:** W449  
**Issue:** #1424  
**Branch:** `wave-loop-449`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Quantified golden transaction theorem

### Theorem statement

```lean
theorem golden_w449_all_corners_transaction_ok
  (oscfsel : Nat) (h : oscfsel ≤ 7) (corner : ProcessCorner) (bits : Nat) :
  let period_ns := cclk_period_ns oscfsel
  let low_ns := period_ns / 2
  let high_ns := period_ns - low_ns
  transaction_satisfies_flash_spec
    (measured_boot_transaction_from_raw_ns_with_pvt period_ns low_ns high_ns bits)
    = true
```

This closes the golden-fixture → raw-ns → PVT-context → flash-spec loop in a
single quantified statement. It covers every documented Artix-7 Master SPI
CCLK variant (OSCFSEL 0..7) and every documented process corner (`ff`/`tt`/`ss`).

### Location

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`, W449 golden section.

### Verification

```text
$ cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
Build completed successfully (2967 jobs).
```

---

## 2. Standalone-build suite metric

### Schema additions

- `bootstrap/src/suite.rs` — `FpgaSmokeResult` gained
  `validate_lean_standalone_status` and `validate_lean_standalone_elapsed_ms`.
- `bootstrap/src/suite.rs` — `SuiteSummary` gained
  `validate_lean_standalone_elapsed_ms`.

### Smoke-gate report fragment

```json
{
  "schema_version": "1.0",
  "theorem_matrix": { "status": "ok", "variant_count": 24, ... },
  "validate_lean_standalone": {
    "status": "ok",
    "source": "synthetic",
    "lean_file": ".../theorem_matrix_validate_standalone_synthetic_0.lean",
    "elapsed_ms": 311446
  },
  "passed": true
}
```

### Suite summary fragment

```json
{
  "repo": "/Users/playra/t27",
  "phases": [ ... ],
  "fpga_smoke_passed": true,
  "fpga_smoke_gate_elapsed_ms": 10,
  "fpga_smoke_gate_replay_elapsed_ms": 8,
  "validate_lean_standalone_elapsed_ms": 311446,
  "total_failures": 7,
  "acceptable": true,
  "passed": false
}
```

`passed` is `false` only because the 7 documented `gen-verilog` yosys smoke
failures are present; `acceptable` is `true` because those failures exactly
match the documented baseline and every other phase is clean.

### Verification

```text
$ ./scripts/tri test --json /tmp/t27_w449_suite.json
...
=== SUMMARY ===
Parse failures:           0
...
TOTAL FAILURES:    7
BASELINE FAILURES: 7
ACCEPTABLE:        yes
[suite] JSON summary: /tmp/t27_w449_suite.json
```

---

## 3. Rust regression tests

### Unit test: standalone smoke-gate phase

```text
$ cd cli/tri && cargo test test_smoke_gate_json_synthetic_validate_lean_standalone -- --nocapture
...
[smoke-gate] validate-lean-standalone OK (source=synthetic, 321260 ms)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete (passed: true)
test fpga::tests::test_smoke_gate_json_synthetic_validate_lean_standalone ... ok
```

### Suite schema tests

```text
$ cd bootstrap && cargo test --bin t27c test_suite_summary_schema_roundtrip test_run_fpga_smoke_gate_passes_with_good_report -- --nocapture
...
test suite::tests::test_suite_summary_schema_roundtrip ... ok
test suite::tests::test_run_fpga_smoke_gate_passes_with_good_report ... ok
```

---

## 4. Full suite status

| Phase | Result |
|-------|--------|
| Parse | 576 PASS |
| Typecheck | 576 PASS |
| GF16 conformance | PASS |
| Gen Zig | 576 PASS |
| Gen Rust | 576 PASS |
| Gen Verilog | 576 PASS |
| Gen Verilog Yosys Smoke | 49 passed, 7 pre-existing failures (#1245) |
| FPGA Board-Less Smoke Gate | PASS, 24-variant matrix, envelope OK |
| FPGA Board-Less Smoke Gate Replay | PASS |
| Gen C | 576 PASS |
| Seal Verify | 576 PASS |
| Fixed Point | 0 divergences |

---

## 5. Notes / caveats

- The standalone lake-package build takes ~5–6 minutes on a warm cache. The new
  metric makes this cost visible so future waves can decide whether to split or
  optimize it.
- Full `lake build` from the repo root still fails on unrelated physics proofs
  (`Trinity/NeutrinoMasses.lean`, `Trinity/H4Lagrangian.lean`). The boot-evidence
  target `Trinity.TernaryFPGABoot` builds independently and the standalone temp
  package depends only on it.
- Physical bench tasks remain blocked by missing hardware.

---

*φ² + φ⁻² = 3 | TRINITY*
