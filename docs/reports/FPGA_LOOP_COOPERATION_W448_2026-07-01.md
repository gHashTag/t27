# FPGA Loop Cooperation Plan — Wave Loop 448 (2026-07-01)

**Issue:** #1422 (W447) → **#1423** (W448)
**Branch:** `wave-loop-447` → **`wave-loop-448`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W447 outcome summary

Wave Loop 447 executed **Variant B**: the FPGA boot-evidence pipeline is now
ready to consume a real board capture even though the bench remains blocked.
A synthetic dry-run-live path (`tri fpga smoke-gate --theorem-matrix
--dry-run-live`) emits the same fixture directory structure a real capture would
produce, and a new Rust unit test replays both the committed golden fixtures and
the synthetic dry-run-live fixtures, asserting identical 24-variant report shape.

A quantified Lean theorem, `golden_w447_all_oscfsel_combined_check_true`, now
states that the dashboard gate evaluates to `true` for every OSCFSEL 0..7 under
the committed golden operating point. The standalone `measured-to-lean` path was
fixed to build only the boot target in a temporary lake package.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
gen-verilog yosys smoke failures, and `./scripts/tri test --json` reports
`acceptable: true` with both `fpga_smoke_gate_elapsed_ms` and
`fpga_smoke_gate_replay_elapsed_ms` populated.

Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** Sparkle/Verilean landed a July 4 2026 FIDO2/crypto burst
(PR #97–#100) and has PR #101 open. CIRCT `firtool-1.152.0` shipped July 4 2026
and is still the latest public release. No new public Sparkle signals have
appeared after 2026-07-11. t27's sealed spec → generated code → seal hash →
physical CCLK/PVT boot-evidence loop is still unmatched.

---

## Constraint landscape for W448

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Dry-run-live fixture set | generated on demand, not yet committed as a second regression anchor |
| Suite timing metrics | both generation and replay elapsed ms are now in the summary |
| Standalone `measured-to-lean` | now builds in isolation; not yet wired into the smoke gate |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `Trinity/H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor; W447 added a golden-point quantified theorem |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal expansion |

---

## Three cooperation variants for Wave Loop 448

### Variant A — Physical cold-POR capture with live fixture archive (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w448/`, add a CI regression test that
replays the live fixtures, and mint a `XADC_LIVE_W448_OPERATING_POINT` theorem
block plus a quantified combined-check theorem over all documented process
corners.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w448_pvt.json --json out/w448_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w448/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w448 --json
   out/w448_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w448_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w448_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W448_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7 and `ff`/`tt`/`ss` corners.
8. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new competitor signals
   appear.
9. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 140+/140 (no new regressions).
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean` and `pvt-envelope` reports `inside_envelope: true`.
- Live fixture replay test passes and reports `elapsed_ms`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Commit dry-run-live fixtures as a second regression anchor + wire standalone build into the smoke gate + formal adversarial envelope test (default)

**Goal:** while the bench is blocked, harden the FPGA boot-evidence loop by (1)
committing the synthetic dry-run-live fixture set as a second regression anchor,
(2) extending the smoke gate to build a standalone `measured-to-lean` theorem for
at least one golden variant, and (3) adding a Lean theorem that proves the
dashboard gate returns `false` for operating points outside the PVT envelope.

**Work items:**
1. Generate a deterministic dry-run-live fixture set and commit it under
   `tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/`.
2. Add a snapshot diff test that replays the dry-run-live fixtures and diffs
   the report shape against a committed `expected_report.json`.
3. Add `SmokeGateOpts.dry_run_live` source handling so replay can select the
   committed dry-run-live set in CI without regeneration.
4. Extend `tri fpga smoke-gate --theorem-matrix` with an optional
   `--validate-lean-standalone` flag that calls `measured-to-lean --standalone`
   for one golden variant and asserts `lake build` succeeds inside a temp
   lake package.
5. In `proofs/lean4/Trinity/TernaryFPGABoot.lean`, add:
   - a quantified theorem over the dry-run-live operating point,
   - an `outside_envelope_operating_point` witness,
   - a theorem `cclk_variant_and_xadc_envelope_check_outside_envelope_false`
     stating the dashboard gate returns `false` when the operating point is
     outside the envelope.
6. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W448 boundary.
7. Mint W448 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 142+/142 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and both elapsed-ms fields populated.
- Dry-run-live replay report matches the new committed snapshot.
- Standalone `measured-to-lean` builds inside the smoke gate for at least one
  golden variant.
- New adversarial envelope theorem builds in `Trinity.TernaryFPGABoot`.

**Risk:** low. No hardware dependency; expands formal coverage and hardens the
standalone Lean artifact path.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-448`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W449.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W448 order

1. **Default to Variant B** — it does not depend on the bench, turns the
   dry-run-live path into a committed regression anchor, and wires the standalone
   `measured-to-lean` build into the smoke gate so that Variant A's real capture
   will be automatically validated when hardware unblocks.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail, the snapshot diff gates, or the IGLA seal set.

---

## W448 issue/branch action

- Create GitHub issue for Wave Loop 448 titled **“Wave Loop 448 — dry-run-live
  fixture anchor + standalone Lean smoke gate + adversarial envelope theorem
  (Variant B, A optional)”**.
- Create branch **`wave-loop-448`** from the W447 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W448.

---

*φ² + φ⁻² = 3 | TRINITY*
