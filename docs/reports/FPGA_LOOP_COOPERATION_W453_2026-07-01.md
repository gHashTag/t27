# FPGA Loop Cooperation Plan — Wave Loop 453 (2026-07-01)

**Issue:** #1422 (W452) → **#1421** (W453)
**Branch:** `wave-loop-452` → **`wave-loop-453`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W452 outcome summary

Wave Loop 452 executed **Variant B**: the formal boot-evidence lattice was
expanded with a quantified end-to-end transaction theorem at the symmetric
boundary operating point (`boundary_cold_highv_w452_all_corners_transaction_ok`).
The theorem states that the ideal raw-ns capture produces a flash-spec-compliant
SPI read transaction for every OSCFSEL 0..7 and every process corner
(`ff`/`tt`/`ss`) at -40 °C and 1100 mV, the coldest/highest-voltage corner
inside the documented operating envelope.

W452 also added the first adversarial VCCINT witness:
- `OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT` (800 mV, below the 900 mV minimum)
- `outside_vccint_low_w452_operating_point_not_within_envelope`
- `cclk_variant_and_xadc_envelope_check_outside_vccint_low_false`

and an OSCFSEL range-gate theorem:
- `oscfsel_out_of_range_combined_check_false`

On the CI side, `bootstrap/src/suite.rs` now distinguishes passed, skipped, and
failed smoke-gate states in the machine-readable `SuiteSummary`:
- `fpga_smoke_skipped: Option<bool>`
- `fpga_smoke_failed: Option<bool>`
- `fpga_smoke_failure_reason: Option<String>`

The all-ok smoke-gate report shape is now snapshot-protected in
`tests/fixtures/fpga/smoke-gate/all_ok_snapshot.json` via
`test_smoke_gate_all_ok_matches_snapshot` in `cli/tri/src/fpga.rs`.

Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** Sparkle/Verilean remains the only fresh Lean-native HDL
signal in early July 2026. The FIDO2/CTAPHID + P-256 proof burst (PR #97–#100,
merged 2026-07-04) is still the most recent public evidence of a broad,
formally verified IP catalog inside Lean 4. No new public Sparkle commits,
CIRCT/firtool releases, Clash promotions, or Lean-native ternary-FPGA projects
appeared between the W451 close-out and the W452 boundary.

---

## Constraint landscape for W453

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Dry-run-live fixture set | committed as a second regression anchor (W448) with a quantified theorem (W450) |
| Boundary-corner theorems | committed at hot/low-v (W451) and cold/high-v (W452) |
| VCCAUX independence | formally captured in `TernaryFPGABoot.lean` (W451) |
| Adversarial envelope witnesses | temperature (W448) and low VCCINT (W452) |
| Suite smoke-gate state | passed/skipped/failed + failure reason now explicit |
| All-ok snapshot | committed and regression-tested |
| Full Trinity `lake build` | still broken on unrelated physics proofs; boot target `Trinity.TernaryFPGABoot` still builds |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal expansion |

---

## Three cooperation variants for Wave Loop 453

### Variant A — Live-capture rectangle theorem and fixture archive (preferred if bench unblocks)

**Goal:** obtain real boot-captured sweeps for OSCFSEL 0..7 with live XADC
readouts at multiple operating points, persist the fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w453/`, add a CI regression test that
replays the live fixtures, and mint an `XADC_LIVE_W453_OPERATING_POINT` theorem
block plus a quantified transaction theorem over all documented process corners
at the captured operating point.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w453_pvt.json --json out/w453_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w453/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w453 --json
   out/w453_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w453_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w453_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W453_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and prove
   `xadc_live_w453_all_corners_transaction_ok` using the existing envelope bridge.
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

### Variant B — Envelope rectangle closure + smoke-gate report schema hardening (default)

**Goal:** while the bench is blocked, close the four-corner operating-rectangle
in `TernaryFPGABoot.lean` (hot/low-v W451, cold/high-v W452, and the two
remaining hot/high-v and cold/low-v corners) in a single quantified theorem,
and harden the smoke-gate JSON report itself with `deny_unknown_fields`-style
schema checks and an explicit all-ok regression anchor.

**Work items:**
1. Define the remaining two envelope corners in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean`:
   - `BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT` (+85 °C, 1100 mV)
   - `BOUNDARY_COLD_LOWV_W453_OPERATING_POINT` (-40 °C, 900 mV)
2. Prove `all_envelope_corners_w453_all_corners_transaction_ok`: a single `∀`
   theorem covering all four envelope corners, all OSCFSEL 0..7, and all
   process corners.
3. Add a schema-version / field-presence regression test for the smoke-gate
   JSON report (e.g., a Rust unit test that asserts the report object carries
   all required top-level keys and `schema_version: "1.0"`).
4. Add a `deny_unknown_fields` deserialization check for the smoke-gate report
   shape, or a snapshot test that rejects unexpected fields.
5. Extend `SuiteSummary` with a `fpga_smoke_report_schema_ok: Option<bool>` field
   if useful for the dashboard.
6. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W453 boundary.
7. Mint W453 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` passes with no new regressions.
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --fast --json suite-summary.json` produces a parseable
  summary with `acceptable: true` and explicit skipped/failed state for the
  smoke gate.
- New four-corner envelope transaction theorem builds in `Trinity.TernaryFPGABoot`.
- Smoke-gate report schema regression test passes.

**Risk:** low. No hardware dependency; expands formal coverage and hardens the
CI metric path.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-453`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W454.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W453 order

1. **Default to Variant B** — it does not depend on the bench, completes the
   four-corner operating-rectangle in proof, and hardens the smoke-gate report
   schema so the snapshot diff gates and machine-readable dashboard remain
   trustworthy across waves.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail, the snapshot diff gates, or the IGLA seal set.

---

## W453 issue/branch action

- Create GitHub issue for Wave Loop 453 titled **“Wave Loop 453 — Envelope
  rectangle closure + smoke-gate report schema hardening (Variant B, A optional)”**.
- Create branch **`wave-loop-453`** from the W452 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W453.

---

*φ² + φ⁻² = 3 | TRINITY*
