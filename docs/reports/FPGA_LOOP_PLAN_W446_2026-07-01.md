# FPGA Loop Plan — Wave Loop 446 (2026-07-01)

**Issue:** #1420 (W446)  
**Branch:** `wave-loop-446`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

Wave Loop 445 left the FPGA boot-evidence line in a **software-only, deterministic
state**: the W444 synthetic theorem-matrix fixtures are checked in as a golden
regression set and the suite summary records the theorem-matrix generation
time. The physical bench is still unreachable, and the `gen-verilog` fix set on
`master` is still not merged.

| Weak point | Why it matters | Risk if ignored |
|---|---|---|
| **Golden fixtures can drift silently** | 75 fixture files are now part of the repo; a future change to `synthetic_pvt_context`, `cclk_period_ns`, `measured_to_lean`, or `verify_lean` can change the replayed report without changing the fixtures themselves. | CI stops catching regressions in the boot-evidence artifact trail. |
| **No committed report snapshot** | The `test_theorem_matrix_golden_replay_passes` only checks per-variant fields. It does not assert that the overall report shape, variant count, and timing metadata remain stable. | Accidental report-schema changes slip through. |
| **Generation vs. replay cost is not separated** | `fpga_smoke_gate_elapsed_ms` mixes fixture generation + verify-lean time. A fast replay path exists but is not visible in the suite summary. | We cannot trend replay cost independently, which matters when live fixtures replace synthetic ones. |
| **Competitor signals need continuous refresh** | Sparkle/Verilean landed a July 4 2026 FIDO2/crypto burst (PR #97–#100). CIRCT `firtool-1.152.0` shipped July 4 2026. The W446 boundary must re-verify these claims and watch for post-2026-07-11 signals. | Stale competitive intelligence weakens the close-out report and the next-wave rationale. |
| **Hardware blockers are unchanged** | DLC10 cable not detected, P12 unwired, no relay gate. Variant A (live capture) remains blocked. | W446 must not depend on the bench. |
| **`gen-verilog` master-merge still too risky to mix** | 7 residual yosys smoke failures are documented baseline; the full fix set lives on `master` (`701d79b3b`) and touches tuple-return / `let` destructuring / ROM arrays / CORDIC. | A side-task compiler merge would destabilize the boot-evidence branch. |
| **Full Trinity `lake build` is still broken** | `NeutrinoMasses.lean` / `H4Lagrangian.lean` fail; only `Trinity.TernaryFPGABoot` builds. | New Lean theorems must be limited to the boot target. |

---

## 2. Competitor scan (W446 boundary)

**Sparkle / Verilean** remains the closest Lean-native threat.

- PR #66 "IP.Net: USB Web server + memcached server + Compiler perf" merged
  2026-06-30.
- A FIDO2/crypto burst landed on 2026-07-04:
  - PR #97 — FIDO2/CTAP2 pure data layer + P-256 sign (M1).
  - PR #98 — P-256 HW sign stack + SHA-256 streaming (FIDO2 M2).
  - PR #99 — FIDO2 CTAPHID + CTAP2 dispatch + UART-bridge top (M3).
  - PR #100 — circuits-only IP refactor with P-256 math-property proofs.
  - PR #96 — policy-enforcing Ethereum signer (Tang Nano 50K).
- PR #101 "docs(tutorial): Ch11 web3 signer — flash, sign, and broadcast to
  local anvil (+ M2)" is open as of the W446 boundary.
- No new public Sparkle signals have appeared **after 2026-07-11**; the
  関数型まつり2026 talk remains the next checkpoint.

**CIRCT / firtool:** `firtool-1.152.0` is still the latest public release
(shipped 2026-07-04). No `1.153.0` exists as of the boundary.

**Ternary-FPGA niche:** TernaryCore, ternfpga, KULeuven ternary-lut-dse, and
BitNet-RISCV-Multicore continue to validate {-1,0,+1} compute hardware, but
none combines it with a Lean-native proof pipeline. t27's differentiation at
this intersection is intact.

---

## 3. Decomposed plan — Variant B (default)

Selected variant: **Variant B — Golden fixture report-shape diff gate + timing
dashboard**. It does not depend on the bench, hardens the deterministic
artifact trail, and keeps the `gen-verilog` debt isolated.

### 3.1 Golden fixture report-shape diff gate

- Refactor `cli/tri/src/fpga.rs` so `replay_theorem_matrix` can return a
  deterministic report block (or add a helper that builds the same JSON object
  the CLI emits).
- Generate `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json` as a
  committed snapshot. Paths inside the snapshot are normalized to be relative to
  the fixture directory so the file is stable across machines.
- Add `test_theorem_matrix_golden_replay_matches_snapshot`:
  - Replays the golden fixtures.
  - Serializes the theorem-matrix report block.
  - Compares actual vs. expected. The test allows the actual report to be a
    strict superset (additional fields are OK) but requires that every
    expected field matches, that all 24 variants are present, that each
    variant has `envelope_check: "ok"`, and that a `fixtures` block is present.
  - Provides an `UPDATE_EXPECTED=1` mode to regenerate the snapshot when the
    fixture set is intentionally updated.

### 3.2 Suite-level replay timing dashboard

- Add `theorem_matrix_replay_elapsed_ms: Option<u64>` to `FpgaSmokeResult`.
- Add `fpga_smoke_gate_replay_elapsed_ms: Option<u64>` to `SuiteSummary`.
- In `bootstrap/src/suite.rs`, run a second FPGA smoke-gate invocation in
  `--replay-fixtures tests/fixtures/fpga/theorem-matrix/golden` mode and parse
  `theorem_matrix.elapsed_ms` into the new summary field.
- Update schema-roundtrip and fake-report tests to exercise the new fields.
- Update `fpga/HARDWARE_SSOT.md` §3.6.26 with the new metric semantics and the
  expected snapshot path.

### 3.3 Competitor refresh

- Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` with a W446 boundary section.
- Correct/confirm the Sparkle July 4 2026 FIDO2/crypto burst and PR #101 status.
- Note that CIRCT `firtool-1.152.0` remains latest and no post-2026-07-11
  Sparkle signals exist yet.

### 3.4 Triage and close-out

- Record the W446 `gen-verilog` triage decision in
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
- Write evidence and cooperation files for W447.
- Update `docs/NOW.md` and `.trinity/current-issue.md` for W446 close-out /
  W447 setup.

---

## 4. Acceptance criteria

- `cargo check -p tri` passes.
- `cargo test -p tri` passes with **138+/138 active** tests and **0 ignored**.
- `cargo test -p t27c --bin t27c suite::tests` passes.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` passes with the documented baseline of 7 pre-existing
  `gen-verilog` yosys smoke failures and **FPGA smoke fails: 0**.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true`, `fpga_smoke_gate_elapsed_ms` populated, and
  `fpga_smoke_gate_replay_elapsed_ms` populated.
- `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json` exists and
  the snapshot diff test passes.

---

## 5. Issue/branch action

- Use issue **#1420** for W446.
- Keep branch **`wave-loop-446`**.
- Wave Loop 447 will use issue **#1422** and branch **`wave-loop-447`**
  (skipping #1421 because it is already a PR number for W443).

---

*φ² + φ⁻² = 3 | TRINITY*
