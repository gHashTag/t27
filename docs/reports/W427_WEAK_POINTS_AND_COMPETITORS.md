# Wave Loop 427 — Weak Points and Competitor Scan

**Date:** 2026-07-05  
**Issue:** #1379  
**Branch:** `wave-loop-427`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points

### 1. Bench state is unchanged from W426
- **P12 CCLK probe:** still unwired. Variant A (real CCLK capture for OSCFSEL 6/7)
  cannot run.
- **Relay/remote-power gate:** still absent. True cold-POR automation remains
  manual and non-reproducible enough for a wave close-out.
- **DLC10 cable:** still missing. The working path is openFPGALoader + Digilent
  HS2 (`0x0403:0x6014`).
- **Board is reachable:** `openFPGALoader --detect -c digilent_hs2` finds the
  XC7A200T (`idcode 0x03636093`). This opens Variant B only if real XADC readout
  over HS2 is feasible, which is too large for a single wave.
- **No external captures:** no new CSV/VCD files for OSCFSEL 6/7 were provided.

### 2. Gen-verilog #1245 residual failures (7)
The `gen-verilog-yosys-smoke` phase still fails on 7 specs:

- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

The full fix set exists on `master` at commit `701d79b3b`, but it touches major
features (tuple-return generation, `let` destructuring, ROM arrays, CORDIC). The
wave-loop strategy is narrow, regression-free sub-fixes only. None of the 7
failures qualifies as a narrow single-wave fix, so W427 defers the fix set and
updates `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

### 3. PVT model is still placeholder
The linear derating coefficients in `n25q128_pvt_temp_derating_ns` and
`n25q128_pvt_voltage_derating_ns` are conservative upper envelopes, not real
Micron N25Q128_3V PVT data. The finite-grid theorems and per-OSCFSEL envelope
proofs hold for the placeholder coefficients and survive a future coefficient
update as long as the shape constraints (monotonicity, antitonicity, upper
envelope) are preserved.

### 4. XADC readout is a placeholder
`tri fpga boot-log` / `cold-por` / `cclk-sweep` still embed `xadc.source:
"not_read"`. Real readout requires JTAG XADC register access over the HS2 path;
this is feasible but too large for W427.

### 5. Competitor gap is narrowing
- **Sparkle/Verilean** pushed a public Functional Matsuri 2026 talk on July 11 and
  an RV32 divider correctness proof (PR #65) in late June. Its IP catalog and
  formal depth are growing faster than t27's ternary catalog.
- **Clash 1.10** (April 2026) keeps the ecosystem moving; **Clash Formal** is
  funded through 2025+ with a Clash 2.0 formal-verification roadmap.
- **CIRCT/firtool** delivered the major LTL/Verif/BTOR2 work in March–June 2026;
  the July 4 2026 firtool 1.152.0 release is maintenance, not a formal headline,
  but the pipeline remains current.

t27 must keep the physical boot-evidence line, the spec-first sealed `gen/`
pipeline, and the ternary proof lattice as clear differentiators.
