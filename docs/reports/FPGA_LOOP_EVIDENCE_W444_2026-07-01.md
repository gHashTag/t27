# FPGA Loop Evidence — Wave Loop 444 (2026-07-01)

**Issue:** #1418  
**Branch:** `wave-loop-444`  
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
```

Result: release `tri` binary built successfully (6 pre-existing warnings).

---

## 2. `tri` crate unit tests

```bash
cargo test -p tri --bin tri
```

Result:

```text
test result: ok. 136 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.47s
```

Key new regression tests:
- `fpga::tests::test_theorem_matrix_fixture_roundtrip`
- `fpga::tests::test_theorem_matrix_replay_does_not_regenerate`
- Existing `test_smoke_gate_json_synthetic_verify_lean` and the W443 envelope tests
  still pass.

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

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1495 filtered out; finished in 0.85s
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
./scripts/tri test --json build/suite_report_w444_final.json
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
python3 -c "import json; d=json.load(open('build/suite_report_w444_final.json')); print(d['acceptable'], len(d['known_failures']), d['baseline_failures'], d['fpga_smoke_passed'])"
```

Output: `True 7 7 True`.

The 7 `known_failures` exactly match `docs/reports/gen_verilog_smoke_baseline.json`.

---

## 6. Theorem-matrix fixture generation

```bash
rm -rf build/fpga/theorem-matrix-fixtures
./target/release/tri fpga smoke-gate \
  --synthetic-operating-point \
  --verify-lean \
  --theorem-matrix \
  --json build/fpga/smoke_gate_report.json
```

Result (selected excerpts):

```text
[smoke-gate] theorem-matrix: generating OSCFSEL 0..7 synthetic theorems for ff/tt/ss
[smoke-gate] theorem-matrix OK (24 variants, source=synthetic, 10 ms)
[smoke-gate] yosys synthesis OK
[smoke-gate] JSON report: build/fpga/smoke_gate_report.json
[smoke-gate] complete (passed: true)
```

JSON report sanity:

```bash
python3 -c "import json; d=json.load(open('build/fpga/smoke_gate_report.json')); print(d['passed'], d['schema_version'], d['theorem_matrix']['status'], d['theorem_matrix']['variant_count']); print([v['envelope_check'] for v in d['theorem_matrix']['variants'][:4]]); print(all('fixtures' in v for v in d['theorem_matrix']['variants']))"
```

Output:

```text
True 1.0 ok 24
['ok', 'ok', 'ok', 'ok']
True
```

All 24 variants carry `envelope_check: "ok"` and a `fixtures` block.

Sample fixture tree:

```text
build/fpga/theorem-matrix-fixtures/
├── theorem_matrix_pvt_ff.json
├── theorem_matrix_pvt_ss.json
├── theorem_matrix_pvt_tt.json
├── theorem_matrix_raw_ns_ff_0.json
├── theorem_matrix_raw_ns_ff_1.json
...
├── theorem_matrix_summary_ss_7.json
├── theorem_matrix_ss_oscfsel_7.lean
...
```

---

## 7. Theorem-matrix fixture replay

```bash
./target/release/tri fpga smoke-gate \
  --synthetic-operating-point \
  --verify-lean \
  --theorem-matrix \
  --replay-fixtures build/fpga/theorem-matrix-fixtures \
  --json build/fpga/smoke_gate_report_replay.json
```

Result (selected excerpts):

```text
[smoke-gate] theorem-matrix: replaying from fixtures build/fpga/theorem-matrix-fixtures
[smoke-gate] theorem-matrix replay OK (24 variants, 3 ms)
[smoke-gate] yosys synthesis OK
[smoke-gate] JSON report: build/fpga/smoke_gate_report_replay.json
[smoke-gate] complete (passed: true)
```

JSON report sanity:

```bash
python3 -c "import json; d=json.load(open('build/fpga/smoke_gate_report_replay.json')); tm=d['theorem_matrix']; print(tm['replay'], tm['elapsed_ms'], len(tm['variants']), sum(1 for v in tm['variants'] if v['envelope_check']=='ok'))"
```

Output: `True 3 24 24`.

Generation took ~10 ms; replay took ~3 ms. Both produced 24 variants with all
`envelope_check: "ok"`.

---

## 8. Files changed

- `cli/tri/src/fpga.rs` — added `--replay-fixtures`, `generate_theorem_matrix`,
  `replay_theorem_matrix`, per-variant `fixtures` block, `replay`/`elapsed_ms`
  report fields, and unit tests.
- `bootstrap/src/suite.rs` — suite runner now passes `--theorem-matrix`; fake
  report test exercises the new report shape.
- `fpga/HARDWARE_SSOT.md` — §3.6.26 documents fixture replay.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — W444 boundary competitor note.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — W444 triage note.
- `docs/reports/WAVE_LOOP_444_REPORT.md` — close-out report (new).
- `docs/reports/FPGA_LOOP_PLAN_W444_2026-07-01.md` — decomposed plan (new).
- `docs/reports/FPGA_LOOP_EVIDENCE_W444_2026-07-01.md` — this file (new).
- `docs/reports/FPGA_LOOP_COOPERATION_W445_2026-07-01.md` — next-wave variants
  (new).

---

## 9. Still blocked / deferred

- **Physical bench:** DLC10 JTAG cable not detected (`VID=0x03FD`); P12 unwired;
  no relay gate.
- **Gen-verilog debt:** 7 residual yosys smoke failures remain; master-merge fix
  set on `master` (`701d79b3b`) deferred to a dedicated wave.
- **Full Trinity `lake build`:** still broken on unrelated physics proofs
  (`Trinity.NeutrinoMasses`, `Trinity.H4Lagrangian`); the targeted
  `lake build Trinity.TernaryFPGABoot` target remains green.
- **Issue #1418:** not yet created; `Closes #1418` is intentionally omitted from
  commits until the issue exists (HR-15 candidate rule).

---

*φ² + φ⁻² = 3 | TRINITY*
