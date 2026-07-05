# FPGA Loop Cooperation Plan — Wave Loop 446 (2026-07-01)

**Issue:** #1419 (W445) → **#1420** (W446)  
**Branch:** `wave-loop-445` → **`wave-loop-446`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W445 outcome summary

Wave Loop 445 executed **Variant B**: the W444 synthetic theorem-matrix fixtures
are now committed as a **golden regression set** under
`tests/fixtures/fpga/theorem-matrix/golden/`. A new Rust unit test replays the
golden fixtures and asserts 24 variants with all `envelope_check: "ok"` and a
`fixtures` block on every variant. The suite-level JSON summary from
`./scripts/tri test --json` now carries `fpga_smoke_gate_elapsed_ms`, populated
from the smoke-gate report's `theorem_matrix.elapsed_ms`.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
`gen-verilog` yosys smoke failures, and the summary reports `ACCEPTABLE: yes`.
Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan correction.** A previous W444 note incorrectly claimed no
Sparkle/Verilean signals after 2026-07-11. In reality Sparkle was highly active:
PR #66 (IP.Net + compiler perf) merged 2026-06-30, and a FIDO2/crypto burst
landed on 2026-07-04 — PR #97 FIDO2/CTAP2 data layer, PR #98 P-256 HW sign
stack + SHA-256 streaming, PR #99 CTAPHID + CTAP2 dispatch top, and PR #100
crypto refactor with P-256 math-property proofs. Sparkle remains the only
competitor at the ternary + Lean 4 + FPGA synthesis intersection (BitNet b1.58
accelerator with 60+ theorems). t27's differentiation is intact: the sealed
spec → generated code → seal hash → physical CCLK/PVT boot-evidence loop is
still unmatched.

---

## Constraint landscape for W446

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed and replayable; no live fixture set yet |
| Suite timing metric | `fpga_smoke_gate_elapsed_ms` is now in the summary |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and fixture diff gates |

---

## Three cooperation variants for Wave Loop 446

### Variant A — Physical cold-POR capture with golden live fixtures (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w446/`, add a CI regression test that
replays the live fixtures, and mint a `XADC_LIVE_W446_OPERATING_POINT` theorem
block plus a quantified combined-check theorem over all documented process
corners.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w446_pvt.json --json out/w446_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w446/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w446 --json
   out/w446_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w446_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w446_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W446_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7 and `ff`/`tt`/`ss` corners.
8. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 138+/138.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean` and `pvt-envelope` reports `inside_envelope: true`.
- Live fixture replay test passes and reports `elapsed_ms`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Golden fixture report-shape diff gate + timing dashboard (default)

**Goal:** protect the committed golden fixture set against accidental drift by
adding a report-shape diff test and a lightweight elapsed_ms dashboard, and
extend the `fpga_smoke_gate_elapsed_ms` metric to also record the replay path.

**Work items:**
1. Add an integration test that runs the golden replay, serializes the report,
   and asserts it matches a recorded snapshot (or is a strict superset of it).
2. Add a `--json` path to the golden replay unit test so the report shape is
   exercised in the Rust test suite, not only via CLI.
3. Persist a `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`
   snapshot and add a test that diffs the actual replay report against it.
4. Extend `SuiteSummary` with `fpga_smoke_gate_replay_elapsed_ms` so CI can
   separately trend generation and replay cost.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals after 2026-07-11.
6. Mint W446 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 138+/138 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and both elapsed-ms fields populated.
- Golden fixture replay report matches the committed snapshot.

**Risk:** low. No hardware dependency; hardens the deterministic artifact trail.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-446`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W447.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W446 order

1. **Default to Variant B** — a report-shape diff gate prevents silent fixture
   drift and gives CI a timing dashboard. It does not depend on the bench.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail or the new CI harness.

---

## W446 issue/branch action

- Create GitHub issue for Wave Loop 446 titled **“Wave Loop 446 — theorem-matrix
  golden fixture diff gate + live-capture fallback + gen-verilog debt
  (Variant B, A optional)”**.
- Create branch **`wave-loop-446`** from the W445 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W446.

---

*φ² + φ⁻² = 3 | TRINITY*
