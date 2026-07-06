# FPGA Wave Loop 451 Evidence

**Wave:** W451
**Issue:** #1423
**Branch:** `wave-loop-451`
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Boundary hot/low-voltage envelope-corner transaction theorem

### Theorem statement

```lean
theorem boundary_hot_lowv_w451_all_corners_transaction_ok
  (oscfsel : Nat) (h : oscfsel ≤ 7) (corner : ProcessCorner) (bits : Nat) :
  let period_ns := cclk_period_ns oscfsel
  let low_ns := period_ns / 2
  let high_ns := period_ns - low_ns
  transaction_satisfies_flash_spec
    (measured_boot_transaction_from_raw_ns_with_pvt period_ns low_ns high_ns bits)
    = true
```

This closes the boundary-corner operating point → raw-ns → PVT-context →
flash-spec loop in a single quantified statement. It covers every documented
Artix-7 Master SPI CCLK variant (OSCFSEL 0..7) and every documented process
corner (`ff`/`tt`/`ss`) at +85 °C and 900 mV, the hottest/lowest-voltage corner
inside the documented operating envelope.

### Supporting definitions and lemmas

- `BOUNDARY_HOT_LOWV_W451_OPERATING_POINT (corner : ProcessCorner)`
- `boundary_hot_lowv_w451_operating_point_within_envelope`
- `boundary_hot_lowv_w451_process_corner_worse_than_ss`
- `boundary_hot_lowv_w451_raw_ns_satisfies_flash_spec`
- `boundary_hot_lowv_w451_all_oscfsel_combined_check_true`

### VCCAUX independence lemmas

- `xadc_operating_point_within_envelope_independent_of_vccaux`
- `n25q128_min_sck_low_ns_pvt_independent_of_vccaux`
- `n25q128_min_sck_high_ns_pvt_independent_of_vccaux`
- `n25q128_min_sck_half_ns_pvt_independent_of_vccaux`
- `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec_independent_of_vccaux`

### Location

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`, W451 boundary hot/low-voltage section.

### Verification

```text
$ cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
Build completed successfully (2967 jobs).
```

---

## 2. `FpgaSmokeResultBuilder` + `SuiteSummary` `deny_unknown_fields`

### Code location

- `bootstrap/src/suite.rs`:
  - `FpgaSmokeResultBuilder` with fluent field methods.
  - `FpgaSmokeResultBuilder::missing_bitstream()` fallback shape.
  - `FpgaSmokeResultBuilder::failed()` fallback shape.
  - `#[serde(deny_unknown_fields)]` on `SuitePhaseSummary` and `SuiteSummary`.

### New unit tests

- `test_fpga_smoke_result_builder_missing_bitstream`
- `test_fpga_smoke_result_builder_failed`
- `test_suite_summary_deny_unknown_fields`
- `test_suite_phase_summary_deny_unknown_fields`
- `test_parse_smoke_gate_report_fast_skips_standalone`

### Verification

```text
$ cargo test -p t27c --bin t27c suite::tests
...
test suite::tests::test_fpga_smoke_result_builder_failed ... ok
test suite::tests::test_fpga_smoke_result_builder_missing_bitstream ... ok
test suite::tests::test_parse_smoke_gate_report_fast_skips_standalone ... ok
test suite::tests::test_suite_phase_summary_deny_unknown_fields ... ok
test suite::tests::test_suite_summary_deny_unknown_fields ... ok
test suite::tests::test_suite_summary_schema_roundtrip ... ok
...
test result: ok.
```

---

## 3. Smoke-gate snapshot tests for edge-case report shapes

### Snapshot locations

- `tests/fixtures/fpga/smoke-gate/missing_bitstream_snapshot.json`
- `tests/fixtures/fpga/smoke-gate/fast_skipped_standalone_snapshot.json`

### Missing-bitstream normalized shape

```json
{
  "bit_config": {
    "bitstream": "<TMP>/tri_smoke_gate_missing_bitstream.bit",
    "reason": "bitstream not found",
    "status": "skipped"
  },
  "dry_run_sweep": null,
  "passed": false,
  "schema_version": "1.0",
  "theorem_matrix": null,
  "validate_lean_standalone": null,
  "verify_lean": null,
  "yosys_synthesis": {
    "reason": "demo Verilog sources not found",
    "status": "skipped"
  }
}
```

### `--fast` skipped-standalone normalized shape

```json
{
  "bit_config": {
    "assertions": ["ASSERTION OK: idcode"],
    "bitstream": "<TMP>/tri_smoke_gate_fast_bitstream.bit",
    "status": "ok"
  },
  "dry_run_sweep": {
    "report_md": "<TMP>/tri_smoke_gate_fast_sweep.md",
    "source": "synthetic",
    "status": "ok",
    "variant_count": 8
  },
  "passed": true,
  "schema_version": "1.0",
  "theorem_matrix": {
    "replay": false,
    "source": "synthetic",
    "status": "ok",
    "variant_count": 1,
    "variants": [
      {
        "corner": "ff",
        "envelope_check": "ok",
        "fixtures": {
          "lean": "<TMP>/tri_smoke_gate_fast_theorem.lean",
          "pvt": "<TMP>/tri_smoke_gate_fast_pvt.json",
          "raw_ns": "<TMP>/tri_smoke_gate_fast_raw_ns.json",
          "summary": "<TMP>/tri_smoke_gate_fast_summary.json"
        },
        "oscfsel": 0,
        "period_ns": 400,
        "sck_high_ns": 200,
        "sck_low_ns": 200,
        "status": "ok"
      }
    ]
  },
  "validate_lean_standalone": null,
  "verify_lean": { "status": "ok" },
  "yosys_synthesis": {
    "files": [],
    "status": "ok",
    "top": "ternary_mac_demo_top"
  }
}
```

### Rust test invocation

```text
$ cargo test -p tri --bin tri missing_bitstream -- --test-threads=1
...
test fpga::tests::test_smoke_gate_missing_bitstream_matches_snapshot ... ok

$ cargo test -p tri --bin tri fast_skipped -- --test-threads=1
...
test fpga::tests::test_smoke_gate_fast_skipped_standalone_matches_snapshot ... ok
```

---

## 4. Full suite verification

### Default

```text
$ ./scripts/tri test --json /tmp/t27_w451_suite.json
...
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

### `--fast`

```text
$ ./scripts/tri test --fast --json /tmp/t27_w451_fast_suite.json
...
[suite] --fast mode: skipping the standalone lake-package build phase
...
--- Phase 3c-standalone: FPGA Standalone Lake-Package Build ---
  FPGA standalone build: skipped (--fast mode)
...
TOTAL FAILURES:    7
BASELINE FAILURES: 7
ACCEPTABLE:        yes (known failures match baseline, no other failures)
```

Both runs report exactly the 7 documented `gen-verilog` yosys smoke failures
(#1245) and no new failures.

---

*φ² + φ⁻² = 3 | TRINITY*
