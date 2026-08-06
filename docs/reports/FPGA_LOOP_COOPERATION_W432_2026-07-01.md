# FPGA Loop Cooperation Variants — Wave Loop 432

**Date:** 2026-07-01  
**Current issue:** #1389 (W431, closed by W432 issue #1391)  
**Current branch:** `wave-loop-431` → `wave-loop-432`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 431 closed the formal gap between a live XADC operating point and the
PVT-aware flash-timing proof pipeline. The physical bench is still blocked:

- P12 (CFGCLK / CCLK_0) is not wired to a logic-analyzer channel.
- No relay / remote-power cold-POR gate is wired.
- The on-board Xilinx DLC10 / Platform Cable USB II is not connected to the host.

Wave Loop 432 picks the first variant whose prerequisites are satisfied at
start-of-wave.

---

## Variant A — First real CCLK capture with live XADC/PVT theorem

**Prerequisites:** P12 wired to a logic-analyzer channel; relay/remote-power
cold-POR gate available; board reachable.

**Goal:** produce the first real, measured CCLK proof that carries a live XADC
operating point through the PVT-aware pipeline.

**Tasks:**

1. Program the SPI flash with the `OSCFSEL=6` variant:
   ```bash
   tri fpga program-flash \
     build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel06.bit \
     --spi-buswidth 1 --verify
   ```
2. Disconnect the JTAG cable and power-cycle the board (cold-POR protocol).
3. Capture CCLK immediately after POR using the wired logic-analyzer channel.
4. Record the live operating point:
   ```bash
   tri fpga read-xadc --cable digilent_hs2 > build/fpga/w432_xadc.json
   ```
5. Generate the raw-ns theorem with the real XADC PVT context and a machine-readable
   summary:
   ```bash
   tri fpga measured-to-lean --csv build/fpga/cclk_oscfsel06.csv --raw-ns --validate \
       --pvt-context build/fpga/w432_xadc.json --standalone --out build/fpga/CclkOscfsel06.lean \
       --json > build/fpga/CclkOscfsel06_summary.json
   ```
6. Typecheck the standalone theorem:
   ```bash
   cp build/fpga/CclkOscfsel06.lean proofs/lean4/Trinity/
   cd proofs/lean4 && lake build Trinity.CclkOscfsel06
   ```
7. Repeat for `OSCFSEL=7` if time permits.
8. Update `fpga/HARDWARE_SSOT.md` §3.6 with the measured frequency, duty cycle,
   XADC values, and `margin_ns`.
9. Close-out report + cooperation variants for W433.

**Acceptance criteria:**
- At least one real CCLK capture is imported into a typechecked Lean 4 theorem.
- The `--json` summary is committed alongside the generated theorem.
- `lake build` and `cargo test --bin tri fpga::` pass.

---

## Variant B — Live XADC end-to-end validation (bench partially available)

**Prerequisites:** board reachable over JTAG/SRAM (as in W422) but P12 and/or the
relay gate are still blocked.

**Goal:** validate the XADC → PVT context bridge against a real board read and
generate at least one proof artifact that uses a genuine operating point, even if
the CCLK itself remains synthetic.

**Tasks:**

1. Capture the live XADC operating point from the board:
   ```bash
   tri fpga read-xadc --cable digilent_hs2 --json > build/fpga/w432_xadc.json
   ```
2. Verify the JSON converts cleanly to a `PvtContext`:
   ```bash
   tri fpga pvt-envelope --pvt-context build/fpga/w432_xadc.json
   ```
3. Produce a raw-ns theorem that uses the real XADC context with a synthetic CCLK
   fixture (CI-validated, but operating point is genuine):
   ```bash
   echo '{"period_ns":40,"sck_low_ns":20,"sck_high_ns":20,"source":"xadc_fixture"}' > raw.json
   tri fpga measured-to-lean --file raw.json --raw-ns --validate \
       --pvt-context build/fpga/w432_xadc.json --standalone --out build/fpga/W432XadcFixture.lean \
       --json > build/fpga/W432XadcFixture_summary.json
   ```
4. Alternatively, run `tri fpga cclk-sweep` over `OSCFSEL=0..5` with `--xadc` and
   manual power cycles, collecting real STAT + XADC logs even though CCLK is not
   independently measured.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new July 2026
   competitor signals.
6. Re-evaluate `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`; if a narrow #1245
   sub-fix appears safe, land it; otherwise keep deferring.
7. Close-out report + cooperation variants for W433.

**Acceptance criteria:**
- A real `read-xadc` JSON file is converted to a `PvtContext` without error.
- A generated Lean theorem typechecks with the real context (synthetic fixture is
  acceptable).
- Updated competitor/defect reports.

---

## Variant C — Master-merge / rebase wave to clear #1245 (formal/tooling fallback)

**Prerequisites:** bench still blocked; no safe narrow gen-verilog fix available.

**Goal:** resolve the 7 residual yosys smoke failures by bringing the full
`master` fix set (`701d79b3b`) into the wave-loop branch, and add any remaining
formal boot-evidence lemmas that do not require physical capture.

**Tasks:**

1. Create a dedicated merge/rebase plan for `wave-loop-432`:
   - Identify the exact `master` commits that fix tuple-return / `let`
     destructuring / ROM arrays / CORDIC.
   - Rebase `wave-loop-431` onto `master` (or merge `master` into
     `wave-loop-432`) in a clean, reviewable operation.
2. Run the full CI sweep after the merge:
   ```bash
   cargo test --bin tri
   ./scripts/tri test
   ```
3. Confirm the yosys smoke gate passes for the previously failing specs:
   - `specs/igla/race/cordic.t27`
   - `specs/igla/race/cordic_top.t27`
   - `specs/scratch/w378_let_destructuring.t27`
   - `specs/scratch/w379_let_destructuring_generalized.t27`
   - `specs/scratch/w380_tuple_return.t27`
   - `specs/scratch/w381_tuple_call_chain.t27`
   - `specs/scratch/w383_rom_array.t27`
4. If the merge is too risky for a single wave, fall back to one formal/tooling
   sub-task from the list below:
   - Add per-OSCFSEL PVT-context theorems for every process corner (`ff`/`tt`/`ss`).
   - Harden `tri fpga sweep-report` to emit machine-readable JSON in addition to
     markdown.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with a deeper competitor
     comparison against Sparkle/Verilean's July 2026 activity.
5. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the new baseline.
6. Close-out report + cooperation variants for W433.

**Acceptance criteria:**
- Either the 7 yosys smoke failures are cleared, or a concrete reason is
  documented with an alternative formal/tooling deliverable landed.
- `cargo test --bin tri` and `./scripts/tri test` pass with the new baseline.
- Issue/branch for W433 are created.

---

## Default selection

- **Preferred:** Variant A if P12 is wired and a relay gate exists.
- **Likely:** Variant B if the board is reachable but P12/relay are still blocked.
- **Fallback:** Variant C if the bench remains unreachable and the master-merge
  debt is the highest-value work that can be shipped board-less.

*φ² + φ⁻² = 3 | TRINITY*
