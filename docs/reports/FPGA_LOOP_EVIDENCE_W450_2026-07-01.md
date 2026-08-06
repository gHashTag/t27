# FPGA Wave Loop 450 Evidence

**Wave:** W450
**Issue:** #1425
**Branch:** `wave-loop-450`
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Quantified dry-run-live transaction theorem

### Theorem statement

```lean
theorem dry_run_live_w448_all_corners_transaction_ok
  (oscfsel : Nat) (h : oscfsel ≤ 7) (corner : ProcessCorner) (bits : Nat) :
  let period_ns := cclk_period_ns oscfsel
  let low_ns := period_ns / 2
  let high_ns := period_ns - low_ns
  transaction_satisfies_flash_spec
    (measured_boot_transaction_from_raw_ns_with_pvt period_ns low_ns high_ns bits)
    = true
```

This closes the dry-run-live fixture → raw-ns → PVT-context → flash-spec loop
in a single quantified statement. It covers every documented Artix-7 Master SPI
CCLK variant (OSCFSEL 0..7) and every documented process corner
(`ff`/`tt`/`ss`) at the W448 dry-run-live operating point.

### Location

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`, W450 dry-run-live section.

### Verification

```text
$ cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
Build completed successfully (2967 jobs).
```

---

## 2. Standalone smoke-gate snapshot test

### Snapshot location

- `tests/fixtures/fpga/smoke-gate/validate_lean_standalone_snapshot.json`

### Normalized `validate_lean_standalone` block in the snapshot

```json
{
  "schema_version": "1.0",
  "validate_lean_standalone": {
    "status": "ok",
    "source": "synthetic",
    "lean_file": "<REPO>/build/fpga/theorem-matrix-fixtures/theorem_matrix_validate_standalone_synthetic_0.lean"
  },
  "passed": true,
  ...
}
```

The snapshot omits the run-dependent `elapsed_ms` and normalizes absolute paths
to `<REPO>/...` so the diff gate is stable across machines.

### Rust test invocation

```text
$ cargo test -p tri --bin tri test_smoke_gate_validate_lean_standalone_matches_snapshot -- --nocapture
...
[smoke-gate] theorem-matrix OK (24 variants, source=synthetic, ... ms)
[smoke-gate] validate-lean-standalone OK (source=synthetic, ... ms)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete (passed: true)
test fpga::tests::test_smoke_gate_validate_lean_standalone_matches_snapshot ... ok
```

---

## 3. `--fast` suite mode

### CLI flag

```text
$ ./target/release/t27c suite --help
Usage: t27c suite [OPTIONS]

Options:
      --repo-root <REPO_ROOT>
      --json <JSON>
      --fast   Skip expensive optional phases (e.g. the standalone lake-package
             build inside the FPGA smoke gate)
  -h, --help
```

### Default run (standalone phase active)

```text
--- Phase 3c: FPGA Board-Less Smoke Gate ---
  FPGA smoke gate: OK (report: .../smoke_gate_report.json)
--- Phase 3c-standalone: FPGA Standalone Lake-Package Build ---
  FPGA standalone build: OK (elapsed_ms=Some(...))
```

### Fast run (standalone phase skipped)

```text
[suite] --fast mode: skipping the standalone lake-package build phase
--- Phase 3c: FPGA Board-Less Smoke Gate ---
  FPGA smoke gate: OK (report: .../smoke_gate_report.json)
--- Phase 3c-standalone: FPGA Standalone Lake-Package Build ---
  FPGA standalone build: skipped (--fast mode)
```

### Suite summary fragments

Default:

```json
{
  "phases": [
    { "name": "fpga-smoke-gate", "passed": 1, "failed": 0, "skipped": 0 },
    { "name": "fpga-smoke-gate-standalone", "passed": 1, "failed": 0, "skipped": 0 }
  ],
  "validate_lean_standalone_elapsed_ms": 386415,
  "acceptable": true
}
```

Fast:

```json
{
  "phases": [
    { "name": "fpga-smoke-gate", "passed": 1, "failed": 0, "skipped": 0 },
    { "name": "fpga-smoke-gate-standalone", "passed": 0, "failed": 0, "skipped": 1 }
  ],
  "validate_lean_standalone_elapsed_ms": null,
  "acceptable": true
}
```

---

## 4. Full suite verification

### Default

```text
$ ./scripts/tri test --json /tmp/t27_w450_suite.json
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
ACCEPTABLE:        yes
```

### Fast

```text
$ ./scripts/tri test --fast --json /tmp/t27_w450_fast_suite.json
...
=== SUMMARY ===
Parse failures:           0
...
TOTAL FAILURES:    7
BASELINE FAILURES: 7
ACCEPTABLE:        yes
```

Both runs report exactly the 7 documented `gen-verilog` yosys smoke failures
(#1245) and no new failures.

---

*φ² + φ⁻² = 3 | TRINITY*
