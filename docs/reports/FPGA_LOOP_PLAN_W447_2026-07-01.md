# FPGA Loop Plan — Wave Loop 447 (2026-07-01)

**Issue:** #1422 (W447)  
**Branch:** `wave-loop-447`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

Wave Loop 446 closed the golden theorem-matrix fixture set behind a
report-shape diff gate and added separate suite-level generation/replay timing
metrics. The bench remains blocked, the `gen-verilog` fix set on `master` is still
not merged, and the full Trinity `lake build` is still broken on unrelated
physics proofs.

| Weak point | Why it matters | Risk if ignored |
|---|---|---|
| **No live fixture archive yet** | All replayed fixtures are synthetic/golden. When the bench unblocks, the live-capture procedure is not yet exercised end-to-end in CI. | First real capture requires manual debugging instead of running a known-good path. |
| **No quantified theorem over the 24-variant golden matrix** | The golden fixtures prove each variant individually, but there is no single Lean theorem stating the combined-check gate holds for every OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner under the committed operating point. | The formal lattice is one abstraction short of a single machine-checkable statement about the whole matrix. |
| **Standalone `measured-to-lean` is not actually built** | The test only inspects the generated `.lean` file; it does not compile it in isolation because the full `lake build` fails on physics proofs. | A generated theorem could drift out of sync with `Trinity.TernaryFPGABoot` without CI catching it. |
| **Competitor refresh is static** | Sparkle/Verilean and CIRCT signals need continuous re-verification at each wave boundary. | Stale competitive intelligence weakens close-out reports. |
| **Hardware blockers unchanged** | DLC10 cable, P12 header, relay gate all still blocked. Variant A remains unavailable. | W447 must not depend on the bench. |
| **`gen-verilog` master-merge still too risky to mix** | 7 residual yosys smoke failures remain the baseline; the full fix set lives on `master`. | A side-task compiler merge would destabilize the boot-evidence branch. |

---

## 2. Competitor scan (W447 boundary)

**Sparkle / Verilean** remains the closest Lean-native threat.

- PR #97–#100 and PR #96 merged on 2026-07-04 (FIDO2/crypto burst).
- PR #101 “docs(tutorial): Ch11 web3 signer — flash, sign, and broadcast to local
  anvil (+ M2)” is still open.
- PR #65 “Prove that Divider divides” is open (RV32 divider formal proof).
- No new public Sparkle signals after 2026-07-11; the 関数型まつり2026 talk remains
  the next checkpoint.

**CIRCT / firtool:** `firtool-1.152.0` (2026-07-04) is still the latest public
release. No `1.153.0` has shipped as of the W447 boundary.

**Clash:** `clash-ghc-1.11.0` remains a Hackage candidate; latest published is
still `1.10.0` (April 2026).

**Ternary-FPGA niche:** TernaryCore and BitNet-RISCV-Multicore remain the closest
non-Lean signals. No Lean-native ternary-FPGA competitor appeared this wave.

t27's differentiation — sealed spec → generated code → seal hash → physical
CCLK/PVT boot-evidence loop — remains unmatched.

---

## 3. Decomposed plan — Variant B (default)

Selected variant: **Variant B — Live-capture fallback + formal combined-check
over the golden matrix + competitor refresh**. It does not depend on the bench,
expands formal coverage, and documents the live-capture procedure.

### 3.1 Dry-run live-capture fixture path

- Add `--dry-run-live` to `tri fpga smoke-gate --theorem-matrix`.
- The mode emits fixtures under `build/fpga/theorem-matrix-dry-run-live/` with the
  same directory structure a real board capture would produce:
  - `pvt.json` per corner (same shape as `synthetic_pvt_context`).
  - `raw_ns_<corner>_<oscfsel>.json` with synthetic periods matching the golden
    matrix.
  - `.lean` theorem per variant.
  - `summary_<corner>_<oscfsel>.json` per variant.
- Mark every fixture with `source: "dry_run_live"` so replay is distinguishable
  from `synthetic` or `xadc`.

### 3.2 Golden + dry-run-live replay regression test

- Add `test_theorem_matrix_dry_run_live_replay_matches_golden_shape`.
- Generate dry-run-live fixtures to a temp directory.
- Replay both `tests/fixtures/fpga/theorem-matrix/golden/` and the dry-run-live
  directory.
- Assert both produce 24 variants, all `envelope_check: "ok"`, matching report
  shape, and that dry-run-live variants carry `source: "dry_run_live"`.

### 3.3 Quantified combined-check theorem over the golden matrix

- Define a `GOLDEN_W447_OPERATING_POINT : XadcOperatingPoint` in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` matching the synthetic PVT context
  (`temp_c = 42`, `vccint_mv = 1000`, `vccaux_mv = 1800`, `process_corner = ss`).
- Prove `golden_w447_operating_point_within_envelope`.
- Mint `golden_w447_all_oscfsel_combined_check_true`:
  for every `oscfsel ≤ 7`, `cclk_variant_and_xadc_envelope_check oscfsel
  GOLDEN_W447_OPERATING_POINT = true`.
- Mint corner-specific theorems for `ff`/`tt`/`ss` that record the golden matrix
  PVT context and justify each OSCFSEL variant.

### 3.4 Standalone `measured-to-lean` build gate

- Extend `measured_to_lean` with `--standalone` so the generated file can be
  dropped into a temporary lake package that depends only on
  `Trinity.TernaryFPGABoot`.
- Add a test that creates a temp `lakefile.lean` pointing at the in-repo
  `proofs/lean4`, copies the generated `.lean` file into the package, and runs
  `lake build` against it.
- Keep the existing lightweight content check as a fast path.

### 3.5 Competitor refresh and close-out

- Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the W447 boundary section.
- Document the W447 triage decision in
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
- Mint W447 evidence and cooperation files for W448.
- Update `docs/NOW.md` and `.trinity/current-issue.md` for W447 close-out /
  W448 setup.

---

## 4. Acceptance criteria

- `cargo check -p tri` passes.
- `cargo test -p tri` passes with **138+/138 active** tests and **0 ignored**.
- `cargo test -p t27c --bin t27c suite::tests` passes.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` passes with the documented baseline of 7 pre-existing
  `gen-verilog` yosys smoke failures and **FPGA smoke fails: 0**.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and both elapsed-ms fields populated.
- Golden fixture replay report matches the committed snapshot.
- New combined-check theorem builds in `Trinity.TernaryFPGABoot`.
- Standalone `measured-to-lean` output builds in a temporary lake package.

---

## 5. Issue/branch action

- Use issue **#1422** for W447.
- Keep branch **`wave-loop-447`**.
- Wave Loop 448 will use issue **#1423** and branch **`wave-loop-448`**.

---

*φ² + φ⁻² = 3 | TRINITY*
