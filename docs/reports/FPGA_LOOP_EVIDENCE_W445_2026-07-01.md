# FPGA Loop Evidence — Wave Loop 445 (2026-07-01)

**Issue:** #1419  
**Branch:** `wave-loop-445`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Rust build / check

```bash
cargo check -p tri
```

Result: `Finished dev profile [unoptimized + debuginfo]` with warnings only,
no errors.

```bash
cargo build --release -p tri
cargo build --release -p t27c
```

Result: release `tri` and `t27c` binaries built successfully.

---

## 2. `tri` crate unit tests

```bash
cargo test -p tri --bin tri
```

Result:

```text
test result: ok. 137 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.43s
```

Key new regression tests:
- `fpga::tests::test_theorem_matrix_golden_replay_passes`
- Existing `test_theorem_matrix_fixture_roundtrip` and
  `test_theorem_matrix_replay_does_not_regenerate` still pass.

---

## 3. Bootstrap `t27c` suite regression tests

```bash
cargo test -p t27c --bin t27c suite::tests
```

Result:

```text
running 8 tests
test suite::tests::test_parse_smoke_gate_report_missing_file ... ok
test suite::tests::test_suite_summary_acceptable_computation ... ok
test suite::tests::test_parse_smoke_gate_report_schema_tolerant_without_theorem_matrix ... ok
test suite::tests::test_suite_summary_schema_roundtrip ... ok
test suite::tests::test_load_gen_verilog_smoke_baseline ... ok
test suite::tests::test_tri_exe_finds_target_debug_tri ... ok
test suite::tests::test_run_fpga_smoke_gate_fails_with_bad_report ... ok
test suite::tests::test_run_fpga_smoke_gate_passes_with_good_report ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1495 filtered out; finished in 0.94s
```

---

## 4. Lean boot-evidence build

```bash
cd proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
Build completed successfully (2967 jobs).
```

---

## 5. Full repo test sweep

```bash
./scripts/tri test --json build/suite_report_w445.json
```

Result:

```text
--- Phase 1: Parse ---
Parse: 576 passed, 0 failed
--- Phase 1b: Typecheck ---
Typecheck: 576 passed, 0 failed
--- Phase 1c: GF16 Conformance ---
GF16: conformance OK (typecheck clean)
--- Phase 2: Gen Zig ---
Gen Zig: 576 passed, 0 failed
--- Phase 2b: Gen Rust ---
Gen Rust: 576 passed, 0 failed
--- Phase 3: Gen Verilog ---
Gen Verilog: 576 passed, 0 failed
--- Phase 3b: Gen Verilog Yosys Smoke ---
Gen Verilog Yosys Smoke: 49 passed, 7 failed
--- Phase 3c: FPGA Board-Less Smoke Gate ---
  FPGA smoke gate: OK (report: .../build/fpga/smoke_gate_report.json)
--- Phase 4: Gen C ---
Gen C: 576 passed, 0 failed
--- Phase 5: Seal Verify ---
Seal Verify: 576 passed, 0 failed
--- Phase 6: Fixed Point ---
Fixed Point: 0 divergences

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

JSON summary sanity:

```bash
python3 -c "import json; d=json.load(open('build/suite_report_w445.json')); print(d['acceptable'], len(d['known_failures']), d['baseline_failures'], d['fpga_smoke_passed'], d['fpga_smoke_gate_elapsed_ms'])"
```

Output: `True 7 7 True 9`.

The 7 `known_failures` exactly match `docs/reports/gen_verilog_smoke_baseline.json`.

---

## 6. Golden fixture replay test

```bash
cargo test -p tri --bin tri fpga::tests::test_theorem_matrix_golden_replay_passes -- --nocapture
```

Result:

```text
running 1 test
golden theorem-matrix replay elapsed_ms: 4
test fpga::tests::test_theorem_matrix_golden_replay_passes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.02s
```

The checked-in golden fixture set under
`tests/fixtures/fpga/theorem-matrix/golden/` replays in ~4 ms and produces 24
variants with all `envelope_check: "ok"`.

---

## 7. Files changed

- `cli/tri/src/fpga.rs` — added `test_theorem_matrix_golden_replay_passes`.
- `bootstrap/src/suite.rs` — added `theorem_matrix_elapsed_ms` to
  `FpgaSmokeResult`, `fpga_smoke_gate_elapsed_ms` to `SuiteSummary`, and updated
  schema regression tests.
- `tests/fixtures/fpga/theorem-matrix/golden/` — new golden fixture set (75
  files) plus `README.md`.
- `fpga/HARDWARE_SSOT.md` — §3.6.26 extended with golden fixture path and
  elapsed_ms metric documentation.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — W445 boundary competitor note.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — W445 triage note.
- Close-out artifacts (new):
  `docs/reports/WAVE_LOOP_445_REPORT.md`,
  `docs/reports/FPGA_LOOP_PLAN_W445_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W445_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W446_2026-07-01.md`.

---

## 8. Still blocked / deferred

- **Physical bench:** DLC10 JTAG cable not detected (`VID=0x03FD`); P12 unwired;
  no relay gate.
- **Gen-verilog debt:** 7 residual yosys smoke failures remain; master-merge fix
  set on `master` (`701d79b3b`) deferred to a dedicated wave.
- **Full Trinity `lake build`:** still broken on unrelated physics proofs
  (`Trinity.NeutrinoMasses`, `Trinity.H4Lagrangian`); the targeted
  `lake build Trinity.TernaryFPGABoot` target remains green.
- **Issue #1419:** not yet created; `Closes #1419` is intentionally omitted
  from commits until the issue exists (HR-15 candidate rule).

---

*φ² + φ⁻² = 3 | TRINITY*
