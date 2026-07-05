# FPGA Boot-Evidence — Wave Loop 435 Cooperation Variants (2026-07-01)

**Issue:** #1395 (Wave Loop 434 closes this; W435 issue to be created)  
**Branch:** `wave-loop-434` → next `wave-loop-435`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

Wave Loop 434 executed **Variant B**: live XADC readout succeeded, the board is
reachable over JTAG, but P12 and the relay gate are still blocked, so real CCLK
capture and automated cold-POR remain impossible. The W435 variants below are
ordered by leverage, not by probability.

---

## Variant A — Real CCLK capture with live XADC context (preferred if bench unblocks)

**Prerequisites:**
- P12 is wired to a logic-analyzer channel.
- A relay / remote-power gate is available for automated cold-POR.

**Goals:**
1. Program SPI flash with the canonical bitstream patched for OSCFSEL=6 (and
   OSCFSEL=7 if time permits).
2. Capture real CCLK during cold-POR boot with the logic analyzer.
3. Run `tri fpga cclk-sweep ... --values 6,7 --xadc` so each boot log records the
   live operating point.
4. Import each capture with `tri fpga measured-to-lean --csv/--vcd --raw-ns
   --standalone --validate --pvt-context <xadc.json> --out <theorem.lean> --json`.
5. Reference `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` (or the generic
   W433 theorem) in the generated proof so the live XADC context is part of the
   machine-checked claim.
6. Update `fpga/HARDWARE_SSOT.md` §3.6 with measured frequency, duty cycle, margin,
   and a note that the claim is grounded in live XADC data.

**Acceptance criteria:**
- At least one real CCLK capture for OSCFSEL=6 is imported into a `measured-to-lean`
  theorem with the live XADC context.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` passes with the documented 7 pre-existing gen-verilog
  failures.

---

## Variant B — Harden the live XADC → PVT context pipeline (default if P12/relay stay blocked)

**Prerequisites:**
- Board remains reachable over JTAG.
- No additional wiring needed.

**Goals:**
1. Add a CLI option to export the rounded `PvtContext` directly from
   `tri fpga read-xadc` (e.g. `--to-pvt-context <file>` or `--process-corner <corner>`).
2. Add a unit test and an integration test for the full
   `read-xadc → pvt-envelope → measured-to-lean` pipeline using the live capture or
   a saved fixture.
3. Extend `measured-to-lean --json` summary to include the source operating point
   (`temp_c`, `vccint_mv`, `vccaux_mv`, `process_corner`) so downstream dashboards
   can correlate the theorem with the live silicon state.
4. Generate a `measured-to-lean` theorem for each OSCFSEL 0..7 using the live XADC
   context and synthetic CCLK fixtures, producing a machine-checked coverage matrix.
5. Optionally add a Lean theorem that quantifies over all OSCFSEL values and the
   live point simultaneously.

**Acceptance criteria:**
- `tri fpga read-xadc` can emit a valid `--pvt-context` JSON directly.
- At least one new unit/integration test exercises the end-to-end live XADC
  → theorem pipeline.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` passes with the documented baseline.

---

## Variant C — Master-merge retry or formal/tooling fallback (fallback if bench still blocked and Variant B is too small)

**Prerequisites:**
- No physical bench work required.

**Goals (choose one sub-variant):**

**C1 — Master-merge of the `gen-verilog` fix set.**
- Re-attempt to bring commit `701d79b3b` (tuple-return / `let` destructuring /
  ROM array / CORDIC fixes) into the wave-loop line from a fresh topic branch.
- Clear the 7 residual yosys smoke failures (#1245).
- This is risky and should only be attempted when the FPGA boot-evidence line is
  not the primary wave focus.

**C2 — Formal envelope extension.**
- Add a computable `Bool` check that combines OSCFSEL validity (`oscfsel ≤ 7`)
  with the XADC envelope check (`xadc_operating_point_within_envelope_dec`).
- Prove equivalence with the propositional form.
- Add a theorem linking the combined check directly to
  `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` for any OSCFSEL and any
  in-envelope point.

**C3 — Competitor and tooling refresh.**
- Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new Sparkle/Clash/
  CIRCT/firtool/Aria-HDL/TernaryCore signals.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` baseline if no compiler work is
  done.

**Acceptance criteria:**
- If C1: `./scripts/tri test` shows 0 gen-verilog-yosys-smoke failures.
- If C2: new computable check and equivalence theorem added; `lake build` passes.
- If C3: refreshed reports; `./scripts/tri test` passes with documented baseline.

---

## Recommended default for W435

**Variant B** unless P12 or the relay gate becomes available. The live XADC
pipeline is now proven end-to-end in principle, but it needs CLI hardening and
integration tests before it can be run reliably by a non-developer. If the bench
unblocks during the wave, switch to **Variant A** immediately.

---

*φ² + φ⁻² = 3 | TRINITY*
