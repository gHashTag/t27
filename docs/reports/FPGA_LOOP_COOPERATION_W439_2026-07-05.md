# FPGA Loop Cooperation Plan — Wave Loop 439 (2026-07-05)

**Issue:** #1407 (W438) → **#1409** (W439)  
**Branch:** `wave-loop-438` → **`wave-loop-439`**  
**Date:** 2026-07-05  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W438 outcome summary

Wave Loop 438 executed **Variant B**: the dry-run synthetic path and
`tri fpga verify-lean` were integrated into `tri fpga smoke-gate`.
`--synthetic-operating-point` asserts `operating_point.source == "synthetic"` in
the JSON sweep report, and `--verify-lean` generates a synthetic `.lean`
theorem and verifies it end-to-end. Edge-case unit tests for `verify_lean` and
documentation for the `--json` schema were added. The full repo sweep remains
576/576 non-smoke PASS with the documented 7 `gen-verilog` yosys smoke failures.

---

## Constraint landscape for W439

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Smoke-gate artifact trail | now implemented but not yet wired into the default `./scripts/tri test` FPGA phase |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; the software-only path is hardened, so the next safe step is either CI wiring, a real capture, or clearing the gen-verilog debt |

---

## Three cooperation variants for Wave Loop 439

### Variant A — Physical cold-POR capture (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts and verify the artifacts end-to-end with `tri fpga verify-lean`.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w439_pvt.json --json out/w439_sweep.json`.
3. Run `tri fpga sweep-report --json out/w439_sweep.json`.
4. Run `tri fpga measured-to-lean --pvt-context out/w439_pvt.json` and
   `tri fpga verify-lean` on the output.
5. Mint a new `XADC_LIVE_W439_OPERATING_POINT` theorem block in
   `TernaryFPGABoot.lean` and a quantified combined-check theorem.
6. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 126+/126.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Wire the artifact gate into the default CI sweep (default)

**Goal:** make the synthetic dry-run + verify-lean artifact trail run on every
`./scripts/tri test` invocation, add a machine-readable smoke-gate report, and
harden the edge cases.

**Work items:**
1. Extend the FPGA smoke phase in `./scripts/tri test` to invoke
   `tri fpga smoke-gate --synthetic-operating-point --verify-lean --json
   <report.json>` when the demo bitstream is present.
2. Add a `--json` output mode to `tri fpga smoke-gate` that emits a single
   JSON object with bit-config, dry-run sweep, verify-lean, and yosys results.
3. Document the smoke-gate `--json` schema in `fpga/HARDWARE_SSOT.md`.
4. Add a regression test that exercises `tri fpga smoke-gate --verify-lean`
   end-to-end (or a lighter unit test around the smoke_gate helper when the
   bitstream is available).
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any post-2026-07-11
   Sparkle 関数型まつり notes if public summaries appear.
6. Mint the W439 evidence file and cooperation variants for W440.

**Acceptance criteria:**
- `cargo test -p tri` 126+/126.
- `./scripts/tri test` invokes the synthetic/verify-lean artifact gate in the
  FPGA phase and still reports exactly the documented 7 gen-verilog failures.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --json
  <path>` produces a parseable JSON report.

**Risk:** low. No hardware dependency; only touches the CI harness and CLI
output.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / verilog changes into `wave-loop-439`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W440.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave.

---

## Recommended W439 order

1. **Default to Variant B** — it makes the W438 artifact gate run automatically
   on every CI sweep and does not depend on the bench.
2. **If the bench unblocks, execute Variant A** — real capture is still the
   highest-leverage outcome.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail.

---

## W439 issue/branch action

- Create GitHub issue **#1409** titled **“Wave Loop 439 — CI artifact trail
  hardening for dry-run boot-evidence + real-capture fallback + gen-verilog
  debt (Variant B, A optional)”**.
- Create branch **`wave-loop-439`** from the W438 land commit.
- Update `docs/NOW.md` to reference W439 / #1409.

---

*φ² + φ⁻² = 3 | TRINITY*
