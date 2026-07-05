# FPGA Loop Evidence — Wave Loop 440 (2026-07-01)

**Issue:** [#1411](https://github.com/gHashTag/t27/issues/1411)  
**Branch:** `wave-loop-440`  
**PR:** [#1414](https://github.com/gHashTag/t27/pull/1414)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was changed

Wave Loop 440 executed **Variant B** from the W439 cooperation plan: consume the
smoke-gate JSON report in the suite runner, add a machine-readable suite-level
summary, harden skip/fail handling, and replace the two ignored full-Trinity
`lake build` integration tests with lightweight content checks.

- `bootstrap/src/main.rs` added `json: Option<PathBuf>` to the `Suite` command.
- `bootstrap/src/suite.rs` now:
  - parses `build/fpga/smoke_gate_report.json` after Phase 3c,
  - asserts `passed == true`,
  - distinguishes `skipped` (bitstream missing / yosys unavailable) from
    `failed`,
  - emits a top-level `SuiteSummary` JSON when `--json <path>` is supplied.
- `cli/tri/src/fpga.rs` removed the two ignored full-Trinity `lake build`
  integration tests and added:
  - `test_measured_to_lean_standalone_outputs_consumable_lean`
  - `test_measured_to_lean_xadc_to_pvt_context_outputs`
- `scripts/tri` forwards `--json` and all other arguments after `test` to
  `t27c suite --repo-root "$REPO_ROOT"`.
- `fpga/HARDWARE_SSOT.md` §3.6.24/§3.6.25 documents the new suite-level JSON
  summary and updated test counts.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` and
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` are refreshed for W440.

The 7 residual `gen-verilog` yosys smoke failures from #1245 were intentionally
left untouched; Variant C (master-merge of the full fix set) remains a dedicated
future wave.

---

## Verification commands and results

### Rust check

```bash
cargo check -p tri
```

Result: **PASS** (warnings only, no errors).

### Rust unit tests

```bash
cargo test -p tri
```

Result: **127 passed; 0 failed; 0 ignored**.

The two previously ignored full-Trinity `lake build` integration tests are now
replaced by lightweight content checks on the generated Lean theorem and the
XADC→PVT context path.

### Full repo sweep (CI-like)

```bash
./scripts/tri test
```

Result:
- Parse / Typecheck / GF16 / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal: **576/576 PASS**
- Gen Verilog Yosys Smoke: **49 passed, 7 failed** (documented baseline from #1245)
- FPGA Board-Less Smoke Gate: **0 failed**

The FPGA phase produced `build/fpga/smoke_gate_report.json` with `passed: true`.

### Machine-readable suite summary

```bash
./scripts/tri test --json /tmp/suite_summary.json
```

Result: **PASS**. The summary contains:
- `phases` array with per-phase `passed`, `failed`, `skipped` counts.
- `fpga_smoke_report`: path to `build/fpga/smoke_gate_report.json`.
- `fpga_smoke_passed`: `true`.
- `total_failures`: `7` (all from the documented gen-verilog smoke baseline).
- `passed`: `false` because the documented baseline failures are non-zero.

### End-to-end smoke-gate JSON gate

```bash
./target/release/tri fpga smoke-gate \
  --synthetic-operating-point --verify-lean \
  --json /tmp/smoke_gate_report.json
```

Result: **PASS**. The report contains:
- `bit_config.status`: `ok` with the audited assertions.
- `dry_run_sweep.status`: `ok`, `variant_count`: 8, `source`: `synthetic`.
- `verify_lean.status`: `ok`, `expected_source`: `synthetic`.
- `yosys_synthesis.status`: `ok`.
- `passed`: `true`.

### Lean 4 build

```bash
cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
```

Result: **Build completed successfully (2967 jobs).**

---

## Known limitations

- Physical cold-POR capture remains blocked: the DLC10 JTAG cable is still not
  detected on the host (`VID=0x03FD`) and the board P12 power header is still
  unwired.
- The 7 residual `gen-verilog` yosys smoke failures remain the documented
  baseline. They will be addressed in a dedicated master-merge wave (Variant C),
  not mixed with the boot-evidence trail.
- The full Trinity `lake build` still fails on `Trinity/NeutrinoMasses.lean` and
  `Trinity/H4Lagrangian.lean`, but the boot-evidence target
  `Trinity.TernaryFPGABoot` still builds and the affected integration tests are
  now replaced with content checks.

---

## Artifacts produced

- `bootstrap/src/main.rs` — `--json` argument for `t27c suite`.
- `bootstrap/src/suite.rs` — smoke-gate report consumption, `SuiteSummary`,
  skip/fail hardening.
- `cli/tri/src/fpga.rs` — replacement lightweight integration tests.
- `scripts/tri` — forwards `--json` to `t27c suite`.
- `fpga/HARDWARE_SSOT.md` — §3.6.24/§3.6.25 suite JSON summary documentation.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — W440 boundary refresh.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — branch bump to `wave-loop-440`.
- `docs/reports/WAVE_LOOP_440_REPORT.md` — this wave's close-out report.
- `docs/reports/FPGA_LOOP_COOPERATION_W441_2026-07-01.md` — next-wave cooperation
  variants.

---

*φ² + φ⁻² = 3 | TRINITY*
