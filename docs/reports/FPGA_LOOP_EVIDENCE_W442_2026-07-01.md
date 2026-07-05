# FPGA Loop Evidence — Wave Loop 442 (2026-07-01)

**Issue:** #1415  
**Branch:** `wave-loop-442`  
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
test result: ok. 129 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.43s
```

Key FPGA regression tests included:
- `fpga::tests::test_cclk_period_ns_oscfsel_0_7` — asserts documented Artix-7 periods.
- `fpga::tests::test_theorem_matrix_synthetic_fixture_and_summary` — full
  temporary-directory matrix fixture/summary/verify-lean path.
- `fpga::tests::test_smoke_gate_json_synthetic_verify_lean` — existing board-less
  smoke-gate path.

---

## 3. Bootstrap `t27c` suite regression tests

```bash
cargo test -p t27c --bin t27c suite::tests
```

Result:

```text
running 4 tests
test suite::tests::test_parse_smoke_gate_report_missing_file ... ok
test suite::tests::test_parse_smoke_gate_report_schema_tolerant_without_theorem_matrix ... ok
test suite::tests::test_run_fpga_smoke_gate_passes_with_good_report ... ok
test suite::tests::test_run_fpga_smoke_gate_fails_with_bad_report ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1499 filtered out; finished in 0.93s
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

The 7 `known_failures` exactly match `docs/reports/gen_verilog_smoke_baseline.json`:

- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

---

## 6. Board-less corner×OSCFSEL theorem matrix

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
python3 -c "import json; d=json.load(open('build/fpga/smoke_gate_report.json')); print(d['passed'], d['schema_version'], d['theorem_matrix']['status'], d['theorem_matrix']['variant_count'], d['theorem_matrix']['corner_count'], d['theorem_matrix']['oscfsel_count'])"
```

Output: `True 1.0 ok 24 3 8`.

Per-corner, per-OSCFSEL periods (ns) from the report:

| corner | OSCFSEL | period_ns | sck_low_ns | sck_high_ns |
|---|---|---|---|---|
| ff | 0 | 400 | 200 | 200 |
| ff | 1 | 238 | 119 | 119 |
| ff | 2 | 151 | 75 | 76 |
| ff | 3 | 100 | 50 | 50 |
| ff | 4 | 80 | 40 | 40 |
| ff | 5 | 59 | 29 | 30 |
| ff | 6 | 40 | 20 | 20 |
| ff | 7 | 30 | 15 | 15 |
| tt | 0 | 400 | 200 | 200 |
| ... | ... | ... | ... | ... |
| ss | 7 | 30 | 15 | 15 |

The `period_ns` values are corner-independent in the Artix-7 documentation, but
each corner receives a distinct synthetic PVT context (`ff`/`tt`/`ss`) so the
PVT-aware flash-spec margin is recomputed per corner. All 24 variants generated
a `.lean` theorem and passed `verify-lean --expected-source synthetic`.

---

## 7. Files changed

- `cli/tri/src/fpga.rs` — theorem matrix now covers `ff`/`tt`/`ss` corners (24
  variants); smoke-gate report gains `schema_version: "1.0"`; new unit tests for
  `cclk_period_ns` and the theorem-matrix fixture/summary path.
- `bootstrap/src/suite.rs` — `FpgaSmokeResult` exposes `schema_version` and
  `theorem_matrix_status`; new schema-v1 and backward-tolerance tests.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — refreshed for W442 boundary.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — W442 triage note.
- `docs/reports/FPGA_LOOP_PLAN_W442_2026-07-01.md` — W442 decomposed plan (new).
- `docs/reports/WAVE_LOOP_442_REPORT.md` — this wave's close-out report (new).
- `docs/reports/FPGA_LOOP_EVIDENCE_W442_2026-07-01.md` — this file (new).
- `docs/reports/FPGA_LOOP_COOPERATION_W443_2026-07-01.md` — next-wave variants
  (new).

---

## 8. Still blocked / deferred

- **Physical bench:** DLC10 JTAG cable not detected (`VID=0x03FD`); P12 unwired;
  no relay gate.
- **Gen-verilog debt:** 7 residual yosys smoke failures remain; master-merge fix
  set on `master` (`701d79b3b`) deferred to a dedicated wave.
- **Full Trinity `lake build`:** still broken on unrelated physics proofs
  (`Trinity.NeutrinoMasses`, `Trinity.H4Lagrangian`); the targeted
  `lake build Trinity.TernaryFPGABoot` target remains green.

---

*φ² + φ⁻² = 3 | TRINITY*
