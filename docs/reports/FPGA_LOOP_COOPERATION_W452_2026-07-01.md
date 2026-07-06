# FPGA Loop Cooperation Plan — Wave Loop 452 (2026-07-01)

**Issue:** #1423 (W451) → **#1422** (W452)
**Branch:** `wave-loop-451` → **`wave-loop-452`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W451 outcome summary

Wave Loop 451 executed **Variant B**: the formal boot-evidence lattice was
expanded with a quantified end-to-end transaction theorem at the PVT envelope
corner (`boundary_hot_lowv_w451_all_corners_transaction_ok`). The theorem states
that the ideal raw-ns capture produces a flash-spec-compliant SPI read
transaction for every OSCFSEL 0..7 and every process corner (`ff`/`tt`/`ss`) at
+85 °C and 900 mV — the hottest, lowest-VCCINT point inside the documented
operating envelope.

W451 also formalized the VCCAUX-agnostic design of the envelope and the timing
predicate:
- `xadc_operating_point_within_envelope_independent_of_vccaux`
- `n25q128_min_sck_low_ns_pvt_independent_of_vccaux`
- `n25q128_min_sck_high_ns_pvt_independent_of_vccaux`
- `n25q128_min_sck_half_ns_pvt_independent_of_vccaux`
- `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec_independent_of_vccaux`

On the CI side, `bootstrap/src/suite.rs` now uses `FpgaSmokeResultBuilder` to
construct `FpgaSmokeResult` values, centralizing the missing-bitstream and
failure fallback shapes. `SuiteSummary` and `SuitePhaseSummary` carry
`#[serde(deny_unknown_fields)]`, preventing silent schema drift in the
machine-readable suite summary.

Two new synthetic snapshot tests in `cli/tri/src/fpga.rs` protect the report
normalization path:
- `test_smoke_gate_missing_bitstream_matches_snapshot`
- `test_smoke_gate_fast_skipped_standalone_matches_snapshot`

Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** Sparkle/Verilean remains the only fresh Lean-native HDL
signal in early July 2026. The FIDO2/CTAPHID + P-256 proof burst (PR #97–#100,
merged 2026-07-04) is still the most recent public evidence of a broad,
formally verified IP catalog inside Lean 4. No new public Sparkle commits,
CIRCT/firtool releases, Clash promotions, or Lean-native ternary-FPGA projects
appeared between the W450 close-out and the W451 boundary.

---

## Constraint landscape for W452

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Dry-run-live fixture set | committed as a second regression anchor (W448) and has a matching quantified theorem (W450) |
| Boundary-corner theorem | committed (W451) at +85 °C / 900 mV, all corners |
| VCCAUX independence | formally captured in `TernaryFPGABoot.lean` (W451) |
| Suite timing metrics | generation, replay, and standalone elapsed ms are in the summary |
| Standalone `measured-to-lean` | wired into the smoke gate, report block is snapshot-protected, and cost is now visible |
| Full Trinity `lake build` | still broken on unrelated physics proofs; boot target `Trinity.TernaryFPGABoot` still builds |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal expansion |

---

## Three cooperation variants for Wave Loop 452

### Variant A — Live-capture transaction theorem and fixture archive (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w452/`, add a CI regression test that
replays the live fixtures, and mint an `XADC_LIVE_W452_OPERATING_POINT` theorem
block plus a quantified transaction theorem over all documented process corners
at the captured operating point.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w452_pvt.json --json out/w452_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w452/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w452 --json
   out/w452_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w452_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w452_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W452_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and prove
   `xadc_live_w452_all_corners_transaction_ok` using the existing envelope bridge.
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

### Variant B — Envelope theorem lattice continuation + CI metric hardening (default)

**Goal:** while the bench is blocked, continue expanding the formal boot-evidence
lattice and harden the suite's board-less fallback. Add a second boundary
operating point (e.g., the low-temp/high-voltage corner `-40 °C`, `1100 mV`) or
a deliberately adversarial inside-envelope witness, and wire the
missing-bitstream / `--fast` report shapes into the suite summary so the
machine-readable dashboard can distinguish "skipped due to missing bitstream"
from "failed". Optionally add a golden/dry-run-live fixture replay phase to the
suite summary so replay cost is trended alongside generation cost.

**Work items:**
1. Define a second adversarial / envelope-corner operating point in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` (e.g., `-40 °C`, `1100 mV`, nominal
   VCCAUX) and prove a quantified transaction theorem over all corners.
2. Add a `missing-bitstream` unit test or suite-level regression that asserts
   `FpgaSmokeResultBuilder::missing_bitstream()` is the shape used when the demo
   bitstream is absent.
3. Extend `SuiteSummary` with an explicit `fpga_smoke_skipped: Option<bool>`
   field so the JSON dashboard can distinguish skipped from failed smoke gates.
4. Optionally add `fpga_smoke_gate_replay_elapsed_ms` trend chart support, or add
   a dry-run-live replay phase cost to the suite summary.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W452 boundary.
6. Mint W452 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` passes with no new regressions.
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --fast --json suite-summary.json` produces a parseable
  summary with `acceptable: true` and an explicit skipped/failed state for the
  smoke gate.
- New envelope-corner transaction theorem builds in `Trinity.TernaryFPGABoot`.
- Schema tests still pass after any summary-struct change.

**Risk:** low. No hardware dependency; expands formal coverage and hardens the
CI metric path.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-452`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W453.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W452 order

1. **Default to Variant B** — it does not depend on the bench, continues the
   formal boot-evidence expansion, and hardens the suite summary schema so the
   snapshot diff gates and the machine-readable dashboard remain trustworthy
   across waves.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail, the snapshot diff gates, or the IGLA seal set.

---

## W452 issue/branch action

- Create GitHub issue for Wave Loop 452 titled **“Wave Loop 452 — Envelope
  theorem lattice continuation + CI metric hardening (Variant B, A optional)”**.
- Create branch **`wave-loop-452`** from the W451 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W452.

---

*φ² + φ⁻² = 3 | TRINITY*
