# FPGA Loop Cooperation Plan — Wave Loop 445 (2026-07-01)

**Issue:** #1418 (W444) → **#1419** (W445)  
**Branch:** `wave-loop-444` → **`wave-loop-445`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W444 outcome summary

Wave Loop 444 executed **Variant B**: the 24-variant board-less theorem matrix is
now deterministic and replayable from JSON fixtures.
`tri fpga smoke-gate --theorem-matrix` persists `pvt.json`, `raw_ns.json`,
`summary.json`, and `theorem.lean` for each `ff`/`tt`/`ss` × OSCFSEL 0..7
variant under `build/fpga/theorem-matrix-fixtures/`, and a new
`--replay-fixtures <dir>` mode reproduces the matrix report from those fixtures
without regenerating the Lean theorems. Every per-variant report entry now
carries a structured `fixtures` object, an `envelope_check: "ok"` verdict, and
the matrix block records `replay: true/false` and `elapsed_ms`.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
`gen-verilog` yosys smoke failures, and the summary reports `ACCEPTABLE: yes`
because those failures exactly match the documented baseline. Physical bench
execution is still blocked by the missing DLC10 cable / unwired P12 header, and
the full `gen-verilog` fix set remains unmerged on `master`.

---

## Constraint landscape for W445

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Fixture replay | generation and replay paths both pass; no checked-in golden fixture set yet |
| Smoke-gate report schema | `schema_version: "1.0"`, 24 variants, per-variant `fixtures` + `envelope_check` |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and fixture diff gates |

---

## Three cooperation variants for Wave Loop 445

### Variant A — Physical cold-POR capture with fixture persistence (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under `build/fpga/theorem-matrix-fixtures-live/`,
verify them with the new `--replay-fixtures` path, and mint a
`XADC_LIVE_W445_OPERATING_POINT` theorem block plus a quantified combined-check
theorem over all documented process corners.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w445_pvt.json --json out/w445_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to a stable
   `build/fpga/theorem-matrix-fixtures-live/` tree.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures build/fpga/theorem-matrix-fixtures-live --json
   out/w445_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w445_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w445_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W445_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7 and `ff`/`tt`/`ss` corners.
8. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 137+/137.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean` and `pvt-envelope` reports `inside_envelope: true`.
- Fixture replay from the live directory produces 24 variants with
  `envelope_check: "ok"`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Checked-in golden fixture regression gate + elapsed_ms metric tracking (default)

**Goal:** commit a golden synthetic fixture set under
`tests/fixtures/fpga/theorem-matrix/` and add a CI test that replays the matrix,
diffs the report against expected JSON, and tracks `elapsed_ms` across waves so
regressions in replay time or report shape are caught automatically.

**Work items:**
1. Copy the W444 synthetic fixture set into
   `tests/fixtures/fpga/theorem-matrix/golden/` and add it to git.
2. Add a unit/integration test that runs
   `tri fpga smoke-gate --theorem-matrix --replay-fixtures
   tests/fixtures/fpga/theorem-matrix/golden --json /tmp/w445_golden_replay.json`
   and asserts:
   - `passed: true`,
   - 24 variants,
   - all `envelope_check: "ok"`,
   - report shape matches a recorded golden snapshot (or is a superset of it).
3. Add an `elapsed_ms` metric to the suite-level JSON summary so CI can trend
   generation vs replay cost over time.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals after 2026-07-11.
5. Mint W445 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 137+/137 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true`.
- Golden fixture replay test passes and reports `elapsed_ms`.

**Risk:** low. No hardware dependency; hardens the deterministic artifact trail.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-445`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
  failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W446.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W445 order

1. **Default to Variant B** — a checked-in golden fixture gate makes the matrix
   reproducible across machines, protects against silent report-shape drift, and
   provides a time metric. It does not depend on the bench.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail or the new CI harness.

---

## W445 issue/branch action

- Create GitHub issue for Wave Loop 445 titled **“Wave Loop 445 — theorem-matrix
  golden fixture gate + real-capture fallback + gen-verilog debt (Variant B, A
  optional)”**.
- Create branch **`wave-loop-445`** from the W444 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W445.

---

*φ² + φ⁻² = 3 | TRINITY*
