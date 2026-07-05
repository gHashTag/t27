# FPGA Loop Evidence — Wave Loop 439 (2026-07-05)

**Issue:** [#1409](https://github.com/gHashTag/t27/issues/1409)  
**Branch:** `wave-loop-439`  
**PR:** [#1412](https://github.com/gHashTag/t27/pull/1412) (predicted)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was changed

Wave Loop 439 executed **Variant B** from the W438 cooperation plan: wire the
W438 dry-run synthetic + `verify-lean` artifact trail into the default
`./scripts/tri test` FPGA phase and make the smoke gate machine-readable.

- `bootstrap/src/suite.rs` Phase 3c now invokes `tri fpga smoke-gate
  --synthetic-operating-point --verify-lean --json build/fpga/smoke_gate_report.json`
  when the demo bitstream is present, replacing the older direct Python/yosys
  calls.
- `tri fpga smoke-gate` gained `--json <path>` and emits a single JSON report
  with per-phase status for bit-config audit, dry-run CCLK sweep, verify-lean,
  and yosys synthesis.
- The bit-config audit now captures the `ASSERTION OK:` result lines from
  `scripts/dump_bit_config.py` in the JSON report.
- A new regression test, `test_smoke_gate_json_synthetic_verify_lean`, exercises
  the full board-less artifact path end-to-end.
- `repo_root()` was fixed to prefer a `.git` directory over a `Cargo.toml` file
  when climbing from the current working directory, so unit tests in `cli/tri`
  resolve to the workspace root.
- `fpga/HARDWARE_SSOT.md` §3.6.24 documents the smoke-gate `--json` schema.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` was refreshed; no new post-2026-07-11
  public competitor signals appeared.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` was updated to `wave-loop-439`.

The 7 residual `gen-verilog` yosys smoke failures from #1245 were intentionally
left untouched; Variant C (master-merge of the full fix set) remains a
dedicated future wave.

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

Result: **125 passed; 0 failed; 2 ignored**.

The 2 ignored tests are full-Trinity `lake build` integration tests blocked by
pre-existing build failures in `Trinity/NeutrinoMasses.lean` and
`Trinity/H4Lagrangian.lean` (unrelated physics proofs).

### Full repo sweep (CI-like)

```bash
./scripts/tri test
```

Result:
- Parse / Typecheck / GF16 / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal: **576/576 PASS**
- Gen Verilog Yosys Smoke: **49 passed, 7 failed** (documented baseline from #1245)
- FPGA Board-Less Smoke Gate: **0 failed**

The FPGA phase produced `build/fpga/smoke_gate_report.json` with
`passed: true`.

### Lean 4 build

```bash
cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
```

Result: **Build completed successfully (2967 jobs).**

### End-to-end smoke-gate JSON gate

```bash
./target/release/tri fpga smoke-gate \
  --synthetic-operating-point --verify-lean \
  --json /tmp/smoke_gate_report.json
```

Result: **PASS**. The report contains:
- `bit_config.status`: `ok` with the five audited assertions.
- `dry_run_sweep.status`: `ok`, `variant_count`: 8, `source`: `synthetic`.
- `verify_lean.status`: `ok`, `expected_source`: `synthetic`.
- `yosys_synthesis.status`: `ok`.
- `passed`: `true`.

### Default smoke gate (regression check)

```bash
./target/release/tri fpga smoke-gate
```

Result: **PASS**. Default behaviour is unchanged; no synthetic source assertion
or verify-lean step is performed without the new flags.

---

## Known limitations

- Physical cold-POR capture remains blocked: the DLC10 JTAG cable is still not
  detected on the host (`VID=0x03FD`) and the board P12 power header is still
  unwired.
- The 7 residual `gen-verilog` yosys smoke failures remain the documented
  baseline. They will be addressed in a dedicated master-merge wave (Variant C),
  not mixed with the boot-evidence trail.
- `lake build` for the full Trinity package currently fails on
  `Trinity/NeutrinoMasses.lean` and `Trinity/H4Lagrangian.lean`, so two
  standalone integration tests are ignored. The boot-evidence target
  `Trinity.TernaryFPGABoot` still builds.

---

## Artifacts produced

- `cli/tri/src/fpga.rs` — `--json` output for smoke-gate, assertion capture,
  regression test, `repo_root()` heuristic fix.
- `bootstrap/src/suite.rs` — FPGA Phase 3c integration of `tri fpga smoke-gate`.
- `fpga/HARDWARE_SSOT.md` — §3.6.24 smoke-gate `--json` schema.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — W439 boundary refresh.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — branch bump to `wave-loop-439`.
- `docs/reports/WAVE_LOOP_439_REPORT.md` — this wave's close-out report.
- `docs/reports/FPGA_LOOP_COOPERATION_W440_2026-07-05.md` — next-wave cooperation
  variants.

---

*φ² + φ⁻² = 3 | TRINITY*
