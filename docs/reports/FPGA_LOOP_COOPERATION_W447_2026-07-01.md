# FPGA Loop Cooperation Plan — Wave Loop 447 (2026-07-01)

**Issue:** #1420 (W446) → **#1422** (W447)  
**Branch:** `wave-loop-446` → **`wave-loop-447`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W446 outcome summary

Wave Loop 446 executed **Variant B**: the golden theorem-matrix fixture set is
now protected by a committed `expected_report.json` snapshot and a Rust unit
test that diffs the replayed report shape against it. The suite-level JSON
summary from `./scripts/tri test --json` now carries both
`fpga_smoke_gate_elapsed_ms` and `fpga_smoke_gate_replay_elapsed_ms`, allowing
CI to trend generation and replay cost independently.

The full repo sweep remains 576/576 non-smoke PASS with the documented 7
`gen-verilog` yosys smoke failures, and the summary reports `ACCEPTABLE: yes`.
A field-access keyword-escape regression was fixed in
`bootstrap/src/compiler.rs` so that `specs/igla/coder/benchmark.t27` continues to
pass yosys. 52 stale seals were resynced to the current compiler output.

Physical bench execution is still blocked by the missing DLC10 cable / unwired
P12 header, and the full `gen-verilog` fix set remains unmerged on `master`.

**Competitor scan.** Sparkle/Verilean landed a July 4 2026 FIDO2/crypto burst
(PR #97–#100) and has PR #101 open. CIRCT `firtool-1.152.0` shipped July 4 2026
and is still the latest public release. No new public Sparkle signals have
appeared after 2026-07-11. t27's sealed spec → generated code → seal hash →
physical CCLK/PVT boot-evidence loop is still unmatched.

---

## Constraint landscape for W447

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the baseline |
| Golden fixture set | committed, replayable, and snapshot-protected |
| Suite timing metrics | both generation and replay elapsed ms are now in the summary |
| Full Trinity `lake build` | still broken on `NeutrinoMasses.lean` / `Trinity/H4Lagrangian.lean`; boot target `Trinity.TernaryFPGABoot` still builds |
| Lean theorem lattice | W434 live point is still the latest silicon-backed anchor |
| Time pressure | Medium; safe software-only paths remain CI hardening and formal expansion |

---

## Three cooperation variants for Wave Loop 447

### Variant A — Physical cold-POR capture with live fixture archive (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, persist the resulting fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w447/`, add a CI regression test that
replays the live fixtures, and mint a `XADC_LIVE_W447_OPERATING_POINT` theorem
block plus a quantified combined-check theorem over all documented process
corners.

**Work items:**
1. Wire P12 or implement a documented manual/relay cold-POR gate.
2. Run `tri fpga cclk-sweep --process-corner ss --xadc --to-pvt-context
   out/w447_pvt.json --json out/w447_sweep.json` for OSCFSEL 0..7.
3. Copy the resulting per-variant fixtures to
   `tests/fixtures/fpga/theorem-matrix/live-w447/`.
4. Run `tri fpga smoke-gate --verify-lean --theorem-matrix
   --replay-fixtures tests/fixtures/fpga/theorem-matrix/live-w447 --json
   out/w447_replay_report.json` and confirm 24 variants with
   `envelope_check: "ok"`.
5. Run `tri fpga pvt-envelope --pvt-context out/w447_pvt.json --json` and confirm
   `inside_envelope: true`.
6. Run `tri fpga measured-to-lean --pvt-context out/w447_pvt.json --raw-ns
   --standalone --validate --json` and `tri fpga verify-lean --expected-source
   xadc` on the output.
7. Mint `XADC_LIVE_W447_OPERATING_POINT` in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` and a quantified combined-check
   theorem over OSCFSEL 0..7 and `ff`/`tt`/`ss` corners.
8. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new competitor signals
   appear.
9. Update evidence / cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 138+/138.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer.
- At least one real captured `operating_point` with `source: "xadc"` passes
  `verify-lean` and `pvt-envelope` reports `inside_envelope: true`.
- Live fixture replay test passes and reports `elapsed_ms`.

**Risk:** high dependency on external hardware state.

---

### Variant B — Live-capture fallback + formal combined-check over the golden matrix + competitor refresh (default)

**Goal:** keep the pipeline ready for real capture while the bench is blocked,
by adding a synthetic-to-golden equivalence assertion, a quantified Lean
combined-check theorem over the 24-variant golden matrix, and a live-capture
fallback path that records exactly what would be persisted when the board
becomes available.

**Work items:**
1. Add a `tri fpga smoke-gate --theorem-matrix --dry-run-live` mode that emits
   the fixture directory structure and PVT context file that would be produced
   by a live capture, without requiring the board.
2. Add a CI regression test that replays the golden fixtures, the synthetic
   dry-run live fixtures, and asserts both produce 24 variants with
   `envelope_check: "ok"` and matching report shape.
3. Mint a quantified Lean theorem in `Trinity.TernaryFPGABoot.lean` that states
   the combined-check gate holds for every OSCFSEL 0..7 and every `ff`/`tt`/`ss`
   corner under the committed golden operating point.
4. Extend `tri fpga measured-to-lean --standalone` so the generated standalone
   package can be built with only `Trinity.TernaryFPGABoot` (avoiding the
   broken full-Trinity physics targets).
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W447 boundary.
6. Mint W447 evidence and cooperation files.

**Acceptance criteria:**
- `cargo test -p tri` 138+/138 (no new regressions).
- `./scripts/tri test` still reports exactly the documented 7 gen-verilog
  failures and FPGA smoke fails: 0.
- `./scripts/tri test --json suite-summary.json` produces a parseable summary
  with `acceptable: true` and both elapsed-ms fields populated.
- New combined-check theorem builds in `Trinity.TernaryFPGABoot`.
- Golden fixture replay report still matches the committed snapshot.

**Risk:** low. No hardware dependency; expands formal coverage and keeps the
live-capture procedure documented.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the compiler / Verilog changes into `wave-loop-447`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test for each previously failing scratch spec.
5. Update the gen-verilog baseline and competitor report.
6. Mint evidence and cooperation variants for W448.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set touches the code generator and could perturb
bitstream / seal hashes. Should be its own wave if selected.

---

## Recommended W447 order

1. **Default to Variant B** — it does not depend on the bench, expands the
   formal boot-evidence lattice, and documents the live-capture fallback
   procedure so Variant A can execute immediately when hardware unblocks.
2. **If the bench unblocks, execute Variant A** — real capture remains the
   highest-leverage outcome and the pipeline is ready to consume it.
3. **Schedule Variant C as a separate wave** so the gen-verilog merge does not
   risk the boot-evidence trail or the snapshot diff gate.

---

## W447 issue/branch action

- Create GitHub issue for Wave Loop 447 titled **“Wave Loop 447 — live-capture
  fallback + golden-matrix combined-check theorem + competitor refresh
  (Variant B, A optional)”**.
- Create branch **`wave-loop-447`** from the W446 land commit.
- Update `docs/NOW.md` and `.trinity/current-issue.md` to reference W447.

---

*φ² + φ⁻² = 3 | TRINITY*
