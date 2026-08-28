# FPGA Loop Evidence — Wave Loop 418 (2026-07-04)

**Issue:** #1353  
**Branch:** `wave-loop-418`  
**Scope:** Variant C fallback — formal tooling, instrument import coverage,
PVT regression test, and standalone Lean integration test.

---

## 1. PVT-envelope lower-bound regression

Evidence: `cli/tri/src/fpga.rs`, `proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Rust test `test_pvt_half_ns_lower_bound_across_operating_rectangle` exhaustively
  checks the operating rectangle and confirms
  `n25q128_min_sck_half_ns_pvt(ctx) >= 6` for every sampled context.
- Lean lemma `pvt_half_ns_at_least_nominal` proves the same bound symbolically for
  any context inside the operating envelope.
- Worst-case context (ss corner, 900 mV, +85 °C) produces a 13 ns bound.

---

## 2. VCD parser header hardening

Evidence: `cli/tri/src/fpga.rs`

- Added state machine for `$date`, `$version`, and `$comment` sections in
  `parse_vcd_to_raw_ns`.
- Multi-line header contents are skipped before `$var` / value-change parsing.
- Regression test `test_parse_vcd_multiline_header_sections_skipped` verifies a
  vendor-style multi-line header is parsed correctly.

---

## 3. Analog CSV voltage-column auto-detection

Evidence: `cli/tri/src/fpga.rs`

- Header names `voltage`, `v`, and `analog` are recognized as the signal column.
- The named column is preferred over the first-numeric-column fallback.
- Regression test `test_parse_cclk_csv_named_voltage_column` verifies a
  three-column CSV where the signal is in the third column.

---

## 4. Standalone Lean integration test

Evidence: `cli/tri/src/fpga.rs`

- Rust test `test_measured_to_lean_standalone_lake_package_builds`:
  1. writes a synthetic raw-ns capture (`period=40 ns`, `low=20 ns`, `high=20 ns`),
  2. generates a self-contained `.lean` file via `--standalone --raw-ns`,
  3. creates a temporary `lake` package requiring the local Trinity library,
  4. builds the package with `lake build`,
  5. asserts the build succeeds.

This proves the `--standalone` output is a valid, buildable lake package snippet.

---

## 5. Local verification commands

```bash
# Rust checks
cd .
cargo check -p tri
cargo test -p tri pvt
cargo test -p tri vcd
cargo test -p tri csv
cargo test -p tri test_measured_to_lean_standalone_lake_package_builds

# Lean check
cd proofs/lean4
lake build Trinity.TernaryFPGABoot
```

All commands above passed in the W418 development session.

---

## 6. Hardware status

- P12 CCLK capture: **not wired**.
- DLC10 cable: **missing** (VID 0x03FD not detected).
- Relay board / USB power switch: **not available**.

Therefore W418 executed Variant C and produced no new silicon evidence. W419 will
again evaluate A/B/C variants.

---

*φ² + φ⁻² = 3 | TRINITY*
