# FPGA Loop Cooperation Plan — Wave Loop 455 (2026-07-01)

**Issue:** #1424 (W454) → **#1425** (W455)
**Branch:** `wave-loop-454` → **`wave-loop-455`**
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W454 outcome summary

Wave Loop 454 executed **Variant C**: adversarial and robustness theorems in
`proofs/lean4/Trinity/TernaryFPGABoot.lean` while the physical bench remained
blocked and the `gen-verilog` master-merge was found insufficient.

- Added `OUTSIDE_VCCINT_HIGH_W454_OPERATING_POINT` (1200 mV VCCINT) and proved
  the dashboard gate rejects it.
- Added `cclk_oscfsel_7_duty_asymmetry_w454` and
  `cclk_ideal_split_robust_to_1ns_jitter_w454`, proving the ideal 50 % CCLK split
  tolerates ±1 ns jitter and moderate duty-cycle asymmetry even at the fastest
  documented OSCFSEL (~33.3 MHz, 30 ns period).
- Added Rust counterparts and unit tests in `cli/tri/src/fpga.rs`.
- Refreshed `docs/reports/T27_VS_FORMAL_HDL_2026.md` and
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

Variant B (master-merge of `gen-verilog` fixes from `master` `701d79b3b`) was
investigated and rejected: the commit fixes narrow pre-existing issues (const
order, decimal literals, early-return if-else chaining, struct-field reg naming,
zero-arg function dummy input, named function begin blocks) but does **not**
address the current failure modes rooted in missing backend support for tuple
return types, `let` destructuring, and module-level `const` array literal
lowering. A blind merge would also risk regressing the wave-loop branch's own
sub-fixes.

Physical bench execution remains blocked by the missing DLC10 cable / unwired P12
header, and the full `gen-verilog` tuple/array fix set remains unimplemented.

---

## Constraint landscape for W455

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog tuple/array gaps | unimplemented; 7 yosys smoke failures remain the baseline |
| Gen-verilog master-merge (`701d79b3b`) | rejected as insufficient for the current failures |
| High-VCCINT adversarial witness | committed (W454) |
| Duty-cycle / jitter robustness | committed (W454) |
| Four-corner rectangle theorem | committed (W453) |
| VCCAUX independence | formally captured (W451) |
| Smoke-gate report schema | `deny_unknown_fields` guard on generator + consumer (W453) |
| All-ok / missing-bitstream / fast snapshots | committed and regression-tested |
| Full Trinity `lake build` | still broken on unrelated physics proofs; boot target `Trinity.TernaryFPGABoot` still builds |
| Time pressure | Medium; the `gen-verilog` backend gaps are the largest remaining safe lever |

---

## Three cooperation variants for Wave Loop 455

### Variant A — Live-capture high-voltage adversarial replay and fixture archive (preferred if bench unblocks)

**Goal:** obtain real boot-captured CCLK sweeps with live XADC readouts, persist
the fixtures under `tests/fixtures/fpga/theorem-matrix/live-w455/`, add a CI
regression test that replays the live fixtures, and mint an
`XADC_LIVE_W455_OPERATING_POINT` theorem block plus a quantified transaction
theorem over all documented process corners at the captured operating point.
Use the captured point to demonstrate that a real in-envelope measurement
justifies the nominal CCLK timing even with the W454 jitter budget.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w455_pvt.json --json out/w455_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w455/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w455 --json
   out/w455_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w455_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w455_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W455_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and prove
   `xadc_live_w455_all_corners_transaction_ok` using the existing envelope bridge.
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

### Variant B — Implement the missing `gen-verilog` tuple/array backend (default)

**Goal:** close the 7 residual yosys smoke failures by implementing the missing
backend support in `bootstrap/src/compiler.rs`:

- Tuple return types in function signatures (`-> (u32, u32, u32)`).
- `let (a, b, c) = ...` tuple destructuring.
- Module-level `const [N]T = [N]T{...}` array literal lowering to Verilog
  `reg [W-1:0] lut [0:N-1]` with `initial` block.

This is the largest remaining safe lever because it directly attacks the
baseline failures without relying on external hardware or a risky master-merge.

**Work items:**
1. Extend `parse_fn_decl` to recognize `LParen` tuple return types and store the
   tuple width/fields in the AST.
2. Extend `parse_local_decl` to recognize `LParen` after `let` and produce a
   destructuring binding node.
3. Add an AST node for tuple expressions / tuple literals and teach the parser
   to handle comma-separated values inside parentheses.
4. Implement `gen_verilog` lowering for tuple return values (packed function
   result, callee-aware destructuring) and `let` destructuring.
5. Implement module-level `const` array literal lowering to Verilog ROM with
   `reg [W-1:0] lut [0:N-1]` and `initial $readmemh` or inline initialization.
6. Add regression scratch specs under `specs/scratch/` for each new backend
   feature.
7. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
8. Update the gen-verilog baseline in
   `docs/reports/gen_verilog_smoke_baseline.json`.
9. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` and
   `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
10. Mint W455 evidence and cooperation files.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.
- New scratch regression specs pass `yosys read_verilog`.
- FPGA smoke-gate JSON report schema tests still pass.
- The W453/W454 formal boot-evidence theorems still build.

**Risk:** medium-high. The work touches the parser and code generator. Work in
small reviewable chunks; re-seal after each compiler change.

---

### Variant C — Extend the adversarial/robustness lattice (fallback)

**Goal:** if neither the bench unblocks nor the compiler backend work can be
completed in one wave, extend the formal boot-evidence lattice with additional
adversarial or robustness theorems that do not require hardware or compiler
changes.

Candidate theorems:
- Jitter budget refinement: prove a larger, symbolic jitter bound for each OSCFSEL
  by computing the available slack from the worst-case PVT half-period bound.
- Frequency-asymmetry interaction: prove that duty-cycle asymmetry combined with
  the fastest allowed frequency still satisfies the spec.
- OSCL / OSCH robustness: a theorem tying the `oscfsel > 7` rejection to the
  concrete `OSCFSEL_BITS` width and the `cclk_nominal_hz` mapping.
- Measured-point monotonicity: prove that if a measured operating point is inside
  the envelope, moving it closer to best-case (lower temp, higher VCCINT, faster
  corner) does not increase the required half-period bound.

**Work items:**
1. Pick one or two robustness axes that add new falsifiable claims without
   duplicating the existing rectangle/jitter theorems.
2. Add the operating points, helper lemmas, and theorems to
   `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
3. Add unit tests or snapshot coverage in `cli/tri/src/fpga.rs` if the theorem
   has a computable gate counterpart.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
5. Mint evidence and cooperation files for W456.

**Acceptance criteria:**
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` still reports the documented 7 baseline gen-verilog
  failures and `acceptable: true`.
- New adversarial/robustness theorem builds and is covered by at least one
  computable gate or unit test.

**Risk:** low. No hardware dependency and no compiler merge.

---

## Recommended W455 order

1. **Default to Variant B** — the `gen-verilog` tuple/array backend gaps are the
   largest remaining safe lever and directly attack the 7 residual yosys smoke
   failures. Closing them removes the documented baseline and unblocks the
   bitstream synthesis path.
2. **If the bench unblocks during Variant B, execute Variant A** — real capture
   remains the highest-leverage outcome and the pipeline is ready to consume it.
3. **If Variant B is too large for one wave, execute Variant C** — additional
   adversarial/robustness theorems keep the formal lattice moving without
   hardware or compiler changes, while Variant B is split across W455 and W456.

---

## W455 issue/branch action

- GitHub issue for Wave Loop 455: **#1425**.
- Create branch **`wave-loop-455`** from the W454 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W455.

---

*φ² + φ⁻² = 3 | TRINITY*
