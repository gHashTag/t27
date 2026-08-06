# FPGA Loop Cooperation Plan — Wave Loop 438 (2026-07-01)

**Issue:** #1405 (W437) → **#1407** (W438)  
**Branch:** `wave-loop-437` → **`wave-loop-438`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W437 outcome summary

Wave Loop 437 executed **Variant B**: added deterministic
`--synthetic-operating-point` modes to `tri fpga cold-por` and `tri fpga
cclk-sweep`, introduced `tri fpga verify-lean` for checking generated `.lean`
theorem blocks, refactored the PVT source resolver into a public unit-tested
helper, and added round-trip tests for the `operating_point` source label.

All new code passes Rust tests (123/123), Lean builds (`TernaryFPGABoot`), and the
full repo sweep (576/576 non-smoke phases; 7 known gen-verilog smoke failures from
#1245).

---

## Constraint landscape for W438

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; the software-only path is now hardened, so the next safe step is either a real capture or clearing the gen-verilog debt |

---

## Three cooperation variants for Wave Loop 438

### Variant A — Physical cold-POR capture (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts and verify the artifacts end-to-end with `tri fpga verify-lean`.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w438_pvt.json --json out/w438_sweep.json`.
3. Run `tri fpga sweep-report --json out/w438_sweep.json`.
4. Run `tri fpga measured-to-lean --pvt-context out/w438_pvt.json` and
   `tri fpga verify-lean` on the output.
5. Mint a new `XADC_LIVE_W438_OPERATING_POINT` theorem block in
   `TernaryFPGABoot.lean` and a quantified combined-check theorem.
6. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 123+/123.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean`.

**Risk:** high dependency on external hardware state.

---

### Variant B — CI harness and artifact audit trail (default)

**Goal:** integrate the new `verify-lean` / `synthetic` path into the existing
FPGA smoke gate so every green run produces a machine-checkable artifact trail,
and add a regression test that exercises the full dry-run pipeline.

**Work items:**
1. Extend `tri fpga smoke-gate` dry-run path to optionally run
   `cclk-sweep --synthetic-operating-point` and produce a JSON sweep report,
   then assert the report contains `operating_point.source == "synthetic"`.
2. Add a `tri fpga smoke-gate --verify-lean` mode that, after the dry-run
   sweep, generates a synthetic `.lean` theorem and runs `verify-lean` on it.
3. Add unit tests for `verify_lean` edge cases: missing theorem, missing
   summary + missing source comment, mismatched expected source.
4. Add a `--json` machine-readable output mode to `verify-lean` and document its
   schema.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new competitor
   signals (post-2026-07-11 Sparkle talk if public notes appear).
6. Mint the W438 evidence file and cooperation variants for W439.

**Acceptance criteria:**
- `cargo test -p tri` 123+/123.
- `tri fpga smoke-gate --dry-run-verify-lean` (or equivalent) passes end-to-end.
- `./scripts/tri test` baseline remains 7 gen-verilog failures; no new failures.

**Risk:** low. This continues the W437 focus without hardware dependency.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / verilog changes into `wave-loop-438`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W439.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave.

---

## Recommended W438 order

1. **Default to Variant B** — it turns the W437 dry-run path into a CI gate and
   does not depend on the bench.
2. **If the bench unblocks, execute Variant A** — real capture is still the
   highest-leverage outcome.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail.

---

## W438 issue/branch action

- Create GitHub issue **#1407** titled **“Wave Loop 438 — CI artifact audit trail
  for dry-run boot-evidence + real-capture fallback (Variant B, A optional)”**.
- Create branch **`wave-loop-438`** from the W437 land commit.
- Update `docs/NOW.md` to reference W438 / #1407.

---

*φ² + φ⁻² = 3 | TRINITY*
