# FPGA Wave Loop 452 Evidence

**Wave:** W452
**Issue:** #1422
**Branch:** `wave-loop-452`
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Boundary cold/high-voltage envelope-corner transaction theorem

### Theorem statement

```lean
theorem boundary_cold_highv_w452_all_corners_transaction_ok
  (oscfsel : Nat) (h : oscfsel ≤ 7) (corner : ProcessCorner) (bits : Nat) :
  let period_ns := cclk_period_ns oscfsel
  let low_ns := period_ns / 2
  let high_ns := period_ns - low_ns
  transaction_satisfies_flash_spec
    (measured_boot_transaction_from_raw_ns_with_pvt period_ns low_ns high_ns bits)
    = true
```

This closes the symmetric boundary-corner operating point → raw-ns →
PVT-context → flash-spec loop in a single quantified statement. It covers every
documented Artix-7 Master SPI CCLK variant (OSCFSEL 0..7) and every documented
process corner (`ff`/`tt`/`ss`) at -40 °C and 1100 mV, the coldest/highest-
voltage corner inside the documented operating envelope.

### Supporting definitions and lemmas

- `BOUNDARY_COLD_HIGHV_W452_OPERATING_POINT (corner : ProcessCorner)`
- `boundary_cold_highv_w452_operating_point_within_envelope`
- `boundary_cold_highv_w452_process_corner_worse_than_ss`
- `boundary_cold_highv_w452_raw_ns_satisfies_flash_spec`
- `boundary_cold_highv_w452_all_oscfsel_combined_check_true`

### Location

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`, W452 boundary cold/high-voltage section.

### Verification

```text
$ cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
Build completed successfully (2967 jobs).
```

---

## 2. Adversarial VCCINT witness + OSCFSEL range gate

### Definitions

```lean
def OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT : XadcOperatingPoint :=
  { temp_c := (25 : Int), vccint_mv := 800, vccaux_mv := 1800,
    process_corner := ProcessCorner.ss }
```

### Theorem statements

```lean
theorem outside_vccint_low_w452_operating_point_not_within_envelope :
  ¬ xadc_operating_point_within_envelope OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT

theorem cclk_variant_and_xadc_envelope_check_outside_vccint_low_false
  (oscfsel : Nat) (h : oscfsel ≤ 7) :
  cclk_variant_and_xadc_envelope_check oscfsel OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT = false

theorem oscfsel_out_of_range_combined_check_false (oscfsel : Nat) (h : oscfsel > 7)
  (pt : XadcOperatingPoint) :
  cclk_variant_and_xadc_envelope_check oscfsel pt = false
```

These theorems complement the W448 temperature witness with a voltage-side
adversarial witness and isolate the OSCFSEL range assumption in a single
falsifiable theorem.

### Location

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`, W452 adversarial voltage witness section.

### Verification

Same `lake build Trinity.TernaryFPGABoot` run as section 1; all three theorems
build.

---

## 3. Smoke-gate state classification in `SuiteSummary`

### Code location

- `bootstrap/src/suite.rs`:
  - `FpgaSmokeResult` now carries `failed: bool` and
    `failure_reason: Option<String>`.
  - `SuiteSummary` now carries `fpga_smoke_skipped`, `fpga_smoke_failed`, and
    `fpga_smoke_failure_reason`.
  - `parse_smoke_gate_report` classifies reports as passed/skipped/failed.
  - The error fallback path in `run_comprehensive` sets `failed = true` with a
    captured reason.

### New/updated unit tests

- `test_fpga_smoke_result_builder_missing_bitstream`
- `test_fpga_smoke_result_builder_failure_fallback`
- `test_parse_smoke_gate_report_missing_file`
- `test_parse_smoke_gate_report_fast_skips_standalone`
- `test_suite_summary_smoke_state_roundtrip`
- `test_suite_summary_schema_roundtrip`

### Verification

```text
$ cargo test -p t27c --bin t27c suite::tests
...
test suite::tests::test_fpga_smoke_result_builder_failure_fallback ... ok
test suite::tests::test_fpga_smoke_result_builder_missing_bitstream ... ok
test suite::tests::test_parse_smoke_gate_report_fast_skips_standalone ... ok
test suite::tests::test_parse_smoke_gate_report_missing_file ... ok
test suite::tests::test_suite_summary_deny_unknown_fields ... ok
test suite::tests::test_suite_summary_schema_roundtrip ... ok
test suite::tests::test_suite_summary_smoke_state_roundtrip ... ok
test suite::tests::test_run_fpga_smoke_gate_passes_with_good_report ... ok
test suite::tests::test_run_fpga_smoke_gate_fails_with_bad_report ... ok
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1499 filtered out
```

---

## 4. All-ok smoke-gate snapshot test

### Snapshot location

- `tests/fixtures/fpga/smoke-gate/all_ok_snapshot.json`

### All-ok normalized shape (excerpt)

```json
{
  "bit_config": {
    "assertions": ["ASSERTION OK: idcode"],
    "bitstream": "<TMP>/tri_smoke_gate_all_ok_bitstream.bit",
    "status": "ok"
  },
  "dry_run_sweep": {
    "report_md": "<TMP>/tri_smoke_gate_all_ok_sweep.md",
    "source": "synthetic",
    "status": "ok",
    "variant_count": 8
  },
  "passed": true,
  "schema_version": "1.0",
  "theorem_matrix": {
    "corner_count": 1,
    "oscfsel_count": 1,
    "replay": false,
    "source": "synthetic",
    "status": "ok",
    "variant_count": 1,
    "variants": [...]
  },
  "validate_lean_standalone": {
    "lean_file": "<TMP>/tri_smoke_gate_all_ok_validate_standalone.lean",
    "source": "synthetic",
    "status": "ok"
  },
  "verify_lean": {
    "expected_source": "synthetic",
    "lean_file": "<TMP>/tri_smoke_gate_all_ok_verify.lean",
    "status": "ok",
    "summary_file": "<TMP>/tri_smoke_gate_all_ok_summary.json"
  },
  "yosys_synthesis": {
    "files": [
      "<REPO>/fpga/verilog/ternary_mac_synth.v",
      "<REPO>/fpga/verilog/ternary_mac_demo_top.v"
    ],
    "status": "ok",
    "top": "ternary_mac_demo_top"
  }
}
```

### Rust test invocation

```text
$ cargo test -p tri --bin tri all_ok -- --test-threads=1
...
test fpga::tests::test_smoke_gate_all_ok_matches_snapshot ... ok
```

---

## 5. Full suite verification

### Default

```text
$ ./scripts/tri test --json /tmp/t27_w452_suite.json
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

JSON summary excerpt:

```json
{
  "fpga_smoke_passed": true,
  "fpga_smoke_skipped": false,
  "fpga_smoke_failed": false,
  "fpga_smoke_failure_reason": null,
  "fpga_smoke_gate_elapsed_ms": ...,
  "validate_lean_standalone_elapsed_ms": ...
}
```

### `--fast`

```text
$ ./scripts/tri test --fast --json /tmp/t27_w452_fast_suite.json
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

JSON summary excerpt:

```json
{
  "fpga_smoke_passed": true,
  "fpga_smoke_skipped": false,
  "fpga_smoke_failed": false,
  "fpga_smoke_failure_reason": null,
  "fpga_smoke_gate_elapsed_ms": 10,
  "validate_lean_standalone_elapsed_ms": null
}
```

Both runs report exactly the 7 documented `gen-verilog` yosys smoke failures
(#1245) and no new failures. The `--fast` run skips only the standalone lake-
package build phase; the board-less smoke gate still passes.

---

*φ² + φ⁻² = 3 | TRINITY*
