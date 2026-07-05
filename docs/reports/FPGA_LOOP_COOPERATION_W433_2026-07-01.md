# FPGA Loop Cooperation Variants — Wave Loop 433

**Date:** 2026-07-01  
**Current issue:** #1391 (W432, closed by W433 issue #1393)  
**Current branch:** `wave-loop-432` → `wave-loop-433`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 432 extended the FPGA boot-evidence line with per-process-corner
raw-ns OSCFSEL theorems in Lean 4 while the physical bench remained blocked:

- P12 (CFGCLK / CCLK_0) is not wired to a logic-analyzer channel.
- No relay / remote-power cold-POR gate is wired.
- The on-board Xilinx DLC10 / Platform Cable USB II is not connected to the host.
- The `gen-verilog` fix set (`701d79b3b`) is on a divergent `master` lineage not
  safely reachable from `wave-loop-432`.

Wave Loop 433 picks the first variant whose prerequisites are satisfied at
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
   tri fpga read-xadc --cable digilent_hs2 --json > build/fpga/w433_xadc.json
   ```
5. Generate the raw-ns theorem with the real XADC PVT context and a machine-readable
   summary:
   ```bash
   tri fpga measured-to-lean --csv build/fpga/cclk_oscfsel06.csv --raw-ns --validate \
       --pvt-context build/fpga/w433_xadc.json --standalone --out build/fpga/CclkOscfsel06.lean \
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
9. Close-out report + cooperation variants for W434.

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
   tri fpga read-xadc --cable digilent_hs2 --json > build/fpga/w433_xadc.json
   ```
2. Verify the JSON converts cleanly to a `PvtContext`:
   ```bash
   tri fpga pvt-envelope --pvt-context build/fpga/w433_xadc.json
   ```
3. Produce a raw-ns theorem that uses the real XADC context with a synthetic CCLK
   fixture (CI-validated, but operating point is genuine):
   ```bash
   echo '{"period_ns":40,"sck_low_ns":20,"sck_high_ns":20,"source":"xadc_fixture"}' > raw.json
   tri fpga measured-to-lean --file raw.json --raw-ns --validate \
       --pvt-context build/fpga/w433_xadc.json --standalone --out build/fpga/W433XadcFixture.lean \
       --json > build/fpga/W433XadcFixture_summary.json
   ```
4. Alternatively, run `tri fpga cclk-sweep` over `OSCFSEL=0..5` with `--xadc` and
   manual power cycles, collecting real STAT + XADC logs even though CCLK is not
   independently measured.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new late-July 2026
   competitor signals.
6. Re-evaluate `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`; if a narrow #1245
   sub-fix appears safe, land it; otherwise keep deferring.
7. Close-out report + cooperation variants for W434.

**Acceptance criteria:**
- A real `read-xadc` JSON file is converted to a `PvtContext` without error.
- A generated Lean theorem typechecks with the real context (synthetic fixture is
  acceptable).
- Updated competitor/defect reports.

---

## Variant C — Master-merge feasibility from a fresh topic branch, or another formal/tooling fallback

**Prerequisites:** bench still blocked; no safe narrow gen-verilog fix available.

**Goal:** resolve or make further progress on the 7 residual yosys smoke failures
(#1245) by bringing the `master` fix set into the wave-loop line in a controlled
way, or ship another board-less formal/tooling deliverable if the merge remains
too risky.

**Tasks:**

1. From a fresh topic branch off `wave-loop-433`, re-attempt the merge/rebase of
   the reachable `master` fix set:
   - Identify the exact `master` commits that fix tuple-return / `let`
     destructuring / ROM arrays / CORDIC.
   - If the reachable `origin/master` still does not contain `701d79b3b`,
     investigate the branch topology and decide whether to rebase `wave-loop-*`
     onto the commit that carries the fix set.
2. If the merge is feasible, run the full CI sweep:
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
4. If the merge is still too risky, fall back to one formal/tooling sub-task from
   the list below:
   - Add a machine-readable `--json` mode to `tri fpga sweep-report` if any gaps
     remain in the current JSON schema (e.g., per-variant PVT context or
     `process_corner`).
   - Add a Lean theorem linking a live `XadcOperatingPoint` to the new
     per-process-corner raw-ns OSCFSEL theorem.
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with a deeper competitor
     comparison against Sparkle/Verilean's late-July 2026 activity.
5. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the new baseline.
6. Close-out report + cooperation variants for W434.

**Acceptance criteria:**
- Either the 7 yosys smoke failures are cleared, or a concrete reason is
  documented with an alternative formal/tooling deliverable landed.
- `cargo test --bin tri` and `./scripts/tri test` pass with the new baseline.
- Issue/branch for W434 are created.

---

## Default selection

- **Preferred:** Variant A if P12 is wired and a relay gate exists.
- **Likely:** Variant B if the board is reachable but P12/relay are still blocked.
- **Fallback:** Variant C if the bench remains unreachable and the master-merge
  debt is the highest-value work that can be shipped board-less.

*φ² + φ⁻² = 3 | TRINITY*
