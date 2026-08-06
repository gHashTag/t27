# FPGA Loop Evidence — Wave Loop 448 (2026-07-01)

**Issue:** #1423
**Branch:** `wave-loop-448`
**Variant:** B (bench still blocked)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Artifact list

| Artifact | Path | Status |
|---|---|---|
| W448 plan | `docs/reports/FPGA_LOOP_PLAN_W448_2026-07-01.md` | committed |
| W448 report | `docs/reports/WAVE_LOOP_448_REPORT.md` | committed |
| W448 cooperation (W449) | `docs/reports/FPGA_LOOP_COOPERATION_W449_2026-07-01.md` | committed |
| Dry-run-live fixtures | `tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/` (75 files + snapshot) | committed |
| CLI changes | `cli/tri/src/fpga.rs` | committed |
| Lean formal model | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | committed |
| Competitor report | `docs/reports/T27_VS_FORMAL_HDL_2026.md` | committed |

---

## Weak points addressed

1. **No committed dry-run-live anchor.** Now fixed: 75 fixtures committed with
   `expected_report.json` snapshot.
2. **Standalone Lean not in smoke gate.** Now fixed: `--validate-lean-standalone`
   runs inside `tri fpga smoke-gate --theorem-matrix` and asserts a temp lake
   package build.
3. **One-sided envelope formal story.** Now fixed: adversarial theorem proves the
   dashboard gate returns `false` for an outside-envelope operating point.
4. **Stale competitor report.** Now fixed: W448 boundary section added.

---

## Competitor snapshot (W448 boundary)

- **Sparkle / Verilean:** last push 2026-07-03; PR #66 open (~27K additions);
  RV32 divider proof merged 2026-06-25; README cites **102 formal theorems**.
- **CIRCT / firtool:** `firtool-1.152.0` shipped 2026-07-04; no `1.153.0` yet.
- **Clash:** `1.11.0` candidate unchanged; latest official `1.10.0`.
- **Ternary-FPGA niche:** no Lean-native formal competitor.

---

## Verification logs

```
cargo test -p tri
  test result: ok. 141 passed; 0 failed; 0 ignored; 0 measured

cargo test -p t27c --bin t27c suite::tests
  test result: ok. 8 passed; 0 failed; 0 ignored

lake build Trinity.TernaryFPGABoot
  Build completed successfully (2967 jobs)

./scripts/tri test --json build/suite_summary_w448.json
  Parse failures: 0
  Typecheck fails: 0
  GF16 conformance: 0
  Gen Zig failures: 0
  Gen Rust failures: 0
  Gen Verilog fails: 0
  Gen Verilog smoke fails: 7 (baseline)
  FPGA smoke fails: 0
  Gen C failures: 0
  Seal mismatches: 0
  FP divergences: 0
  TOTAL FAILURES: 7
  BASELINE FAILURES: 7
  ACCEPTABLE: yes
```

---

## Blockers remaining

- Physical DLC10 cable / P12 header still unavailable.
- Gen-verilog fix set on `master` still not merged; 7 yosys smoke failures remain.
- GitHub issue #1423 creation failed with HTTP 401 (needs `gh auth login`).

---

*φ² + φ⁻² = 3 | TRINITY*
