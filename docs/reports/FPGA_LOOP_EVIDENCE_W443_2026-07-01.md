# FPGA Loop Evidence — Wave Loop 443 (2026-07-01)

**Issue:** #1417  
**Branch:** `wave-loop-443`  
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

Result: release binary built successfully.

---

## 2. `tri` crate unit tests

```bash
cargo test -p tri --bin tri
```

Result:

```text
test result: ok. 134 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.33s
```

Key new regression tests:
- `fpga::tests::test_pvt_envelope_json_report_inside_envelope_true`
- `fpga::tests::test_pvt_envelope_json_report_no_context_skipped`
- `fpga::tests::test_synthetic_pvt_context_inside_envelope_all_corners`
- `fpga::tests::test_pvt_context_outside_envelope_detected`
- `fpga::tests::test_theorem_matrix_synthetic_context_envelope_check_ok`
- Existing `test_cclk_period_ns_oscfsel_0_7` and
  `test_theorem_matrix_synthetic_fixture_and_summary` still pass.

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

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1495 filtered out; finished in 0.93s
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
./scripts/tri test --json build/suite_report.json
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
python3 -c "import json; d=json.load(open('build/suite_report.json')); print(d['acceptable'], len(d['known_failures']), d['baseline_failures'], d['fpga_smoke_passed'])"
```

Output: `True 7 7 True`.

The 7 `known_failures` exactly match `docs/reports/gen_verilog_smoke_baseline.json`.

---

## 6. PVT-envelope JSON verdict

```bash
echo '{"temp_c":42,"vccint_mv":1000,"vccaux_mv":1800,"process_corner":"tt"}' > /tmp/tri_pvt_ok.json
./target/release/tri fpga pvt-envelope --pvt-context /tmp/tri_pvt_ok.json --json | python3 -m json.tool
```

Result:

```json
{
    "envelope_check": "ok",
    "inside_envelope": true,
    "margin_ns": 3,
    "min_sck_half_ns": 9,
    "nominal_min_sck_half_ns": 6,
    "operating_envelope": {
        "temp_c_max": 85,
        "temp_c_min": -40,
        "vccint_mv_max": 1100,
        "vccint_mv_min": 900
    },
    "pvt_context": {
        "process_corner": "tt",
        "source": "pvt_context_file",
        "temp_c": 42,
        "vccaux_mv": 1800,
        "vccint_mv": 1000
    },
    "warnings": []
}
```

Out-of-envelope context is rejected:

```bash
echo '{"temp_c":200,"vccint_mv":1000,"vccaux_mv":1800,"process_corner":"tt"}' > /tmp/tri_pvt_bad.json
./target/release/tri fpga pvt-envelope --pvt-context /tmp/tri_pvt_bad.json --json
```

Result:

```text
Error: PVT temp_c 200 is outside operating envelope [-40..85] °C
```

---

## 7. Board-less corner×OSCFSEL theorem matrix with envelope checks

```bash
./target/release/tri fpga smoke-gate \
  --synthetic-operating-point \
  --verify-lean \
  --theorem-matrix \
  --json build/fpga/smoke_gate_report.json
```

Result (selected excerpts):

```text
[smoke-gate] theorem-matrix: generating OSCFSEL 0..7 synthetic theorems for ff/tt/ss
[smoke-gate] theorem-matrix OK (24 variants, source=synthetic)
[smoke-gate] yosys synthesis OK
[smoke-gate] JSON report: build/fpga/smoke_gate_report.json
[smoke-gate] complete (passed: true)
```

JSON report sanity:

```bash
python3 -c "import json; d=json.load(open('build/fpga/smoke_gate_report.json')); print(d['passed'], d['schema_version'], d['theorem_matrix']['status'], d['theorem_matrix']['variant_count']); print([v['envelope_check'] for v in d['theorem_matrix']['variants'][:4]])"
```

Output:

```text
True 1.0 ok 24
['ok', 'ok', 'ok', 'ok']
```

All 24 variants carry `envelope_check: "ok"`.

---

## 8. Files changed

- `cli/tri/src/fpga.rs` — `build_pvt_envelope_report` emits
  `inside_envelope`/`envelope_check`; theorem matrix validates and records
  `envelope_check`; new unit tests.
- `bootstrap/src/suite.rs` — fake smoke-gate report includes `envelope_check`
  in the theorem-matrix variant.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — refreshed for W443 boundary.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — W443 triage note.
- `docs/reports/FPGA_LOOP_PLAN_W443_2026-07-01.md` — W443 decomposed plan (new).
- `docs/reports/WAVE_LOOP_443_REPORT.md` — close-out report (new).
- `docs/reports/FPGA_LOOP_EVIDENCE_W443_2026-07-01.md` — this file (new).
- `docs/reports/FPGA_LOOP_COOPERATION_W444_2026-07-01.md` — next-wave variants
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

---

*φ² + φ⁻² = 3 | TRINITY*
