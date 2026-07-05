# FPGA Loop Cooperation Plan — Wave Loop 441 (2026-07-01)

**Issue:** #1411 (W440) → **#1413** (W441)  
**Branch:** `wave-loop-440` → **`wave-loop-441`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W440 outcome summary

Wave Loop 440 executed **Variant B**: the W439 smoke-gate JSON report is now
consumed by `bootstrap/src/suite.rs`, `./scripts/tri test --json <path>` emits a
machine-readable suite-level summary, skip/fail handling is hardened for
bitstream-missing and yosys-unavailable cases, and the test suite is restored to
**127 active passes with 0 ignored tests**. Two previously ignored full-Trinity
`lake build` integration tests were replaced with lightweight content checks on
the generated Lean theorem and the XADC→PVT context path.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
`gen-verilog` yosys smoke failures. Physical bench execution is still blocked by
the missing DLC10 cable / unwired P12 header, and the full `gen-verilog` fix set
remains unmerged on `master`.

---

## Constraint landscape for W441

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Smoke-gate artifact trail | wired into `./scripts/tri test`; report is consumed and a suite-level JSON summary is produced |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and schema enforcement |

---

## Three cooperation variants for Wave Loop 441

### Variant A — Physical cold-POR capture (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts and verify the artifacts end-to-end with `tri fpga verify-lean`, then
mint a `XADC_LIVE_W441_OPERATING_POINT` theorem block and a quantified
combined-check theorem.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w441_pvt.json --json out/w441_sweep.json`.
3. Run `tri fpga sweep-report --json out/w441_sweep.json`.
4. Run `tri fpga measured-to-lean --pvt-context out/w441_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
5. Mint a new `XADC_LIVE_W441_OPERATING_POINT` theorem block in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7.
6. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 127+/127.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Harden the machine-readable CI gate and add schema regression tests (default)

**Goal:** make the suite-level JSON summary and the smoke-gate JSON report
first-class CI artifacts by adding schema regression tests, deterministic
skip/fail unit tests, and a board-less dry-run theorem matrix over OSCFSEL 0..7.

**Work items:**
1. Add a Rust unit test in `bootstrap/src/suite.rs` that exercises `tri_exe()`
   path resolution and the `SuiteSummary` JSON schema.
2. Add a Rust unit test that feeds a synthetic `smoke_gate_report.json` into the
   suite runner and verifies the resulting `SuiteSummary` counts and
   `fpga_smoke_passed` field.
3. Harden `cmd_fpga_smoke_gate` to produce deterministic `skipped` records for
   bitstream-missing and yosys-unavailable cases, and add unit tests for those
   branches.
4. Add a board-less command (or extend `smoke-gate`) that generates a synthetic
   OSCFSEL 0..7 raw-ns theorem matrix under a synthetic PVT context, runs
   `verify-lean --expected-source synthetic` on each theorem, and records the
   result in the smoke-gate JSON report.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that appear after 2026-07-11.
6. Mint W441 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 130+/130 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  and the schema regression tests validate it.

**Risk:** low. No hardware dependency; only touches CI harness and output
formatting.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / verilog changes into `wave-loop-441`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W442.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave.

---

## Recommended W441 order

1. **Default to Variant B** — it hardens the new CI artifacts from W440, makes
   the suite summary and smoke-gate report schema-testable, and adds a board-less
   OSCFSEL theorem matrix. It does not depend on the bench.
2. **If the bench unblocks, execute Variant A** — real capture is still the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail or the new CI harness.

---

## W441 issue/branch action

- Create GitHub issue **#1413** titled **“Wave Loop 441 — CI schema hardening +
  board-less theorem matrix + real-capture fallback + gen-verilog debt (Variant
  B, A optional)”**.
- Create branch **`wave-loop-441`** from the W440 land commit.
- Update `docs/NOW.md` to reference W441 / #1414.

---

*φ² + φ⁻² = 3 | TRINITY*
