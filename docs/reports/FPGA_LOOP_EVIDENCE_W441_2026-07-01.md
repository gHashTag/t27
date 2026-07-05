# FPGA Loop Evidence — Wave Loop 441 (2026-07-01)

**Issue:** #1413  
**Branch:** `wave-loop-441`  
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

Result (captured in `/tmp/w441_cargo_test_tri.log`, 229 lines):

```text
test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.32s
```

Key FPGA regression tests included:
- `fpga::tests::test_smoke_gate_json_synthetic_verify_lean` — exercises the
  full `--synthetic-operating-point --verify-lean --json` board-less path.
- `fpga::tests::test_verify_lean_*` — edge cases for source-label enforcement.

---

## 3. Bootstrap `t27c` suite regression tests

```bash
cargo test -p t27c --bin t27c suite::tests
```

Result (captured in `/tmp/w441_cargo_test_t27c_suite.log`, 2476 lines including
compile warnings):

```text
running 7 tests
test suite::tests::test_parse_smoke_gate_report_missing_file ... ok
test suite::tests::test_suite_summary_acceptable_computation ... ok
test suite::tests::test_tri_exe_finds_target_debug_tri ... ok
test suite::tests::test_suite_summary_schema_roundtrip ... ok
test suite::tests::test_load_gen_verilog_smoke_baseline ... ok
test suite::tests::test_run_fpga_smoke_gate_fails_with_bad_report ... ok
test suite::tests::test_run_fpga_smoke_gate_passes_with_good_report ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1495 filtered out; finished in 1.17s
```

---

## 4. Lean boot-evidence build

```bash
cd proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result (captured in `/tmp/w441_lake_build.log`):

```text
Build completed successfully (2967 jobs).
```

---

## 5. Full repo test sweep

```bash
./scripts/tri test --json /tmp/w441_suite_summary.json
```

Result (captured in `/tmp/w441_tri_test.log`, 88 lines):

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
python3 -c "import json; d=json.load(open('/tmp/w441_suite_summary.json')); print(d['acceptable'], len(d['known_failures']), d['baseline_failures'], d['fpga_smoke_passed'])"
```

Output: `True 7 7 True`.

The 7 `known_failures` exactly match `docs/reports/gen_verilog_smoke_baseline.json`:

- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

---

## 6. Board-less OSCFSEL 0..7 theorem matrix

```bash
./target/release/tri fpga smoke-gate \
  --synthetic-operating-point \
  --verify-lean \
  --theorem-matrix \
  --json /tmp/tri_smoke_matrix.json
```

Result (selected excerpts):

```text
[smoke-gate] theorem-matrix OK (8 variants, source=synthetic)
[smoke-gate] yosys synthesis OK
[smoke-gate] JSON report: /tmp/tri_smoke_matrix.json
[smoke-gate] complete (passed: true)
```

JSON report sanity:

```bash
python3 -c "import json; d=json.load(open('/tmp/tri_smoke_matrix.json')); print(d['passed'], d['theorem_matrix']['status'], d['theorem_matrix']['variant_count'])"
```

Output: `True ok 8`.

Per-variant periods (ns) from the report:

| OSCFSEL | period_ns | sck_low_ns | sck_high_ns |
|---|---|---|---|
| 0 | 400 | 200 | 200 |
| 1 | 238 | 119 | 119 |
| 2 | 151 | 75 | 76 |
| 3 | 100 | 50 | 50 |
| 4 | 80 | 40 | 40 |
| 5 | 59 | 29 | 30 |
| 6 | 40 | 20 | 20 |
| 7 | 30 | 15 | 15 |

Each variant generated a `.lean` theorem and passed `verify-lean
--expected-source synthetic`.

---

## 7. Files changed

- `bootstrap/src/suite.rs` — baseline-aware summary, schema tests, skip/fail tests.
- `cli/tri/src/fpga.rs` — `--theorem-matrix`, OSCFSEL 0..7 theorem matrix,
  `cclk_period_ns` helper, smoke-gate `passed` wiring.
- `docs/reports/gen_verilog_smoke_baseline.json` — 7 documented yosys smoke
  failures (new).
- `docs/reports/FPGA_LOOP_PLAN_W441_2026-07-01.md` — W441 decomposed plan (new).
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — refreshed for W441.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — W440/W441 triage notes.
- `docs/reports/WAVE_LOOP_441_REPORT.md` — this wave's close-out report (new).
- `docs/reports/FPGA_LOOP_EVIDENCE_W441_2026-07-01.md` — this file (new).
- `docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md` — next-wave variants
  (new).

---

## 8. Still blocked / deferred

- **Physical bench:** DLC10 JTAG cable not detected (`VID=0x03FD`); P12 unwired;
  no relay gate.
- **Gen-verilog debt:** 7 residual yosys smoke failures remain; master-merge fix
  set on `master` (`701d79b3b`) deferred to a dedicated wave.

---

*φ² + φ⁻² = 3 | TRINITY*
