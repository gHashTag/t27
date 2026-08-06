# FPGA Loop Cooperation Plan — Wave Loop 451 (2026-07-01)

**Issue:** #1425 (W450) → **#1426** (W451)
**Branch:** `wave-loop-450` → **`wave-loop-451`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W450 outcome summary

Wave Loop 450 executed **Variant B**: the formal boot-evidence lattice was
expanded with a quantified end-to-end transaction theorem over the committed W448
dry-run-live operating point, `dry_run_live_w448_all_corners_transaction_ok`.
This theorem states that the ideal raw-ns capture produces a flash-spec-compliant
SPI read transaction for every OSCFSEL 0..7 and every process corner
(`ff`/`tt`/`ss`) at the W448 dry-run-live temperature/voltage point. The proof
reuses the W431 XADC-envelope bridge and the W448 adversarial envelope theorem,
so it introduces no new ad-hoc computation.

The smoke-gate `validate_lean_standalone` report block is now protected by a
committed snapshot (`tests/fixtures/fpga/smoke-gate/validate_lean_standalone_snapshot.json`)
and a Rust snapshot diff gate that normalizes run-dependent paths and elapsed
time before comparing actual reports to the expected shape.

The suite gained an opt-in `--fast` mode that skips the ~5–6 min standalone
lake-package build while keeping all deterministic board-less phases and the
documented 7-gen-verilog baseline unchanged. Default CI runs still execute the
standalone build and populate `validate_lean_standalone_elapsed_ms`.

Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** No new public competitor signals appeared between the W449
close-out and the W450 boundary. Sparkle/Verilean's repository last pushed
2026-07-03; PR #66 remains open, the FIDO2/crypto burst (PR #97–#100) remains
merged 2026-07-04, and the README still cites 102 formal theorems. CIRCT
`firtool-1.152.0` (2026-07-04) is still the latest public release. The new
dry-run-live transaction theorem and the `--fast`/snapshot CI hardening keep
t27's sealed spec→generated code→seal hash→physical boot-evidence loop
unmatched.

---

## Constraint landscape for W451

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Dry-run-live fixture set | committed as a second regression anchor (W448) and now has a matching quantified theorem |
| Suite timing metrics | generation, replay, and standalone elapsed ms are in the summary |
| Standalone `measured-to-lean` | wired into the smoke gate, report block is snapshot-protected, and cost is now visible |
| Full Trinity `lake build` | still broken on unrelated physics proofs (`NeutrinoMasses.lean`, `Trinity/H4Lagrangian.lean`); boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W450 added a dry-run-live quantified transaction theorem over all corners |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal expansion |

---

## Three cooperation variants for Wave Loop 451

### Variant A — Live-capture transaction theorem and fixture archive (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w451/`, add a CI regression test that
replays the live fixtures, and mint a `XADC_LIVE_W451_OPERATING_POINT` theorem
block plus a quantified transaction theorem over all documented process corners
at the captured operating point.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w451_pvt.json --json out/w451_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w451/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w451 --json
   out/w451_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w451_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w451_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W451_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and prove
   `xadc_live_w451_all_corners_transaction_ok` using the existing envelope bridge.
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

### Variant B — Formal boot-evidence expansion + adversarial envelope theorem + CI metric hardening (default)

**Goal:** while the bench is blocked, continue expanding the formal boot-evidence
lattice and harden the suite's board-less fallback. Add a quantified
transaction theorem over an adversarial operating point (e.g. envelope corner or
worst-case synthetic context), extend the `--fast` path to produce a minimal
machine-readable summary when the bitstream is absent, and add a non-default
field guard or builder helper to `FpgaSmokeResult`/`SuiteSummary` so future phases
cannot silently drop metrics.

**Work items:**
1. Define an adversarial / envelope-corner operating point in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` (e.g. a point on the PVT envelope
   boundary or a deliberately worse-than-ss context if the lattice supports it).
2. Prove `adversarial_w451_all_corners_transaction_ok`: for every OSCFSEL 0..7
   and every process corner, the raw-ns capture at the adversarial point
   produces a flash-spec-compliant transaction.
3. Extend the smoke-gate snapshot test to also cover the `--fast` path when the
   bitstream is present, or add a dedicated schema test for the skipped
   standalone phase object.
4. Harden `FpgaSmokeResult`/`SuiteSummary` construction so new phases must
   explicitly opt into missing metrics (e.g. a builder that panics on unset
   required fields, or a non-default derive guard).
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W451 boundary.
6. Mint W451 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` passes with no new regressions.
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --fast --json suite-summary.json` produces a parseable
  summary with `acceptable: true` and an explicit `fpga-smoke-gate-standalone`
  phase entry.
- New adversarial transaction theorem builds in `Trinity.TernaryFPGABoot`.
- Snapshot / schema tests still pass after any summary-struct change.

**Risk:** low. No hardware dependency; expands formal coverage and hardens the
CI metric path.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-451`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W452.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W451 order

1. **Default to Variant B** — it does not depend on the bench, continues the
   formal boot-evidence expansion, and hardens the suite summary schema so the
   snapshot diff gates remain trustworthy across waves.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail, the snapshot diff gates, or the IGLA seal set.

---

## W451 issue/branch action

- Create GitHub issue for Wave Loop 451 titled **“Wave Loop 451 — Formal
  boot-evidence expansion + adversarial envelope theorem + CI metric hardening
  (Variant B, A optional)”**.
- Create branch **`wave-loop-451`** from the W450 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W451.

---

*φ² + φ⁻² = 3 | TRINITY*
