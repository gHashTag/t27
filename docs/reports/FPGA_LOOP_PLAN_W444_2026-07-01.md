# FPGA Loop Decomposed Implementation Plan — Wave Loop 444 (2026-07-01)

**Issue:** #1418  
**Branch:** `wave-loop-444`  
**Variant:** B (default) — theorem-matrix fixture replay + deterministic CI artifact  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified in W443

| Weak point | Why it matters | Where it lives |
|---|---|---|
| Theorem matrix regenerates theorems every run | CI cannot diff the inputs/outputs between waves, and re-running the bitstream path is slower than replaying fixtures. | `cli/tri/src/fpga.rs` theorem-matrix block inside `smoke_gate` |
| Fixture files are not persisted in a stable location | Consumers of the smoke-gate report cannot inspect the raw PVT context, raw-ns capture, or generated Lean theorem per variant. | `build/fpga/` scratch directories |
| No replay mode exists | A captured or synthetic fixture set cannot be re-run cheaply for regression verification. | `FpgaCmd::SmokeGate` CLI surface |
| Smoke-gate report does not record per-variant fixture paths | Downstream CI cannot validate that a matrix entry was produced from a specific artifact on disk. | `report["theorem_matrix"]["variants"]` JSON shape |
| No unit test protects fixture round-trip | A regression in generation or replay path could silently drop fixtures or change report shape. | `cli/tri/src/fpga.rs` tests |
| Competitor checkpoint | Sparkle's July 4 2026 FIDO2/crypto burst needed to be folded into the W444 boundary snapshot. | `docs/reports/T27_VS_FORMAL_HDL_2026.md` |

---

## 2. Competitor signals used for W444 planning

- **Sparkle / Verilean**
  - PR #66 (IP.Net + compiler perf) merged **2026-06-30**.
  - A burst of FIDO2/crypto work landed on **2026-07-04**: PR #97 FIDO2/CTAP2 data layer,
    PR #98 P-256 hardware sign stack + SHA-256 streaming, PR #99 CTAPHID + CTAP2
    dispatch top, PR #100 crypto refactor with P-256 math-property proofs.
  - This confirms Sparkle is building a broad, formally verified IP catalog inside
    Lean 4 HDL — the closest strategic threat to t27's Lean-native positioning.
- **CIRCT / firtool**
  - `firtool-1.152.0` (2026-07-04) remains the latest public release; no
    `1.153.0` has appeared.
- **Ternary FPGA ecosystem**
  - **TernaryCore** (`shepherdscientific/ternarycore`): last push 2026-05-27;
    RTL simulation 31/31 passing; no formal proofs.
  - **ternfpga** (`Neumann-Labs/ternfpga`): created June 2026; sub-watt BitNet
    inference on Arty A7-35T; no Lean-native proof pipeline.
  - Neither project combines ternary compute with a Lean-native proof pipeline,
    so t27's differentiation remains intact.

Strategic takeaway for W444: continue hardening the **machine-readable CI gate +
board-less formal coverage matrix + deterministic fixture replay** because no
competitor matches that combination, and keep monitoring Sparkle for a
spec-first sealed pipeline or physical boot-evidence path.

Sources:
- [Verilean/sparkle](https://github.com/Verilean/sparkle)
- [Sparkle PR #66 — IP.Net + compiler perf](https://github.com/Verilean/sparkle/pull/66)
- [Sparkle PR #97 — FIDO2/CTAP2 data layer](https://github.com/Verilean/sparkle/pull/97)
- [Sparkle PR #98 — P-256 hardware sign stack](https://github.com/Verilean/sparkle/pull/98)
- [Sparkle PR #99 — CTAPHID + CTAP2 dispatch](https://github.com/Verilean/sparkle/pull/99)
- [Sparkle PR #100 — crypto refactor](https://github.com/Verilean/sparkle/pull/100)
- [CIRCT releases](https://github.com/llvm/circt/releases)
- [TernaryCore](https://github.com/shepherdscientific/ternarycore)
- [ternfpga](https://github.com/Neumann-Labs/ternfpga)

---

## 3. Decomposed implementation tasks

### 3.1 Persist theorem-matrix fixtures to a stable directory

1. Extract the existing inline theorem-matrix loop in `cli/tri/src/fpga.rs` into
   a dedicated `generate_theorem_matrix(fixture_dir, _report)` function.
2. Write per-corner `theorem_matrix_pvt_{corner}.json` and per-variant
   `theorem_matrix_raw_ns_{corner}_{oscfsel}.json`,
   `theorem_matrix_{corner}_oscfsel_{oscfsel}.lean`, and
   `theorem_matrix_summary_{corner}_{oscfsel}.json` under the supplied fixture
   directory.
3. Default fixture directory: `build/fpga/theorem-matrix-fixtures/`.

### 3.2 Add fixture replay mode

1. Add `--replay-fixtures <dir>` to `FpgaCmd::SmokeGate`.
2. Pass the option through `smoke_gate(...)` as `Option<&PathBuf>`.
3. Implement `replay_theorem_matrix(fixture_dir)` that:
   - reads the four fixture files per variant,
   - calls `verify_lean` on the persisted Lean + summary,
   - re-checks the PVT envelope,
   - returns report entries identical in shape to the generation path plus an
     elapsed-ms metric.
4. When `--replay-fixtures` is supplied, emit `theorem_matrix.replay: true` and
   `theorem_matrix.elapsed_ms`; otherwise emit `replay: false` and generation
   time.

### 3.3 Extend per-variant report with `fixtures` block

1. Add a `fixtures` object to every theorem-matrix variant entry:
   - `pvt`
   - `raw_ns`
   - `lean`
   - `summary`
2. Keep `schema_version: "1.0"` unchanged; the new field is an additive
   extension.
3. Update the schema-tolerant tests in `bootstrap/src/suite.rs` to exercise the
   new shape.

### 3.4 Add Rust unit tests

1. `test_theorem_matrix_fixture_roundtrip` — generate fixtures into a temp
   directory, replay them, and assert that the replayed entries match the
   generated entries in shape and envelope verdict.
2. `test_theorem_matrix_replay_does_not_regenerate` — assert that replay mode
   does not call `measured_to_lean` and instead verifies the persisted fixtures.
3. Update `test_smoke_gate_json_synthetic_verify_lean` for the new
   `smoke_gate(...)` signature.

### 3.5 Documentation refresh

1. Document fixture file patterns and the `--replay-fixtures` workflow in
   `fpga/HARDWARE_SSOT.md` §3.6.26.
2. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the W444 boundary note
   (Sparkle July 4 2026 activity).
3. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W444 triage
   decision.

### 3.6 Close-out and next-wave hand-off

1. Run the full verification matrix.
2. Write `docs/reports/WAVE_LOOP_444_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W444_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_COOPERATION_W445_2026-07-01.md` with three
   variants for Wave Loop 445.
5. Update `docs/NOW.md` and `.trinity/current-issue.md` for W445.
6. Create issue #1419 and branch `wave-loop-445` only after #1418 exists and W444
   is closed.

---

## 4. Acceptance criteria

- `cargo test -p tri` and `cargo test -p t27c --bin t27c suite::tests` both pass
  with no new regressions.
- `cargo test -p tri` target: 136+/136 active tests.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test --json suite-summary.json` emits:
  - `known_failures` = 7 baseline specs,
  - `acceptable: true`,
  - `fpga_smoke_passed: true`.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix
  --json ...` produces a 24-variant `theorem_matrix` with per-variant
  `envelope_check: "ok"` and `fixtures` present.
- Fixture replay path reproduces the same report shape and passes; record
  `elapsed_ms` as a metric rather than gating unit tests on a fixed time bound.

---

*φ² + φ⁻² = 3 | TRINITY*
