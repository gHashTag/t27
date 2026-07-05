# FPGA Loop Cooperation Plan — Wave Loop 450 (2026-07-01)

**Issue:** #1424 (W449) → **#1425** (W450)
**Branch:** `wave-loop-449` → **`wave-loop-450`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W449 outcome summary

Wave Loop 449 executed **Variant B**: the formal boot-evidence lattice was
expanded with a single quantified end-to-end transaction theorem,
`golden_w449_all_corners_transaction_ok`, which states that the ideal raw-ns
capture produces a flash-spec-compliant SPI read transaction for every OSCFSEL
0..7 and every process corner (`ff`/`tt`/`ss`) under the W447/W448 golden
operating point. The proof reuses the W431 XADC-envelope bridge and the W442
worst-case raw-ns theorem, so it adds no new ad-hoc computation.

The suite-level CI dashboard now tracks the standalone `lake build` cost via
`validate_lean_standalone_elapsed_ms`, parsed from the smoke-gate JSON report and
emitted by `./scripts/tri test --json`. A schema regression test and a new Rust
unit test protect the field and the standalone smoke-gate phase end-to-end.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
gen-verilog yosys smoke failures, and `./scripts/tri test --json` reports
`acceptable: true` with all elapsed-ms fields populated, including the new
standalone metric.

Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** No new public competitor signals appeared between the W448
close-out and the W449 boundary. Sparkle/Verilean's repository last pushed
2026-07-03; PR #66 remains open, the FIDO2/crypto burst (PR #97–#100) remains
merged 2026-07-04, and the README still cites 102 formal theorems. CIRCT
`firtool-1.152.0` (2026-07-04) is still the latest public release. t27's sealed
spec → generated code → seal hash → physical CCLK/PVT boot-evidence loop is
still unmatched.

---

## Constraint landscape for W450

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Dry-run-live fixture set | committed as a second regression anchor (W448) |
| Suite timing metrics | generation, replay, and standalone elapsed ms are now in the summary |
| Standalone `measured-to-lean` | wired into the smoke gate and producing a report block; cost is now visible |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `Trinity/H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W449 added a golden-point quantified transaction theorem over all corners |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal expansion |

---

## Three cooperation variants for Wave Loop 450

### Variant A — Live-capture transaction theorem and fixture archive (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w450/`, add a CI regression test that
replays the live fixtures, and mint a `XADC_LIVE_W450_OPERATING_POINT` theorem
block plus a quantified transaction theorem over all documented process corners at
the captured operating point.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w450_pvt.json --json out/w450_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w450/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w450 --json
   out/w450_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w450_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w450_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W450_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and prove
   `xadc_live_w450_all_corners_transaction_ok` using the existing envelope bridge.
8. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new competitor signals
   appear.
9. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` passes with no new regressions.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean` and `pvt-envelope` reports `inside_envelope: true`.
- Live fixture replay test passes and reports `elapsed_ms`.
- New live-capture transaction theorem builds in `Trinity.TernaryFPGABoot`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Formal boot-evidence expansion + standalone-build snapshot + CI hardening (default)

**Goal:** while the bench is blocked, continue expanding the formal boot-evidence
lattice and harden the standalone-build path. Add a quantified transaction theorem
over the W448 dry-run-live operating point, commit an expected snapshot for the
standalone smoke-gate report block, and optionally reduce the standalone build
overhead by caching the temporary lake package or moving it behind a dedicated
CI job.

**Work items:**
1. Define `DRY_RUN_LIVE_W448_OPERATING_POINT` (or reuse the W448 synthetic
   fixture PVT context) in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
2. Prove `dry_run_live_w448_all_corners_transaction_ok`: for every OSCFSEL 0..7
   and every process corner, the dry-run-live raw-ns capture produces a
   flash-spec-compliant transaction.
3. Add a snapshot test that records the expected shape of the smoke-gate
   `validate_lean_standalone` report block (status, source, elapsed_ms present,
   lean_file path pattern) and fails if the schema changes.
4. Optionally split the standalone build into its own suite phase so it can be
   skipped independently when `lake` is unavailable, and add a `--fast` suite mode
   that suppresses the standalone build.
5. Harden the `FpgaSmokeResult` / `SuiteSummary` schema so future phases cannot
   silently drop metrics: add a builder-style helper or a non-default field guard.
6. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W450 boundary.
7. Mint W450 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` passes with no new regressions.
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and `validate_lean_standalone_elapsed_ms` populated.
- New dry-run-live transaction theorem builds in `Trinity.TernaryFPGABoot`.
- Standalone smoke-gate snapshot test passes.

**Risk:** low. No hardware dependency; expands formal coverage and hardens the
standalone metric path.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-450`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W451.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W450 order

1. **Default to Variant B** — it does not depend on the bench, continues the
   formal boot-evidence expansion, and protects the standalone-build report
   schema with a snapshot test so the metric is trustworthy across waves.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail, the snapshot diff gates, or the IGLA seal set.

---

## W450 issue/branch action

- Create GitHub issue for Wave Loop 450 titled **“Wave Loop 450 — Formal
  boot-evidence expansion + standalone-build snapshot + CI hardening (Variant B,
  A optional)”**.
- Create branch **`wave-loop-450`** from the W449 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W450.

---

*φ² + φ⁻² = 3 | TRINITY*
