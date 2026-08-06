# FPGA Boot-Evidence — Wave Loop 436 Cooperation Variants (2026-07-01)

**Issue:** #1398 (Wave Loop 435 closes this; W436 issue #1402)
**Branch:** `wave-loop-435` → next `wave-loop-436`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

Wave Loop 435 executed **Variant B**: the live XADC → PVT context pipeline is now hardened, the OSCFSEL 0..7 synthetic theorem matrix is in the formal library, and the board is still reachable over JTAG. P12 and the relay gate remain blocked, and the master-merge debt for the 7 residual gen-verilog yosys smoke failures is still deferred. The W436 variants below are ordered by leverage, not by probability.

---

## Variant A — Real CCLK capture for OSCFSEL=6/7 with live XADC context (preferred if bench unblocks)

**Prerequisites:**
- P12 is wired to a logic-analyzer channel.
- A relay / remote-power gate is available for automated cold-POR.

**Goals:**
1. Program SPI flash with the canonical bitstream patched for OSCFSEL=6 (and OSCFSEL=7 if time permits).
2. Capture real CCLK during cold-POR boot with the logic analyzer.
3. Run `tri fpga cclk-sweep ... --values 6,7 --xadc` so each boot log records the live operating point via `--to-pvt-context`.
4. Import each capture with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --validate --pvt-context <xadc_pvt.json> --out <theorem.lean> --json`.
5. Reference `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` or the W435 combined gate `cclk_variant_and_xadc_envelope_check` in the generated proof so the live XADC context is part of the machine-checked claim.
6. Update `fpga/HARDWARE_SSOT.md` §3.6 with measured frequency, duty cycle, margin, and a note that the claim is grounded in live XADC data.

**Acceptance criteria:**
- At least one real CCLK capture for OSCFSEL=6 is imported into a `measured-to-lean` theorem with the live XADC context.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` passes with the documented 7 pre-existing gen-verilog failures.

---

## Variant B — Extend the live XADC pipeline to cold-POR boot logs and dashboard JSON (default if P12/relay stay blocked)

**Prerequisites:**
- Board remains reachable over JTAG.
- No additional wiring needed.

**Goals:**
1. Add `--to-pvt-context` support to `tri fpga cold-por` / `tri fpga cclk-sweep` so every boot log JSON contains the rounded PVT context recorded at boot time.
2. Extend the sweep/boot log JSON schema with the `operating_point` field (mirroring `measured-to-lean --json`).
3. Add a `tri fpga sweep-report --pvt-context` path that produces a machine-readable JSON report correlating each OSCFSEL variant, its live XADC point, its PVT margin, and its recommendation.
4. Harden the `measured-to-lean` path to accept an `operating_point` source label `"xadc"` when the PVT context is derived from a live `read-xadc` export, not from a hand-written file or `--pvt-worstcase`.
5. Add a Lean theorem that evaluates `cclk_variant_and_xadc_envelope_check` on the actual W434 live point and returns `true` (already present as `xadc_live_w434_oscfsel_6_combined_check_true`; generalize to a quantified example over OSCFSEL 0..7).

**Acceptance criteria:**
- `tri fpga read-xadc --to-pvt-context` output can be consumed directly by `tri fpga measured-to-lean --pvt-context` with a single round-trip documented in `fpga/HARDWARE_SSOT.md`.
- A Rust integration test exercises the full `read-xadc --to-pvt-context` → `measured-to-lean --pvt-context --json` round-trip.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` passes with the documented baseline.

---

## Variant C — Master-merge retry or formal/tooling fallback (fallback if bench still blocked and Variant B is too small)

**Prerequisites:**
- No physical bench work required.

**Goals (choose one sub-variant):**

**C1 — Master-merge of the `gen-verilog` fix set.**
- Re-attempt to bring commit `701d79b3b` (tuple-return / `let` destructuring / ROM array / CORDIC fixes) into the wave-loop line from a fresh topic branch.
- Clear the 7 residual yosys smoke failures (#1245).
- This is risky and should only be attempted when the FPGA boot-evidence line is not the primary wave focus.

**C2 — Formal envelope extension.**
- Add PVT-aware minimum half-period bounds for the remaining Artix-7 configuration modes or for a second flash part (if one is targeted).
- Quantify the combined gate `cclk_variant_and_xadc_envelope_check` over all process corners and prove a worst-case bound monotone in the corner.

**C3 — Competitor and tooling refresh.**
- Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new Sparkle/Clash/CIRCT/firtool/Aria-HDL/TernaryCore/Takahe signals.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` baseline if no compiler work is done.

**Acceptance criteria:**
- If C1: `./scripts/tri test` shows 0 gen-verilog-yosys-smoke failures.
- If C2: new formal bounds or quantified theorems added; `lake build` passes.
- If C3: refreshed reports; `./scripts/tri test` passes with documented baseline.

---

## Recommended default for W436

**Variant B** unless P12 or the relay gate becomes available. The live XADC pipeline is now formally closed end-to-end, but it needs tighter integration with the boot-log / sweep-report JSON so the dashboard can display a single machine-readable artifact per boot attempt. If the bench unblocks during the wave, switch to **Variant A** immediately.

---

*φ² + φ⁻² = 3 | TRINITY*
