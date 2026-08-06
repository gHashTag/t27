# FPGA Loop Cooperation Plan — Wave Loop 444 (2026-07-01)

**Issue:** #1417 (W443) → **#1418** (W444)  
**Branch:** `wave-loop-443` → **`wave-loop-444`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W443 outcome summary

Wave Loop 443 executed **Variant B**: the 24-variant board-less theorem matrix
now carries explicit PVT-envelope validation. `tri fpga pvt-envelope
--pvt-context <ctx.json> --json` emits `inside_envelope: true/false` and a
closed-vocabulary `envelope_check` (`"ok"` / `"failed"` / `"skipped"`), and every
synthetic `ff`/`tt`/`ss` corner context is checked before a theorem is generated.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
`gen-verilog` yosys smoke failures, and the summary reports `ACCEPTABLE: yes`
because those failures exactly match the documented baseline. Physical bench
execution is still blocked by the missing DLC10 cable / unwired P12 header, and
the full `gen-verilog` fix set remains unmerged on `master`.

---

## Constraint landscape for W444

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| PVT-envelope hardening | every synthetic matrix variant now validates and records `envelope_check` |
| Smoke-gate report schema | `schema_version: "1.0"`, 24 variants, per-variant `envelope_check` |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal-coverage expansion |

---

## Three cooperation variants for Wave Loop 444

### Variant A — Physical cold-POR capture (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, verify the artifacts end-to-end with `tri fpga verify-lean`, then mint a
`XADC_LIVE_W444_OPERATING_POINT` theorem block and a quantified combined-check
theorem that spans all documented process corners and validates the envelope.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w444_pvt.json --json out/w444_sweep.json`.
3. Run `tri fpga sweep-report --json out/w444_sweep.json` and confirm every
   variant carries `operating_point.source = "xadc"`.
4. Run `tri fpga pvt-envelope --pvt-context out/w444_pvt.json --json` and confirm
   `inside_envelope: true`.
5. Run `tri fpga measured-to-lean --pvt-context out/w444_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
6. Mint `XADC_LIVE_W444_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7 and `ff`/`tt`/`ss` corners.
7. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 135+/135.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean` and `pvt-envelope` reports `inside_envelope: true`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Matrix replay from JSON fixtures and deterministic CI artifact (default)

**Goal:** make the 24-variant theorem matrix reproducible from checked-in or
machine-generated JSON fixtures so CI can replay the matrix without invoking the
full bitstream path, and add a regression test that asserts the matrix report
matches the fixture set exactly.

**Work items:**
1. Persist the 24 synthetic PVT-context fixtures and raw-ns fixtures under
   `build/fpga/theorem-matrix-fixtures/` (or a similar stable path) when
   `--theorem-matrix` runs.
2. Add a `tri fpga smoke-gate --theorem-matrix --replay-fixtures <dir>` mode that
   re-uses existing fixtures instead of regenerating them.
3. Extend the smoke-gate report `theorem_matrix` block with a `fixtures` object
   recording the paths to the PVT, raw-ns, summary, and Lean files for each variant.
4. Add Rust unit tests for fixture replay and report-to-fixture consistency.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that surface after 2026-07-11.
6. Mint W444 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 135+/135 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and the schema tests validate it.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix
  --json ...` still produces 24 variants with `envelope_check: "ok"` and
  `passed: true`.
- Fixture replay path produces identical report shape and passes in under 30
  seconds.

**Risk:** low. No hardware dependency; extends existing software-only paths.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-444`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W445.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave.

---

## Recommended W444 order

1. **Default to Variant B** — fixture replay makes the matrix deterministic,
   faster in CI, and easier to diff between waves. It does not depend on the
   bench.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail or the new CI harness.

---

## W444 issue/branch action

- Create GitHub issue for Wave Loop 444 titled **“Wave Loop 444 — theorem-matrix
  fixture replay + deterministic CI artifact + real-capture fallback +
  gen-verilog debt (Variant B, A optional)”**.
- Create branch **`wave-loop-444`** from the W443 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W444.

---

*φ² + φ⁻² = 3 | TRINITY*
