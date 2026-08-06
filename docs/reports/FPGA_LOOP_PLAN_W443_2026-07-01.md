# FPGA Loop Decomposed Implementation Plan — Wave Loop 443 (2026-07-01)

**Issue:** #1417  
**Branch:** `wave-loop-443`  
**Variant:** B (default) — PVT-envelope hardening for the 24-variant theorem matrix  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified in W442

| Weak point | Why it matters | Where it lives |
|---|---|---|
| Theorem matrix generates theorems without an explicit PVT-envelope check | The synthetic corner contexts are hand-picked to be inside the envelope, but CI consumers cannot see this fact per variant. | `cli/tri/src/fpga.rs` `smoke_gate` theorem-matrix block |
| Smoke-gate report `theorem_matrix` variants do not record envelope status | Downstream CI cannot distinguish "theorem generated" from "context validated against operating rectangle". | `cli/tri/src/fpga.rs` report object |
| No unit test for per-variant envelope validation in the matrix path | A regression in `parse_pvt_context` or `synthetic_pvt_context` could silently move a context outside the envelope. | `cli/tri/src/fpga.rs` tests |
| `tri fpga pvt-envelope --json` does not expose an `inside_envelope` boolean | The command prints bounds and examples, but does not return a machine-readable verdict for a supplied context. | `cli/tri/src/fpga.rs` `pvt_envelope` / `build_pvt_envelope_report` |
| Competitor checkpoint is stale | Sparkle's public activity is still the July 3 2026 push; no post-2026-07-11 public signals exist yet. | `docs/reports/T27_VS_FORMAL_HDL_2026.md` |

---

## 2. Competitor signals used for W443 planning

- **Sparkle / Verilean**
  - Last public push on `main`: **2026-07-03 19:27:50Z**.
  - Open PR #66 remains large (`+27,013 / -120` across 204 files) — USB web server, memcached ASCII server, compiler performance and robustness fixes.
  - PR #57 (analogue circuit simulation) is closed; the analogue work is being spun out to a separate repo.
  - No new public commits or PRs appeared between the W442 boundary and this planning date.
- **CIRCT / firtool**
  - No `firtool-1.153.0` release exists yet; `firtool-1.152.0` (2026-07-04) remains the latest public release.
- **Ternary FPGA ecosystem**
  - **TernaryCore** (`shepherdscientific/ternarycore`): last push 2026-05-27; RTL simulation 31/31 passing; no formal proofs.
  - **ternfpga** (`Neumann-Labs/ternfpga`): created June 2026; full sub-watt BitNet inference engine on Arty A7-35T; no Lean-native proof pipeline.
  - Neither project combines ternary compute with a Lean-native proof pipeline, so t27's differentiation remains intact.

Strategic takeaway for W443: continue hardening the **machine-readable CI gate + board-less formal coverage matrix** because Sparkle still does not match that combination, and keep the post-2026-07-11 Sparkle talk window as the next refresh trigger.

Sources:
- [Verilean/sparkle](https://github.com/Verilean/sparkle)
- [Sparkle PR #66 — IP.Net + compiler perf](https://github.com/Verilean/sparkle/pull/66)
- [Sparkle PR #57 — analogue circuits](https://github.com/Verilean/sparkle/pull/57)
- [CIRCT releases](https://github.com/llvm/circt/releases)
- [TernaryCore](https://github.com/shepherdscientific/ternarycore)
- [ternfpga](https://github.com/Neumann-Labs/ternfpga)

---

## 3. Decomposed implementation tasks

### 3.1 Add machine-readable `inside_envelope` verdict to `tri fpga pvt-envelope --json`

1. Extend `build_pvt_envelope_report` in `cli/tri/src/fpga.rs` so that when a
   `pvt_context` file is supplied the report includes:
   - `operating_point` object with `temp_c`, `vccint_mv`, `vccaux_mv`, `process_corner`, `source`.
   - `inside_envelope`: boolean.
   - `envelope_check`: `"ok"` | `"failed"` | `"skipped"`.
2. Keep backward compatibility: when no context is supplied, `inside_envelope` is
   `null` and `envelope_check` is `"skipped"`.
3. Add unit tests for:
   - A context inside the envelope → `inside_envelope: true`, `envelope_check: "ok"`.
   - A context outside temperature bounds → `inside_envelope: false`,
     `envelope_check: "failed"`.
   - No context → `inside_envelope: null`, `envelope_check: "skipped"`.

### 3.2 Validate every synthetic corner context before generating a matrix theorem

1. In the theorem-matrix block of `cli/tri/src/fpga.rs`, after building the
   synthetic `PvtContext` for a corner, run the same envelope check used by
   `parse_pvt_context`.
2. Record the result as `envelope_check: "ok"` / `"failed"` in the per-variant
   matrix entry.
3. If any variant fails the envelope check, set `theorem_matrix.status` to
   `"failed"` and bail; synthetic contexts should never fail, so a failure is a
   regression worth surfacing.

### 3.3 Extend smoke-gate `theorem_matrix` report with `envelope_check`

1. Add `envelope_check: "ok"` (or `"failed"`) to every per-variant entry.
2. Keep `schema_version: "1.0"` unchanged; the new field is an additive
   extension.
3. Update the schema-tolerant test in `bootstrap/src/suite.rs` to accept the new
   field.

### 3.4 Add Rust unit tests

1. `test_synthetic_pvt_context_inside_envelope_ff_tt_ss` — assert that
   `synthetic_pvt_context(Ff/Tt/Ss)` is inside the operating envelope.
2. `test_theorem_matrix_envelope_check_recorded` — extend the existing matrix
   fixture/summary test to also parse the smoke-gate report fragment and assert
   `envelope_check == "ok"`.
3. Update `test_run_fpga_smoke_gate_passes_with_good_report` fake report to
   include `envelope_check: "ok"`.

### 3.5 Competitor and baseline documentation refresh

1. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the W443 boundary notes:
   Sparkle last push 2026-07-03, no new post-2026-07-11 signals, firtool-1.152.0
   remains latest, ternary ecosystem unchanged.
2. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W443 triage
   decision.

### 3.6 Close-out and next-wave hand-off

1. Run the full verification matrix.
2. Write `docs/reports/WAVE_LOOP_443_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W443_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_COOPERATION_W444_2026-07-01.md` with three
   variants for Wave Loop 444.
5. Update `docs/NOW.md` and `.trinity/current-issue.md` for W444.
6. Create issue #1418 and branch `wave-loop-444`.
7. Open PR for W443.

---

## 4. Acceptance criteria

- `cargo test -p tri` and `cargo test -p t27c --bin t27c suite::tests` both pass
  with no new regressions.
- `cargo test -p tri` target: 131+/131 active tests.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test --json /tmp/suite_summary.json` emits:
  - `known_failures` = 7 baseline specs,
  - `acceptable: true`,
  - `fpga_smoke_passed: true`.
- `tri fpga pvt-envelope --pvt-context <ctx.json> --json` emits
  `inside_envelope: true/false` and `envelope_check: "ok" / "failed" / "skipped"`.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json ...`
  produces a 24-variant `theorem_matrix` with per-variant `envelope_check: "ok"`
  and `passed: true`.

---

*φ² + φ⁻² = 3 | TRINITY*
