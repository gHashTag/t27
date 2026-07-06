# FPGA Loop Cooperation Plan — Wave Loop 454 (2026-07-01)

**Issue:** #1421 (W453) → **#1424** (W454)
**Branch:** `wave-loop-453` → **`wave-loop-454`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W453 outcome summary

Wave Loop 453 executed **Variant B**: the four-corner PVT operating rectangle
was closed in `proofs/lean4/Trinity/TernaryFPGABoot.lean` with a single
quantified theorem:

- `all_envelope_corners_w453_all_corners_transaction_ok`
- `all_envelope_corners_w453_all_oscfsel_combined_check_true`

The new `EnvelopeCorner` inductive enumerates hot/low-v (W451), hot/high-v
(W453), cold/low-v (W453), and cold/high-v (W452). New boundary operating points
`BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT` (85 °C, 1100 mV) and
`BOUNDARY_COLD_LOWV_W453_OPERATING_POINT` (-40 °C, 900 mV) cover the remaining
rectangle diagonal, each quantified over `ff`/`tt`/`ss` process corners and all
OSCFSEL 0..7.

On the CI side, the FPGA smoke-gate JSON report is now schema-guarded by
`#[serde(deny_unknown_fields)]` on both the generator (`cli/tri/src/fpga.rs`)
and the consumer (`bootstrap/src/suite.rs`). New unit tests verify acceptance of
a canonical report and rejection of unknown fields.

Physical bench execution remains blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** Sparkle/Verilean remains the only fresh Lean-native HDL
signal in early July 2026. No new public Sparkle commits, CIRCT/firtool releases,
Clash promotions, or Lean-native ternary-FPGA projects appeared between the W452
close-out and the W453 boundary.

---

## Constraint landscape for W454

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master` (`701d79b3b`), not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Dry-run-live fixture set | committed as a regression anchor (W448) with a quantified theorem (W450) |
| Four-corner rectangle theorem | committed in `TernaryFPGABoot.lean` (W453) |
| VCCAUX independence | formally captured in `TernaryFPGABoot.lean` (W451) |
| Adversarial envelope witnesses | temperature (W448), low VCCINT (W452) |
| Smoke-gate report schema | `deny_unknown_fields` guard on generator + consumer (W453) |
| All-ok snapshot | committed and regression-tested (W452) |
| Full Trinity `lake build` | still broken on unrelated physics proofs; boot target `Trinity.TernaryFPGABoot` still builds |
| Time pressure | Medium; the master-merge gen-verilog fix set is the next largest safe lever |

---

## Three cooperation variants for Wave Loop 454

### Variant A — Live-capture four-corner rectangle and fixture archive (preferred if bench unblocks)

**Goal:** obtain real boot-captured CCLK sweeps with live XADC readouts, persist
the fixtures under `tests/fixtures/fpga/theorem-matrix/live-w454/`, add a CI
regression test that replays the live fixtures, and mint an
`XADC_LIVE_W454_OPERATING_POINT` theorem block plus a quantified transaction
theorem over all documented process corners at the captured operating point.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w454_pvt.json --json out/w454_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w454/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w454 --json
   out/w454_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w454_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w454_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W454_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and prove
   `xadc_live_w454_all_corners_transaction_ok` using the existing envelope bridge.
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

### Variant B — Master-merge the safe gen-verilog fix set from `master` (default)

**Goal:** close the 7 residual yosys smoke failures by merging the safe
`gen-verilog` fix set already present on `master` (`701d79b3b`) into the
wave-loop branch, while preserving the W453 boot-evidence theorems and
snapshot diff gates.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b` and identify the subset of fixes that are safe to merge without
   perturbing the IGLA seal set or the boot-evidence pipeline.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-454`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline in
   `docs/reports/gen_verilog_smoke_baseline.json` if the set of expected
   failures changes.
6. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
7. Mint W454 evidence and cooperation files.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.
- FPGA smoke-gate JSON report schema tests still pass.
- The four-corner rectangle theorem still builds.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Work in small reviewable chunks and re-seal after each
compiler change.

---

### Variant C — Adversarial envelope / duty-cycle / jitter theorems (fallback)

**Goal:** if neither the bench unblocks nor the master-merge is safe enough to
complete in one wave, extend the formal boot-evidence lattice with adversarial
or robustness theorems that do not require hardware or compiler changes.

Candidate theorems:
- Duty-cycle asymmetry: prove that a non-50/50 low/high split still satisfies the
  flash spec as long as each half remains above the minimum low/high times.
- Jitter bound: given a bounded timing perturbation around the ideal raw-ns
  values, the transaction still meets the spec.
- VCCINT adversarial high witness: an operating point above 1100 mV is rejected
  by `xadc_operating_point_within_envelope`.
- OSCFSEL robustness: a theorem tying the `oscfsel > 7` rejection to the
  concrete `OSCFSEL_BITS` width.

**Work items:**
1. Pick one or two robustness axes that add new falsifiable claims without
   duplicating the existing rectangle theorem.
2. Add the operating points, helper lemmas, and theorems to
   `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
3. Add unit tests or snapshot coverage in `cli/tri/src/fpga.rs` if the theorem
   has a computable gate counterpart.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
5. Mint evidence and cooperation files for W455.

**Acceptance criteria:**
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` still reports the documented 7 baseline gen-verilog
  failures and `acceptable: true`.
- New adversarial/robustness theorem builds and is covered by at least one
  computable gate or unit test.

**Risk:** low. No hardware dependency and no compiler merge.

---

## Recommended W454 order

1. **Default to Variant B** — the `master` fix set is the largest remaining safe
   lever and directly attacks the 7 residual yosys smoke failures. Close them
   now so the FPGA loop stops carrying documented baseline failures.
2. **If the bench unblocks during Variant B, execute Variant A** — real capture
   remains the highest-leverage outcome and the pipeline is ready to consume it.
3. **If Variant B is blocked by merge risk, execute Variant C** — adversarial
   robustness theorems keep the formal lattice moving without hardware or
   compiler changes.

---

## W454 issue/branch action

- GitHub issue for Wave Loop 454: **#1424**.
- Create branch **`wave-loop-454`** from the W453 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W454.

---

*φ² + φ⁻² = 3 | TRINITY*
