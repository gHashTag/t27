# FPGA Loop Cooperation Plan — Wave Loop 443 (2026-07-01)

**Issue:** #1415 (W442) → **#1417** (W443)  
**Branch:** `wave-loop-442` → **`wave-loop-443`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W442 outcome summary

Wave Loop 442 executed **Variant B**: the board-less `tri fpga smoke-gate
--theorem-matrix` now covers all three documented Artix-7 process corners
(`ff`/`tt`/`ss`) across OSCFSEL 0..7, producing 24 verified PVT-aware raw-ns
theorems. The smoke-gate JSON report carries `schema_version: "1.0"` and a
structured `theorem_matrix` block, and new Rust unit tests protect the
fixture/summary path and the report schema.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
`gen-verilog` yosys smoke failures, and the summary reports `ACCEPTABLE: yes`
because those failures exactly match the documented baseline. Physical bench
execution is still blocked by the missing DLC10 cable / unwired P12 header, and
the full `gen-verilog` fix set remains unmerged on `master`.

---

## Constraint landscape for W443

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Smoke-gate report schema | `schema_version: "1.0"` and 24-variant theorem matrix now wired into the suite |
| Board-less theorem matrix | OSCFSEL 0..7 × `ff`/`tt`/`ss` synthetic coverage now runs in the smoke gate |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal-coverage expansion |

---

## Three cooperation variants for Wave Loop 443

### Variant A — Physical cold-POR capture (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, verify the artifacts end-to-end with `tri fpga verify-lean`, then mint a
`XADC_LIVE_W443_OPERATING_POINT` theorem block and a quantified combined-check
theorem that spans all documented process corners.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w443_pvt.json --json out/w443_sweep.json`.
3. Run `tri fpga sweep-report --json out/w443_sweep.json` and confirm every
   variant carries `operating_point.source = "xadc"`.
4. Run `tri fpga measured-to-lean --pvt-context out/w443_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
5. Mint `XADC_LIVE_W443_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7 and `ff`/`tt`/`ss` corners.
6. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 130+/130.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Harden the theorem-matrix artifacts and add machine-readable PVT envelope checks (default)

**Goal:** make the 24-variant theorem matrix consumable by downstream CI and add
a machine-readable PVT-envelope gate that verifies every synthetic corner point
lies inside the documented Artix-7 operating rectangle.

**Work items:**
1. Add a `tri fpga pvt-envelope --json` command (or extend the existing one) to
   emit a closed-vocabulary report with `operating_point`, `envelope`, and
   `inside_envelope` booleans for each synthetic corner.
2. Wire the PVT-envelope check into `tri fpga smoke-gate --theorem-matrix` so
   each corner context is validated before the theorem is generated.
3. Extend the smoke-gate report `theorem_matrix` record with a per-variant
   `envelope_check` field (`ok`/`failed`/`skipped`).
4. Add Rust unit tests for the envelope check and the new report fields.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that surface after 2026-07-11.
6. Mint W443 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 131+/131 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and the schema tests validate it.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix
  --json ...` still produces 24 variants, `passed: true`, and each variant carries
  an `envelope_check: "ok"` record.

**Risk:** low. No hardware dependency; extends existing software-only paths.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-443`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W444.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave.

---

## Recommended W443 order

1. **Default to Variant B** — it hardens the 24-variant theorem matrix with
   explicit PVT-envelope validation and makes the smoke-gate report even more
   useful for CI consumers. It does not depend on the bench.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail or the new CI harness.

---

## W443 issue/branch action

- Create GitHub issue for Wave Loop 443 titled **“Wave Loop 443 — PVT-envelope
  hardening for the 24-variant theorem matrix + real-capture fallback +
  gen-verilog debt (Variant B, A optional)”**.
- Branch **`wave-loop-443`** has been created from the W442 close-out commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W443 / #1417.

---

*φ² + φ⁻² = 3 | TRINITY*
