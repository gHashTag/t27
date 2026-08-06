# FPGA Loop Cooperation Plan — Wave Loop 442 (2026-07-01)

**Issue:** #1413 (W441) → **#1415** (W442)  
**Branch:** `wave-loop-441` → **`wave-loop-442`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W441 outcome summary

Wave Loop 441 executed **Variant B**: the suite-level JSON summary is now
baseline-aware (`known_failures`, `baseline_failures`, `acceptable`),
`bootstrap/src/suite.rs` has schema and skip/fail regression tests, and
`tri fpga smoke-gate` can run a board-less OSCFSEL 0..7 theorem matrix under a
synthetic PVT context.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
`gen-verilog` yosys smoke failures, and the summary now reports `ACCEPTABLE: yes`
because those failures exactly match the documented baseline. Physical bench
execution is still blocked by the missing DLC10 cable / unwired P12 header, and
the full `gen-verilog` fix set remains unmerged on `master`.

---

## Constraint landscape for W442

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Suite summary | baseline-aware and schema-tested; CI can now distinguish baseline from regression |
| Board-less theorem matrix | OSCFSEL 0..7 synthetic coverage now runs in the smoke gate |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal-coverage expansion |

---

## Three cooperation variants for Wave Loop 442

### Variant A — Physical cold-POR capture (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, verify the artifacts end-to-end with `tri fpga verify-lean`, then mint a
`XADC_LIVE_W442_OPERATING_POINT` theorem block and a quantified combined-check
theorem.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w442_pvt.json --json out/w442_sweep.json`.
3. Run `tri fpga sweep-report --json out/w442_sweep.json`.
4. Run `tri fpga measured-to-lean --pvt-context out/w442_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
5. Mint `XADC_LIVE_W442_OPERATING_POINT` in
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

### Variant B — Expand the board-less formal coverage matrix and harden CI artifacts (default)

**Goal:** build on the W441 baseline-aware summary and OSCFSEL theorem matrix by
extending the board-less proof coverage and making the smoke-gate report even
more deterministic for CI consumers.

**Work items:**
1. Add a board-less theorem-matrix Rust unit test in `cli/tri/src/fpga.rs` that
   exercises `cclk_period_ns` and the per-OSCFSEL fixture/summary generation in
   a temporary directory.
2. Add a `--process-corner` matrix mode to the theorem-matrix loop so it
   generates and verifies the OSCFSEL 0..7 theorems under `ff`, `tt`, and `ss`
   corners (matching the W432 per-corner theorems in Lean).
3. Harden `run_fpga_smoke_gate` to return structured `skipped`/`failed`/`ok`
   records and add a JSON-schema assertion test for the smoke-gate report
   itself.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that surface after 2026-07-11.
5. Mint W442 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 130+/130 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and the schema tests validate it.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix
  --json ...` still produces an 8-element matrix and `passed: true`.

**Risk:** low. No hardware dependency; extends existing software-only paths.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-442`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W443.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave.

---

## Recommended W442 order

1. **Default to Variant B** — it extends the board-less formal coverage matrix,
   adds Rust unit tests for the matrix, and hardens the smoke-gate report schema.
   It does not depend on the bench.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail or the new CI harness.

---

## W442 issue/branch action

- Create GitHub issue **#1415** titled **“Wave Loop 442 — expanded board-less
  theorem matrix + CI artifact hardening + real-capture fallback + gen-verilog
  debt (Variant B, A optional)”**.
- Create branch **`wave-loop-442`** from the W441 land commit.
- Update `docs/NOW.md` to reference W442 / #1415.

---

*φ² + φ⁻² = 3 | TRINITY*
