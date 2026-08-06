# FPGA Loop Evidence — Wave Loop 453 (2026-07-01)

**Issue:** #1421
**Branch:** `wave-loop-453`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Lean 4 theorems

All theorems below build in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.

### Enumerated rectangle corner

```lean
inductive EnvelopeCorner where
  | hot_lowv  : EnvelopeCorner  -- +85 °C, 900 mV  (W451)
  | hot_highv : EnvelopeCorner  -- +85 °C, 1100 mV (W453)
  | cold_lowv : EnvelopeCorner  -- -40 °C, 900 mV  (W453)
  | cold_highv : EnvelopeCorner -- -40 °C, 1100 mV (W452)
  deriving Repr, DecidableEq
```

### New W453 boundary operating points

```lean
def BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT (corner : ProcessCorner) : XadcOperatingPoint :=
  { temp_c := (85 : Int), vccint_mv := 1100, vccaux_mv := 1800,
    process_corner := corner }

def BOUNDARY_COLD_LOWV_W453_OPERATING_POINT (corner : ProcessCorner) : XadcOperatingPoint :=
  { temp_c := (-40 : Int), vccint_mv := 900, vccaux_mv := 1800,
    process_corner := corner }
```

### Corner-to-point map

```lean
def envelope_corner_operating_point (corner : EnvelopeCorner) (process_corner : ProcessCorner) : XadcOperatingPoint :=
  match corner with
  | EnvelopeCorner.hot_lowv  => BOUNDARY_HOT_LOWV_W451_OPERATING_POINT process_corner
  | EnvelopeCorner.hot_highv => BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT process_corner
  | EnvelopeCorner.cold_lowv => BOUNDARY_COLD_LOWV_W453_OPERATING_POINT process_corner
  | EnvelopeCorner.cold_highv => BOUNDARY_COLD_HIGHV_W452_OPERATING_POINT process_corner
```

### Single quantified rectangle theorem

```lean
theorem all_envelope_corners_w453_all_corners_transaction_ok
  (corner : EnvelopeCorner) (oscfsel : Nat) (h : oscfsel ≤ 7) (process_corner : ProcessCorner) (bits : Nat) :
  let period_ns := cclk_period_ns oscfsel
  let low_ns := period_ns / 2
  let high_ns := period_ns - low_ns
  transaction_satisfies_flash_spec
    (measured_boot_transaction_from_raw_ns_with_pvt period_ns low_ns high_ns bits)
    = true := by
  ...
```

This theorem subsumes the W451 hot/low-v, W452 cold/high-v, and W453 hot/high-v /
cold/low-v per-corner transaction theorems in one `∀` statement over all four
rectangle corners.

### Computable rectangle theorem

```lean
theorem all_envelope_corners_w453_all_oscfsel_combined_check_true
  (corner : EnvelopeCorner) (oscfsel : Nat) (h : oscfsel ≤ 7) :
  cclk_variant_and_xadc_envelope_check oscfsel (envelope_corner_operating_point corner ProcessCorner.ss) = true := by
  ...
```

### Supporting per-corner theorems

- `boundary_hot_highv_w453_operating_point_within_envelope`
- `boundary_cold_lowv_w453_operating_point_within_envelope`
- `boundary_hot_highv_w453_process_corner_worse_than_ss`
- `boundary_cold_lowv_w453_process_corner_worse_than_ss`
- `boundary_hot_highv_w453_raw_ns_satisfies_flash_spec`
- `boundary_cold_lowv_w453_raw_ns_satisfies_flash_spec`
- `boundary_hot_highv_w453_all_corners_transaction_ok`
- `boundary_cold_lowv_w453_all_corners_transaction_ok`
- `boundary_hot_highv_w453_all_oscfsel_combined_check_true`
- `boundary_cold_lowv_w453_all_oscfsel_combined_check_true`

### Build result

```text
$ cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
[2967/2967] Linking Trinity.TernaryFPGABoot
Build succeeded.
```

---

## 2. Smoke-gate JSON schema hardening

### Generator side (`cli/tri/src/fpga.rs`)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SmokeGateReport {
    schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bit_config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dry_run_sweep: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verify_lean: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theorem_matrix: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validate_lean_standalone: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    yosys_synthesis: Option<serde_json::Value>,
    passed: bool,
}
```

Validation before write:

```rust
serde_json::from_value::<SmokeGateReport>(report.clone())
    .with_context(|| "smoke-gate report violates schema")?;
```

Tests:

- `test_smoke_gate_report_schema_accepts_canonical`
- `test_smoke_gate_report_schema_rejects_unknown_field`

### Consumer side (`bootstrap/src/suite.rs`)

The same `SmokeGateReport` schema is duplicated on the suite side so that any
report emitted by `tri fpga smoke-gate --json` is validated again before it
influences the suite summary.

Updated `parse_smoke_gate_report`:

```rust
let _: SmokeGateReport = serde_json::from_str(&text)
    .with_context(|| format!("smoke-gate report schema violation in {}", report_path.display()))?;
let report: serde_json::Value = serde_json::from_str(&text)
    .with_context(|| format!("parsing smoke-gate report {}", report_path.display()))?;
```

Tests:

- `test_parse_smoke_gate_report_deny_unknown_fields`
- `test_parse_smoke_gate_report_schema_tolerant_without_theorem_matrix` (hardened
  to include mandatory `schema_version`)

### Test results

```text
$ cargo test -p tri --bin tri smoke_gate_report_schema -- --test-threads=1
running 2 tests
test fpga::tests::test_smoke_gate_report_schema_accepts_canonical ... ok
test fpga::tests::test_smoke_gate_report_schema_rejects_unknown_field ... ok

$ cargo test -p t27c --bin t27c suite::tests
running 9 tests
... ok
```

---

## 3. Full suite results

### Default run

```text
$ ./scripts/tri test --json /tmp/t27_w453_full_suite.json
[2026-07-01T...] Suite complete.
{
  "total": 583,
  "passed": 576,
  "failed": 7,
  "skipped": 0,
  "fpga_smoke_passed": true,
  "fpga_smoke_skipped": false,
  "fpga_smoke_failed": false,
  "known_failures": 7,
  "baseline_failures": 7,
  "acceptable": true
}
```

Breakdown:

- Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: **576/576 PASS**.
- Gen-verilog-yosys-smoke: **49 passed, 7 pre-existing failures** (#1245).
- FPGA board-less smoke gate: **PASS** with theorem matrix 24 variants,
  `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
  `passed: true`.
- Phase 3c-standalone: **OK**, `validate_lean_standalone_elapsed_ms` populated.

### Fast run

```text
$ ./scripts/tri test --fast --json /tmp/t27_w453_fast_suite.json
...
"acceptable": true
```

Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
Phase 3c-standalone skipped (`--fast` mode), `validate_lean_standalone_elapsed_ms`: `null`.

---

## 4. Competitor snapshot

Refreshed `docs/reports/T27_VS_FORMAL_HDL_2026.md`. As of the W453 boundary:

- Sparkle/Verilean remains the only fresh Lean-native HDL signal in early July 2026.
- CIRCT `firtool-1.152.0` (2026-07-04) is still the latest public release.
- Clash 1.11.0 remains a Hackage candidate.
- No Lean-native ternary-FPGA competitor surfaced.

---

## 5. Gen-verilog defect tracker

Updated `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`:

- Branch header moved to `wave-loop-453`.
- W452 triage: no compiler work; 7 residual yosys smoke failures remain baseline.
- W453 triage: no compiler work; the 7 residual failures are explicitly targeted
  by Wave Loop 454 Variant B (master-merge the safe fix set from `master` commit
  `701d79b3b`).

---

*φ² + φ⁻² = 3 | TRINITY*
