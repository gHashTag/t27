# FPGA Loop Cooperation Plan — Wave Loop 449 (2026-07-01)

**Issue:** #1423 (W448) → **#1424** (W449)
**Branch:** `wave-loop-448` → **`wave-loop-449`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W448 outcome summary

Wave Loop 448 executed **Variant B**: the synthetic dry-run-live theorem-matrix
fixtures are now committed as a second regression anchor under
`tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/` and protected by a
snapshot diff test. The smoke gate now supports `--validate-lean-standalone`,
which builds a standalone generated theorem in a temporary lake package. The
formal envelope story is now two-sided: `TernaryFPGABoot.lean` contains both
positive in-envelope theorems and the adversarial
`cclk_variant_and_xadc_envelope_check_outside_envelope_false` theorem.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
gen-verilog yosys smoke failures, and `./scripts/tri test --json` reports
`acceptable: true` with both `fpga_smoke_gate_elapsed_ms` and
`fpga_smoke_gate_replay_elapsed_ms` populated.

Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** Sparkle/Verilean repo last pushed 2026-07-03; PR #66
(IP.Net + compiler perf) remains open with ~27K additions; the RV32 divider
correctness proof landed 2026-06-25; README now cites **102 formal theorems**.
CIRCT `firtool-1.152.0` (2026-07-04) is still the latest public release. No new
Lean-native ternary-FPGA competitor appeared. t27's sealed spec → generated code
→ seal hash → physical CCLK/PVT boot-evidence loop is still unmatched.

---

## Constraint landscape for W449

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Dry-run-live fixture set | now committed as a second regression anchor |
| Suite timing metrics | both generation and replay elapsed ms are in the summary |
| Standalone `measured-to-lean` | now builds inside the smoke gate |
| Adversarial envelope theorem | minted in `TernaryFPGABoot.lean` |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `Trinity/H4Lagrangian.lean`; boot target still builds |
| Lean theorem lattice | W448 added outside-envelope adversarial theorem; W434 live point still latest silicon anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal expansion |

---

## Three cooperation variants for Wave Loop 449

### Variant A — Physical cold-POR capture with live fixture archive (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w449/`, add a CI regression test that
replays the live fixtures, and mint a `XADC_LIVE_W449_OPERATING_POINT` theorem
block plus a quantified combined-check theorem over all documented process
corners.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w449_pvt.json --json out/w449_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w449/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w449 --json
   out/w449_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w449_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w449_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W449_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7 and `ff`/`tt`/`ss` corners.
8. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new competitor signals
   appear.
9. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 143+/143 (no new regressions).
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean` and `pvt-envelope` reports `inside_envelope: true`.
- Live fixture replay test passes and reports `elapsed_ms`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Formal boot-evidence lattice expansion + CI hardening (default)

**Goal:** while the bench is blocked, expand the formal boot-evidence lattice by
adding a `∀`-quantified transaction-theorem over all 24 matrix variants under
the golden operating point, harden the smoke-gate report schema with a
`validate_lean_standalone_elapsed_ms` suite-level metric, and add a regression
test that exercises `--validate-lean-standalone` in the Rust unit-test suite.

**Work items:**
1. Add a quantified theorem in `TernaryFPGABoot.lean` stating that every
   OSCFSEL 0..7 under every `ff`/`tt`/`ss` corner satisfies the PVT-aware flash
   transaction spec when the operating point is the W448 golden point.
2. Add `validate_lean_standalone_elapsed_ms` to `SuiteSummary` and populate it
   from the smoke-gate report in `bootstrap/src/suite.rs`.
3. Add a schema regression test for the new suite-summary field.
4. Add a Rust unit test that runs `smoke_gate` with
   `--validate-lean-standalone` directly and asserts the report block.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W449 boundary.
6. Mint W449 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 143+/143 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and the new `validate_lean_standalone_elapsed_ms` field
  populated.
- New quantified transaction theorem builds in `Trinity.TernaryFPGABoot`.

**Risk:** low. No hardware dependency; expands formal coverage and CI metrics.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-449`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W450.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W449 order

1. **Default to Variant B** — it does not depend on the bench, expands the
   formal boot-evidence lattice with a quantified transaction theorem, and adds a
   suite-level metric for the standalone Lean build so that cost can be trended.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail, the snapshot diff gates, or the IGLA seal set.

---

## W449 issue/branch action

- Create GitHub issue for Wave Loop 449 titled **“Wave Loop 449 — formal
  boot-evidence lattice expansion + standalone-build suite metric + competitor
  refresh (Variant B, A optional)”**.
- Create branch **`wave-loop-449`** from the W448 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W449.

---

*φ² + φ⁻² = 3 | TRINITY*
