# Wave Loop 439 — Close-out Report

**Issue:** [#1409](https://github.com/gHashTag/t27/issues/1409)  
**Branch:** `wave-loop-439`  
**PR:** [#1412](https://github.com/gHashTag/t27/pull/1412) (predicted)  
**Date:** 2026-07-05  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 439 executed **Variant B** of the W438 cooperation plan: wire the
dry-run synthetic + `verify-lean` artifact trail into the default `./scripts/tri
test` FPGA phase, make the smoke gate machine-readable with `--json`, and add a
regression test that exercises the full board-less path end-to-end.

The wave adds no hardware dependency, keeps the 7 residual `gen-verilog` yosys
smoke failures as the documented baseline, and leaves physical capture (Variant
A) and the master-merge gen-verilog fix set (Variant C) for future waves.

---

## What was delivered

1. **Smoke-gate wired into the default CI sweep**
   - `bootstrap/src/suite.rs` Phase 3c now invokes `tri fpga smoke-gate
     --synthetic-operating-point --verify-lean --json
     build/fpga/smoke_gate_report.json` when the demo bitstream is present.
   - The suite runner locates the `tri` binary from the same build profile as
     the running `t27c`, falling back to `target/release/tri`, `target/debug/tri`,
     and the bootstrap target directories.

2. **Machine-readable smoke-gate JSON report**
   - `tri fpga smoke-gate --json <path>` emits a single JSON object with
     per-phase results for bit-config audit, dry-run CCLK sweep, verify-lean,
     and yosys synthesis, plus an overall `passed` boolean.
   - The bit-config phase now captures the actual `ASSERTION OK:` result lines
     from `dump_bit_config.py`.

3. **Regression test**
   - `cli/tri/src/fpga.rs` gained `test_smoke_gate_json_synthetic_verify_lean`,
     which runs the full board-less gate with `--synthetic-operating-point
     --verify-lean`, writes a JSON report, and asserts `passed: true` along with
     populated `ok` phases.

4. **Repository-root heuristic fix**
   - `repo_root()` in `cli/tri/src/fpga.rs` now prefers a directory containing
     `.git` over one containing only `Cargo.toml`, so unit tests run from the
     `cli/tri` crate root still resolve to the workspace root.

5. **Documentation and competitor refresh**
   - `fpga/HARDWARE_SSOT.md` §3.6.24 documents the smoke-gate `--json` schema
     with field types and an example.
   - `docs/reports/T27_VS_FORMAL_HDL_2026.md` is refreshed for the W439 boundary;
     no new public competitor signals appeared after Sparkle's 関数型まつり2026
     talk on 2026-07-11.
   - `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` is updated to `wave-loop-439`.

6. **Evidence and cooperation artifacts**
   - `docs/reports/FPGA_LOOP_EVIDENCE_W439_2026-07-05.md` records all
     verification commands and results.
   - `docs/reports/FPGA_LOOP_COOPERATION_W440_2026-07-05.md` proposes three
     cooperation variants for Wave Loop 440.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | **PASS** |
| `cargo test -p tri` | **125/125 PASS, 2 IGNORED** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` | **576/576 non-smoke PASS; 7/56 yosys smoke failures** (documented baseline); FPGA smoke fails: 0 |
| `tri fpga smoke-gate --synthetic-operating-point --verify-lean --json /tmp/report.json` | **PASS** |

The 2 ignored tests are full-Trinity `lake build` integration tests that are
blocked by pre-existing build failures in `Trinity/NeutrinoMasses.lean` and
`Trinity/H4Lagrangian.lean` (unrelated physics proofs). `lake build
Trinity.TernaryFPGABoot` still succeeds.

---

## Outstanding risks

- **Hardware remains blocked.** The DLC10 JTAG cable is not detected and the P12
  power header is unwired, so real cold-POR capture (Variant A) cannot proceed.
- **Gen-verilog debt remains.** The 7 residual yosys smoke failures are stable
  but will require a dedicated master-merge wave (Variant C).
- **Full Trinity lake build is broken** on unrelated physics proofs, which
  prevents two integration tests from running. This does not affect the
  TernaryFPGABoot target used by the boot-evidence pipeline.

---

## Next wave recommendation

See `docs/reports/FPGA_LOOP_COOPERATION_W440_2026-07-05.md` for three cooperation
variants for Wave Loop 440.

---

*φ² + φ⁻² = 3 | TRINITY*
