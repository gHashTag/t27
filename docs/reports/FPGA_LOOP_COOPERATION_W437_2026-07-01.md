# FPGA Loop Cooperation Plan — Wave Loop 437 (2026-07-01)

**Issue:** #1402 (W436) → **#1404** (W437)  
**Branch:** `wave-loop-436` → **`wave-loop-437`**  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## W436 outcome summary

Wave Loop 436 executed **Variant B**: extended the live XADC → PVT context
pipeline into cold-POR boot logs and CCLK sweep reports, added closed-vocabulary
`operating_point` source labels, and proved the quantified combined-check theorem
for OSCFSEL 0..7 under the W434 live XADC operating point.

All new code passes Rust tests (117/117), Lean builds (`TernaryFPGABoot`), and the
full repo sweep (576/576 non-smoke phases; 7 known gen-verilog smoke failures from
#1245).

---

## Constraint landscape for W437

| Constraint | State |
|---|---|
| DLC10 JTAG cable | still not detected on host (`VID=0x03FD`) |
| Board P12 power header | still unwired — no automated cold-POR |
| Gen-verilog fix set | still on `master`, not merged; 7 yosys smoke failures remain the documented baseline |
| Lean theorem lattice | W434 live point is now the most complete formal anchor; needs a fresh real capture to extend it |
| Time pressure | Medium; every wave that advances the evidence trail counts, but no single wave should merge risky unrelated changes |

---

## Three cooperation variants for Wave Loop 437

### Variant A — Physical cold-POR capture (preferred if bench unblocks)

**Goal:** obtain a real boot-captured sweep for OSCFSEL 0..7 with live XADC
readouts, then run `tri fpga measured-to-lean` end-to-end to mint a new
`XADC_LIVE_W437_OPERATING_POINT` theorem that supersedes the W434 point.

**Work items:**
1. Wire P12 or implement a manual/relay cold-POR gate documented in
   `fpga/HARDWARE_SSOT.md`.
2. Run `tri fpga cclk-sweep --process-corner ss --to-pvt-context
   out/w437_pvt.json --json out/w437_sweep.json` (requires the bench to be
   connected).
3. Run `tri fpga sweep-report --json out/w437_sweep.json`.
4. Run `tri fpga measured-to-lean --pvt-context out/w437_pvt.json` to emit a
   new `xadc_live_w437_*` theorem block.
5. Add a quantified combined-check theorem over the W437 operating point.
6. Update `docs/reports/FPGA_LOOP_EVIDENCE_W437_*.md` and
   `docs/reports/T27_VS_FORMAL_HDL_2026.md`.

**Acceptance criteria:**
- `cargo test -p tri` 117/117.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test` baseline 7 gen-verilog failures or fewer; no new failures.
- At least one real captured `operating_point` with `source: "xadc"` appears in
  the generated evidence file.

**Risk:** high dependency on external hardware state. If the bench does not
unblock, the wave stalls without a fallback deliverable.

---

### Variant B — Soft end-to-end sweep and Lean harness hardening (default)

**Goal:** make the new W436 pipeline fully runnable in software/dry-run mode,
add machine-readable validation of the generated `.lean` theorem block, and
prepare the harness so that a future real capture is a one-command replay.

**Work items:**
1. Add a `--dry-run-xadc` / `--synthetic-operating-point` flag to `tri fpga
cclk-sweep` and `cold-por` so CI can exercise the JSON shape and source labels
without a board.
2. Add a `tri fpga verify-lean` subcommand that parses the generated `.lean`
file, checks that every `operating_point` source label matches the CLI
invocation, and reports the quantified theorem count.
3. Add unit tests for `operating_point` round-tripping across boot log → sweep
report → `.lean` JSON → `.lean` theorem comment.
4. Refactor `resolve_pvt_context_for_boot` into a public helper with doc-tests
   for the four source-label priority cases.
5. Update `fpga/HARDWARE_SSOT.md` with a dry-run protocol.
6. Mint the W437 evidence file and cooperation variants for W438.

**Acceptance criteria:**
- `cargo test -p tri` 117+/117.
- Dry-run path produces valid JSON and `.lean` artifacts with
  `source: "not_read"` or `source: "worstcase"`.
- `lake build` of generated dry-run theorem block succeeds.
- No new gen-verilog smoke failures.

**Risk:** low. This is the safe continuation of the W436 focus and does not
depend on the physical bench.

---

### Variant C — Master-merge the gen-verilog fix set (dedicated technical-debt wave)

**Goal:** close the 7 residual yosys smoke failures by merging the full
`gen-verilog` fix set from `master` into the wave-loop branch.

**Work items:**
1. Audit `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` against `master` commit
   `701d79b3b`.
2. Cherry-pick or merge the `bootstrap/src/compiler.rs` and
   `bootstrap/src/verilog.rs` changes into `wave-loop-437`.
3. Regenerate all IGLA seals, run `./scripts/tri test`, and drive yosys smoke
   failures to 0.
4. Add a regression test that captures each previously failing spec.
5. Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the gen-verilog
   milestone.
6. Mint evidence and cooperation variants for W438.

**Acceptance criteria:**
- `./scripts/tri test` reports **0 gen-verilog yosys smoke failures**.
- All 27 IGLA seals regenerate cleanly.
- Lean generic-∀ count does not regress.

**Risk:** medium. The fix set is known-good on `master` but touches the code
generator, which could perturb the bitstream and seal hashes. Should be done as
its own wave, not mixed with FPGA evidence capture.

---

## Recommended W437 order

1. **Default to Variant B** — it continues the W436 evidence pipeline in
   software, keeps the branch green, and does not block on hardware.
2. **If the bench unblocks during the week, execute Variant A** — it is the
   highest-leverage outcome and supersedes Variant B's soft artifacts.
3. **Schedule Variant C as a separate wave** (W438 or later) so the
   gen-verilog merge does not risk the boot-evidence trail.

---

## W437 issue/branch action

- Create GitHub issue **#1404** titled **“Wave Loop 437 — dry-run XADC→PVT
  boot-evidence validation and real-capture fallback (Variant B, A optional)”**.
- Create branch **`wave-loop-437`** from the W436 land commit.
- Update `docs/NOW.md` to reference W437 / #1404.

---

*φ² + φ⁻² = 3 | TRINITY*
